use std::collections::BTreeMap;
use std::io::Read;
use std::time::{Duration, Instant};

use crate::chat::models::ChatResult;
use crate::chat::process::{spawn_provider_process, ProviderProcessConfig};

pub(super) const BENCHMARK_CHILD_ENV: &str = "GANBARU_CHAT_BENCHMARK_CHILD";

/// Runs the inert child mode used to measure supervised process shutdown.
pub fn run_benchmark_child_if_requested() -> bool {
    if std::env::var(BENCHMARK_CHILD_ENV).as_deref() != Ok("1") {
        return false;
    }
    let mut buffer = [0_u8; 1024];
    while std::io::stdin()
        .read(&mut buffer)
        .is_ok_and(|read| read > 0)
    {}
    true
}

/// Measures graceful shutdown of an owned shell-free process.
pub async fn measure_provider_stop() -> ChatResult<f64> {
    let executable = std::env::current_exe().map_err(|error| {
        crate::chat::models::ChatError::driver_unavailable(format!(
            "Chat benchmark executable is unavailable: {error}"
        ))
    })?;
    let mut environment = BTreeMap::new();
    environment.insert(BENCHMARK_CHILD_ENV.to_string(), "1".to_string());
    #[cfg(test)]
    let arguments = vec![
        "--ignored".to_string(),
        "--exact".to_string(),
        "chat::benchmark::tests::benchmark_child_fixture".to_string(),
        "--nocapture".to_string(),
    ];
    #[cfg(not(test))]
    let arguments = Vec::new();
    let mut process = spawn_provider_process(ProviderProcessConfig {
        executable,
        arguments,
        working_directory: std::env::temp_dir(),
        environment,
        stderr_limit_bytes: 8 * 1024,
    })?;
    tokio::time::sleep(Duration::from_millis(25)).await;
    let started = Instant::now();
    process
        .stop(Duration::from_secs(2), Duration::from_secs(1))
        .await?;
    Ok(started.elapsed().as_secs_f64() * 1_000.0)
}

/// Returns cumulative CPU milliseconds for the app process tree.
pub fn process_tree_cpu_time_ms() -> Result<f64, String> {
    platform_process_tree_cpu_time_ms()
}

#[cfg(target_os = "linux")]
fn platform_process_tree_cpu_time_ms() -> Result<f64, String> {
    use std::collections::{HashMap, HashSet};

    let mut processes = HashMap::<u32, (u32, u64)>::new();
    let entries =
        std::fs::read_dir("/proc").map_err(|error| format!("read process list: {error}"))?;
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|value| value.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            continue;
        };
        let Some(after_name) = stat.rfind(')') else {
            continue;
        };
        let fields = stat[after_name + 2..]
            .split_whitespace()
            .collect::<Vec<_>>();
        let (Some(parent), Some(user), Some(system)) =
            (fields.get(1), fields.get(11), fields.get(12))
        else {
            continue;
        };
        let (Ok(parent), Ok(user), Ok(system)) =
            (parent.parse(), user.parse::<u64>(), system.parse::<u64>())
        else {
            continue;
        };
        processes.insert(pid, (parent, user.saturating_add(system)));
    }
    let root = std::process::id();
    let mut included = HashSet::from([root]);
    loop {
        let before = included.len();
        for (&pid, &(parent, _)) in &processes {
            if included.contains(&parent) {
                included.insert(pid);
            }
        }
        if included.len() == before {
            break;
        }
    }
    let ticks = included
        .iter()
        .filter_map(|pid| processes.get(pid).map(|(_, ticks)| *ticks))
        .sum::<u64>();
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks_per_second <= 0 {
        return Err("read process clock rate: unavailable".to_string());
    }
    Ok(ticks as f64 * 1_000.0 / ticks_per_second as f64)
}

#[cfg(target_os = "windows")]
fn platform_process_tree_cpu_time_ms() -> Result<f64, String> {
    use std::collections::{HashMap, HashSet};
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    fn filetime_ticks(value: FILETIME) -> u64 {
        (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|error| format!("snapshot process list: {error}"))?;
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut parents = HashMap::<u32, u32>::new();
    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
        loop {
            parents.insert(entry.th32ProcessID, entry.th32ParentProcessID);
            if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
                break;
            }
        }
    }
    unsafe { CloseHandle(snapshot) }.ok();
    let root = std::process::id();
    let mut included = HashSet::from([root]);
    loop {
        let before = included.len();
        for (&pid, &parent) in &parents {
            if included.contains(&parent) {
                included.insert(pid);
            }
        }
        if included.len() == before {
            break;
        }
    }
    let mut total_ticks = 0_u64;
    for pid in included {
        let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) })
        else {
            continue;
        };
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        if unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) }
            .is_ok()
        {
            total_ticks = total_ticks
                .saturating_add(filetime_ticks(kernel))
                .saturating_add(filetime_ticks(user));
        }
        unsafe { CloseHandle(process) }.ok();
    }
    Ok(total_ticks as f64 / 10_000.0)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn platform_process_tree_cpu_time_ms() -> Result<f64, String> {
    Err("Chat CPU benchmark is supported on Linux and Windows".to_string())
}
