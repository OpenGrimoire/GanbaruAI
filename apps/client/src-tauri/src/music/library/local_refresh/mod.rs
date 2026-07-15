mod helpers;
mod metadata;
mod persistence;
mod traversal;

pub(in crate::music::library) use traversal::strong_fingerprint as strong_fingerprint_for_repair;
pub(super) use traversal::{
    inspect_media as inspect_for_relink, ArtworkCache as RelinkArtworkCache,
    LocalMediaEvidence as RelinkMediaEvidence,
};

use sqlx::SqlitePool;
use std::path::{Path, PathBuf};

use super::{
    MusicLibraryError, MusicLibraryResult, MusicLocalRefreshRequest, MusicRefreshJobProgress,
};

pub(crate) async fn prepare(
    pool: &SqlitePool,
    request: &MusicLocalRefreshRequest,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    persistence::prepare(pool, request).await
}

pub(crate) async fn progress(
    pool: &SqlitePool,
    job_id: &str,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    persistence::load_progress(pool, job_id).await
}

pub(crate) async fn cancel(
    pool: &SqlitePool,
    job_id: &str,
    cancelled_at: i64,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    persistence::cancel(pool, job_id, cancelled_at).await
}

pub(crate) async fn run_prepared(
    pool: &SqlitePool,
    request: MusicLocalRefreshRequest,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    persistence::mark_running(pool, &request.job_id).await?;
    let root = PathBuf::from(&request.folder_path);
    let progress = persistence::load_progress(pool, &request.job_id).await?;
    let result = run_current(pool, &request, &root, progress.generation).await;
    if let Err(error) = result {
        persistence::finish_incomplete(pool, &request, &error.message).await?;
    }
    persistence::load_progress(pool, &request.job_id).await
}

async fn run_current(
    pool: &SqlitePool,
    request: &MusicLocalRefreshRequest,
    root: &Path,
    generation: i64,
) -> MusicLibraryResult<()> {
    discover(pool, request, root).await?;
    if !persistence::is_current(pool, &request.job_id).await? {
        return Ok(());
    }
    catalog(pool, request, root, generation).await?;
    if !persistence::is_current(pool, &request.job_id).await? {
        return Ok(());
    }
    persistence::finalize_success(pool, request, generation).await
}

async fn discover(
    pool: &SqlitePool,
    request: &MusicLocalRefreshRequest,
    root: &Path,
) -> MusicLibraryResult<()> {
    while let Some(directory) = persistence::next_pending_directory(pool, &request.job_id).await? {
        if !persistence::is_current(pool, &request.job_id).await? {
            return Ok(());
        }
        let mut reader = traversal::open_directory(root, &directory)
            .map_err(|message| MusicLibraryError::runtime("discover local music", message))?;
        loop {
            let entries = traversal::next_discovery_batch(root, &mut reader)
                .map_err(|message| MusicLibraryError::runtime("discover local music", message))?;
            let complete = entries.len() < traversal::DISCOVERY_BATCH_SIZE;
            persistence::save_discovery_batch(
                pool,
                &request.job_id,
                &directory,
                &entries,
                complete,
            )
            .await?;
            if complete {
                break;
            }
            if !persistence::is_current(pool, &request.job_id).await? {
                return Ok(());
            }
        }
    }
    Ok(())
}

async fn catalog(
    pool: &SqlitePool,
    request: &MusicLocalRefreshRequest,
    root: &Path,
    generation: i64,
) -> MusicLibraryResult<()> {
    let mut artwork_cache = traversal::ArtworkCache::new();
    loop {
        if !persistence::is_current(pool, &request.job_id).await? {
            return Ok(());
        }
        let paths =
            persistence::next_media_batch(pool, &request.job_id, traversal::RECONCILE_BATCH_SIZE)
                .await?;
        if paths.is_empty() {
            return Ok(());
        }
        let mut evidence = Vec::with_capacity(paths.len());
        for relative_path in paths {
            match traversal::inspect_media(root, &relative_path, &mut artwork_cache) {
                Ok(media) => evidence.push(media),
                Err(message) => {
                    persistence::record_media_failure(pool, request, &relative_path, &message)
                        .await?;
                }
            }
        }
        if !evidence.is_empty() {
            persistence::reconcile_batch(pool, request, generation, &evidence).await?;
        }
    }
}

#[cfg(test)]
mod tests;
