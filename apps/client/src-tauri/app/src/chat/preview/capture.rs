use super::*;

pub(super) async fn capture_recording_frames(
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

pub(super) fn recording_archive(frames: &[Vec<u8>], duration: Duration) -> ChatResult<Vec<u8>> {
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

pub(super) async fn capture_preview_png(
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
pub(super) fn capture_platform_png(
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
pub(super) fn capture_platform_png(
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
pub(super) fn read_windows_stream(
    stream: &windows::Win32::System::Com::IStream,
) -> ChatResult<Vec<u8>> {
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
pub(super) fn capture_platform_png(
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
pub(super) fn capture_platform_png(
    _webview: &tauri::Webview,
    sender: tokio::sync::oneshot::Sender<ChatResult<Vec<u8>>>,
) -> ChatResult<()> {
    let _ = sender.send(Err(ChatError::unsupported(
        "Browser capture is unavailable on this platform",
    )));
    Ok(())
}
