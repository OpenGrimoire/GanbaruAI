use super::*;

#[cfg(windows)]
fn windows_foreground_window() -> Option<::windows::Win32::Foundation::HWND> {
    use ::windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

#[cfg(windows)]
fn windows_foreground_process_id(hwnd: ::windows::Win32::Foundation::HWND) -> Option<u32> {
    use ::windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    let mut process_id = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut process_id as *mut u32));
    }
    if process_id == 0 {
        None
    } else {
        Some(process_id)
    }
}

#[cfg(windows)]
fn windows_process_image_path(process_id: u32) -> Option<String> {
    use ::windows::core::PWSTR;
    use ::windows::Win32::Foundation::CloseHandle;
    use ::windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()? };
    let mut buffer = vec![0u16; 32_768];
    let mut size = buffer.len() as u32;
    let result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size as *mut u32,
        )
    };
    let _ = unsafe { CloseHandle(handle) };
    result.ok()?;
    if size == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..size as usize]))
}

#[cfg(windows)]
fn windows_status_for_window(
    hwnd: ::windows::Win32::Foundation::HWND,
) -> DoomscrollingForegroundDesktopAppStatus {
    let Some(process_id) = windows_foreground_process_id(hwnd) else {
        return unavailable_foreground_desktop_app_status(
            "foreground window process is unavailable",
        );
    };
    let process_path = windows_process_image_path(process_id);
    let process_name = process_path
        .as_deref()
        .and_then(normalize_process_match_name)
        .or_else(|| Some(format!("process-{process_id}")));
    let app_name = process_path
        .as_deref()
        .and_then(|path| Path::new(path).file_stem().and_then(|name| name.to_str()))
        .and_then(normalize_app_candidate_name)
        .or_else(|| process_name.clone())
        .unwrap_or_else(|| format!("process-{process_id}"));
    let mut match_names = Vec::new();
    if let Some(path) = process_path {
        match_names.push(path);
    }
    foreground_status_from_parts(app_name, process_name, Some(process_id), match_names)
}

#[cfg(windows)]
pub(in crate::doomscrolling) fn foreground_desktop_app_status(
) -> DoomscrollingForegroundDesktopAppStatus {
    let Some(hwnd) = windows_foreground_window() else {
        return unavailable_foreground_desktop_app_status("no foreground window is active");
    };
    windows_status_for_window(hwnd)
}

#[cfg(windows)]
pub(in crate::doomscrolling) fn close_current_foreground_desktop_app(
    expected: DoomscrollingForegroundDesktopAppExpectation,
    authorize: &mut dyn FnMut(&DoomscrollingForegroundDesktopAppStatus) -> Result<(), String>,
) -> Result<(), String> {
    use ::windows::Win32::Foundation::{LPARAM, WPARAM};
    use ::windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

    let hwnd =
        windows_foreground_window().ok_or_else(|| "no foreground window is active".to_string())?;
    let status = windows_status_for_window(hwnd);
    validate_foreground_status_is_closeable(&status)?;
    if !foreground_expectation_matches(&status, &expected) {
        return Err("foreground app changed before it could be closed".to_string());
    }
    authorize(&status)?;
    unsafe {
        PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0))
            .map_err(|e| format!("close foreground window: {e}"))?;
    }
    Ok(())
}
