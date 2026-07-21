//! Cursor executable, probe, launch, environment, and continuation identity.

use super::protocol::{protocol_error, CursorProviderSettings, MINIMUM_CURSOR_VERSION};
use crate::chat::models::*;
use crate::chat::process::{spawn_provider_process, ProviderProcessConfig};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

pub const CURSOR_STDERR_LIMIT_BYTES: usize = 256 * 1024;
const VERSION_OUTPUT_LIMIT_BYTES: u64 = 16 * 1024;
const ABOUT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CursorVersion {
    pub date: u32,
}

impl std::fmt::Display for CursorVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let year = self.date / 10_000;
        let month = (self.date / 100) % 100;
        let day = self.date % 100;
        write!(formatter, "{year:04}.{month:02}.{day:02}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CursorAbout {
    pub version: CursorVersion,
    pub account_label: Option<String>,
    pub authenticated: Option<bool>,
}

pub fn process_environment(
    configuration: &ProviderInstanceConfig,
) -> ChatResult<BTreeMap<String, String>> {
    let mut environment = BTreeMap::new();
    for name in inherited_environment_names() {
        if let Ok(value) = std::env::var(name) {
            if !value.contains('\0') {
                environment.insert((*name).to_string(), value);
            }
        }
    }
    for (name, value) in &configuration.environment {
        if name.is_empty()
            || name.contains(['=', '\0'])
            || value.contains('\0')
            || matches!(name.to_ascii_uppercase().as_str(), "HOME" | "USERPROFILE")
        {
            return Err(ChatError::validation(
                "environment",
                "Cursor environment contains a prohibited entry",
            ));
        }
        environment.insert(name.clone(), value.clone());
    }
    Ok(environment)
}

pub fn resolve_executable(
    configured: &str,
    environment: &BTreeMap<String, String>,
) -> ChatResult<PathBuf> {
    let configured = configured.trim();
    if configured.is_empty() || configured.contains('\0') {
        return Err(ChatError::validation(
            "executable",
            "Cursor executable is required",
        ));
    }
    let candidate = if path_has_separator(configured) {
        let path = PathBuf::from(configured);
        if !path.is_absolute() || path.components().any(|part| part == Component::ParentDir) {
            return Err(ChatError::validation(
                "executable",
                "Cursor executable path must be absolute",
            ));
        }
        path
    } else {
        find_on_path(configured, environment).ok_or_else(executable_missing)?
    };
    let resolved = fs::canonicalize(candidate).map_err(|_| executable_missing())?;
    if !resolved.is_file() || !is_executable(&resolved) {
        return Err(executable_missing());
    }
    if matches!(
        resolved.extension().and_then(|value| value.to_str()),
        Some("cmd" | "bat" | "ps1")
    ) {
        return Err(ChatError::unsupported(
            "Cursor command shims are unsupported; configure the native cursor-agent executable",
        ));
    }
    Ok(resolved)
}

pub async fn probe_about(
    executable: &Path,
    working_directory: &Path,
    environment: BTreeMap<String, String>,
) -> ChatResult<CursorAbout> {
    let output = tokio::process::Command::new(executable)
        .arg("about")
        .arg("--json")
        .current_dir(working_directory)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output();
    let output = tokio::time::timeout(ABOUT_TIMEOUT, output)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Timeout, "Cursor about probe timed out", true))?
        .map_err(|_| executable_missing())?;
    if output.stdout.len() as u64 > VERSION_OUTPUT_LIMIT_BYTES
        || output.stderr.len() as u64 > VERSION_OUTPUT_LIMIT_BYTES
    {
        return Err(protocol_error("Cursor about output"));
    }
    parse_about(&output.stdout, &output.stderr, output.status.success())
}

pub fn parse_about(stdout: &[u8], stderr: &[u8], success: bool) -> ChatResult<CursorAbout> {
    let stdout = String::from_utf8_lossy(stdout);
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let version = value
            .get("cliVersion")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| protocol_error("Cursor CLI version"))?;
        let account = value
            .get("userEmail")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let authenticated = if value
            .get("userEmail")
            .is_some_and(serde_json::Value::is_null)
        {
            Some(false)
        } else {
            account.as_ref().map(|_| true)
        };
        return Ok(CursorAbout {
            version: parse_version(version)?,
            account_label: account,
            authenticated,
        });
    }
    let combined = format!("{stdout}\n{}", String::from_utf8_lossy(stderr));
    let version = field(&combined, "CLI Version")
        .or_else(|| {
            combined
                .split_whitespace()
                .find(|value| parse_version(value).is_ok())
        })
        .ok_or_else(|| protocol_error("Cursor CLI version"))?;
    let account = field(&combined, "User Email").map(str::to_string);
    let lower = combined.to_ascii_lowercase();
    let authenticated = if lower.contains("not logged in")
        || lower.contains("authentication required")
        || lower.contains("login required")
    {
        Some(false)
    } else if account.is_some() {
        Some(true)
    } else if success {
        None
    } else {
        Some(false)
    };
    Ok(CursorAbout {
        version: parse_version(version)?,
        account_label: account,
        authenticated,
    })
}

pub fn ensure_supported_version(version: CursorVersion) -> ChatResult<()> {
    if version < parse_version(MINIMUM_CURSOR_VERSION)? {
        return Err(ChatError::new(
            ChatErrorCode::UnsupportedVersion,
            format!(
                "Cursor Agent {version} is unsupported; version {MINIMUM_CURSOR_VERSION} or newer is required"
            ),
            false,
        ));
    }
    Ok(())
}

pub fn parse_version(value: &str) -> ChatResult<CursorVersion> {
    let prefix = value.trim().split('-').next().unwrap_or_default();
    let parts = prefix.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(protocol_error("Cursor CLI version"));
    }
    let year = parts[0].parse::<u32>().ok();
    let month = parts[1].parse::<u32>().ok();
    let day = parts[2].parse::<u32>().ok();
    match (year, month, day) {
        (Some(year), Some(month), Some(day))
            if (2020..=9999).contains(&year)
                && (1..=12).contains(&month)
                && (1..=31).contains(&day) =>
        {
            Ok(CursorVersion {
                date: year * 10_000 + month * 100 + day,
            })
        }
        _ => Err(protocol_error("Cursor CLI version")),
    }
}

pub fn launch_arguments(
    configured: &[String],
    settings: &CursorProviderSettings,
) -> ChatResult<Vec<String>> {
    if configured.iter().any(|argument| {
        let normalized = argument.trim().to_ascii_lowercase();
        argument.contains('\0')
            || matches!(
                normalized.as_str(),
                "acp" | "-e" | "--endpoint" | "--api-key" | "-a"
            )
            || normalized.starts_with("--endpoint=")
            || normalized.starts_with("--api-key=")
    }) {
        return Err(ChatError::validation(
            "launchArguments",
            "Cursor launch arguments cannot override protocol, endpoint, or credentials",
        ));
    }
    let mut arguments = configured.to_vec();
    if let Some(endpoint) = settings.api_endpoint.as_ref() {
        arguments.extend(["-e".to_string(), endpoint.clone()]);
    }
    arguments.push("acp".to_string());
    Ok(arguments)
}

pub fn spawn_connection_process(
    configuration: &ProviderInstanceConfig,
    settings: &CursorProviderSettings,
    working_directory: &Path,
) -> ChatResult<crate::chat::process::ProviderProcessHandle> {
    let environment = process_environment(configuration)?;
    let executable = resolve_executable(&configuration.executable, &environment)?;
    let arguments = launch_arguments(&configuration.launch_arguments, settings)?;
    spawn_provider_process(ProviderProcessConfig {
        executable,
        arguments,
        working_directory: working_directory.to_path_buf(),
        environment,
        stderr_limit_bytes: CURSOR_STDERR_LIMIT_BYTES,
    })
}

pub fn continuation_group(
    configuration: &ProviderInstanceConfig,
    settings: &CursorProviderSettings,
    account_identity: Option<&str>,
) -> ChatResult<ContinuationGroupId> {
    let mut digest = Sha256::new();
    digest.update(b"ganbaru-chat-cursor-account-v1\0");
    if let Some(home) = configuration.provider_home.as_deref() {
        digest.update(home.as_bytes());
    }
    digest.update(b"\0");
    digest.update(
        settings
            .api_endpoint
            .as_deref()
            .unwrap_or("default")
            .as_bytes(),
    );
    digest.update(b"\0");
    digest.update(account_identity.unwrap_or("default").as_bytes());
    ContinuationGroupId::new(format!("cursor-account-{:x}", digest.finalize()))
        .map_err(|_| protocol_error("continuation identity"))
}

fn field<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    value.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim())
            .filter(|value| !value.is_empty())
    })
}

fn path_has_separator(value: &str) -> bool {
    value.contains('/') || value.contains('\\')
}

fn find_on_path(name: &str, environment: &BTreeMap<String, String>) -> Option<PathBuf> {
    let path = environment
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))?
        .1
        .as_str();
    std::env::split_paths(path).find_map(|directory| {
        executable_names(name)
            .into_iter()
            .map(|candidate| directory.join(candidate))
            .find(|candidate| candidate.is_file() && is_executable(candidate))
    })
}

fn executable_names(name: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        if Path::new(name).extension().is_some() {
            vec![name.to_string()]
        } else {
            vec![format!("{name}.exe"), name.to_string()]
        }
    }
    #[cfg(not(windows))]
    {
        vec![name.to_string()]
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn inherited_environment_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &[
            "PATH",
            "PATHEXT",
            "SYSTEMROOT",
            "WINDIR",
            "TEMP",
            "TMP",
            "USERPROFILE",
        ]
    }
    #[cfg(not(windows))]
    {
        &["PATH", "HOME", "LANG", "LC_ALL", "TMPDIR"]
    }
}

fn executable_missing() -> ChatError {
    ChatError::new(
        ChatErrorCode::ExecutableMissing,
        "Cursor Agent executable is unavailable",
        true,
    )
}
