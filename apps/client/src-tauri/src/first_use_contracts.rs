use serde::Serialize;
use sqlx::{SqliteConnection, SqlitePool};
use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use std::sync::{Arc, Mutex};

use crate::{db::run_migrations, notes, projects};

const SQLITE_OK: c_int = 0;
const SQLITE_TRACE_STMT: c_uint = 0x01;
const EMPTY_PROJECTS_SQL_READS: usize = 36;
const EMPTY_PROJECTS_SQL_WRITES: usize = 52;
const EMPTY_PROJECTS_RESPONSE_BYTES: usize = 23_564;
const EMPTY_NOTES_SQL_READS: usize = 8;
const EMPTY_NOTES_SQL_WRITES: usize = 1;
const EMPTY_NOTES_RESPONSE_BYTES: usize = 368;

unsafe extern "C" {
    fn sqlite3_trace_v2(
        database: *mut c_void,
        mask: c_uint,
        callback: Option<
            unsafe extern "C" fn(c_uint, *mut c_void, *mut c_void, *mut c_void) -> c_int,
        >,
        context: *mut c_void,
    ) -> c_int;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FirstUseIpcCommand {
    ProjectsLoadSnapshot,
    NotesListSidebarPages,
    NotesListPages,
    NotesListFolders,
    NotesListPageTemplates,
    NotesGetLocalUser,
    NotesGetPageHistorySettings,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct SqlStatementCounts {
    reads: usize,
    writes: usize,
}

#[derive(Debug, Default, Eq, PartialEq)]
struct FirstUseContractMetrics {
    commands: Vec<FirstUseIpcCommand>,
    sql: SqlStatementCounts,
    serialized_response_bytes: usize,
}

impl FirstUseContractMetrics {
    fn record_response<T: Serialize>(&mut self, command: FirstUseIpcCommand, response: &T) {
        self.commands.push(command);
        self.serialized_response_bytes += serde_json::to_vec(response)
            .expect("first-use response should serialize")
            .len();
    }
}

struct SqlTrace {
    pool: SqlitePool,
    counts: Arc<Mutex<SqlStatementCounts>>,
    context: *const Mutex<SqlStatementCounts>,
}

impl SqlTrace {
    async fn start(pool: &SqlitePool) -> Self {
        let counts = Arc::new(Mutex::new(SqlStatementCounts::default()));
        let context = Arc::into_raw(Arc::clone(&counts));
        let mut connection = pool.acquire().await.expect("acquire trace connection");
        install_trace(
            &mut connection,
            Some(sql_trace_callback),
            context.cast_mut().cast(),
        )
        .await;
        drop(connection);
        Self {
            pool: pool.clone(),
            counts,
            context,
        }
    }

    async fn finish(self) -> SqlStatementCounts {
        let mut connection = self
            .pool
            .acquire()
            .await
            .expect("acquire trace connection for cleanup");
        install_trace(&mut connection, None, std::ptr::null_mut()).await;
        drop(connection);

        // SAFETY: `context` came from `Arc::into_raw` in `start`. The callback has
        // been removed from SQLite, so balancing that strong reference is safe.
        unsafe {
            drop(Arc::from_raw(self.context));
        }
        Arc::try_unwrap(self.counts)
            .expect("trace counter should have one owner")
            .into_inner()
            .expect("trace counter lock should not be poisoned")
    }
}

async fn install_trace(
    connection: &mut SqliteConnection,
    callback: Option<unsafe extern "C" fn(c_uint, *mut c_void, *mut c_void, *mut c_void) -> c_int>,
    context: *mut c_void,
) {
    let mut handle = connection.lock_handle().await.expect("lock SQLite handle");
    // SAFETY: SQLx has locked the connection's native handle. SQLite retains only
    // the callback and context pointer, whose lifetime is managed by `SqlTrace`.
    let result = unsafe {
        sqlite3_trace_v2(
            handle.as_raw_handle().as_ptr().cast(),
            if callback.is_some() {
                SQLITE_TRACE_STMT
            } else {
                0
            },
            callback,
            context,
        )
    };
    assert_eq!(result, SQLITE_OK, "install SQLite statement trace");
}

unsafe extern "C" fn sql_trace_callback(
    event: c_uint,
    context: *mut c_void,
    _statement: *mut c_void,
    sql: *mut c_void,
) -> c_int {
    if event != SQLITE_TRACE_STMT || context.is_null() || sql.is_null() {
        return SQLITE_OK;
    }
    // SAFETY: SQLite supplies a null-terminated SQL string for SQLITE_TRACE_STMT.
    let sql = unsafe { CStr::from_ptr(sql.cast::<c_char>()) }.to_string_lossy();
    let Some(kind) = statement_kind(&sql) else {
        return SQLITE_OK;
    };
    // SAFETY: `context` points to the `Arc` allocation held alive by `SqlTrace`.
    let counts = unsafe { &*context.cast::<Mutex<SqlStatementCounts>>() };
    let mut counts = counts
        .lock()
        .expect("trace counter lock should not be poisoned");
    match kind {
        StatementKind::Read => counts.reads += 1,
        StatementKind::Write => counts.writes += 1,
    }
    SQLITE_OK
}

#[derive(Clone, Copy)]
enum StatementKind {
    Read,
    Write,
}

fn statement_kind(sql: &str) -> Option<StatementKind> {
    let first_word = sql
        .trim_start()
        .split_ascii_whitespace()
        .next()?
        .to_ascii_uppercase();
    match first_word.as_str() {
        "SELECT" | "WITH" | "EXPLAIN" => Some(StatementKind::Read),
        "INSERT" | "UPDATE" | "DELETE" | "REPLACE" | "CREATE" | "DROP" | "ALTER" => {
            Some(StatementKind::Write)
        }
        _ => None,
    }
}

async fn migrated_empty_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect empty first-use fixture");
    sqlx::raw_sql("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .expect("enable foreign keys");
    run_migrations(&pool)
        .await
        .expect("migrate empty first-use fixture");
    pool
}

async fn assert_no_user_content(pool: &SqlitePool) {
    for table in [
        "notes_pages",
        "notes_blocks",
        "notes_folders",
        "notes_page_templates",
        "project_tasks",
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .unwrap_or_else(|error| panic!("count {table}: {error}"));
        assert_eq!(count, 0, "fixture should have no rows in {table}");
    }
}

#[test]
fn empty_projects_first_use_has_a_fixed_backend_contract() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_empty_pool().await;
        projects::load_projects_snapshot_for_first_use_contract(&pool, None)
            .await
            .expect("seed required built-in projects");
        assert_no_user_content(&pool).await;

        let trace = SqlTrace::start(&pool).await;
        let mut metrics = FirstUseContractMetrics::default();
        let metadata = projects::load_projects_snapshot_for_first_use_contract(&pool, None)
            .await
            .expect("load Projects metadata snapshot");
        metrics.record_response(FirstUseIpcCommand::ProjectsLoadSnapshot, &metadata);
        let metadata_json = serde_json::to_value(&metadata).expect("serialize Projects metadata");
        let selected_project_id = metadata_json["projects"][0]["id"]
            .as_str()
            .expect("built-in project id");
        let selected = projects::load_projects_snapshot_for_first_use_contract(
            &pool,
            Some(selected_project_id),
        )
        .await
        .expect("load selected Projects snapshot");
        metrics.record_response(FirstUseIpcCommand::ProjectsLoadSnapshot, &selected);
        metrics.sql = trace.finish().await;

        assert_eq!(
            metrics,
            FirstUseContractMetrics {
                commands: vec![FirstUseIpcCommand::ProjectsLoadSnapshot; 2],
                sql: SqlStatementCounts {
                    reads: EMPTY_PROJECTS_SQL_READS,
                    writes: EMPTY_PROJECTS_SQL_WRITES,
                },
                serialized_response_bytes: EMPTY_PROJECTS_RESPONSE_BYTES,
            }
        );
    });
}

#[test]
fn empty_notes_first_use_has_a_fixed_backend_contract() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_empty_pool().await;
        notes::get_local_user_for_first_use_contract(&pool)
            .await
            .expect("seed required local Notes user");
        notes::get_page_history_settings_for_first_use_contract(&pool)
            .await
            .expect("seed required Notes history settings");
        assert_no_user_content(&pool).await;

        let trace = SqlTrace::start(&pool).await;
        let mut metrics = FirstUseContractMetrics::default();
        let sidebar = notes::list_sidebar_pages_for_first_use_contract(&pool)
            .await
            .expect("list Notes sidebar pages");
        metrics.record_response(FirstUseIpcCommand::NotesListSidebarPages, &sidebar);
        let pages = notes::list_pages_for_first_use_contract(&pool)
            .await
            .expect("list Notes pages");
        metrics.record_response(FirstUseIpcCommand::NotesListPages, &pages);
        let folders = notes::list_folders_for_first_use_contract(&pool)
            .await
            .expect("list Notes folders");
        metrics.record_response(FirstUseIpcCommand::NotesListFolders, &folders);
        let templates = notes::list_page_templates_for_first_use_contract(&pool)
            .await
            .expect("list Notes page templates");
        metrics.record_response(FirstUseIpcCommand::NotesListPageTemplates, &templates);
        let local_user = notes::get_local_user_for_first_use_contract(&pool)
            .await
            .expect("load local Notes user");
        metrics.record_response(FirstUseIpcCommand::NotesGetLocalUser, &local_user);
        let history_settings = notes::get_page_history_settings_for_first_use_contract(&pool)
            .await
            .expect("load Notes history settings");
        metrics.record_response(
            FirstUseIpcCommand::NotesGetPageHistorySettings,
            &history_settings,
        );
        metrics.sql = trace.finish().await;

        assert_eq!(
            metrics,
            FirstUseContractMetrics {
                commands: vec![
                    FirstUseIpcCommand::NotesListSidebarPages,
                    FirstUseIpcCommand::NotesListPages,
                    FirstUseIpcCommand::NotesListFolders,
                    FirstUseIpcCommand::NotesListPageTemplates,
                    FirstUseIpcCommand::NotesGetLocalUser,
                    FirstUseIpcCommand::NotesGetPageHistorySettings,
                ],
                sql: SqlStatementCounts {
                    reads: EMPTY_NOTES_SQL_READS,
                    writes: EMPTY_NOTES_SQL_WRITES,
                },
                serialized_response_bytes: EMPTY_NOTES_RESPONSE_BYTES,
            }
        );
    });
}
