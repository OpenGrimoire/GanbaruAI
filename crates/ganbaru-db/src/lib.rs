use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../apps/client/src-tauri/migrations");
const WRITE_CONTENTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Shared registry for SQLite pools keyed by their authorized filesystem path.
#[derive(Clone, Default)]
pub struct DatabasePoolRegistry {
    pools: Arc<Mutex<HashMap<PathBuf, SqlitePool>>>,
}

impl DatabasePoolRegistry {
    /// Connects to an authorized SQLite path or returns its existing pool.
    pub async fn connect_path(&self, path: impl AsRef<Path>) -> Result<SqlitePool, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(pool) = self
            .pools
            .lock()
            .map_err(|_| "database pool lock poisoned".to_string())?
            .get(&path)
            .cloned()
        {
            return Ok(pool);
        }

        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(WRITE_CONTENTION_TIMEOUT);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|error| format!("connect: {error}"))?;
        run_migrations(&pool).await?;
        sqlx::raw_sql("PRAGMA optimize")
            .execute(&pool)
            .await
            .map_err(|error| format!("pragma optimize: {error}"))?;

        let existing = {
            let mut pools = self
                .pools
                .lock()
                .map_err(|_| "database pool lock poisoned".to_string())?;
            if let Some(existing) = pools.get(&path).cloned() {
                Some(existing)
            } else {
                pools.insert(path, pool.clone());
                None
            }
        };
        if let Some(existing) = existing {
            pool.close().await;
            return Ok(existing);
        }
        Ok(pool)
    }

    /// Closes and removes the pool registered for a filesystem path.
    pub async fn close_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let pool = self
            .pools
            .lock()
            .map_err(|_| "database pool lock poisoned".to_string())?
            .remove(path.as_ref());
        if let Some(pool) = pool {
            pool.close().await;
        }
        Ok(())
    }

    /// Closes every pool currently held by the registry.
    pub async fn close_all(&self) -> Result<(), String> {
        let pools = std::mem::take(
            &mut *self
                .pools
                .lock()
                .map_err(|_| "database pool lock poisoned".to_string())?,
        );
        for pool in pools.into_values() {
            pool.close().await;
        }
        Ok(())
    }
}

/// Applies the embedded Ganbaru AI migration chain to a SQLite pool.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), String> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|error| format!("run database migrations: {error}"))
}

#[doc(hidden)]
pub mod __private {
    pub use sqlx;
}

/// Implements `sqlx::FromRow` by reading fields with their Rust names.
#[macro_export]
macro_rules! impl_sqlite_from_row {
    ($type:ty { $($field:ident),+ $(,)? }) => {
        impl<'r> $crate::__private::sqlx::FromRow<'r, $crate::__private::sqlx::sqlite::SqliteRow>
            for $type
        {
            fn from_row(
                row: &'r $crate::__private::sqlx::sqlite::SqliteRow,
            ) -> Result<Self, $crate::__private::sqlx::Error> {
                use $crate::__private::sqlx::Row as _;
                Ok(Self {
                    $($field: row.try_get(stringify!($field))?,)+
                })
            }
        }
    };
}

#[cfg(test)]
mod tests;
