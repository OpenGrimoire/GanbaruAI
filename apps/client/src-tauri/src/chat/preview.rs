//! Isolated child-webview preview state and bounded browser controls.

use super::models::{
    ChatError, ChatErrorCode, ChatResult, ChatThreadId, ProjectWorkingFolderId, UtcTimestamp,
};
use super::repository::resources::{
    self, ChatResourceKind, ChatResourceRead, StoreBrowserArtifact,
};
use crate::db_path;
use crate::vault;
use base64::{engine::general_purpose, Engine as _};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Manager, WebviewUrl};

const MAX_PREVIEW_URL_BYTES: usize = 8_192;
const MAX_PREVIEW_SCRIPT_BYTES: usize = 256 * 1024;
const MAX_PREVIEW_RESULT_BYTES: usize = 2 * 1024 * 1024;
const PREVIEW_EVALUATION_TIMEOUT: Duration = Duration::from_secs(15);
const PREVIEW_CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const RECORDING_FRAME_INTERVAL: Duration = Duration::from_millis(500);
const MAX_RECORDING_DURATION: Duration = Duration::from_secs(15);
const MAX_CAPTURE_BYTES: usize = 32 * 1024 * 1024;
const MAX_RECORDING_BYTES: usize = 192 * 1024 * 1024;
static NEXT_ARTIFACT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPreviewRequest {
    pub thread_id: ChatThreadId,
    pub tab_id: String,
    pub url: String,
    pub bounds: PreviewBounds,
    pub external_navigation_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTabRead {
    pub thread_id: ChatThreadId,
    pub tab_id: String,
    pub current_url: String,
    pub title: String,
    pub visible: bool,
    pub loading: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub external_origin: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredPreviewServer {
    pub url: String,
    pub source_label: String,
}

#[derive(Clone, Debug)]
struct RuntimePreviewTab {
    read: PreviewTabRead,
    webview_label: String,
    allowed_url: Arc<Mutex<reqwest::Url>>,
}

struct ActiveRecording {
    thread_id: ChatThreadId,
    tab_id: String,
    started: Instant,
    stop: tokio::sync::oneshot::Sender<()>,
    completed: tokio::sync::oneshot::Receiver<ChatResult<Vec<Vec<u8>>>>,
}

struct BrowserArtifactPayload<'a> {
    kind: ChatResourceKind,
    display_name: &'a str,
    mime_type: &'a str,
    duration_milliseconds: Option<u64>,
    frame_count: Option<u64>,
    bytes: &'a [u8],
}

#[derive(Clone, Default)]
pub struct ChatPreviewManager {
    tabs: Arc<Mutex<HashMap<String, RuntimePreviewTab>>>,
    recording: Arc<Mutex<Option<ActiveRecording>>>,
}

impl ChatPreviewManager {
    fn cancel_recording(&self, matches: impl FnOnce(&ActiveRecording) -> bool) {
        if let Ok(mut active) = self.recording.lock() {
            if active.as_ref().is_some_and(matches) {
                if let Some(recording) = active.take() {
                    let _ = recording.stop.send(());
                }
            }
        }
    }

    fn read(&self, tab_id: &str) -> ChatResult<RuntimePreviewTab> {
        self.tabs
            .lock()
            .map_err(|_| preview_state_error())?
            .get(tab_id)
            .cloned()
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "Browser preview tab was not found",
                    true,
                )
            })
    }

    fn update(&self, tab_id: &str, update: impl FnOnce(&mut PreviewTabRead)) {
        if let Ok(mut tabs) = self.tabs.lock() {
            if let Some(tab) = tabs.get_mut(tab_id) {
                update(&mut tab.read);
            }
        }
    }

    pub(crate) fn thread_reads(&self, thread_id: &ChatThreadId) -> ChatResult<Vec<PreviewTabRead>> {
        let mut reads = self
            .tabs
            .lock()
            .map_err(|_| preview_state_error())?
            .values()
            .filter(|tab| &tab.read.thread_id == thread_id)
            .map(|tab| tab.read.clone())
            .collect::<Vec<_>>();
        reads.sort_by(|left, right| left.tab_id.cmp(&right.tab_id));
        Ok(reads)
    }

    fn active_thread_tab(&self, thread_id: &ChatThreadId) -> ChatResult<RuntimePreviewTab> {
        let tabs = self.tabs.lock().map_err(|_| preview_state_error())?;
        tabs.values()
            .find(|tab| &tab.read.thread_id == thread_id && tab.read.visible)
            .or_else(|| tabs.values().find(|tab| &tab.read.thread_id == thread_id))
            .cloned()
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "Open the Browser panel before using preview tools",
                    true,
                )
            })
    }

    pub fn close_thread<R: tauri::Runtime>(
        &self,
        app: &tauri::AppHandle<R>,
        thread_id: &ChatThreadId,
    ) {
        self.cancel_recording(|recording| &recording.thread_id == thread_id);
        let labels = if let Ok(mut tabs) = self.tabs.lock() {
            let ids = tabs
                .iter()
                .filter_map(|(id, tab)| (&tab.read.thread_id == thread_id).then_some(id.clone()))
                .collect::<Vec<_>>();
            ids.into_iter()
                .filter_map(|id| tabs.remove(&id).map(|tab| tab.webview_label))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        for label in labels {
            if let Some(webview) = app.get_webview(&label) {
                let _ = webview.close();
            }
        }
    }

    pub fn close_all<R: tauri::Runtime>(&self, app: &tauri::AppHandle<R>) {
        self.cancel_recording(|_| true);
        let labels = if let Ok(mut tabs) = self.tabs.lock() {
            std::mem::take(&mut *tabs)
                .into_values()
                .map(|tab| tab.webview_label)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        for label in labels {
            if let Some(webview) = app.get_webview(&label) {
                let _ = webview.close();
            }
        }
    }
}

pub(crate) async fn mcp_navigate_preview(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    url: &str,
) -> ChatResult<PreviewTabRead> {
    let url = validate_navigation(url, false)?;
    let tab = app
        .state::<ChatPreviewManager>()
        .active_thread_tab(thread_id)?;
    navigate_existing(app, &tab, url.clone())?;
    app.state::<ChatPreviewManager>()
        .update(&tab.read.tab_id, |read| {
            read.current_url = url.to_string();
            read.loading = true;
            read.external_origin = false;
        });
    let read = app
        .state::<ChatPreviewManager>()
        .read(&tab.read.tab_id)?
        .read;
    persist_tab(pool, &read).await?;
    Ok(read)
}

pub(crate) async fn mcp_resize_preview(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    width: u32,
    height: u32,
) -> ChatResult<PreviewTabRead> {
    if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
        return Err(ChatError::validation(
            "viewport",
            "Browser viewport is invalid",
        ));
    }
    let tab = app
        .state::<ChatPreviewManager>()
        .active_thread_tab(thread_id)?;
    app.get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?
        .set_size(tauri::PhysicalSize::new(width, height))
        .map_err(|_| preview_unavailable())?;
    app.state::<ChatPreviewManager>()
        .update(&tab.read.tab_id, |read| {
            read.viewport_width = width;
            read.viewport_height = height;
        });
    let read = app
        .state::<ChatPreviewManager>()
        .read(&tab.read.tab_id)?
        .read;
    persist_tab(pool, &read).await?;
    Ok(read)
}

pub(crate) fn mcp_active_tab_id(
    app: &tauri::AppHandle,
    thread_id: &ChatThreadId,
) -> ChatResult<String> {
    Ok(app
        .state::<ChatPreviewManager>()
        .active_thread_tab(thread_id)?
        .read
        .tab_id)
}

pub(crate) async fn mcp_screenshot_preview(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    vault_root: &std::path::Path,
    thread_id: &ChatThreadId,
) -> ChatResult<ChatResourceRead> {
    let tab = app
        .state::<ChatPreviewManager>()
        .active_thread_tab(thread_id)?;
    let bytes = capture_preview_png(app, &tab).await?;
    persist_browser_artifact_with_pool(
        pool,
        vault_root,
        &tab,
        BrowserArtifactPayload {
            kind: ChatResourceKind::BrowserScreenshot,
            display_name: "Browser screenshot",
            mime_type: "image/png",
            duration_milliseconds: None,
            frame_count: Some(1),
            bytes: &bytes,
        },
    )
    .await
}

#[tauri::command]
pub async fn chat_preview_status(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
) -> ChatResult<Vec<PreviewTabRead>> {
    let mut reads = app
        .state::<ChatPreviewManager>()
        .tabs
        .lock()
        .map_err(|_| preview_state_error())?
        .values()
        .filter(|tab| tab.read.thread_id == thread_id)
        .map(|tab| tab.read.clone())
        .collect::<Vec<_>>();
    let pool = chat_pool(&app, db_url).await?;
    let stored = sqlx::query_as::<_, (String, String, String, Option<i64>, Option<i64>)>(
        "SELECT id, current_url, title, viewport_width, viewport_height
         FROM chat_preview_tabs WHERE thread_id = ? ORDER BY position, id",
    )
    .bind(thread_id.as_str())
    .fetch_all(&pool)
    .await
    .map_err(|_| persistence_error())?;
    for (tab_id, current_url, title, width, height) in stored {
        if reads.iter().any(|read| read.tab_id == tab_id) {
            continue;
        }
        let external_origin = reqwest::Url::parse(&current_url)
            .ok()
            .is_some_and(|url| !is_loopback(&url));
        reads.push(PreviewTabRead {
            thread_id: thread_id.clone(),
            tab_id,
            current_url,
            title,
            visible: false,
            loading: false,
            viewport_width: u32::try_from(width.unwrap_or(800)).unwrap_or(800),
            viewport_height: u32::try_from(height.unwrap_or(600)).unwrap_or(600),
            external_origin,
        });
    }
    reads.sort_by(|left, right| left.tab_id.cmp(&right.tab_id));
    Ok(reads)
}

#[tauri::command]
pub async fn chat_preview_discover_servers(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
) -> ChatResult<Vec<DiscoveredPreviewServer>> {
    let pool = chat_pool(&app, db_url).await?;
    let working_folder_id =
        sqlx::query_scalar::<_, String>("SELECT working_folder_id FROM chat_threads WHERE id = ?")
            .bind(thread_id.as_str())
            .fetch_optional(&pool)
            .await
            .map_err(|_| persistence_error())?
            .ok_or_else(|| {
                ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true)
            })?;
    let working_folder_id =
        ProjectWorkingFolderId::new(working_folder_id).map_err(|_| persistence_error())?;
    let registry = app.state::<super::terminal::ChatTerminalRegistry>();
    let terminals = registry.list(&thread_id, &working_folder_id)?;
    let mut candidates = HashMap::<String, String>::new();
    for terminal in terminals {
        let snapshot = registry.snapshot(&terminal.id)?;
        for chunk in snapshot.scrollback {
            let Ok(bytes) = general_purpose::STANDARD.decode(chunk.data_base64) else {
                continue;
            };
            for candidate in loopback_urls(&bytes) {
                candidates
                    .entry(candidate)
                    .or_insert_with(|| terminal.name.clone());
            }
        }
    }
    let mut discovered = Vec::new();
    for (url, source_label) in candidates.into_iter().take(64) {
        let Ok(parsed) = reqwest::Url::parse(&url) else {
            continue;
        };
        let Some(port) = parsed.port_or_known_default() else {
            continue;
        };
        let address = format!("127.0.0.1:{port}");
        if tokio::time::timeout(
            Duration::from_millis(250),
            tokio::net::TcpStream::connect(address),
        )
        .await
        .is_ok_and(|result| result.is_ok())
        {
            discovered.push(DiscoveredPreviewServer { url, source_label });
        }
    }
    discovered.sort_by(|left, right| left.url.cmp(&right.url));
    discovered.dedup_by(|left, right| left.url == right.url);
    Ok(discovered)
}

#[tauri::command]
pub async fn chat_preview_open(
    app: tauri::AppHandle,
    db_url: String,
    request: OpenPreviewRequest,
) -> ChatResult<PreviewTabRead> {
    let url = validate_navigation(&request.url, request.external_navigation_confirmed)?;
    let bounds = validate_bounds(&request.bounds)?;
    validate_tab_id(&request.tab_id)?;
    let pool = chat_pool(&app, db_url).await?;
    require_thread(&pool, &request.thread_id).await?;
    if let Ok(existing) = app.state::<ChatPreviewManager>().read(&request.tab_id) {
        if existing.read.thread_id != request.thread_id {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Browser preview tab belongs to another thread",
                true,
            ));
        }
        navigate_existing(&app, &existing, url.clone())?;
        resize_existing(&app, &existing, &bounds)?;
        if let Some(webview) = app.get_webview(&existing.webview_label) {
            webview.show().map_err(|_| preview_unavailable())?;
        }
        app.state::<ChatPreviewManager>()
            .update(&request.tab_id, |read| {
                read.current_url = url.to_string();
                read.visible = true;
                read.loading = true;
                read.viewport_width = bounds.width.round() as u32;
                read.viewport_height = bounds.height.round() as u32;
            });
        persist_tab(
            &pool,
            &app.state::<ChatPreviewManager>()
                .read(&request.tab_id)?
                .read,
        )
        .await?;
        return Ok(app
            .state::<ChatPreviewManager>()
            .read(&request.tab_id)?
            .read);
    }
    create_child_preview(&app, &pool, &request, url, bounds).await
}

#[tauri::command]
pub async fn chat_preview_navigate(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    tab_id: String,
    url: String,
    external_navigation_confirmed: bool,
) -> ChatResult<PreviewTabRead> {
    let url = validate_navigation(&url, external_navigation_confirmed)?;
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    navigate_existing(&app, &tab, url.clone())?;
    app.state::<ChatPreviewManager>().update(&tab_id, |read| {
        read.current_url = url.to_string();
        read.loading = true;
        read.external_origin = !is_loopback(&url);
    });
    let read = app.state::<ChatPreviewManager>().read(&tab_id)?.read;
    persist_tab(&chat_pool(&app, db_url).await?, &read).await?;
    Ok(read)
}

#[tauri::command]
pub async fn chat_preview_resize(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    tab_id: String,
    bounds: PreviewBounds,
) -> ChatResult<PreviewTabRead> {
    let bounds = validate_bounds(&bounds)?;
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    resize_existing(&app, &tab, &bounds)?;
    app.state::<ChatPreviewManager>().update(&tab_id, |read| {
        read.viewport_width = bounds.width.round() as u32;
        read.viewport_height = bounds.height.round() as u32;
    });
    let read = app.state::<ChatPreviewManager>().read(&tab_id)?.read;
    persist_tab(&chat_pool(&app, db_url).await?, &read).await?;
    Ok(read)
}

#[tauri::command]
pub async fn chat_preview_set_visible(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    tab_id: String,
    visible: bool,
) -> ChatResult<PreviewTabRead> {
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    let webview = app
        .get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?;
    if visible {
        webview.show()
    } else {
        webview.hide()
    }
    .map_err(|_| preview_unavailable())?;
    app.state::<ChatPreviewManager>()
        .update(&tab_id, |read| read.visible = visible);
    let read = app.state::<ChatPreviewManager>().read(&tab_id)?.read;
    persist_tab(&chat_pool(&app, db_url).await?, &read).await?;
    Ok(read)
}

#[tauri::command]
pub async fn chat_preview_back(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<()> {
    eval_fixed(&app, &thread_id, &tab_id, "history.back()")
}

#[tauri::command]
pub async fn chat_preview_forward(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<()> {
    eval_fixed(&app, &thread_id, &tab_id, "history.forward()")
}

#[tauri::command]
pub async fn chat_preview_refresh(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<()> {
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    app.get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?
        .reload()
        .map_err(|_| preview_unavailable())
}

#[tauri::command]
pub async fn chat_preview_snapshot(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<String> {
    evaluate_script(
        &app,
        &thread_id,
        &tab_id,
        "JSON.stringify({url:location.href,title:document.title,text:(document.body?.innerText??'').slice(0,1000000),html:(document.documentElement?.outerHTML??'').slice(0,1000000)})",
    )
    .await
}

#[tauri::command]
pub async fn chat_preview_screenshot(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<ChatResourceRead> {
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    let bytes = capture_preview_png(&app, &tab).await?;
    persist_browser_artifact(
        &app,
        db_url,
        &tab,
        BrowserArtifactPayload {
            kind: ChatResourceKind::BrowserScreenshot,
            display_name: "Browser screenshot",
            mime_type: "image/png",
            duration_milliseconds: None,
            frame_count: Some(1),
            bytes: &bytes,
        },
    )
    .await
}

#[tauri::command]
pub async fn chat_preview_recording_start(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
    approved: bool,
) -> ChatResult<()> {
    if !approved {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Browser recording requires approval",
            true,
        ));
    }
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    let manager = app.state::<ChatPreviewManager>();
    let mut active = manager
        .recording
        .lock()
        .map_err(|_| preview_state_error())?;
    if active.is_some() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Another browser recording is already active",
            true,
        ));
    }
    let (stop, stop_receiver) = tokio::sync::oneshot::channel();
    let (completed_sender, completed) = tokio::sync::oneshot::channel();
    let recording_app = app.clone();
    let recording_tab = tab.clone();
    tauri::async_runtime::spawn(async move {
        let result = capture_recording_frames(recording_app, recording_tab, stop_receiver).await;
        let _ = completed_sender.send(result);
    });
    *active = Some(ActiveRecording {
        thread_id,
        tab_id,
        started: Instant::now(),
        stop,
        completed,
    });
    Ok(())
}

#[tauri::command]
pub async fn chat_preview_recording_stop(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<ChatResourceRead> {
    let recording = app
        .state::<ChatPreviewManager>()
        .recording
        .lock()
        .map_err(|_| preview_state_error())?
        .take()
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::NotFound,
                "No browser recording is active",
                true,
            )
        })?;
    if recording.thread_id != thread_id || recording.tab_id != tab_id {
        app.state::<ChatPreviewManager>()
            .recording
            .lock()
            .map_err(|_| preview_state_error())?
            .replace(recording);
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Browser recording belongs to another preview tab",
            true,
        ));
    }
    let duration = recording.started.elapsed().min(MAX_RECORDING_DURATION);
    let _ = recording.stop.send(());
    let frames = tokio::time::timeout(PREVIEW_CAPTURE_TIMEOUT, recording.completed)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Timeout, "Browser recording timed out", true))?
        .map_err(|_| preview_unavailable())??;
    if frames.is_empty() {
        return Err(preview_unavailable());
    }
    let archive = recording_archive(&frames, duration)?;
    let tab = owned_tab(&app, &thread_id, &tab_id)?;
    persist_browser_artifact(
        &app,
        db_url,
        &tab,
        BrowserArtifactPayload {
            kind: ChatResourceKind::BrowserRecording,
            display_name: "Browser recording",
            mime_type: "application/zip",
            duration_milliseconds: Some(duration.as_millis() as u64),
            frame_count: Some(frames.len() as u64),
            bytes: &archive,
        },
    )
    .await
}

#[tauri::command]
pub async fn chat_preview_evaluate(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
    script: String,
    approved: bool,
) -> ChatResult<String> {
    if !approved {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Browser JavaScript evaluation requires approval",
            true,
        ));
    }
    if script.is_empty() || script.len() > MAX_PREVIEW_SCRIPT_BYTES || script.contains('\0') {
        return Err(ChatError::validation(
            "script",
            "Browser JavaScript is invalid",
        ));
    }
    evaluate_script(&app, &thread_id, &tab_id, &script).await
}

#[tauri::command]
pub async fn chat_preview_click(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
    selector: String,
) -> ChatResult<()> {
    let selector = js_string(&selector, "selector")?;
    eval_fixed(
        &app,
        &thread_id,
        &tab_id,
        &format!("document.querySelector({selector})?.click()"),
    )
}

#[tauri::command]
pub async fn chat_preview_type(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
    selector: String,
    text: String,
) -> ChatResult<()> {
    let selector = js_string(&selector, "selector")?;
    let text = js_string(&text, "text")?;
    eval_fixed(&app, &thread_id, &tab_id, &format!("(()=>{{const e=document.querySelector({selector});if(e){{e.focus();e.value={text};e.dispatchEvent(new Event('input',{{bubbles:true}}));}}}})()"))
}

#[tauri::command]
pub async fn chat_preview_press(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
    key: String,
) -> ChatResult<()> {
    let key = js_string(&key, "key")?;
    eval_fixed(&app, &thread_id, &tab_id, &format!("document.activeElement?.dispatchEvent(new KeyboardEvent('keydown',{{key:{key},bubbles:true}}))"))
}

#[tauri::command]
pub async fn chat_preview_scroll(
    app: tauri::AppHandle,
    thread_id: ChatThreadId,
    tab_id: String,
    x: f64,
    y: f64,
) -> ChatResult<()> {
    if !x.is_finite() || !y.is_finite() || x.abs() > 1_000_000.0 || y.abs() > 1_000_000.0 {
        return Err(ChatError::validation(
            "scroll",
            "Browser scroll distance is invalid",
        ));
    }
    eval_fixed(&app, &thread_id, &tab_id, &format!("scrollBy({x},{y})"))
}

#[tauri::command]
pub async fn chat_preview_close(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    tab_id: String,
) -> ChatResult<()> {
    let pool = chat_pool(&app, db_url).await?;
    require_thread(&pool, &thread_id).await?;
    if let Ok(tab) = app.state::<ChatPreviewManager>().read(&tab_id) {
        if tab.read.thread_id != thread_id {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Browser preview tab belongs to another thread",
                true,
            ));
        }
        app.state::<ChatPreviewManager>()
            .cancel_recording(|recording| recording.tab_id == tab_id);
        if let Some(webview) = app.get_webview(&tab.webview_label) {
            webview.close().map_err(|_| preview_unavailable())?;
        }
        app.state::<ChatPreviewManager>()
            .tabs
            .lock()
            .map_err(|_| preview_state_error())?
            .remove(&tab_id);
    }
    let deleted = sqlx::query("DELETE FROM chat_preview_tabs WHERE id = ? AND thread_id = ?")
        .bind(&tab_id)
        .bind(thread_id.as_str())
        .execute(&pool)
        .await
        .map_err(|_| persistence_error())?;
    if deleted.rows_affected() == 0 {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "Browser preview tab was not found",
            true,
        ));
    }
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn create_child_preview(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: &OpenPreviewRequest,
    url: reqwest::Url,
    bounds: PreviewBounds,
) -> ChatResult<PreviewTabRead> {
    use tauri::webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder};
    let label = preview_label(&request.tab_id);
    let allowed_url = Arc::new(Mutex::new(url.clone()));
    let navigation_url = allowed_url.clone();
    let manager = app.state::<ChatPreviewManager>().inner().clone();
    let loading_tab = request.tab_id.clone();
    let loading_pool = pool.clone();
    let title_manager = manager.clone();
    let title_tab = request.tab_id.clone();
    let title_pool = pool.clone();
    let builder = WebviewBuilder::new(label.clone(), WebviewUrl::External(url.clone()))
        .on_navigation(move |candidate| {
            navigation_url
                .lock()
                .is_ok_and(|allowed| navigation_allowed(&allowed, candidate))
        })
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .on_page_load(move |_, payload| {
            manager.update(&loading_tab, |read| {
                read.current_url = payload.url().to_string();
                read.loading = payload.event() == PageLoadEvent::Started;
                read.external_origin = !is_loopback(payload.url());
            });
            persist_runtime_tab(&manager, &loading_pool, &loading_tab);
        })
        .on_document_title_changed(move |_, title| {
            title_manager.update(&title_tab, |read| read.title = title);
            persist_runtime_tab(&title_manager, &title_pool, &title_tab);
        });
    let main = app.get_window("main").ok_or_else(preview_unavailable)?;
    main.add_child(
        builder,
        tauri::LogicalPosition::new(bounds.x, bounds.y),
        tauri::LogicalSize::new(bounds.width, bounds.height),
    )
    .map_err(|_| preview_unavailable())?;
    let read = PreviewTabRead {
        thread_id: request.thread_id.clone(),
        tab_id: request.tab_id.clone(),
        current_url: url.to_string(),
        title: String::new(),
        visible: true,
        loading: true,
        viewport_width: bounds.width.round() as u32,
        viewport_height: bounds.height.round() as u32,
        external_origin: !is_loopback(&url),
    };
    app.state::<ChatPreviewManager>()
        .tabs
        .lock()
        .map_err(|_| preview_state_error())?
        .insert(
            request.tab_id.clone(),
            RuntimePreviewTab {
                read: read.clone(),
                webview_label: label,
                allowed_url,
            },
        );
    persist_tab(pool, &read).await?;
    Ok(read)
}

fn persist_runtime_tab(manager: &ChatPreviewManager, pool: &SqlitePool, tab_id: &str) {
    let Ok(tab) = manager.read(tab_id) else {
        return;
    };
    let pool = pool.clone();
    tauri::async_runtime::spawn(async move {
        let _ = persist_tab(&pool, &tab.read).await;
    });
}

#[cfg(any(target_os = "android", target_os = "ios"))]
async fn create_child_preview(
    _app: &tauri::AppHandle,
    _pool: &SqlitePool,
    _request: &OpenPreviewRequest,
    _url: reqwest::Url,
    _bounds: PreviewBounds,
) -> ChatResult<PreviewTabRead> {
    Err(ChatError::unsupported(
        "Embedded browser preview is unavailable on this platform",
    ))
}

fn owned_tab(
    app: &tauri::AppHandle,
    thread_id: &ChatThreadId,
    tab_id: &str,
) -> ChatResult<RuntimePreviewTab> {
    let tab = app.state::<ChatPreviewManager>().read(tab_id)?;
    if &tab.read.thread_id != thread_id {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Browser preview tab belongs to another thread",
            true,
        ));
    }
    Ok(tab)
}

fn navigate_existing(
    app: &tauri::AppHandle,
    tab: &RuntimePreviewTab,
    url: reqwest::Url,
) -> ChatResult<()> {
    let webview = app
        .get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?;
    let previous = {
        let mut allowed = tab.allowed_url.lock().map_err(|_| preview_state_error())?;
        let previous = allowed.clone();
        *allowed = url.clone();
        previous
    };
    let result = webview.navigate(url).map_err(|_| preview_unavailable());
    if result.is_err() {
        *tab.allowed_url.lock().map_err(|_| preview_state_error())? = previous;
    }
    result
}

fn resize_existing(
    app: &tauri::AppHandle,
    tab: &RuntimePreviewTab,
    bounds: &PreviewBounds,
) -> ChatResult<()> {
    let webview = app
        .get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?;
    webview
        .set_position(tauri::LogicalPosition::new(bounds.x, bounds.y))
        .and_then(|_| webview.set_size(tauri::LogicalSize::new(bounds.width, bounds.height)))
        .map_err(|_| preview_unavailable())
}

async fn capture_recording_frames(
    app: tauri::AppHandle,
    tab: RuntimePreviewTab,
    mut stop: tokio::sync::oneshot::Receiver<()>,
) -> ChatResult<Vec<Vec<u8>>> {
    let started = Instant::now();
    let mut frames = Vec::new();
    let mut total_bytes = 0_usize;
    loop {
        let frame = capture_preview_png(&app, &tab).await?;
        total_bytes = total_bytes.saturating_add(frame.len());
        if total_bytes > MAX_RECORDING_BYTES {
            return Err(ChatError::new(
                ChatErrorCode::Protocol,
                "Browser recording exceeds the supported limit",
                true,
            ));
        }
        frames.push(frame);
        if started.elapsed() >= MAX_RECORDING_DURATION {
            break;
        }
        tokio::select! {
            _ = &mut stop => break,
            _ = tokio::time::sleep(RECORDING_FRAME_INTERVAL) => {}
        }
    }
    Ok(frames)
}

fn recording_archive(frames: &[Vec<u8>], duration: Duration) -> ChatResult<Vec<u8>> {
    use std::io::{Cursor, Write};
    use zip::write::{SimpleFileOptions, ZipWriter};
    use zip::CompressionMethod;

    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let manifest = serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "mimeType": "image/png",
        "frameIntervalMilliseconds": RECORDING_FRAME_INTERVAL.as_millis(),
        "durationMilliseconds": duration.as_millis(),
        "frameCount": frames.len()
    }))
    .map_err(|_| preview_unavailable())?;
    writer
        .start_file("manifest.json", options)
        .and_then(|_| writer.write_all(&manifest).map_err(Into::into))
        .map_err(|_| preview_unavailable())?;
    for (index, frame) in frames.iter().enumerate() {
        writer
            .start_file(format!("frames/{index:04}.png"), options)
            .and_then(|_| writer.write_all(frame).map_err(Into::into))
            .map_err(|_| preview_unavailable())?;
    }
    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|_| preview_unavailable())
}

async fn persist_browser_artifact(
    app: &tauri::AppHandle,
    db_url: String,
    tab: &RuntimePreviewTab,
    payload: BrowserArtifactPayload<'_>,
) -> ChatResult<ChatResourceRead> {
    let pool = chat_pool(app, db_url).await?;
    let vault_root = vault::active_vault_path(app).map_err(|_| persistence_error())?;
    persist_browser_artifact_with_pool(&pool, &vault_root, tab, payload).await
}

async fn persist_browser_artifact_with_pool(
    pool: &SqlitePool,
    vault_root: &std::path::Path,
    tab: &RuntimePreviewTab,
    payload: BrowserArtifactPayload<'_>,
) -> ChatResult<ChatResourceRead> {
    let working_folder_id =
        sqlx::query_scalar::<_, String>("SELECT working_folder_id FROM chat_threads WHERE id = ?")
            .bind(tab.read.thread_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(|_| persistence_error())?
            .ok_or_else(|| {
                ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true)
            })?;
    let working_folder_id =
        ProjectWorkingFolderId::new(working_folder_id).map_err(|_| persistence_error())?;
    let created_at = UtcTimestamp::new(Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| persistence_error())?;
    let resource_id = new_artifact_id(&tab.read.thread_id, &tab.read.tab_id);
    resources::store_browser_artifact(
        pool,
        vault_root,
        StoreBrowserArtifact {
            resource_id: &resource_id,
            thread_id: &tab.read.thread_id,
            working_folder_id: &working_folder_id,
            preview_tab_id: &tab.read.tab_id,
            kind: payload.kind,
            display_name: payload.display_name,
            mime_type: payload.mime_type,
            source_url: &tab.read.current_url,
            viewport_width: tab.read.viewport_width,
            viewport_height: tab.read.viewport_height,
            duration_milliseconds: payload.duration_milliseconds,
            frame_count: payload.frame_count,
            created_at: &created_at,
            bytes: payload.bytes,
        },
    )
    .await
}

fn new_artifact_id(thread_id: &ChatThreadId, tab_id: &str) -> String {
    let generation = NEXT_ARTIFACT_ID.fetch_add(1, Ordering::Relaxed);
    let seed = format!(
        "{}:{tab_id}:{}:{generation}:{}",
        thread_id.as_str(),
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    format!("browser-{:x}", Sha256::digest(seed.as_bytes()))
}

async fn capture_preview_png(
    app: &tauri::AppHandle,
    tab: &RuntimePreviewTab,
) -> ChatResult<Vec<u8>> {
    let webview = app
        .get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?;
    let (sender, receiver) = tokio::sync::oneshot::channel();
    capture_platform_png(&webview, sender)?;
    let bytes = tokio::time::timeout(PREVIEW_CAPTURE_TIMEOUT, receiver)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Timeout, "Browser capture timed out", true))?
        .map_err(|_| preview_unavailable())??;
    if bytes.is_empty() || bytes.len() > MAX_CAPTURE_BYTES {
        return Err(ChatError::new(
            ChatErrorCode::Protocol,
            "Browser screenshot exceeds the supported limit",
            true,
        ));
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn capture_platform_png(
    webview: &tauri::Webview,
    sender: tokio::sync::oneshot::Sender<ChatResult<Vec<u8>>>,
) -> ChatResult<()> {
    use webkit2gtk::{SnapshotOptions, SnapshotRegion, WebViewExt};
    let sender = Arc::new(Mutex::new(Some(sender)));
    webview
        .with_webview(move |platform| {
            let sender = Arc::clone(&sender);
            platform.inner().snapshot(
                SnapshotRegion::Visible,
                SnapshotOptions::NONE,
                None::<&webkit2gtk::gio::Cancellable>,
                move |result| {
                    let encoded = result
                        .map_err(|_| preview_unavailable())
                        .and_then(|surface| {
                            let mut bytes = Vec::new();
                            surface
                                .write_to_png(&mut bytes)
                                .map_err(|_| preview_unavailable())?;
                            Ok(bytes)
                        });
                    if let Ok(mut sender) = sender.lock() {
                        if let Some(sender) = sender.take() {
                            let _ = sender.send(encoded);
                        }
                    }
                },
            );
        })
        .map_err(|_| preview_unavailable())
}

#[cfg(windows)]
fn capture_platform_png(
    webview: &tauri::Webview,
    sender: tokio::sync::oneshot::Sender<ChatResult<Vec<u8>>>,
) -> ChatResult<()> {
    use webview2_com::{CapturePreviewCompletedHandler, Microsoft::Web::WebView2::Win32::*};
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::Com::IStream;
    use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;

    webview
        .with_webview(move |platform| unsafe {
            let stream = match CreateStreamOnHGlobal(HGLOBAL::default(), true) {
                Ok(stream) => stream,
                Err(_) => {
                    let _ = sender.send(Err(preview_unavailable()));
                    return;
                }
            };
            let callback_stream: IStream = stream.clone();
            let handler = CapturePreviewCompletedHandler::create(Box::new(move |result| {
                let captured = result
                    .map_err(|_| preview_unavailable())
                    .and_then(|_| read_windows_stream(&callback_stream));
                let _ = sender.send(captured);
                Ok(())
            }));
            let core: ICoreWebView2 = match platform.controller().CoreWebView2() {
                Ok(core) => core,
                Err(_) => {
                    return;
                }
            };
            let _ = core.CapturePreview(
                COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_PNG,
                &stream,
                &handler,
            );
        })
        .map_err(|_| preview_unavailable())
}

#[cfg(windows)]
fn read_windows_stream(stream: &windows::Win32::System::Com::IStream) -> ChatResult<Vec<u8>> {
    use windows::Win32::System::Com::{STATFLAG_NONAME, STREAM_SEEK_SET};
    unsafe {
        let stat = stream
            .Stat(STATFLAG_NONAME)
            .map_err(|_| preview_unavailable())?;
        let length = usize::try_from(stat.cbSize).map_err(|_| preview_unavailable())?;
        if length == 0 || length > MAX_CAPTURE_BYTES {
            return Err(preview_unavailable());
        }
        stream
            .Seek(0, STREAM_SEEK_SET, None)
            .map_err(|_| preview_unavailable())?;
        let mut bytes = vec![0_u8; length];
        let mut read = 0_u32;
        stream
            .Read(bytes.as_mut_ptr().cast(), length as u32, Some(&mut read))
            .map_err(|_| preview_unavailable())?;
        bytes.truncate(read as usize);
        Ok(bytes)
    }
}

#[cfg(target_os = "macos")]
fn capture_platform_png(
    webview: &tauri::Webview,
    sender: tokio::sync::oneshot::Sender<ChatResult<Vec<u8>>>,
) -> ChatResult<()> {
    use block2::RcBlock;
    use objc2_app_kit::{NSBitmapImageRep, NSImage, NSPNGFileType};
    use objc2_foundation::{NSDictionary, NSError};
    use objc2_web_kit::WKWebView;

    let sender = Arc::new(Mutex::new(Some(sender)));
    webview
        .with_webview(move |platform| unsafe {
            let sender = Arc::clone(&sender);
            let view: &WKWebView = &*platform.inner().cast();
            let block = RcBlock::new(move |image: *mut NSImage, error: *mut NSError| {
                let result = if !error.is_null() || image.is_null() {
                    Err(preview_unavailable())
                } else {
                    (*image)
                        .TIFFRepresentation()
                        .and_then(|data| NSBitmapImageRep::imageRepWithData(&data))
                        .and_then(|representation| {
                            let properties = NSDictionary::new();
                            representation
                                .representationUsingType_properties(NSPNGFileType, &properties)
                        })
                        .map(|data| data.to_vec())
                        .ok_or_else(preview_unavailable)
                };
                if let Ok(mut sender) = sender.lock() {
                    if let Some(sender) = sender.take() {
                        let _ = sender.send(result);
                    }
                }
            });
            view.takeSnapshotWithConfiguration_completionHandler(None, &block);
        })
        .map_err(|_| preview_unavailable())
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn capture_platform_png(
    _webview: &tauri::Webview,
    sender: tokio::sync::oneshot::Sender<ChatResult<Vec<u8>>>,
) -> ChatResult<()> {
    let _ = sender.send(Err(ChatError::unsupported(
        "Browser capture is unavailable on this platform",
    )));
    Ok(())
}

fn eval_fixed(
    app: &tauri::AppHandle,
    thread_id: &ChatThreadId,
    tab_id: &str,
    script: &str,
) -> ChatResult<()> {
    let tab = owned_tab(app, thread_id, tab_id)?;
    app.get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?
        .eval(script)
        .map_err(|_| preview_unavailable())
}

pub(crate) async fn evaluate_script(
    app: &tauri::AppHandle,
    thread_id: &ChatThreadId,
    tab_id: &str,
    script: &str,
) -> ChatResult<String> {
    let tab = owned_tab(app, thread_id, tab_id)?;
    let webview = app
        .get_webview(&tab.webview_label)
        .ok_or_else(preview_unavailable)?;
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));
    webview
        .eval_with_callback(script, move |result| {
            if let Ok(mut sender) = sender.lock() {
                if let Some(sender) = sender.take() {
                    let _ = sender.send(result);
                }
            }
        })
        .map_err(|_| preview_unavailable())?;
    let result = tokio::time::timeout(PREVIEW_EVALUATION_TIMEOUT, receiver)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Timeout, "Browser evaluation timed out", true))?
        .map_err(|_| preview_unavailable())?;
    if result.len() > MAX_PREVIEW_RESULT_BYTES {
        return Err(ChatError::new(
            ChatErrorCode::Protocol,
            "Browser result exceeds the supported limit",
            true,
        ));
    }
    Ok(result)
}

fn validate_navigation(value: &str, external_confirmed: bool) -> ChatResult<reqwest::Url> {
    if value.is_empty()
        || value.len() > MAX_PREVIEW_URL_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "url",
            "Browser preview URL is invalid",
        ));
    }
    let url = reqwest::Url::parse(value)
        .map_err(|_| ChatError::validation("url", "Browser preview URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(ChatError::validation(
            "url",
            "Browser preview URL is not allowed",
        ));
    }
    if !is_loopback(&url) && !external_confirmed {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "External browser navigation requires explicit confirmation",
            true,
        ));
    }
    Ok(url)
}

fn is_loopback(url: &reqwest::Url) -> bool {
    url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    })
}

fn navigation_allowed(allowed: &reqwest::Url, candidate: &reqwest::Url) -> bool {
    if !matches!(candidate.scheme(), "http" | "https") {
        return false;
    }
    if is_loopback(allowed) {
        return is_loopback(candidate);
    }
    allowed.scheme() == candidate.scheme()
        && allowed.host_str() == candidate.host_str()
        && allowed.port_or_known_default() == candidate.port_or_known_default()
}

fn loopback_urls(bytes: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(bytes);
    let mut urls = Vec::new();
    for prefix in ["http://", "https://"] {
        let mut remaining = text.as_ref();
        while let Some(index) = remaining.find(prefix) {
            let candidate = &remaining[index..];
            let end = candidate
                .find(|character: char| {
                    character.is_whitespace()
                        || matches!(character, '"' | '\'' | '<' | '>' | ')' | ']' | '}')
                })
                .unwrap_or(candidate.len());
            let value = candidate[..end].trim_end_matches(['.', ',', ';', ':']);
            if let Ok(parsed) = reqwest::Url::parse(value) {
                if is_loopback(&parsed) && parsed.port_or_known_default().is_some() {
                    let origin = format!(
                        "{}://{}:{}",
                        parsed.scheme(),
                        parsed.host_str().unwrap_or("localhost"),
                        parsed.port_or_known_default().unwrap_or_default()
                    );
                    urls.push(origin);
                }
            }
            remaining = &candidate[end.min(candidate.len())..];
            if end == 0 {
                break;
            }
        }
    }
    urls.sort();
    urls.dedup();
    urls
}

fn validate_bounds(bounds: &PreviewBounds) -> ChatResult<PreviewBounds> {
    if !bounds.x.is_finite()
        || !bounds.y.is_finite()
        || !bounds.width.is_finite()
        || !bounds.height.is_finite()
        || bounds.x < 0.0
        || bounds.y < 0.0
        || !(1.0..=16_384.0).contains(&bounds.width)
        || !(1.0..=16_384.0).contains(&bounds.height)
    {
        return Err(ChatError::validation(
            "bounds",
            "Browser preview bounds are invalid",
        ));
    }
    Ok(bounds.clone())
}

fn validate_tab_id(tab_id: &str) -> ChatResult<()> {
    if tab_id.is_empty() || tab_id.len() > 1_024 || tab_id.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "tabId",
            "Browser preview tab ID is invalid",
        ));
    }
    Ok(())
}

fn js_string(value: &str, field: &str) -> ChatResult<String> {
    if value.len() > 256 * 1024 || value.contains('\0') {
        return Err(ChatError::validation(
            field,
            "Browser interaction value is invalid",
        ));
    }
    serde_json::to_string(value)
        .map_err(|_| ChatError::validation(field, "Browser interaction value is invalid"))
}

fn preview_label(tab_id: &str) -> String {
    format!("chat-preview-{:x}", Sha256::digest(tab_id.as_bytes()))
}

async fn persist_tab(pool: &SqlitePool, read: &PreviewTabRead) -> ChatResult<()> {
    sqlx::query(
        "INSERT INTO chat_preview_tabs
            (id, thread_id, position, current_url, title, viewport_kind, viewport_width,
             viewport_height, visible, loading_state, created_at, updated_at)
         VALUES (?, ?, (SELECT COALESCE(MAX(position) + 1, 0) FROM chat_preview_tabs WHERE thread_id = ?),
             ?, ?, 'freeform', ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(id) DO UPDATE SET current_url = excluded.current_url, title = excluded.title,
             viewport_width = excluded.viewport_width, viewport_height = excluded.viewport_height,
             visible = excluded.visible, loading_state = excluded.loading_state,
             updated_at = excluded.updated_at",
    )
    .bind(&read.tab_id)
    .bind(read.thread_id.as_str())
    .bind(read.thread_id.as_str())
    .bind(&read.current_url)
    .bind(&read.title)
    .bind(i64::from(read.viewport_width))
    .bind(i64::from(read.viewport_height))
    .bind(read.visible)
    .bind(if read.loading { "loading" } else { "loaded" })
    .execute(pool)
    .await
    .map_err(|_| persistence_error())?;
    Ok(())
}

async fn require_thread(pool: &SqlitePool, thread_id: &ChatThreadId) -> ChatResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chat_threads WHERE id = ?)")
        .bind(thread_id.as_str())
        .fetch_one(pool)
        .await
        .map_err(|_| persistence_error())?;
    if exists {
        Ok(())
    } else {
        Err(ChatError::new(
            ChatErrorCode::NotFound,
            "Chat thread was not found",
            true,
        ))
    }
}

async fn chat_pool(app: &tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app.clone(), db_url)
        .await
        .map_err(|_| persistence_error())
}

fn preview_unavailable() -> ChatError {
    ChatError::new(
        ChatErrorCode::DriverUnavailable,
        "Browser preview is unavailable",
        true,
    )
}

fn preview_state_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Browser preview state is unavailable",
        true,
    )
}

fn persistence_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Browser preview state could not be persisted",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        loopback_urls, navigation_allowed, validate_bounds, validate_navigation, PreviewBounds,
    };

    #[test]
    fn navigation_requires_external_confirmation_and_stays_within_origin() {
        assert!(validate_navigation("http://127.0.0.1:5173", false).is_ok());
        assert!(validate_navigation("https://example.com", false).is_err());
        let external = validate_navigation("https://example.com/path", true).unwrap();
        assert!(navigation_allowed(
            &external,
            &"https://example.com/next".parse().unwrap()
        ));
        assert!(!navigation_allowed(
            &external,
            &"https://other.example/".parse().unwrap()
        ));
        assert!(!navigation_allowed(
            &external,
            &"file:///tmp/secret".parse().unwrap()
        ));
    }

    #[test]
    fn preview_bounds_are_finite_and_bounded() {
        assert!(validate_bounds(&PreviewBounds {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0
        })
        .is_ok());
        assert!(validate_bounds(&PreviewBounds {
            x: -1.0,
            y: 0.0,
            width: 320.0,
            height: 240.0
        })
        .is_err());
    }

    #[test]
    fn preview_server_candidates_accept_only_loopback_urls() {
        assert_eq!(
            loopback_urls(
                b"ready at http://localhost:5173/path and http://127.0.0.1:3000, remote https://example.com"
            ),
            vec![
                "http://127.0.0.1:3000".to_string(),
                "http://localhost:5173".to_string()
            ]
        );
    }
}
