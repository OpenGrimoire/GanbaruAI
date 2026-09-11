use crate::vault;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use tauri::{AppHandle, Manager, Runtime};

pub use ganbaru_db::DatabasePoolRegistry as DatabaseState;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub const BENCHMARK_SQLITE_URL: &str = "sqlite:benchmark.sqlite";

const ALLOWED_SQLITE_FILES: &[&str] = &["ganbaru-ai.sqlite", "benchmark.sqlite"];
static VAULT_CONNECTION_GATE: LazyLock<Arc<tokio::sync::RwLock<()>>> =
    LazyLock::new(|| Arc::new(tokio::sync::RwLock::new(())));

#[cfg(target_os = "android")]
pub(crate) type VaultRestoreGuard = tokio::sync::OwnedRwLockWriteGuard<()>;

/// Prevent new SQLite connections while an active vault is being replaced.
#[cfg(target_os = "android")]
pub(crate) async fn begin_vault_restore() -> VaultRestoreGuard {
    VAULT_CONNECTION_GATE.clone().write_owned().await
}

fn resolve_sqlite_path<R: Runtime>(app: &AppHandle<R>, db_url: &str) -> Result<PathBuf, String> {
    let file_name = db_url
        .strip_prefix("sqlite:")
        .ok_or_else(|| format!("invalid db url '{db_url}', expected 'sqlite:<file>'"))?;

    if !ALLOWED_SQLITE_FILES.contains(&file_name) {
        return Err(format!("unsupported sqlite file '{file_name}'"));
    }

    let mut path = if file_name == vault::APP_SQLITE_FILE {
        vault::active_vault_path(app)?
    } else {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&path)
            .map_err(|error| format!("create app config dir: {error}"))?;
        path
    };
    path.push(file_name);
    Ok(path)
}

pub async fn connect_sqlite<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<sqlx::SqlitePool, String> {
    let _connection_guard = VAULT_CONNECTION_GATE.read().await;
    let path = resolve_sqlite_path(&app, &db_url)?;
    let registry = app.state::<DatabaseState>().inner().clone();
    drop(app);
    drop(db_url);
    registry.connect_path(path).await
}

#[cfg(target_os = "android")]
pub(crate) async fn close_all_sqlite_pools_for_restore<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), String> {
    app.state::<DatabaseState>().close_all().await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn close_sqlite_pool<R: Runtime>(app: &AppHandle<R>, db_url: &str) -> Result<(), String> {
    let path = resolve_sqlite_path(app, db_url)?;
    app.state::<DatabaseState>().close_path(path).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn close_all_sqlite_pools<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.state::<DatabaseState>().close_all().await
}

#[cfg(test)]
mod tests {
    use super::ALLOWED_SQLITE_FILES;

    #[test]
    fn allowed_sqlite_files_are_plain_file_names() {
        for file_name in ALLOWED_SQLITE_FILES {
            assert!(!file_name.contains('/'));
            assert!(!file_name.contains('\\'));
            assert!(!file_name.contains(".."));
        }
    }
}
