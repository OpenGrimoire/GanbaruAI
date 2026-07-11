use super::models::{
    NoteFolderDto, NoteFolderRow, NotePageSummaryDto, NoteWorkspaceShellDto,
    NoteWorkspaceShellRequest,
};
use super::validation::require_uuid;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::{HashMap, HashSet};

const PAGE_WINDOW_SIZE: i64 = 50;
const FOLDER_WINDOW_SIZE: i64 = 100;
const MAX_SHELL_PAGES: usize = 200;
const MAX_SEED_IDS: usize = 50;
const MAX_EXPANDED_IDS: usize = 50;

pub(in crate::notes) async fn load_workspace_shell(
    pool: &SqlitePool,
    request: NoteWorkspaceShellRequest,
) -> Result<NoteWorkspaceShellDto, String> {
    let project_id = normalize_optional_uuid(request.project_id, "project_id")?;
    let selected_page_id = normalize_optional_uuid(request.selected_page_id, "selected_page_id")?;
    let seed_page_ids = normalize_ids(request.seed_page_ids, MAX_SEED_IDS);
    let expanded_page_ids = normalize_ids(request.expanded_page_ids, MAX_EXPANDED_IDS);
    let page_offset = decode_cursor(request.page_cursor.as_deref(), "page")?;
    let folder_offset = decode_cursor(request.folder_cursor.as_deref(), "folder")?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| format!("begin notes workspace shell read: {error}"))?;

    let total_page_count = count_pages(&mut transaction, project_id.as_deref()).await?;
    let total_folder_count = count_folders(&mut transaction, project_id.as_deref()).await?;
    let mut rows_by_id = HashMap::<String, NotePageSummaryDto>::new();

    let initial_pages = if request.destination_candidates {
        fetch_destination_window(
            &mut transaction,
            project_id.as_deref(),
            page_offset,
            PAGE_WINDOW_SIZE,
        )
        .await?
    } else {
        fetch_root_window(
            &mut transaction,
            project_id.as_deref(),
            page_offset,
            PAGE_WINDOW_SIZE,
        )
        .await?
    };
    let initial_page_window_full = initial_pages.len() == PAGE_WINDOW_SIZE as usize;
    add_unique(&mut rows_by_id, initial_pages);

    let mut recovery_ids = seed_page_ids;
    if let Some(selected) = selected_page_id.as_ref() {
        if !recovery_ids.contains(selected) {
            recovery_ids.push(selected.clone());
        }
    }
    add_unique(
        &mut rows_by_id,
        fetch_pages_by_ids(&mut transaction, project_id.as_deref(), &recovery_ids).await?,
    );
    add_unique(
        &mut rows_by_id,
        fetch_ancestors(&mut transaction, project_id.as_deref(), &recovery_ids).await?,
    );
    add_unique(
        &mut rows_by_id,
        fetch_children(&mut transaction, project_id.as_deref(), &expanded_page_ids).await?,
    );

    let mut pages = rows_by_id.into_values().collect::<Vec<_>>();
    pages.sort_by(|left, right| {
        right
            .last_edited_time
            .cmp(&left.last_edited_time)
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });
    pages.truncate(MAX_SHELL_PAGES);
    let loaded_ids = pages.iter().map(|page| page.id.clone()).collect::<Vec<_>>();
    let page_ids_with_children =
        fetch_parent_ids_with_children(&mut transaction, &loaded_ids).await?;
    let (missing_parent_page_ids, trashed_parent_page_ids) =
        fetch_unavailable_parent_ids(&mut transaction, &pages, &loaded_ids).await?;
    let resolved_selected_page_id =
        selected_page_id.filter(|selected| pages.iter().any(|page| page.id == *selected));
    let folder_rows = fetch_folder_window(
        &mut transaction,
        project_id.as_deref(),
        folder_offset,
        FOLDER_WINDOW_SIZE,
    )
    .await?;
    let folder_window_full = folder_rows.len() == FOLDER_WINDOW_SIZE as usize;
    let folders = folder_rows.into_iter().map(NoteFolderDto::new).collect();

    transaction
        .commit()
        .await
        .map_err(|error| format!("commit notes workspace shell read: {error}"))?;
    let next_page_cursor =
        initial_page_window_full.then(|| encode_cursor(page_offset + PAGE_WINDOW_SIZE));
    let next_folder_cursor =
        folder_window_full.then(|| encode_cursor(folder_offset + FOLDER_WINDOW_SIZE));
    Ok(NoteWorkspaceShellDto::new(
        pages,
        folders,
        page_ids_with_children,
        missing_parent_page_ids,
        trashed_parent_page_ids,
        resolved_selected_page_id,
        total_page_count,
        total_folder_count,
        next_page_cursor,
        next_folder_cursor,
    ))
}

fn normalize_optional_uuid(value: Option<String>, field: &str) -> Result<Option<String>, String> {
    value
        .map(|value| {
            let trimmed = value.trim();
            require_uuid(trimmed, field)?;
            Ok(trimmed.to_string())
        })
        .transpose()
}

fn normalize_ids(values: Vec<String>, limit: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(|value| {
            normalize_optional_uuid(Some(value), "page_id")
                .ok()
                .flatten()
        })
        .filter(|value| seen.insert(value.clone()))
        .take(limit)
        .collect()
}

fn decode_cursor(cursor: Option<&str>, field: &str) -> Result<i64, String> {
    let Some(cursor) = cursor else { return Ok(0) };
    cursor
        .strip_prefix("offset:")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .ok_or_else(|| format!("invalid {field} cursor"))
}

fn encode_cursor(offset: i64) -> String {
    format!("offset:{offset}")
}

fn add_unique(
    destination: &mut HashMap<String, NotePageSummaryDto>,
    rows: Vec<NotePageSummaryDto>,
) {
    for row in rows {
        if destination.len() >= MAX_SHELL_PAGES {
            break;
        }
        destination.entry(row.id.clone()).or_insert(row);
    }
}

fn page_projection() -> &'static str {
    "id, parent_type, parent_page_id, parent_block_id, parent_data_source_id, folder_id, title, json_extract(properties, '$.__ganbaru_project_id') AS project_id, icon, created_time, last_edited_time"
}

fn push_project_filter(query: &mut QueryBuilder<'_, Sqlite>, project_id: Option<&str>) {
    if let Some(project_id) = project_id {
        query.push(" AND json_extract(properties, '$.__ganbaru_project_id') = ");
        query.push_bind(project_id.to_string());
    } else {
        query.push(" AND json_extract(properties, '$.__ganbaru_project_id') IS NULL");
    }
}

async fn count_pages(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
) -> Result<i64, String> {
    let mut query =
        QueryBuilder::new("SELECT COUNT(*) FROM notes_pages WHERE in_trash = 0 AND archived = 0");
    push_project_filter(&mut query, project_id);
    query
        .build_query_scalar()
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| format!("count notes workspace pages: {error}"))
}

async fn count_folders(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
) -> Result<i64, String> {
    let mut query = QueryBuilder::new("SELECT COUNT(*) FROM notes_folders WHERE 1 = 1");
    if let Some(project_id) = project_id {
        query
            .push(" AND project_id = ")
            .push_bind(project_id.to_string());
    } else {
        query.push(" AND project_id = ''");
    }
    query
        .build_query_scalar()
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| format!("count notes workspace folders: {error}"))
}

async fn fetch_root_window(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<NotePageSummaryDto>, String> {
    let mut query = QueryBuilder::new(format!(
        "SELECT {} FROM notes_pages WHERE in_trash = 0 AND archived = 0 AND parent_type <> 'page_id' AND parent_type <> 'data_source_id'",
        page_projection()
    ));
    push_project_filter(&mut query, project_id);
    query
        .push(" ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);
    query
        .build_query_as()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes workspace roots: {error}"))
}

async fn fetch_destination_window(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<NotePageSummaryDto>, String> {
    let mut query = QueryBuilder::new(format!(
        "SELECT {} FROM notes_pages WHERE in_trash = 0 AND archived = 0",
        page_projection()
    ));
    push_project_filter(&mut query, project_id);
    query
        .push(" ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);
    query
        .build_query_as()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes destination candidates: {error}"))
}

async fn fetch_pages_by_ids(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
    ids: &[String],
) -> Result<Vec<NotePageSummaryDto>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::new(format!(
        "SELECT {} FROM notes_pages WHERE in_trash = 0 AND archived = 0 AND id IN (",
        page_projection()
    ));
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");
    push_project_filter(&mut query, project_id);
    query
        .build_query_as()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes workspace recovery pages: {error}"))
}

async fn fetch_ancestors(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
    ids: &[String],
) -> Result<Vec<NotePageSummaryDto>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::new("WITH RECURSIVE ancestors(id, depth) AS (SELECT parent_page_id, 1 FROM notes_pages WHERE id IN (");
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(") AND parent_type = 'page_id' UNION ALL SELECT page.parent_page_id, ancestors.depth + 1 FROM notes_pages page JOIN ancestors ON page.id = ancestors.id WHERE page.parent_type = 'page_id' AND ancestors.depth < 64) SELECT DISTINCT ");
    query.push(page_projection()).push(" FROM notes_pages WHERE in_trash = 0 AND archived = 0 AND id IN (SELECT id FROM ancestors WHERE id IS NOT NULL)");
    push_project_filter(&mut query, project_id);
    query.push(" LIMIT 64");
    query
        .build_query_as()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes workspace ancestors: {error}"))
}

async fn fetch_children(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
    ids: &[String],
) -> Result<Vec<NotePageSummaryDto>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::new(format!("SELECT {} FROM notes_pages WHERE in_trash = 0 AND archived = 0 AND parent_type = 'page_id' AND parent_page_id IN (", page_projection()));
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");
    push_project_filter(&mut query, project_id);
    query.push(" ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC LIMIT 100");
    query
        .build_query_as()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes workspace expanded children: {error}"))
}

async fn fetch_parent_ids_with_children(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    ids: &[String],
) -> Result<Vec<String>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::new("SELECT DISTINCT parent_page_id FROM notes_pages WHERE in_trash = 0 AND archived = 0 AND parent_type = 'page_id' AND parent_page_id IN (");
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(") ORDER BY parent_page_id");
    query
        .build_query_scalar()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes workspace child markers: {error}"))
}

async fn fetch_unavailable_parent_ids(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    pages: &[NotePageSummaryDto],
    loaded_ids: &[String],
) -> Result<(Vec<String>, Vec<String>), String> {
    let loaded = loaded_ids.iter().collect::<HashSet<_>>();
    let parent_ids = pages
        .iter()
        .filter(|page| page.parent_type == "page_id")
        .filter_map(|page| page.parent_page_id.as_ref())
        .filter(|id| !loaded.contains(id))
        .cloned()
        .collect::<HashSet<_>>();
    if parent_ids.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut query =
        QueryBuilder::new("SELECT id, in_trash, archived FROM notes_pages WHERE id IN (");
    let mut separated = query.separated(", ");
    for id in &parent_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");
    let states = query
        .build_query_as::<(String, i64, i64)>()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("load notes workspace parent states: {error}"))?;
    let state_by_id = states
        .into_iter()
        .map(|(id, trash, archive)| (id, (trash, archive)))
        .collect::<HashMap<_, _>>();
    let mut missing = Vec::new();
    let mut trashed = Vec::new();
    for id in parent_ids {
        match state_by_id.get(&id) {
            Some((trash, _)) if *trash != 0 => trashed.push(id),
            Some((_, archive)) if *archive != 0 => missing.push(id),
            None => missing.push(id),
            Some(_) => {}
        }
    }
    missing.sort();
    trashed.sort();
    Ok((missing, trashed))
}

async fn fetch_folder_window(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    project_id: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<NoteFolderRow>, String> {
    let mut query = QueryBuilder::new("SELECT id, project_id, parent_folder_id, name, created_time, last_edited_time FROM notes_folders WHERE 1 = 1");
    if let Some(project_id) = project_id {
        query
            .push(" AND project_id = ")
            .push_bind(project_id.to_string());
    } else {
        query.push(" AND project_id = ''");
    }
    query
        .push(" ORDER BY name COLLATE NOCASE ASC, id ASC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);
    query
        .build_query_as()
        .fetch_all(&mut **transaction)
        .await
        .map_err(|error| format!("list notes workspace folders: {error}"))
}
