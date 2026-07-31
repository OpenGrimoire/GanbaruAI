//! Hosted source-control discovery with recoverable local CLI states.

use super::credentials::{CredentialStore, PlatformCredentialStore, SecretValue};
use super::execution_environment::resolve_environment_workspace;
use super::git_service;
use super::models::{
    ChatError, ChatErrorCode, ChatResult, CredentialReferenceId, ProjectWorkingFolderId,
};
use super::workspace::WorkingFolderAuthorizationOperation;
use crate::db_path;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tauri::Manager;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const OPERATION_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_PROBE_OUTPUT_BYTES: usize = 32 * 1024;
const MAX_OPERATION_OUTPUT_BYTES: usize = 2 * 1024 * 1024;
const MAX_CHANGE_REQUESTS: u32 = 100;
const BITBUCKET_API_ROOT: &str = "https://api.bitbucket.org/2.0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostedSourceControlKind {
    Github,
    Gitlab,
    AzureDevops,
    Bitbucket,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostedSourceControlRead {
    pub kind: HostedSourceControlKind,
    pub label: String,
    pub detected_for_repository: bool,
    pub remote_name: Option<String>,
    pub repository_slug: Option<String>,
    pub status: String,
    pub version: Option<String>,
    pub unavailable_reason: Option<String>,
    pub configuration_hint: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostedChangeRequestRead {
    pub provider_kind: HostedSourceControlKind,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: String,
    pub base_branch: String,
    pub head_branch: String,
    pub author: Option<String>,
    pub draft: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHostedChangeRequest {
    pub working_folder_id: ProjectWorkingFolderId,
    pub execution_environment_id: Option<String>,
    pub provider_kind: HostedSourceControlKind,
    pub repository_slug: String,
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub head_branch: String,
    pub draft: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BitbucketCredential {
    username: String,
    token: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureBitbucketCredential {
    pub repository_slug: String,
    pub username: String,
    pub token: String,
}

#[tauri::command]
pub async fn chat_discover_source_control(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    execution_environment_id: Option<String>,
) -> ChatResult<Vec<HostedSourceControlRead>> {
    let pool = db_path::connect_sqlite(app.clone(), db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))?;
    let authorized = super::workspace_commands::authorize_working_folder(
        &app,
        &pool,
        &working_folder_id,
        WorkingFolderAuthorizationOperation::Git,
    )
    .await?;
    let root =
        resolve_environment_workspace(&app, &pool, authorized, execution_environment_id.as_deref())
            .await?
            .canonical_path;
    let remotes = git_service::remotes(&root).await?;
    let detected = remotes
        .iter()
        .filter_map(|remote| {
            remote
                .fetch_url
                .as_deref()
                .and_then(detect_remote)
                .map(|(kind, slug)| (kind, remote.name.clone(), slug))
        })
        .collect::<Vec<_>>();
    let mut reads = Vec::with_capacity(4);
    for (kind, label, executable, install_hint, auth_arguments) in [
        (
            HostedSourceControlKind::Github,
            "GitHub",
            Some("gh"),
            "Install the GitHub CLI, then run gh auth login.",
            &["auth", "status"] as &[&str],
        ),
        (
            HostedSourceControlKind::Gitlab,
            "GitLab",
            Some("glab"),
            "Install the GitLab CLI, then run glab auth login.",
            &["auth", "status"],
        ),
        (
            HostedSourceControlKind::AzureDevops,
            "Azure DevOps",
            Some("az"),
            "Install the Azure CLI and Azure DevOps extension, then run az login.",
            &["account", "show", "--output", "json"],
        ),
        (
            HostedSourceControlKind::Bitbucket,
            "Bitbucket",
            None,
            "Store a Bitbucket access token in the operating-system credential store.",
            &[],
        ),
    ] {
        let repository = detected.iter().find(|entry| entry.0 == kind);
        reads.push(match executable {
            Some(executable) => {
                probe_cli(
                    kind,
                    label,
                    executable,
                    auth_arguments,
                    install_hint,
                    repository,
                )
                .await
            }
            None => probe_bitbucket(repository, install_hint).await,
        });
    }
    reads.sort_by_key(|entry| !entry.detected_for_repository);
    Ok(reads)
}

#[tauri::command]
pub async fn chat_list_hosted_change_requests(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    execution_environment_id: Option<String>,
    provider_kind: HostedSourceControlKind,
    repository_slug: String,
    limit: u32,
) -> ChatResult<Vec<HostedChangeRequestRead>> {
    validate_text(&repository_slug, "repositorySlug", 2_048)?;
    let root = source_control_root(
        &app,
        &db_url,
        &working_folder_id,
        execution_environment_id.as_deref(),
    )
    .await?;
    let limit = limit.clamp(1, MAX_CHANGE_REQUESTS);
    if provider_kind == HostedSourceControlKind::Bitbucket {
        return bitbucket_list_change_requests(&repository_slug, limit).await;
    }
    let arguments = list_arguments(provider_kind, &repository_slug, limit)?;
    let output = run_cli_operation(provider_executable(provider_kind)?, &arguments, &root).await?;
    parse_change_request_list(provider_kind, &output)
}

#[tauri::command]
pub async fn chat_create_hosted_change_request(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateHostedChangeRequest,
) -> ChatResult<HostedChangeRequestRead> {
    validate_text(&request.repository_slug, "repositorySlug", 2_048)?;
    validate_text(&request.title, "title", 1_024)?;
    validate_text(&request.base_branch, "baseBranch", 1_024)?;
    validate_text(&request.head_branch, "headBranch", 1_024)?;
    if request.body.len() > 262_144 || request.body.contains('\0') {
        return Err(ChatError::validation(
            "body",
            "Change request body is invalid",
        ));
    }
    let root = source_control_root(
        &app,
        &db_url,
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    if request.provider_kind == HostedSourceControlKind::Bitbucket {
        return bitbucket_create_change_request(&request).await;
    }
    let arguments = create_arguments(&request)?;
    let output = run_cli_operation(
        provider_executable(request.provider_kind)?,
        &arguments,
        &root,
    )
    .await?;
    let reference = extract_created_reference(request.provider_kind, &output)?;
    let view_arguments =
        view_arguments(request.provider_kind, &request.repository_slug, &reference)?;
    let detail = run_cli_operation(
        provider_executable(request.provider_kind)?,
        &view_arguments,
        &root,
    )
    .await?;
    parse_change_request(request.provider_kind, &detail)
}

#[tauri::command]
pub async fn chat_checkout_hosted_change_request(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    execution_environment_id: Option<String>,
    provider_kind: HostedSourceControlKind,
    reference: String,
    remote_name: Option<String>,
) -> ChatResult<git_service::GitStatusRead> {
    validate_text(&reference, "reference", 2_048)?;
    if let Some(remote_name) = remote_name.as_deref() {
        validate_text(remote_name, "remoteName", 240)?;
    }
    let root = source_control_root(
        &app,
        &db_url,
        &working_folder_id,
        execution_environment_id.as_deref(),
    )
    .await?;
    let _guard = app
        .state::<super::workspace_mutation::ChatWorkspaceMutationRegistry>()
        .try_mutation(&root)?;
    if provider_kind == HostedSourceControlKind::Bitbucket {
        let remote_name = remote_name.as_deref().unwrap_or("origin");
        git_service::checkout_bitbucket_pull_request(&root, remote_name, &reference).await?;
        return git_service::status(&root).await;
    }
    let arguments = checkout_arguments(provider_kind, &reference, remote_name.as_deref())?;
    run_cli_operation(provider_executable(provider_kind)?, &arguments, &root).await?;
    git_service::status(&root).await
}

#[tauri::command]
pub async fn chat_configure_bitbucket_credential(
    request: ConfigureBitbucketCredential,
) -> ChatResult<()> {
    validate_text(&request.repository_slug, "repositorySlug", 2_048)?;
    validate_text(&request.username, "username", 512)?;
    if request.token.is_empty() || request.token.len() > 8_192 || request.token.contains('\0') {
        return Err(ChatError::validation(
            "token",
            "Bitbucket access token is invalid",
        ));
    }
    let credential = BitbucketCredential {
        username: request.username,
        token: request.token,
    };
    bitbucket_authenticated_request(reqwest::Method::GET, "/user", &credential, None).await?;
    let encoded = serde_json::to_string(&credential).map_err(|_| credential_error())?;
    let secret = SecretValue::new(encoded).map_err(|_| credential_error())?;
    PlatformCredentialStore::default()
        .replace(
            &bitbucket_credential_reference(&request.repository_slug)?,
            &secret,
        )
        .map_err(|_| credential_error())?;
    Ok(())
}

#[tauri::command]
pub fn chat_remove_bitbucket_credential(repository_slug: String) -> ChatResult<bool> {
    validate_text(&repository_slug, "repositorySlug", 2_048)?;
    PlatformCredentialStore::default()
        .remove(&bitbucket_credential_reference(&repository_slug)?)
        .map_err(|_| credential_error())
}

async fn source_control_root(
    app: &tauri::AppHandle,
    db_url: &str,
    working_folder_id: &ProjectWorkingFolderId,
    execution_environment_id: Option<&str>,
) -> ChatResult<PathBuf> {
    let pool = db_path::connect_sqlite(app.clone(), db_url.to_string())
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))?;
    let authorized = super::workspace_commands::authorize_working_folder(
        app,
        &pool,
        working_folder_id,
        WorkingFolderAuthorizationOperation::Git,
    )
    .await?;
    Ok(
        resolve_environment_workspace(app, &pool, authorized, execution_environment_id)
            .await?
            .canonical_path,
    )
}

fn provider_executable(kind: HostedSourceControlKind) -> ChatResult<&'static str> {
    match kind {
        HostedSourceControlKind::Github => Ok("gh"),
        HostedSourceControlKind::Gitlab => Ok("glab"),
        HostedSourceControlKind::AzureDevops => Ok("az"),
        HostedSourceControlKind::Bitbucket => Err(ChatError::unsupported(
            "Bitbucket uses the bounded REST adapter instead of a local CLI",
        )),
    }
}

fn list_arguments(
    kind: HostedSourceControlKind,
    repository_slug: &str,
    limit: u32,
) -> ChatResult<Vec<String>> {
    Ok(match kind {
        HostedSourceControlKind::Github => vec![
            "pr".into(),
            "list".into(),
            "--repo".into(),
            repository_slug.into(),
            "--state".into(),
            "all".into(),
            "--limit".into(),
            limit.to_string(),
            "--json".into(),
            "number,title,url,state,baseRefName,headRefName,author,isDraft".into(),
        ],
        HostedSourceControlKind::Gitlab => vec![
            "mr".into(),
            "list".into(),
            "--repo".into(),
            repository_slug.into(),
            "--all".into(),
            "--per-page".into(),
            limit.to_string(),
            "--output".into(),
            "json".into(),
        ],
        HostedSourceControlKind::AzureDevops => vec![
            "repos".into(),
            "pr".into(),
            "list".into(),
            "--detect".into(),
            "true".into(),
            "--status".into(),
            "all".into(),
            "--top".into(),
            limit.to_string(),
            "--only-show-errors".into(),
            "--output".into(),
            "json".into(),
        ],
        HostedSourceControlKind::Bitbucket => {
            return Err(ChatError::unsupported(
                "Bitbucket pull request listing requires configured keyring credentials",
            ));
        }
    })
}

fn view_arguments(
    kind: HostedSourceControlKind,
    repository_slug: &str,
    reference: &str,
) -> ChatResult<Vec<String>> {
    Ok(match kind {
        HostedSourceControlKind::Github => vec![
            "pr".into(),
            "view".into(),
            reference.into(),
            "--repo".into(),
            repository_slug.into(),
            "--json".into(),
            "number,title,url,state,baseRefName,headRefName,author,isDraft".into(),
        ],
        HostedSourceControlKind::Gitlab => vec![
            "mr".into(),
            "view".into(),
            reference.into(),
            "--repo".into(),
            repository_slug.into(),
            "--output".into(),
            "json".into(),
        ],
        HostedSourceControlKind::AzureDevops => vec![
            "repos".into(),
            "pr".into(),
            "show".into(),
            "--detect".into(),
            "true".into(),
            "--id".into(),
            reference.into(),
            "--only-show-errors".into(),
            "--output".into(),
            "json".into(),
        ],
        HostedSourceControlKind::Bitbucket => {
            return Err(ChatError::unsupported(
                "Bitbucket pull request details require configured keyring credentials",
            ));
        }
    })
}

fn create_arguments(request: &CreateHostedChangeRequest) -> ChatResult<Vec<String>> {
    Ok(match request.provider_kind {
        HostedSourceControlKind::Github => {
            let mut arguments = vec![
                "pr".into(),
                "create".into(),
                "--repo".into(),
                request.repository_slug.clone(),
                "--title".into(),
                request.title.clone(),
                "--body".into(),
                request.body.clone(),
                "--base".into(),
                request.base_branch.clone(),
                "--head".into(),
                request.head_branch.clone(),
            ];
            if request.draft {
                arguments.push("--draft".into());
            }
            arguments
        }
        HostedSourceControlKind::Gitlab => {
            let mut arguments = vec![
                "mr".into(),
                "create".into(),
                "--repo".into(),
                request.repository_slug.clone(),
                "--title".into(),
                request.title.clone(),
                "--description".into(),
                request.body.clone(),
                "--target-branch".into(),
                request.base_branch.clone(),
                "--source-branch".into(),
                request.head_branch.clone(),
                "--yes".into(),
            ];
            if request.draft {
                arguments.push("--draft".into());
            }
            arguments
        }
        HostedSourceControlKind::AzureDevops => {
            if request.draft {
                return Err(ChatError::unsupported(
                    "Azure DevOps CLI does not expose draft creation in this adapter",
                ));
            }
            vec![
                "repos".into(),
                "pr".into(),
                "create".into(),
                "--detect".into(),
                "true".into(),
                "--title".into(),
                request.title.clone(),
                "--description".into(),
                request.body.clone(),
                "--target-branch".into(),
                request.base_branch.clone(),
                "--source-branch".into(),
                request.head_branch.clone(),
                "--only-show-errors".into(),
                "--output".into(),
                "json".into(),
            ]
        }
        HostedSourceControlKind::Bitbucket => {
            return Err(ChatError::unsupported(
                "Bitbucket pull request creation requires configured keyring credentials",
            ));
        }
    })
}

fn checkout_arguments(
    kind: HostedSourceControlKind,
    reference: &str,
    remote_name: Option<&str>,
) -> ChatResult<Vec<String>> {
    Ok(match kind {
        HostedSourceControlKind::Github => {
            vec!["pr".into(), "checkout".into(), reference.into()]
        }
        HostedSourceControlKind::Gitlab => {
            vec!["mr".into(), "checkout".into(), reference.into()]
        }
        HostedSourceControlKind::AzureDevops => vec![
            "repos".into(),
            "pr".into(),
            "checkout".into(),
            "--detect".into(),
            "true".into(),
            "--id".into(),
            reference.into(),
            "--remote-name".into(),
            remote_name.unwrap_or("origin").into(),
            "--only-show-errors".into(),
        ],
        HostedSourceControlKind::Bitbucket => {
            return Err(ChatError::unsupported(
                "Bitbucket pull request checkout requires configured keyring credentials",
            ));
        }
    })
}

fn extract_created_reference(kind: HostedSourceControlKind, output: &str) -> ChatResult<String> {
    if kind == HostedSourceControlKind::AzureDevops {
        let value: Value = serde_json::from_str(output).map_err(|_| protocol_error())?;
        return value
            .get("pullRequestId")
            .and_then(Value::as_u64)
            .map(|number| number.to_string())
            .ok_or_else(protocol_error);
    }
    output
        .split_whitespace()
        .find(|value| value.starts_with("https://") || value.starts_with("http://"))
        .map(|value| {
            value
                .trim_end_matches(|character: char| ",.;)".contains(character))
                .to_string()
        })
        .ok_or_else(protocol_error)
}

fn parse_change_request_list(
    kind: HostedSourceControlKind,
    output: &str,
) -> ChatResult<Vec<HostedChangeRequestRead>> {
    let value: Value = serde_json::from_str(output).map_err(|_| protocol_error())?;
    value
        .as_array()
        .ok_or_else(protocol_error)?
        .iter()
        .map(|entry| parse_change_request_value(kind, entry))
        .collect()
}

fn parse_change_request(
    kind: HostedSourceControlKind,
    output: &str,
) -> ChatResult<HostedChangeRequestRead> {
    let value: Value = serde_json::from_str(output).map_err(|_| protocol_error())?;
    parse_change_request_value(kind, &value)
}

fn parse_change_request_value(
    kind: HostedSourceControlKind,
    value: &Value,
) -> ChatResult<HostedChangeRequestRead> {
    let object = value.as_object().ok_or_else(protocol_error)?;
    let number = first_u64(object, &["number", "iid", "pullRequestId"])?;
    let title = first_string(object, &["title"])?;
    let mut url = first_optional_string(object, &["url", "web_url", "webUrl"]);
    if kind == HostedSourceControlKind::AzureDevops {
        if let Some(repository_url) = object
            .get("repository")
            .and_then(Value::as_object)
            .and_then(|repository| first_optional_string(repository, &["webUrl"]))
        {
            url = Some(format!("{repository_url}/pullrequest/{number}"));
        }
    }
    let author = object
        .get("author")
        .or_else(|| object.get("createdBy"))
        .and_then(Value::as_object)
        .and_then(|author| {
            first_optional_string(author, &["login", "username", "name", "displayName"])
        });
    Ok(HostedChangeRequestRead {
        provider_kind: kind,
        number,
        title,
        url: url.ok_or_else(protocol_error)?,
        state: first_optional_string(object, &["state", "status"])
            .unwrap_or_else(|| "unknown".to_string())
            .to_ascii_lowercase(),
        base_branch: normalize_branch(&first_string(
            object,
            &[
                "baseRefName",
                "target_branch",
                "targetBranch",
                "targetRefName",
            ],
        )?),
        head_branch: normalize_branch(&first_string(
            object,
            &[
                "headRefName",
                "source_branch",
                "sourceBranch",
                "sourceRefName",
            ],
        )?),
        author,
        draft: object
            .get("isDraft")
            .or_else(|| object.get("draft"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn first_string(object: &serde_json::Map<String, Value>, fields: &[&str]) -> ChatResult<String> {
    first_optional_string(object, fields).ok_or_else(protocol_error)
}

fn first_optional_string(
    object: &serde_json::Map<String, Value>,
    fields: &[&str],
) -> Option<String> {
    fields
        .iter()
        .find_map(|field| object.get(*field).and_then(Value::as_str))
        .map(str::to_string)
}

fn first_u64(object: &serde_json::Map<String, Value>, fields: &[&str]) -> ChatResult<u64> {
    fields
        .iter()
        .find_map(|field| object.get(*field).and_then(Value::as_u64))
        .ok_or_else(protocol_error)
}

fn normalize_branch(value: &str) -> String {
    value
        .strip_prefix("refs/heads/")
        .unwrap_or(value)
        .to_string()
}

fn validate_text(value: &str, field: &str, maximum: usize) -> ChatResult<()> {
    if value.trim().is_empty()
        || value.len() > maximum
        || value.chars().any(char::is_control)
        || value.starts_with('-')
    {
        return Err(ChatError::validation(
            field,
            "Source-control value is invalid",
        ));
    }
    Ok(())
}

fn protocol_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Protocol,
        "Source-control provider returned an invalid response",
        true,
    )
}

async fn probe_bitbucket(
    repository: Option<&(HostedSourceControlKind, String, String)>,
    configuration_hint: &str,
) -> HostedSourceControlRead {
    let Some((_, remote_name, repository_slug)) = repository else {
        return HostedSourceControlRead {
            kind: HostedSourceControlKind::Bitbucket,
            label: "Bitbucket".to_string(),
            detected_for_repository: false,
            remote_name: None,
            repository_slug: None,
            status: "configuration_required".to_string(),
            version: None,
            unavailable_reason: Some("No Bitbucket remote was detected".to_string()),
            configuration_hint: Some(configuration_hint.to_string()),
        };
    };
    let result = read_bitbucket_credential(repository_slug)
        .and_then(|credential| credential.ok_or_else(credential_missing));
    let (status, unavailable_reason) = match result {
        Ok(credential) => {
            match bitbucket_authenticated_request(reqwest::Method::GET, "/user", &credential, None)
                .await
            {
                Ok(_) => ("available", None),
                Err(error) => ("authentication_required", Some(error.message)),
            }
        }
        Err(error) => ("configuration_required", Some(error.message)),
    };
    HostedSourceControlRead {
        kind: HostedSourceControlKind::Bitbucket,
        label: "Bitbucket".to_string(),
        detected_for_repository: true,
        remote_name: Some(remote_name.clone()),
        repository_slug: Some(repository_slug.clone()),
        status: status.to_string(),
        version: None,
        unavailable_reason,
        configuration_hint: (status != "available").then(|| configuration_hint.to_string()),
    }
}

async fn bitbucket_list_change_requests(
    repository_slug: &str,
    limit: u32,
) -> ChatResult<Vec<HostedChangeRequestRead>> {
    let credential = require_bitbucket_credential(repository_slug)?;
    let (workspace, repository) = bitbucket_repository_parts(repository_slug)?;
    let path = format!(
        "/repositories/{}/{}/pullrequests?pagelen={}&sort=-updated_on&state=OPEN&state=MERGED&state=DECLINED&state=SUPERSEDED",
        percent_encode_segment(workspace),
        percent_encode_segment(repository),
        limit.clamp(1, 50),
    );
    let value =
        bitbucket_authenticated_request(reqwest::Method::GET, &path, &credential, None).await?;
    let entries = value
        .get("values")
        .and_then(Value::as_array)
        .ok_or_else(protocol_error)?;
    entries.iter().map(parse_bitbucket_change_request).collect()
}

async fn bitbucket_create_change_request(
    request: &CreateHostedChangeRequest,
) -> ChatResult<HostedChangeRequestRead> {
    let credential = require_bitbucket_credential(&request.repository_slug)?;
    let (workspace, repository) = bitbucket_repository_parts(&request.repository_slug)?;
    let path = format!(
        "/repositories/{}/{}/pullrequests",
        percent_encode_segment(workspace),
        percent_encode_segment(repository),
    );
    let body = serde_json::json!({
        "title": request.title,
        "description": request.body,
        "draft": request.draft,
        "source": { "branch": { "name": request.head_branch } },
        "destination": { "branch": { "name": request.base_branch } },
    });
    let value =
        bitbucket_authenticated_request(reqwest::Method::POST, &path, &credential, Some(body))
            .await?;
    parse_bitbucket_change_request(&value)
}

fn parse_bitbucket_change_request(value: &Value) -> ChatResult<HostedChangeRequestRead> {
    let object = value.as_object().ok_or_else(protocol_error)?;
    let branch = |name: &str| {
        object
            .get(name)
            .and_then(Value::as_object)
            .and_then(|value| value.get("branch"))
            .and_then(Value::as_object)
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(protocol_error)
    };
    let url = object
        .get("links")
        .and_then(Value::as_object)
        .and_then(|links| links.get("html"))
        .and_then(Value::as_object)
        .and_then(|html| html.get("href"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(protocol_error)?;
    Ok(HostedChangeRequestRead {
        provider_kind: HostedSourceControlKind::Bitbucket,
        number: first_u64(object, &["id"])?,
        title: first_string(object, &["title"])?,
        url,
        state: first_optional_string(object, &["state"])
            .unwrap_or_else(|| "unknown".to_string())
            .to_ascii_lowercase(),
        base_branch: branch("destination")?,
        head_branch: branch("source")?,
        author: object
            .get("author")
            .and_then(Value::as_object)
            .and_then(|author| first_optional_string(author, &["display_name", "nickname"])),
        draft: object
            .get("draft")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

async fn bitbucket_authenticated_request(
    method: reqwest::Method,
    path: &str,
    credential: &BitbucketCredential,
    body: Option<Value>,
) -> ChatResult<Value> {
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(protocol_error());
    }
    let client = reqwest::Client::builder()
        .timeout(OPERATION_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| transport_error())?;
    let mut request = client
        .request(method, format!("{BITBUCKET_API_ROOT}{path}"))
        .basic_auth(&credential.username, Some(&credential.token))
        .header(reqwest::header::ACCEPT, "application/json");
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|_| transport_error())?;
    let status = response.status();
    let bytes = response.bytes().await.map_err(|_| transport_error())?;
    if bytes.len() > MAX_OPERATION_OUTPUT_BYTES {
        return Err(ChatError::new(
            ChatErrorCode::Protocol,
            "Bitbucket response exceeded the supported limit",
            true,
        ));
    }
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(ChatError::new(
            ChatErrorCode::AuthenticationRequired,
            "Bitbucket credentials were rejected",
            true,
        ));
    }
    if !status.is_success() {
        return Err(ChatError::new(
            ChatErrorCode::DriverUnavailable,
            format!(
                "Bitbucket request failed with HTTP status {}",
                status.as_u16()
            ),
            true,
        ));
    }
    serde_json::from_slice(&bytes).map_err(|_| protocol_error())
}

fn require_bitbucket_credential(repository_slug: &str) -> ChatResult<BitbucketCredential> {
    read_bitbucket_credential(repository_slug)?.ok_or_else(credential_missing)
}

fn read_bitbucket_credential(repository_slug: &str) -> ChatResult<Option<BitbucketCredential>> {
    let Some(secret) = PlatformCredentialStore::default()
        .read(&bitbucket_credential_reference(repository_slug)?)
        .map_err(|_| credential_error())?
    else {
        return Ok(None);
    };
    serde_json::from_str(secret.expose())
        .map(Some)
        .map_err(|_| credential_error())
}

fn bitbucket_credential_reference(repository_slug: &str) -> ChatResult<CredentialReferenceId> {
    let digest = format!("{:x}", Sha256::digest(repository_slug.as_bytes()));
    CredentialReferenceId::new(format!("source-control:bitbucket:{}", &digest[..32]))
        .map_err(|_| credential_error())
}

fn bitbucket_repository_parts(repository_slug: &str) -> ChatResult<(&str, &str)> {
    let (workspace, repository) = repository_slug.split_once('/').ok_or_else(|| {
        ChatError::validation("repositorySlug", "Bitbucket repository is invalid")
    })?;
    if workspace.is_empty() || repository.is_empty() || repository.contains('/') {
        return Err(ChatError::validation(
            "repositorySlug",
            "Bitbucket repository is invalid",
        ));
    }
    Ok((workspace, repository))
}

fn percent_encode_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write;
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn credential_missing() -> ChatError {
    ChatError::new(
        ChatErrorCode::AuthenticationRequired,
        "Bitbucket credentials are not configured",
        true,
    )
}

fn credential_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::ConfigurationInvalid,
        "Bitbucket credential operation failed",
        true,
    )
}

fn transport_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::TransportUnavailable,
        "Bitbucket is unavailable",
        true,
    )
}

async fn probe_cli(
    kind: HostedSourceControlKind,
    label: &str,
    executable: &str,
    auth_arguments: &[&str],
    install_hint: &str,
    repository: Option<&(HostedSourceControlKind, String, String)>,
) -> HostedSourceControlRead {
    let version = run_probe(executable, &["--version"]).await;
    let auth = if version.is_ok() {
        run_probe(executable, auth_arguments).await
    } else {
        Err(ProbeFailure::Missing)
    };
    let (status, unavailable_reason) = match (&version, &auth) {
        (Ok(_), Ok(_)) => ("available", None),
        (Ok(_), Err(_)) => (
            "authentication_required",
            Some(format!("{label} CLI is not authenticated")),
        ),
        (Err(ProbeFailure::Missing), _) => (
            "missing",
            Some(format!("{label} CLI is not available on PATH")),
        ),
        (Err(_), _) => ("unavailable", Some(format!("{label} CLI probe failed"))),
    };
    HostedSourceControlRead {
        kind,
        label: label.to_string(),
        detected_for_repository: repository.is_some(),
        remote_name: repository.map(|entry| entry.1.clone()),
        repository_slug: repository.map(|entry| entry.2.clone()),
        status: status.to_string(),
        version: version
            .ok()
            .and_then(|output| output.lines().next().map(str::to_string)),
        unavailable_reason,
        configuration_hint: (status != "available").then(|| install_hint.to_string()),
    }
}

#[derive(Clone, Copy)]
enum ProbeFailure {
    Missing,
    Failed,
}

async fn run_probe(executable: &str, arguments: &[&str]) -> Result<String, ProbeFailure> {
    let mut child = Command::new(executable)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ProbeFailure::Missing
            } else {
                ProbeFailure::Failed
            }
        })?;
    let stdout = child.stdout.take().ok_or(ProbeFailure::Failed)?;
    let stderr = child.stderr.take().ok_or(ProbeFailure::Failed)?;
    let stdout_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stdout
            .take((MAX_PROBE_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map(|_| bytes)
    });
    let stderr_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stderr
            .take((MAX_PROBE_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map(|_| bytes)
    });
    let status = tokio::time::timeout(PROBE_TIMEOUT, child.wait())
        .await
        .map_err(|_| ProbeFailure::Failed)?
        .map_err(|_| ProbeFailure::Failed)?;
    let stdout = stdout_task
        .await
        .map_err(|_| ProbeFailure::Failed)?
        .map_err(|_| ProbeFailure::Failed)?;
    let stderr = stderr_task
        .await
        .map_err(|_| ProbeFailure::Failed)?
        .map_err(|_| ProbeFailure::Failed)?;
    if !status.success()
        || stdout.len() > MAX_PROBE_OUTPUT_BYTES
        || stderr.len() > MAX_PROBE_OUTPUT_BYTES
    {
        return Err(ProbeFailure::Failed);
    }
    let output = if stdout.is_empty() { stderr } else { stdout };
    String::from_utf8(output).map_err(|_| ProbeFailure::Failed)
}

async fn run_cli_operation(
    executable: &str,
    arguments: &[String],
    root: &Path,
) -> ChatResult<String> {
    let mut child = Command::new(executable)
        .args(arguments)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ChatError::new(
                    ChatErrorCode::ExecutableMissing,
                    format!("{executable} is not available on PATH"),
                    true,
                )
            } else {
                ChatError::new(
                    ChatErrorCode::DriverUnavailable,
                    "Source-control command could not be started",
                    true,
                )
            }
        })?;
    let stdout = child.stdout.take().ok_or_else(protocol_error)?;
    let stderr = child.stderr.take().ok_or_else(protocol_error)?;
    let stdout_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stdout
            .take((MAX_OPERATION_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map(|_| bytes)
    });
    let stderr_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stderr
            .take((MAX_OPERATION_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map(|_| bytes)
    });
    let status = tokio::time::timeout(OPERATION_TIMEOUT, child.wait())
        .await
        .map_err(|_| {
            ChatError::new(
                ChatErrorCode::Timeout,
                "Source-control command timed out",
                true,
            )
        })?
        .map_err(|_| {
            ChatError::new(
                ChatErrorCode::DriverUnavailable,
                "Source-control command failed",
                true,
            )
        })?;
    let stdout = stdout_task
        .await
        .map_err(|_| protocol_error())?
        .map_err(|_| protocol_error())?;
    let stderr = stderr_task
        .await
        .map_err(|_| protocol_error())?
        .map_err(|_| protocol_error())?;
    if stdout.len() > MAX_OPERATION_OUTPUT_BYTES || stderr.len() > MAX_OPERATION_OUTPUT_BYTES {
        return Err(ChatError::new(
            ChatErrorCode::Protocol,
            "Source-control command output exceeded the supported limit",
            true,
        ));
    }
    if !status.success() {
        let detail = String::from_utf8_lossy(&stderr);
        let normalized = detail.to_ascii_lowercase();
        let code = if normalized.contains("auth")
            || normalized.contains("login")
            || normalized.contains("credential")
        {
            ChatErrorCode::AuthenticationRequired
        } else {
            ChatErrorCode::DriverUnavailable
        };
        let message = detail.trim();
        return Err(ChatError::new(
            code,
            if message.is_empty() {
                "Source-control command failed".to_string()
            } else {
                message.chars().take(2_000).collect()
            },
            true,
        ));
    }
    String::from_utf8(stdout).map_err(|_| protocol_error())
}

fn detect_remote(remote: &str) -> Option<(HostedSourceControlKind, String)> {
    let normalized = remote.trim_end_matches(".git");
    let (host, path) = if let Ok(url) = reqwest::Url::parse(normalized) {
        (
            url.host_str()?.to_ascii_lowercase(),
            url.path().trim_matches('/').to_string(),
        )
    } else {
        let (host, path) = normalized.split_once(':')?;
        (
            host.trim_start_matches("git@").to_ascii_lowercase(),
            path.trim_matches('/').to_string(),
        )
    };
    let kind = if host == "github.com" {
        HostedSourceControlKind::Github
    } else if host == "gitlab.com" || host.contains("gitlab") {
        HostedSourceControlKind::Gitlab
    } else if host == "dev.azure.com" || host.ends_with("visualstudio.com") {
        HostedSourceControlKind::AzureDevops
    } else if host == "bitbucket.org" || host.contains("bitbucket") {
        HostedSourceControlKind::Bitbucket
    } else {
        return None;
    };
    (!path.is_empty()).then_some((kind, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_https_and_ssh_remotes() {
        assert_eq!(
            detect_remote("git@github.com:owner/project.git"),
            Some((HostedSourceControlKind::Github, "owner/project".to_string()))
        );
        assert_eq!(
            detect_remote("https://gitlab.com/group/nested/project.git"),
            Some((
                HostedSourceControlKind::Gitlab,
                "group/nested/project".to_string()
            ))
        );
        assert_eq!(
            detect_remote("https://bitbucket.org/team/project.git"),
            Some((
                HostedSourceControlKind::Bitbucket,
                "team/project".to_string()
            ))
        );
    }

    #[test]
    fn normalizes_change_requests_from_each_cli_shape() {
        let github = parse_change_request_list(
            HostedSourceControlKind::Github,
            r#"[{"number":17,"title":"Improve chat","url":"https://github.com/o/r/pull/17","state":"OPEN","baseRefName":"dev","headRefName":"feature","author":{"login":"victor"},"isDraft":true}]"#,
        )
        .unwrap();
        assert_eq!(github[0].number, 17);
        assert_eq!(github[0].author.as_deref(), Some("victor"));
        assert!(github[0].draft);

        let gitlab = parse_change_request_list(
            HostedSourceControlKind::Gitlab,
            r#"[{"iid":4,"title":"Improve chat","web_url":"https://gitlab.com/o/r/-/merge_requests/4","state":"opened","target_branch":"dev","source_branch":"feature","author":{"username":"victor"}}]"#,
        )
        .unwrap();
        assert_eq!(gitlab[0].url, "https://gitlab.com/o/r/-/merge_requests/4");

        let azure = parse_change_request_list(
            HostedSourceControlKind::AzureDevops,
            r#"[{"pullRequestId":8,"title":"Improve chat","url":"https://api.invalid/8","status":"active","targetRefName":"refs/heads/dev","sourceRefName":"refs/heads/feature","createdBy":{"displayName":"Victor"},"repository":{"webUrl":"https://dev.azure.com/o/p/_git/r"}}]"#,
        )
        .unwrap();
        assert_eq!(azure[0].base_branch, "dev");
        assert_eq!(
            azure[0].url,
            "https://dev.azure.com/o/p/_git/r/pullrequest/8"
        );
    }

    #[test]
    fn source_control_arguments_keep_values_as_distinct_process_arguments() {
        let request = CreateHostedChangeRequest {
            working_folder_id: ProjectWorkingFolderId::new("workspace").unwrap(),
            execution_environment_id: None,
            provider_kind: HostedSourceControlKind::Github,
            repository_slug: "owner/repository".to_string(),
            title: "Title with shell syntax $(ignored)".to_string(),
            body: "Body with `literal text`".to_string(),
            base_branch: "dev".to_string(),
            head_branch: "feature/chat".to_string(),
            draft: false,
        };
        let arguments = create_arguments(&request).unwrap();
        assert!(arguments.contains(&request.title));
        assert!(arguments.contains(&request.body));
    }

    #[test]
    fn normalizes_bitbucket_change_requests() {
        let change_request = parse_bitbucket_change_request(&serde_json::json!({
            "id": 23,
            "title": "Improve chat",
            "state": "OPEN",
            "destination": { "branch": { "name": "dev" } },
            "source": { "branch": { "name": "feature/chat" } },
            "author": { "display_name": "Victor" },
            "links": { "html": { "href": "https://bitbucket.org/team/project/pull-requests/23" } },
            "draft": true
        }))
        .unwrap();

        assert_eq!(change_request.number, 23);
        assert_eq!(change_request.base_branch, "dev");
        assert_eq!(change_request.head_branch, "feature/chat");
        assert_eq!(change_request.author.as_deref(), Some("Victor"));
        assert!(change_request.draft);
    }

    #[test]
    fn encodes_bitbucket_repository_path_segments() {
        assert_eq!(percent_encode_segment("team name"), "team%20name");
        assert_eq!(percent_encode_segment("project/name"), "project%2Fname");
        assert_eq!(percent_encode_segment("safe._-~"), "safe._-~");
    }
}
