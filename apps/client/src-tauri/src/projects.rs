use crate::db_path::connect_sqlite;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, Runtime};

const PALETTE_SIZE: i64 = 32;
const PROJECT_TEMPLATE_IDS: &[&str] = &[
    "blank", "software", "course", "routine", "reading", "chores",
];
const MAX_PROJECT_EVENT_DURATION_MINUTES: i64 = 24 * 60;
const MAX_PROJECT_POMODORO_FOCUS_MINUTES: i64 = 120;
const MAX_PROJECT_POMODORO_SHORT_BREAK_MINUTES: i64 = 30;
const MAX_PROJECT_POMODORO_LONG_BREAK_MINUTES: i64 = 60;
const MAX_PROJECT_POMODORO_CYCLE_COUNT: i64 = 12;
const MAX_TASK_CHANGE_REASON_LENGTH: usize = 1000;

mod custom_fields;
mod history;
mod models;
mod templates;
mod validation;
use custom_fields::{
    delete_custom_field_option_with_history, delete_custom_field_with_history,
    ensure_custom_field_accepts_options_in_pool, update_custom_field_value_with_history,
};
use history::*;
pub use models::*;
use templates::{insert_default_statuses, insert_template_sections};
use validation::*;

#[tauri::command]
pub async fn projects_load_snapshot<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: Option<String>,
) -> Result<ProjectsSnapshot, String> {
    let pool = connect_sqlite(app, db_url).await?;
    let normalized_project_id = project_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("\0");
    let groups = sqlx::query_as::<_, ProjectGroupRow>(
        "SELECT * FROM project_groups ORDER BY sort_order ASC, name ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project groups: {e}"))?;
    let projects = sqlx::query_as::<_, ProjectRow>(
        "SELECT * FROM projects ORDER BY group_id ASC, sort_order ASC, name ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load projects: {e}"))?;
    let sections = sqlx::query_as::<_, ProjectSectionRow>(
        "SELECT * FROM project_sections
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project sections: {e}"))?;
    let statuses = sqlx::query_as::<_, ProjectStatusRow>(
        "SELECT * FROM project_statuses
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project statuses: {e}"))?;
    let tasks = sqlx::query_as::<_, ProjectTaskRow>(
        "SELECT * FROM project_tasks
         WHERE project_id = ?
         ORDER BY project_id ASC, section_sort_order ASC, created_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project tasks: {e}"))?;
    let checklist_items = sqlx::query_as::<_, ProjectChecklistItemRow>(
        "SELECT project_checklist_items.*
         FROM project_checklist_items
         JOIN project_tasks ON project_tasks.id = project_checklist_items.task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_checklist_items.task_id ASC, project_checklist_items.sort_order ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project checklist items: {e}"))?;
    let labels = sqlx::query_as::<_, ProjectLabelRow>(
        "SELECT * FROM project_labels
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project labels: {e}"))?;
    let task_label_links = sqlx::query_as::<_, ProjectTaskLabelLinkRow>(
        "SELECT project_task_label_links.*
         FROM project_task_label_links
         JOIN project_tasks ON project_tasks.id = project_task_label_links.task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_task_label_links.created_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project task label links: {e}"))?;
    let custom_fields = sqlx::query_as::<_, ProjectCustomFieldRow>(
        "SELECT * FROM project_custom_fields
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project custom fields: {e}"))?;
    let custom_field_options = sqlx::query_as::<_, ProjectCustomFieldOptionRow>(
        "SELECT project_custom_field_options.*
         FROM project_custom_field_options
         JOIN project_custom_fields ON project_custom_fields.id = project_custom_field_options.field_id
         WHERE project_custom_fields.project_id = ?
         ORDER BY project_custom_field_options.field_id ASC, project_custom_field_options.sort_order ASC, project_custom_field_options.name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project custom field options: {e}"))?;
    let custom_field_values = sqlx::query_as::<_, ProjectCustomFieldValueRow>(
        "SELECT project_custom_field_values.*
         FROM project_custom_field_values
         JOIN project_tasks ON project_tasks.id = project_custom_field_values.task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_custom_field_values.updated_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project custom field values: {e}"))?;
    let custom_field_option_values = sqlx::query_as::<_, ProjectCustomFieldOptionValueRow>(
        "SELECT project_custom_field_option_values.*
         FROM project_custom_field_option_values
         JOIN project_tasks ON project_tasks.id = project_custom_field_option_values.task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_custom_field_option_values.created_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project custom field option values: {e}"))?;
    let dependencies = sqlx::query_as::<_, ProjectTaskDependencyRow>(
        "SELECT project_task_dependencies.*
         FROM project_task_dependencies
         JOIN project_tasks ON project_tasks.id = project_task_dependencies.blocked_task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_task_dependencies.created_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project task dependencies: {e}"))?;
    let event_links = sqlx::query_as::<_, ProjectTaskEventLinkRow>(
        "SELECT project_task_event_links.*
         FROM project_task_event_links
         JOIN project_tasks ON project_tasks.id = project_task_event_links.task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_task_event_links.created_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project task event links: {e}"))?;
    let task_change_events = sqlx::query_as::<_, ProjectTaskChangeEventRow>(
        "SELECT project_task_change_events.*
         FROM project_task_change_events
         JOIN project_tasks ON project_tasks.id = project_task_change_events.task_id
         WHERE project_tasks.project_id = ?
         ORDER BY project_task_change_events.occurred_at DESC
         LIMIT 500",
    )
    .bind(normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project task change events: {e}"))?;
    let view_preferences = sqlx::query_as::<_, ProjectViewPreferenceRow>(
        "SELECT * FROM project_view_preferences ORDER BY project_id ASC, view_id ASC, updated_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project view preferences: {e}"))?;
    let custom_emojis = load_project_custom_emojis(&pool).await?;

    Ok(ProjectsSnapshot {
        groups,
        projects,
        sections,
        statuses,
        tasks,
        checklist_items,
        labels,
        task_label_links,
        custom_fields,
        custom_field_options,
        custom_field_values,
        custom_field_option_values,
        dependencies,
        event_links,
        task_change_events,
        view_preferences,
        custom_emojis,
    })
}

async fn load_project_custom_emojis(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<ProjectCustomEmojiRow>, String> {
    sqlx::query_as::<_, ProjectCustomEmojiRow>(
        "SELECT * FROM project_custom_emojis ORDER BY sort_order ASC, name ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load project custom emoji: {e}"))
}

#[tauri::command]
pub async fn projects_create_group<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group: ProjectGroupCreate,
) -> Result<(), String> {
    validate_group_create(&group)?;
    let pool = connect_sqlite(app, db_url).await?;
    sqlx::query(
        "INSERT INTO project_groups (id, name, icon, color, sort_order)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&group.id)
    .bind(group.name.trim())
    .bind(group.icon.trim())
    .bind(group.color)
    .bind(group.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project group: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_group<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group: ProjectGroupUpdate,
) -> Result<(), String> {
    validate_group_update(&group)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_groups
         SET name = ?,
             icon = ?,
             color = ?,
             sort_order = ?,
             collapsed = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(group.name.trim())
    .bind(group.icon.trim())
    .bind(group.color)
    .bind(group.sort_order)
    .bind(if group.collapsed { 1_i64 } else { 0_i64 })
    .bind(&group.id)
    .execute(&pool)
    .await
    .map_err(|e| format!("update project group: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project group not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_group<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group_id: String,
) -> Result<(), String> {
    let pool = connect_sqlite(app, db_url).await?;
    delete_project_group(&pool, group_id.trim()).await
}

async fn delete_project_group(pool: &sqlx::SqlitePool, group_id: &str) -> Result<(), String> {
    let normalized_group_id = group_id.trim();
    require_non_empty(normalized_group_id, "group_id")?;
    let result = sqlx::query("DELETE FROM project_groups WHERE id = ?")
        .bind(normalized_group_id)
        .execute(pool)
        .await
        .map_err(|e| format!("delete project group: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project group not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_set_group_collapsed<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group_id: String,
    collapsed: bool,
) -> Result<(), String> {
    require_non_empty(&group_id, "group_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_groups
         SET collapsed = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(if collapsed { 1_i64 } else { 0_i64 })
    .bind(group_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("set project group collapsed: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project group not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_create_project<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project: ProjectCreate,
) -> Result<(), String> {
    validate_project_create(&project)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    sqlx::query(
        "INSERT INTO projects (
            id, group_id, name, icon, color, sort_order,
            default_event_duration_minutes,
            default_pomodoro_mode, default_pomodoro_preset_key,
            default_pomodoro_focus_minutes, default_pomodoro_short_break_minutes,
            default_pomodoro_long_break_minutes, default_pomodoro_long_break_after_focus_count,
            default_idle_timeout_minutes
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&project.id)
    .bind(&project.group_id)
    .bind(project.name.trim())
    .bind(project.icon.trim())
    .bind(project.color)
    .bind(project.sort_order)
    .bind(project.default_event_duration_minutes)
    .bind(&project.default_pomodoro_mode)
    .bind(&project.default_pomodoro_preset_key)
    .bind(project.default_pomodoro_focus_minutes)
    .bind(project.default_pomodoro_short_break_minutes)
    .bind(project.default_pomodoro_long_break_minutes)
    .bind(project.default_pomodoro_long_break_after_focus_count)
    .bind(project.default_idle_timeout_minutes)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create project: {e}"))?;

    insert_template_sections(&mut tx, &project.id, &project.template_id).await?;
    insert_default_statuses(&mut tx, &project.id).await?;

    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_project<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project: ProjectUpdate,
) -> Result<(), String> {
    validate_project_update(&project)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE projects
         SET group_id = ?,
             name = ?,
             icon = ?,
             color = ?,
             sort_order = ?,
             status = ?,
             default_event_duration_minutes = ?,
             default_pomodoro_mode = ?,
             default_pomodoro_preset_key = ?,
             default_pomodoro_focus_minutes = ?,
             default_pomodoro_short_break_minutes = ?,
             default_pomodoro_long_break_minutes = ?,
             default_pomodoro_long_break_after_focus_count = ?,
             default_idle_timeout_minutes = ?,
             focus_playlist_id = ?,
             break_playlist_id = ?,
             work_environment_id = ?,
             blocker_ruleset_id = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(&project.group_id)
    .bind(project.name.trim())
    .bind(project.icon.trim())
    .bind(project.color)
    .bind(project.sort_order)
    .bind(&project.status)
    .bind(project.default_event_duration_minutes)
    .bind(&project.default_pomodoro_mode)
    .bind(&project.default_pomodoro_preset_key)
    .bind(project.default_pomodoro_focus_minutes)
    .bind(project.default_pomodoro_short_break_minutes)
    .bind(project.default_pomodoro_long_break_minutes)
    .bind(project.default_pomodoro_long_break_after_focus_count)
    .bind(project.default_idle_timeout_minutes)
    .bind(normalized_optional_identifier(
        project.focus_playlist_id.as_deref(),
    ))
    .bind(normalized_optional_identifier(
        project.break_playlist_id.as_deref(),
    ))
    .bind(normalized_optional_identifier(
        project.work_environment_id.as_deref(),
    ))
    .bind(normalized_optional_identifier(
        project.blocker_ruleset_id.as_deref(),
    ))
    .bind(&project.id)
    .execute(&pool)
    .await
    .map_err(|e| format!("update project: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_create_section<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    section: ProjectSectionCreate,
) -> Result<(), String> {
    validate_section_create(&section)?;
    let pool = connect_sqlite(app, db_url).await?;
    sqlx::query(
        "INSERT INTO project_sections (id, project_id, name, sort_order)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&section.id)
    .bind(&section.project_id)
    .bind(section.name.trim())
    .bind(section.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project section: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_section<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    section: ProjectSectionUpdate,
) -> Result<(), String> {
    validate_section_update(&section)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_sections
         SET name = ?,
             sort_order = ?,
             collapsed = ?,
             hidden_at = ?,
             archived_at = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(section.name.trim())
    .bind(section.sort_order)
    .bind(if section.collapsed { 1_i64 } else { 0_i64 })
    .bind(&section.hidden_at)
    .bind(&section.archived_at)
    .bind(&section.id)
    .execute(&pool)
    .await
    .map_err(|e| format!("update project section: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project section not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_create_status<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    status: ProjectStatusCreate,
) -> Result<(), String> {
    validate_status_create(&status)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, &status.project_id).await?;
    sqlx::query(
        "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&status.id)
    .bind(&status.project_id)
    .bind(status.name.trim())
    .bind(&status.category)
    .bind(status.sort_order)
    .bind(if status.terminal { 1_i64 } else { 0_i64 })
    .execute(&pool)
    .await
    .map_err(|e| format!("create project status: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_status<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    status: ProjectStatusUpdate,
) -> Result<(), String> {
    validate_status_update(&status)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_statuses
         SET name = ?,
             category = ?,
             sort_order = ?,
             terminal = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(status.name.trim())
    .bind(&status.category)
    .bind(status.sort_order)
    .bind(if status.terminal { 1_i64 } else { 0_i64 })
    .bind(&status.id)
    .execute(&pool)
    .await
    .map_err(|e| format!("update project status: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project status not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_create_task<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task: ProjectTaskCreate,
) -> Result<(), String> {
    validate_task_create(&task)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    ensure_section_and_status_match_project(
        &mut tx,
        &task.project_id,
        &task.section_id,
        &task.status_id,
    )
    .await?;
    if let Some(parent_id) = &task.parent_task_id {
        ensure_parent_task_matches_project(&mut tx, &task.project_id, parent_id).await?;
    }
    let section_sort_order = next_task_sort_order(
        &mut tx,
        "section_id",
        &task.project_id,
        &task.section_id,
        "section_sort_order",
    )
    .await?;
    let status_sort_order = next_task_sort_order(
        &mut tx,
        "status_id",
        &task.project_id,
        &task.status_id,
        "status_sort_order",
    )
    .await?;
    sqlx::query(
        "INSERT INTO project_tasks (
            id, project_id, section_id, status_id, parent_task_id, title,
            section_sort_order, status_sort_order
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&task.id)
    .bind(&task.project_id)
    .bind(&task.section_id)
    .bind(&task.status_id)
    .bind(&task.parent_task_id)
    .bind(task.title.trim())
    .bind(section_sort_order)
    .bind(status_sort_order)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create project task: {e}"))?;
    insert_task_change_event(&mut tx, &task.id, "created", None, None, None).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_task<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task: ProjectTaskUpdate,
) -> Result<(), String> {
    validate_task_update(&task)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let previous_task =
        sqlx::query_as::<_, ProjectTaskRow>("SELECT * FROM project_tasks WHERE id = ?")
            .bind(&task.id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| format!("load project task: {e}"))?
            .ok_or_else(|| "project task not found".to_string())?;
    let project_id = previous_task.project_id.clone();
    ensure_section_and_status_match_project(
        &mut tx,
        &project_id,
        &task.section_id,
        &task.status_id,
    )
    .await?;
    if let Some(parent_id) = &task.parent_task_id {
        ensure_parent_task_matches_project(&mut tx, &project_id, parent_id).await?;
        if parent_id == &task.id {
            return Err("task cannot be its own parent".to_string());
        }
        ensure_task_has_no_children(&mut tx, &task.id).await?;
    }
    let terminal_status = status_is_terminal(&mut tx, &task.status_id).await?;
    let completed_at = if terminal_status {
        Some(current_timestamp(&mut tx).await?)
    } else {
        None
    };
    sqlx::query(
        "UPDATE project_tasks
         SET section_id = ?,
             status_id = ?,
             parent_task_id = ?,
             title = ?,
             description = ?,
             priority = ?,
             task_type = ?,
             section_sort_order = ?,
             status_sort_order = ?,
             estimate_minutes = ?,
             due_date = ?,
             due_time = ?,
             start_date = ?,
             start_time = ?,
             target_end_date = ?,
             completed_at = ?,
             archived_at = ?,
             blocker_reason = ?,
             milestone = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(&task.section_id)
    .bind(&task.status_id)
    .bind(&task.parent_task_id)
    .bind(task.title.trim())
    .bind(&task.description)
    .bind(&task.priority)
    .bind(&task.task_type)
    .bind(task.section_sort_order)
    .bind(task.status_sort_order)
    .bind(task.estimate_minutes)
    .bind(&task.due_date)
    .bind(&task.due_time)
    .bind(&task.start_date)
    .bind(&task.start_time)
    .bind(&task.target_end_date)
    .bind(&completed_at)
    .bind(&task.archived_at)
    .bind(&task.blocker_reason)
    .bind(if task.milestone { 1_i64 } else { 0_i64 })
    .bind(&task.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update project task: {e}"))?;
    insert_task_update_change_events(&mut tx, &previous_task, &task).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_create_task_dependency<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    dependency: ProjectTaskDependencyCreate,
) -> Result<(), String> {
    validate_task_dependency_create(&dependency)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    ensure_dependency_tasks_match_project(
        &mut tx,
        &dependency.blocking_task_id,
        &dependency.blocked_task_id,
    )
    .await?;
    ensure_dependency_has_no_cycle(
        &mut tx,
        &dependency.blocking_task_id,
        &dependency.blocked_task_id,
    )
    .await?;
    sqlx::query(
        "INSERT INTO project_task_dependencies
            (id, blocking_task_id, blocked_task_id, dependency_type)
         VALUES (?, ?, ?, ?)",
    )
    .bind(dependency.id.trim())
    .bind(&dependency.blocking_task_id)
    .bind(&dependency.blocked_task_id)
    .bind(&dependency.dependency_type)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create project task dependency: {e}"))?;
    insert_task_change_event(
        &mut tx,
        &dependency.blocked_task_id,
        "dependency_added",
        Some("blocking_task_id"),
        None,
        Some(&dependency.blocking_task_id),
    )
    .await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_task_dependency<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    dependency_id: String,
) -> Result<(), String> {
    require_non_empty(&dependency_id, "dependency_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let row = sqlx::query_as::<_, ProjectTaskDependencyRow>(
        "SELECT * FROM project_task_dependencies WHERE id = ?",
    )
    .bind(dependency_id.trim())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| format!("load project task dependency: {e}"))?
    .ok_or_else(|| "project task dependency not found".to_string())?;
    sqlx::query("DELETE FROM project_task_dependencies WHERE id = ?")
        .bind(dependency_id.trim())
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("delete project task dependency: {e}"))?;
    insert_task_change_event(
        &mut tx,
        &row.blocked_task_id,
        "dependency_removed",
        Some("blocking_task_id"),
        Some(&row.blocking_task_id),
        None,
    )
    .await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_create_checklist_item<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    item: ProjectChecklistItemCreate,
) -> Result<(), String> {
    validate_checklist_item_create(&item)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    ensure_task_exists(&mut tx, &item.task_id).await?;
    sqlx::query(
        "INSERT INTO project_checklist_items (id, task_id, title, sort_order)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&item.id)
    .bind(&item.task_id)
    .bind(item.title.trim())
    .bind(item.sort_order)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create project checklist item: {e}"))?;
    insert_task_change_event(
        &mut tx,
        &item.task_id,
        "updated",
        Some("checklist"),
        None,
        None,
    )
    .await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_checklist_item<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    item: ProjectChecklistItemUpdate,
) -> Result<(), String> {
    validate_checklist_item_update(&item)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let task_id = task_id_for_checklist_item(&mut tx, &item.id).await?;
    let completed_at = if item.completed {
        Some(current_timestamp(&mut tx).await?)
    } else {
        None
    };
    sqlx::query(
        "UPDATE project_checklist_items
         SET title = ?,
             completed_at = ?,
             sort_order = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(item.title.trim())
    .bind(&completed_at)
    .bind(item.sort_order)
    .bind(&item.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update project checklist item: {e}"))?;
    insert_task_change_event(&mut tx, &task_id, "updated", Some("checklist"), None, None).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_checklist_item<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    item_id: String,
) -> Result<(), String> {
    require_non_empty(&item_id, "item_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let task_id = task_id_for_checklist_item(&mut tx, &item_id).await?;
    sqlx::query("DELETE FROM project_checklist_items WHERE id = ?")
        .bind(&item_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("delete project checklist item: {e}"))?;
    insert_task_change_event(&mut tx, &task_id, "updated", Some("checklist"), None, None).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_create_label<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    label: ProjectLabelCreate,
) -> Result<(), String> {
    validate_label_create(&label)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, label.project_id.trim()).await?;
    sqlx::query(
        "INSERT INTO project_labels (id, project_id, name, color, sort_order)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(label.id.trim())
    .bind(label.project_id.trim())
    .bind(label.name.trim())
    .bind(label.color)
    .bind(label.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project label: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_label<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    label: ProjectLabelUpdate,
) -> Result<(), String> {
    validate_label_update(&label)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_labels
         SET name = ?,
             color = ?,
             sort_order = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(label.name.trim())
    .bind(label.color)
    .bind(label.sort_order)
    .bind(label.id.trim())
    .execute(&pool)
    .await
    .map_err(|e| format!("update project label: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project label not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_label<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    label_id: String,
) -> Result<(), String> {
    require_non_empty(&label_id, "label_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_label_with_history(&mut tx, label_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_link_task_label<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    link: ProjectTaskLabelLinkCreate,
) -> Result<(), String> {
    validate_task_label_link_create(&link)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let label_name = label_for_task_label_link(&mut tx, &link.task_id, &link.label_id).await?;
    let result = sqlx::query(
        "INSERT INTO project_task_label_links (task_id, label_id)
         VALUES (?, ?)
         ON CONFLICT(task_id, label_id) DO NOTHING",
    )
    .bind(link.task_id.trim())
    .bind(link.label_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("link project task label: {e}"))?;
    if result.rows_affected() > 0 {
        insert_task_change_event(
            &mut tx,
            link.task_id.trim(),
            "updated",
            Some("labels"),
            None,
            Some(&label_name),
        )
        .await?;
    }
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_unlink_task_label<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task_id: String,
    label_id: String,
) -> Result<(), String> {
    require_non_empty(&task_id, "task_id")?;
    require_non_empty(&label_id, "label_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let label_name =
        label_for_existing_task_label_link(&mut tx, task_id.trim(), label_id.trim()).await?;
    sqlx::query(
        "DELETE FROM project_task_label_links
         WHERE task_id = ? AND label_id = ?",
    )
    .bind(task_id.trim())
    .bind(label_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("unlink project task label: {e}"))?;
    insert_task_change_event(
        &mut tx,
        task_id.trim(),
        "updated",
        Some("labels"),
        Some(&label_name),
        None,
    )
    .await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_create_custom_field<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    field: ProjectCustomFieldCreate,
) -> Result<(), String> {
    validate_custom_field_create(&field)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, field.project_id.trim()).await?;
    sqlx::query(
        "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(field.id.trim())
    .bind(field.project_id.trim())
    .bind(field.name.trim())
    .bind(field.field_type.trim())
    .bind(field.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project custom field: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_custom_field<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    field: ProjectCustomFieldUpdate,
) -> Result<(), String> {
    validate_custom_field_update(&field)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_custom_fields
         SET name = ?,
             sort_order = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(field.name.trim())
    .bind(field.sort_order)
    .bind(field.id.trim())
    .execute(&pool)
    .await
    .map_err(|e| format!("update project custom field: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project custom field not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_custom_field<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    field_id: String,
) -> Result<(), String> {
    require_non_empty(&field_id, "field_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_custom_field_with_history(&mut tx, field_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_create_custom_field_option<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    option: ProjectCustomFieldOptionCreate,
) -> Result<(), String> {
    validate_custom_field_option_create(&option)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_custom_field_accepts_options_in_pool(&pool, option.field_id.trim()).await?;
    sqlx::query(
        "INSERT INTO project_custom_field_options (id, field_id, name, sort_order)
         VALUES (?, ?, ?, ?)",
    )
    .bind(option.id.trim())
    .bind(option.field_id.trim())
    .bind(option.name.trim())
    .bind(option.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project custom field option: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_custom_field_option<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    option: ProjectCustomFieldOptionUpdate,
) -> Result<(), String> {
    validate_custom_field_option_update(&option)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_custom_field_options
         SET name = ?,
             sort_order = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(option.name.trim())
    .bind(option.sort_order)
    .bind(option.id.trim())
    .execute(&pool)
    .await
    .map_err(|e| format!("update project custom field option: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project custom field option not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_custom_field_option<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    option_id: String,
) -> Result<(), String> {
    require_non_empty(&option_id, "option_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_custom_field_option_with_history(&mut tx, option_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_update_custom_field_value<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    value: ProjectCustomFieldValueUpdate,
) -> Result<(), String> {
    validate_custom_field_value_update(&value)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    update_custom_field_value_with_history(&mut tx, &value).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_link_task_event<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    link: ProjectTaskEventLinkCreate,
) -> Result<(), String> {
    validate_task_event_link_create(&link)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    link_task_event_with_project_assignment(&mut tx, &link).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

async fn link_task_event_with_project_assignment(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    link: &ProjectTaskEventLinkCreate,
) -> Result<(), String> {
    let task_project_id: String =
        sqlx::query_scalar("SELECT project_id FROM project_tasks WHERE id = ?")
            .bind(&link.task_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load linked project task: {e}"))?
            .ok_or_else(|| "project task not found".to_string())?;
    let event_project_id: Option<String> =
        sqlx::query_scalar("SELECT project_id FROM calendar_events WHERE id = ?")
            .bind(&link.event_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load linked calendar event: {e}"))?
            .ok_or_else(|| "calendar event not found".to_string())?;
    match event_project_id {
        Some(event_project_id) if event_project_id != task_project_id => {
            return Err("calendar event does not belong to the task project".to_string());
        }
        Some(_) => {}
        None => {
            sqlx::query("UPDATE calendar_events SET project_id = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(&task_project_id)
                .bind(&link.event_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("assign calendar event project: {e}"))?;
        }
    }
    sqlx::query(
        "INSERT INTO project_task_event_links (task_id, event_id, link_kind)
         VALUES (?, ?, ?)
         ON CONFLICT(task_id, event_id) DO UPDATE SET link_kind = excluded.link_kind",
    )
    .bind(&link.task_id)
    .bind(&link.event_id)
    .bind(&link.link_kind)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("link project task event: {e}"))?;
    insert_task_change_event(
        tx,
        &link.task_id,
        "scheduled",
        Some("event_id"),
        None,
        Some(&link.event_id),
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn projects_unlink_task_event<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task_id: String,
    event_id: String,
) -> Result<(), String> {
    require_non_empty(&task_id, "task_id")?;
    require_non_empty(&event_id, "event_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let existing_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM project_task_event_links
         WHERE task_id = ? AND event_id = ?",
    )
    .bind(task_id.trim())
    .bind(event_id.trim())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("load project task event link: {e}"))?;
    if existing_count == 0 {
        return Err("project task event link not found".to_string());
    }
    sqlx::query(
        "DELETE FROM project_task_event_links
         WHERE task_id = ? AND event_id = ?",
    )
    .bind(task_id.trim())
    .bind(event_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("unlink project task event: {e}"))?;
    insert_task_change_event(
        &mut tx,
        task_id.trim(),
        "event_unlinked",
        Some("event_id"),
        Some(event_id.trim()),
        None,
    )
    .await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_search_linkable_events<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    search: ProjectLinkableEventSearch,
) -> Result<Vec<ProjectLinkableEvent>, String> {
    require_non_empty(&search.project_id, "project_id")?;
    let normalized_project_id = search.project_id.trim().to_string();
    let normalized_task_id = search.task_id.unwrap_or_default().trim().to_string();
    let result_limit = search.limit.unwrap_or(12).clamp(1, 50);
    let normalized_query = search.query.unwrap_or_default().trim().to_lowercase();
    let normalized_start_date = normalize_optional_date_filter(search.start_date, "start_date")?;
    let normalized_end_date = normalize_optional_date_filter(search.end_date, "end_date")?;
    if let (Some(start), Some(end)) = (&normalized_start_date, &normalized_end_date) {
        if start > end {
            return Err("start_date must be before end_date".to_string());
        }
    }
    let like_query = sql_like_contains_pattern(&normalized_query);
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, &normalized_project_id).await?;
    if !normalized_task_id.is_empty() {
        ensure_task_belongs_to_project_in_pool(&pool, &normalized_task_id, &normalized_project_id)
            .await?;
    }

    let mut event_rows = sqlx::query_as::<_, ProjectLinkableEventRow>(
        "SELECT id,
                project_id,
                title,
                start_time,
                end_time,
                timezone,
                calendar_id,
                color,
                all_day,
                status
         FROM calendar_events
         WHERE project_id = ?
           AND status != 'cancelled'
           AND (
             ? = ''
             OR lower(title) LIKE ? ESCAPE '\\'
             OR lower(start_time) LIKE ? ESCAPE '\\'
             OR lower(end_time) LIKE ? ESCAPE '\\'
           )
           AND (? IS NULL OR substr(start_time, 1, 10) >= ?)
           AND (? IS NULL OR substr(start_time, 1, 10) <= ?)
         ORDER BY
           CASE WHEN start_time >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now') THEN 0 ELSE 1 END,
           CASE WHEN start_time >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now') THEN start_time END ASC,
           start_time DESC
         LIMIT ?",
    )
    .bind(&normalized_project_id)
    .bind(&normalized_query)
    .bind(&like_query)
    .bind(&like_query)
    .bind(&like_query)
    .bind(&normalized_start_date)
    .bind(&normalized_start_date)
    .bind(&normalized_end_date)
    .bind(&normalized_end_date)
    .bind(result_limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("search project linkable events: {e}"))?;
    if !normalized_task_id.is_empty() {
        let linked_event_rows = sqlx::query_as::<_, ProjectLinkableEventRow>(
            "SELECT ce.id,
                    ce.project_id,
                    ce.title,
                    ce.start_time,
                    ce.end_time,
                    ce.timezone,
                    ce.calendar_id,
                    ce.color,
                    ce.all_day,
                    ce.status
             FROM calendar_events ce
             JOIN project_task_event_links ptel ON ptel.event_id = ce.id
             WHERE ce.project_id = ?
               AND ptel.task_id = ?
               AND ce.status != 'cancelled'
             ORDER BY ce.start_time DESC",
        )
        .bind(&normalized_project_id)
        .bind(&normalized_task_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("load linked project events: {e}"))?;
        let mut seen_event_ids: HashSet<String> =
            event_rows.iter().map(|event| event.id.clone()).collect();
        for row in linked_event_rows {
            if seen_event_ids.insert(row.id.clone()) {
                event_rows.push(row);
            }
        }
    }

    let event_ids: HashSet<String> = event_rows.iter().map(|event| event.id.clone()).collect();
    if event_ids.is_empty() {
        return Ok(Vec::new());
    }

    let task_link_rows = sqlx::query_as::<_, ProjectLinkableEventTaskRow>(
        "SELECT ptel.event_id,
                pt.id AS task_id,
                pt.title,
                pt.archived_at
         FROM project_task_event_links ptel
         JOIN project_tasks pt ON pt.id = ptel.task_id
         WHERE pt.project_id = ?
         ORDER BY ptel.created_at DESC",
    )
    .bind(&normalized_project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project event task links: {e}"))?;
    let mut tasks_by_event_id: HashMap<String, Vec<ProjectLinkableEventTask>> = HashMap::new();
    for row in task_link_rows {
        if !event_ids.contains(&row.event_id) {
            continue;
        }
        tasks_by_event_id
            .entry(row.event_id)
            .or_default()
            .push(ProjectLinkableEventTask {
                task_id: row.task_id,
                title: row.title,
                archived_at: row.archived_at,
            });
    }

    Ok(event_rows
        .into_iter()
        .map(|row| ProjectLinkableEvent {
            linked_tasks: tasks_by_event_id.remove(&row.id).unwrap_or_default(),
            id: row.id,
            project_id: row.project_id,
            title: row.title,
            start_time: row.start_time,
            end_time: row.end_time,
            timezone: row.timezone,
            calendar_id: row.calendar_id,
            color: row.color,
            all_day: row.all_day,
            status: row.status,
        })
        .collect())
}

#[tauri::command]
pub async fn projects_upsert_view_preference<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    preference: ProjectViewPreferenceUpsert,
) -> Result<(), String> {
    validate_view_preference(&preference)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, preference.project_id.trim()).await?;
    sqlx::query(
        "INSERT INTO project_view_preferences
            (project_id, view_id, preference_key, preference_value)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(project_id, view_id, preference_key)
         DO UPDATE SET
            preference_value = excluded.preference_value,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(preference.project_id.trim())
    .bind(preference.view_id.trim())
    .bind(preference.preference_key.trim())
    .bind(&preference.preference_value)
    .execute(&pool)
    .await
    .map_err(|e| format!("save project view preference: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_view_preference<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    view_id: String,
    preference_key: String,
) -> Result<(), String> {
    require_non_empty(&project_id, "project_id")?;
    validate_enum(
        &view_id,
        "view_id",
        &["dashboard", "list", "kanban", "calendar", "gantt"],
    )?;
    require_non_empty(&preference_key, "preference_key")?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "DELETE FROM project_view_preferences
         WHERE project_id = ? AND view_id = ? AND preference_key = ?",
    )
    .bind(project_id.trim())
    .bind(view_id.trim())
    .bind(preference_key.trim())
    .execute(&pool)
    .await
    .map_err(|e| format!("delete project view preference: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project view preference not found".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn projects_create_custom_emoji<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    emoji: ProjectCustomEmojiCreate,
) -> Result<(), String> {
    validate_custom_emoji_create(&emoji)?;
    let pool = connect_sqlite(app, db_url).await?;
    insert_project_custom_emoji(&pool, &emoji).await
}

async fn insert_project_custom_emoji(
    pool: &sqlx::SqlitePool,
    emoji: &ProjectCustomEmojiCreate,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO project_custom_emojis (id, name, asset_path, sort_order)
         VALUES (?, ?, ?, ?)",
    )
    .bind(emoji.id.trim())
    .bind(emoji.name.trim())
    .bind(emoji.asset_path.trim())
    .bind(emoji.sort_order)
    .execute(pool)
    .await
    .map_err(|e| format!("create project custom emoji: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn projects_delete_custom_emoji<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    emoji_id: String,
) -> Result<(), String> {
    require_non_empty(&emoji_id, "emoji_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    delete_project_custom_emoji(&pool, &emoji_id).await
}

async fn delete_project_custom_emoji(
    pool: &sqlx::SqlitePool,
    emoji_id: &str,
) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM project_custom_emojis WHERE id = ?")
        .bind(emoji_id.trim())
        .execute(pool)
        .await
        .map_err(|e| format!("delete project custom emoji: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project custom emoji not found".to_string());
    }
    Ok(())
}

async fn next_task_sort_order(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    owner_column: &'static str,
    project_id: &str,
    owner_id: &str,
    sort_column: &'static str,
) -> Result<f64, String> {
    let sql = format!(
        "SELECT CAST(COALESCE(MAX({sort_column}), 0) + 1000 AS REAL)
         FROM project_tasks
         WHERE project_id = ? AND {owner_column} = ?"
    );
    sqlx::query_scalar::<_, f64>(&sql)
        .bind(project_id)
        .bind(owner_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("load next task sort order: {e}"))
}

async fn ensure_section_and_status_match_project(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    section_id: &str,
    status_id: &str,
) -> Result<(), String> {
    let section_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_sections WHERE id = ? AND project_id = ?")
            .bind(section_id)
            .bind(project_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("check project section: {e}"))?;
    if section_count == 0 {
        return Err("section does not belong to project".to_string());
    }
    let status_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_statuses WHERE id = ? AND project_id = ?")
            .bind(status_id)
            .bind(project_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("check project status: {e}"))?;
    if status_count == 0 {
        return Err("status does not belong to project".to_string());
    }
    Ok(())
}

async fn ensure_parent_task_matches_project(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    parent_task_id: &str,
) -> Result<(), String> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
             FROM project_tasks
             WHERE id = ? AND project_id = ? AND parent_task_id IS NULL",
    )
    .bind(parent_task_id)
    .bind(project_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("check parent project task: {e}"))?;
    if count == 0 {
        return Err("parent task must be a top-level task in the same project".to_string());
    }
    Ok(())
}

async fn ensure_task_has_no_children(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
) -> Result<(), String> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_tasks WHERE parent_task_id = ?")
            .bind(task_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("check child project tasks: {e}"))?;
    if count > 0 {
        return Err("task with subtasks cannot become a subtask".to_string());
    }
    Ok(())
}

async fn ensure_project_exists_in_pool(
    pool: &sqlx::SqlitePool,
    project_id: &str,
) -> Result<(), String> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE id = ?")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("check project: {e}"))?;
    if count == 0 {
        return Err("project not found".to_string());
    }
    Ok(())
}

async fn ensure_task_belongs_to_project_in_pool(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    project_id: &str,
) -> Result<(), String> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_tasks WHERE id = ? AND project_id = ?")
            .bind(task_id)
            .bind(project_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("check project task: {e}"))?;
    if count == 0 {
        return Err("project task does not belong to project".to_string());
    }
    Ok(())
}

async fn ensure_task_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
) -> Result<(), String> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_tasks WHERE id = ?")
        .bind(task_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("check project task: {e}"))?;
    if count == 0 {
        return Err("project task not found".to_string());
    }
    Ok(())
}

async fn project_id_for_task(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
) -> Result<String, String> {
    sqlx::query_scalar("SELECT project_id FROM project_tasks WHERE id = ?")
        .bind(task_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load project task: {e}"))?
        .ok_or_else(|| "project task not found".to_string())
}

async fn ensure_dependency_tasks_match_project(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    blocking_task_id: &str,
    blocked_task_id: &str,
) -> Result<(), String> {
    let blocking_project_id = project_id_for_task(tx, blocking_task_id).await?;
    let blocked_project_id = project_id_for_task(tx, blocked_task_id).await?;
    if blocking_project_id != blocked_project_id {
        return Err("dependency tasks must belong to the same project".to_string());
    }
    Ok(())
}

async fn ensure_dependency_has_no_cycle(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    blocking_task_id: &str,
    blocked_task_id: &str,
) -> Result<(), String> {
    let cycle_count: i64 = sqlx::query_scalar(
        "WITH RECURSIVE dependency_chain(task_id) AS (
             SELECT blocked_task_id
             FROM project_task_dependencies
             WHERE blocking_task_id = ?
             UNION
             SELECT project_task_dependencies.blocked_task_id
             FROM project_task_dependencies
             JOIN dependency_chain
               ON project_task_dependencies.blocking_task_id = dependency_chain.task_id
         )
         SELECT COUNT(*)
         FROM dependency_chain
         WHERE task_id = ?",
    )
    .bind(blocked_task_id)
    .bind(blocking_task_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("check project task dependency cycle: {e}"))?;
    if cycle_count > 0 {
        return Err("dependency would create a cycle".to_string());
    }
    Ok(())
}

async fn task_id_for_checklist_item(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: &str,
) -> Result<String, String> {
    sqlx::query_scalar("SELECT task_id FROM project_checklist_items WHERE id = ?")
        .bind(item_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load project checklist item: {e}"))?
        .ok_or_else(|| "project checklist item not found".to_string())
}

async fn label_for_task_label_link(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    label_id: &str,
) -> Result<String, String> {
    sqlx::query_scalar(
        "SELECT pl.name
         FROM project_labels pl
         JOIN project_tasks pt ON pt.project_id = pl.project_id
         WHERE pt.id = ? AND pl.id = ?",
    )
    .bind(task_id.trim())
    .bind(label_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project task label: {e}"))?
    .ok_or_else(|| "label must belong to the task project".to_string())
}

async fn label_for_existing_task_label_link(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    label_id: &str,
) -> Result<String, String> {
    sqlx::query_scalar(
        "SELECT pl.name
         FROM project_task_label_links ptl
         JOIN project_labels pl ON pl.id = ptl.label_id
         WHERE ptl.task_id = ? AND ptl.label_id = ?",
    )
    .bind(task_id.trim())
    .bind(label_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project task label link: {e}"))?
    .ok_or_else(|| "project task label link not found".to_string())
}

async fn delete_label_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    label_id: &str,
) -> Result<(), String> {
    let label = sqlx::query_as::<_, ProjectLabelRow>("SELECT * FROM project_labels WHERE id = ?")
        .bind(label_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load project label: {e}"))?
        .ok_or_else(|| "project label not found".to_string())?;
    let linked_task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id
         FROM project_task_label_links
         WHERE label_id = ?
         ORDER BY created_at ASC",
    )
    .bind(label_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load project label tasks: {e}"))?;
    sqlx::query("DELETE FROM project_labels WHERE id = ?")
        .bind(label_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project label: {e}"))?;
    for task_id in linked_task_ids {
        insert_task_change_event(
            tx,
            &task_id,
            "updated",
            Some("labels"),
            Some(&label.name),
            None,
        )
        .await?;
    }
    Ok(())
}

fn normalized_optional_identifier(value: Option<&str>) -> Option<String> {
    value.and_then(|identifier| {
        let trimmed = identifier.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;
    use sqlx::SqlitePool;

    async fn migrated_memory_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=ON")
            .execute(&pool)
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    async fn insert_project_graph_fixture(pool: &SqlitePool) {
        sqlx::query(
            "INSERT INTO project_groups (id, name, icon, sort_order)
             VALUES ('group-a', 'Group A', 'folder', 100)",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO projects (id, group_id, name, icon, sort_order)
             VALUES ('project-a', 'group-a', 'Project A', 'folder', 100)",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO project_sections (id, project_id, name, sort_order)
             VALUES ('section-a', 'project-a', 'General', 100)",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
             VALUES ('status-a', 'project-a', 'To do', 'not_started', 100, 0)",
        )
        .execute(pool)
        .await
        .unwrap();
        for task_id in ["task-a", "task-b", "task-c"] {
            sqlx::query(
                "INSERT INTO project_tasks (id, project_id, section_id, status_id, title)
                 VALUES (?, 'project-a', 'section-a', 'status-a', ?)",
            )
            .bind(task_id)
            .bind(task_id)
            .execute(pool)
            .await
            .unwrap();
        }
    }

    fn task_update_from_row(task: &ProjectTaskRow) -> ProjectTaskUpdate {
        ProjectTaskUpdate {
            id: task.id.clone(),
            section_id: task.section_id.clone(),
            status_id: task.status_id.clone(),
            parent_task_id: task.parent_task_id.clone(),
            title: task.title.clone(),
            description: task.description.clone(),
            priority: task.priority.clone(),
            task_type: task.task_type.clone(),
            section_sort_order: task.section_sort_order,
            status_sort_order: task.status_sort_order,
            estimate_minutes: task.estimate_minutes,
            due_date: task.due_date.clone(),
            due_time: task.due_time.clone(),
            start_date: task.start_date.clone(),
            start_time: task.start_time.clone(),
            target_end_date: task.target_end_date.clone(),
            archived_at: task.archived_at.clone(),
            blocker_reason: task.blocker_reason.clone(),
            milestone: task.milestone != 0,
            change_reason: None,
        }
    }

    #[test]
    fn next_task_sort_order_handles_empty_project() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            sqlx::query(
                "INSERT INTO project_groups (id, name, icon, sort_order)
                 VALUES ('group-empty', 'Group Empty', 'folder', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO projects (id, group_id, name, icon, sort_order)
                 VALUES ('project-empty', 'group-empty', 'Project Empty', 'folder', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_sections (id, project_id, name, sort_order)
                 VALUES ('section-empty', 'project-empty', 'General', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
                 VALUES ('status-empty', 'project-empty', 'To do', 'not_started', 100, 0)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let section_order = next_task_sort_order(
                &mut tx,
                "section_id",
                "project-empty",
                "section-empty",
                "section_sort_order",
            )
            .await;
            let status_order = next_task_sort_order(
                &mut tx,
                "status_id",
                "project-empty",
                "status-empty",
                "status_sort_order",
            )
            .await;

            assert_eq!(section_order, Ok(1000.0));
            assert_eq!(status_order, Ok(1000.0));
        });
    }

    #[test]
    fn delete_group_cascades_projects_and_keeps_calendar_events() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO calendar_events (id, title, start_time, end_time, project_id)
                 VALUES ('event-a', 'Event A', '2026-06-24T10:00:00Z', '2026-06-24T11:00:00Z', 'project-a')",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_task_event_links (task_id, event_id)
                 VALUES ('task-a', 'event-a')",
            )
            .execute(&pool)
            .await
            .unwrap();

            delete_project_group(&pool, " group-a ").await.unwrap();

            let group_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM project_groups WHERE id = 'group-a'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let project_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE id = 'project-a'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let task_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM project_tasks WHERE project_id = 'project-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let link_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM project_task_event_links WHERE event_id = 'event-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let event_project_id: Option<String> =
                sqlx::query_scalar("SELECT project_id FROM calendar_events WHERE id = 'event-a'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();

            assert_eq!(group_count, 0);
            assert_eq!(project_count, 0);
            assert_eq!(task_count, 0);
            assert_eq!(link_count, 0);
            assert_eq!(event_project_id, None);
        });
    }

    #[test]
    fn custom_emoji_create_list_delete_round_trips() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            let emoji = ProjectCustomEmojiCreate {
                id: " emoji-a ".to_string(),
                name: " Focus ".to_string(),
                asset_path: " project-icons/abcdef.png ".to_string(),
                sort_order: 200,
            };

            insert_project_custom_emoji(&pool, &emoji).await.unwrap();

            let emojis = load_project_custom_emojis(&pool).await.unwrap();
            assert_eq!(emojis.len(), 1);
            assert_eq!(emojis[0].id, "emoji-a");
            assert_eq!(emojis[0].name, "Focus");
            assert_eq!(emojis[0].asset_path, "project-icons/abcdef.png");
            assert_eq!(emojis[0].sort_order, 200);

            delete_project_custom_emoji(&pool, " emoji-a ")
                .await
                .unwrap();

            let emojis = load_project_custom_emojis(&pool).await.unwrap();
            assert!(emojis.is_empty());
            assert_eq!(
                delete_project_custom_emoji(&pool, "emoji-a").await,
                Err("project custom emoji not found".to_string())
            );
        });
    }

    #[test]
    fn dependency_cycle_detection_rejects_reverse_chain() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_task_dependencies
                    (id, blocking_task_id, blocked_task_id, dependency_type)
                 VALUES ('dependency-a-b', 'task-a', 'task-b', 'blocks')",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let result = ensure_dependency_has_no_cycle(&mut tx, "task-b", "task-a").await;

            assert_eq!(result, Err("dependency would create a cycle".to_string()));
        });
    }

    #[test]
    fn dependency_cycle_detection_allows_forward_chain() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_task_dependencies
                    (id, blocking_task_id, blocked_task_id, dependency_type)
                 VALUES ('dependency-a-b', 'task-a', 'task-b', 'blocks')",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let result = ensure_dependency_has_no_cycle(&mut tx, "task-c", "task-a").await;

            assert_eq!(result, Ok(()));
        });
    }

    #[test]
    fn task_label_link_rejects_labels_from_another_project() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO projects (id, group_id, name, icon, sort_order)
                 VALUES ('project-b', 'group-a', 'Project B', 'folder', 200)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_sections (id, project_id, name, sort_order)
                 VALUES ('section-b', 'project-b', 'General', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
                 VALUES ('status-b', 'project-b', 'To do', 'not_started', 100, 0)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_tasks (id, project_id, section_id, status_id, title)
                 VALUES ('task-other', 'project-b', 'section-b', 'status-b', 'Other task')",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_labels (id, project_id, name, sort_order)
                 VALUES ('label-a', 'project-a', 'Backend', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let result = label_for_task_label_link(&mut tx, "task-other", "label-a").await;

            assert_eq!(
                result,
                Err("label must belong to the task project".to_string())
            );
        });
    }

    #[test]
    fn task_event_link_assigns_missing_event_project() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO calendar_events (id, title, start_time, end_time)
                 VALUES ('event-a', 'Focus block', '2026-06-12T15:00:00Z', '2026-06-12T16:00:00Z')",
            )
            .execute(&pool)
            .await
            .unwrap();
            let link = ProjectTaskEventLinkCreate {
                task_id: "task-a".to_string(),
                event_id: "event-a".to_string(),
                link_kind: "scheduled".to_string(),
            };

            let mut tx = pool.begin().await.unwrap();
            link_task_event_with_project_assignment(&mut tx, &link)
                .await
                .unwrap();
            tx.commit().await.unwrap();

            let event_project_id: Option<String> =
                sqlx::query_scalar("SELECT project_id FROM calendar_events WHERE id = 'event-a'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let link_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM project_task_event_links
                 WHERE task_id = 'task-a' AND event_id = 'event-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let history_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM project_task_change_events
                 WHERE task_id = 'task-a'
                   AND event_type = 'scheduled'
                   AND field_name = 'event_id'
                   AND new_value = 'event-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(event_project_id.as_deref(), Some("project-a"));
            assert_eq!(link_count, 1);
            assert_eq!(history_count, 1);
        });
    }

    #[test]
    fn task_event_link_rejects_event_from_another_project() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO projects (id, group_id, name, icon, sort_order)
                 VALUES ('project-b', 'group-a', 'Project B', 'folder', 200)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO calendar_events (id, title, start_time, end_time, project_id)
                 VALUES (
                    'event-b',
                    'Other focus block',
                    '2026-06-12T15:00:00Z',
                    '2026-06-12T16:00:00Z',
                    'project-b'
                 )",
            )
            .execute(&pool)
            .await
            .unwrap();
            let link = ProjectTaskEventLinkCreate {
                task_id: "task-a".to_string(),
                event_id: "event-b".to_string(),
                link_kind: "scheduled".to_string(),
            };

            let mut tx = pool.begin().await.unwrap();
            let result = link_task_event_with_project_assignment(&mut tx, &link).await;

            assert_eq!(
                result,
                Err("calendar event does not belong to the task project".to_string())
            );
        });
    }

    #[test]
    fn delete_label_removes_links_and_records_task_history() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_labels (id, project_id, name, sort_order)
                 VALUES ('label-a', 'project-a', 'Backend', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();
            for task_id in ["task-a", "task-b"] {
                sqlx::query(
                    "INSERT INTO project_task_label_links (task_id, label_id)
                     VALUES (?, 'label-a')",
                )
                .bind(task_id)
                .execute(&pool)
                .await
                .unwrap();
            }

            let mut tx = pool.begin().await.unwrap();
            delete_label_with_history(&mut tx, "label-a").await.unwrap();
            tx.commit().await.unwrap();

            let label_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM project_labels WHERE id = 'label-a'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let link_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM project_task_label_links WHERE label_id = 'label-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let history_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM project_task_change_events
                 WHERE field_name = 'labels'
                   AND old_value = 'Backend'
                   AND new_value IS NULL",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(label_count, 0);
            assert_eq!(link_count, 0);
            assert_eq!(history_count, 2);
        });
    }

    #[test]
    fn task_update_records_trimmed_change_reason() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            let previous = sqlx::query_as::<_, ProjectTaskRow>(
                "SELECT * FROM project_tasks WHERE id = 'task-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let mut next = task_update_from_row(&previous);
            next.estimate_minutes = Some(45);
            next.change_reason = Some("  Scope changed  ".to_string());

            let mut tx = pool.begin().await.unwrap();
            insert_task_update_change_events(&mut tx, &previous, &next)
                .await
                .unwrap();
            tx.commit().await.unwrap();

            let reason: Option<String> = sqlx::query_scalar(
                "SELECT reason
                 FROM project_task_change_events
                 WHERE task_id = 'task-a'
                   AND event_type = 'updated'
                   AND field_name = 'estimate'
                   AND old_value IS NULL
                   AND new_value = '45'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(reason.as_deref(), Some("Scope changed"));
        });
    }

    #[test]
    fn task_update_rejects_oversized_change_reason() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            let previous = sqlx::query_as::<_, ProjectTaskRow>(
                "SELECT * FROM project_tasks WHERE id = 'task-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let mut task = task_update_from_row(&previous);
            task.change_reason = Some("x".repeat(MAX_TASK_CHANGE_REASON_LENGTH + 1));

            assert_eq!(
                validate_task_update(&task),
                Err("change_reason is too long".to_string())
            );
        });
    }

    #[test]
    fn task_update_validates_task_times() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            let previous = sqlx::query_as::<_, ProjectTaskRow>(
                "SELECT * FROM project_tasks WHERE id = 'task-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let mut task = task_update_from_row(&previous);

            task.start_time = Some("09:30".to_string());
            assert_eq!(
                validate_task_update(&task),
                Err("start_time requires start_date".to_string())
            );

            task.start_date = Some("2026-06-21".to_string());
            assert_eq!(validate_task_update(&task), Ok(()));

            task.due_date = Some("2026-06-22".to_string());
            task.due_time = Some("24:00".to_string());
            assert_eq!(
                validate_task_update(&task),
                Err("due_time must use HH:MM".to_string())
            );
        });
    }

    #[test]
    fn project_update_rejects_blank_automation_default_ids() {
        let project = ProjectUpdate {
            id: "project-a".to_string(),
            group_id: "group-a".to_string(),
            name: "Project A".to_string(),
            icon: "folder".to_string(),
            color: None,
            sort_order: 100,
            status: "active".to_string(),
            default_event_duration_minutes: Some(60),
            default_pomodoro_mode: "none".to_string(),
            default_pomodoro_preset_key: None,
            default_pomodoro_focus_minutes: None,
            default_pomodoro_short_break_minutes: None,
            default_pomodoro_long_break_minutes: None,
            default_pomodoro_long_break_after_focus_count: None,
            default_idle_timeout_minutes: None,
            focus_playlist_id: Some(" ".to_string()),
            break_playlist_id: None,
            work_environment_id: None,
            blocker_ruleset_id: None,
        };

        assert_eq!(
            validate_project_update(&project),
            Err("focus_playlist_id cannot be empty".to_string())
        );
    }

    #[test]
    fn custom_field_value_rejects_fields_from_another_project() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO projects (id, group_id, name, icon, sort_order)
                 VALUES ('project-b', 'group-a', 'Project B', 'folder', 200)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
                 VALUES ('field-b', 'project-b', 'Client note', 'text', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let value = ProjectCustomFieldValueUpdate {
                task_id: "task-a".to_string(),
                field_id: "field-b".to_string(),
                text_value: Some("Wrong project".to_string()),
                number_value: None,
                date_value: None,
                checkbox_value: None,
                option_ids: Vec::new(),
            };
            let mut tx = pool.begin().await.unwrap();
            let result = update_custom_field_value_with_history(&mut tx, &value).await;

            assert_eq!(
                result,
                Err("custom field must belong to the task project".to_string())
            );
        });
    }

    #[test]
    fn custom_field_value_rejects_options_from_another_field() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
                 VALUES
                    ('field-a', 'project-a', 'Phase', 'select', 100),
                    ('field-b', 'project-a', 'Risk', 'select', 200)",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO project_custom_field_options (id, field_id, name, sort_order)
                 VALUES ('option-b', 'field-b', 'High', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let value = ProjectCustomFieldValueUpdate {
                task_id: "task-a".to_string(),
                field_id: "field-a".to_string(),
                text_value: None,
                number_value: None,
                date_value: None,
                checkbox_value: None,
                option_ids: vec!["option-b".to_string()],
            };
            let mut tx = pool.begin().await.unwrap();
            let result = update_custom_field_value_with_history(&mut tx, &value).await;

            assert_eq!(
                result,
                Err("custom field option must belong to the field".to_string())
            );
        });
    }

    #[test]
    fn custom_field_value_records_task_history() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
                 VALUES ('field-a', 'project-a', 'Risk', 'text', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let value = ProjectCustomFieldValueUpdate {
                task_id: "task-a".to_string(),
                field_id: "field-a".to_string(),
                text_value: Some("High".to_string()),
                number_value: None,
                date_value: None,
                checkbox_value: None,
                option_ids: Vec::new(),
            };
            let mut tx = pool.begin().await.unwrap();
            update_custom_field_value_with_history(&mut tx, &value)
                .await
                .unwrap();
            tx.commit().await.unwrap();

            let stored_value: String = sqlx::query_scalar(
                "SELECT text_value
                 FROM project_custom_field_values
                 WHERE task_id = 'task-a' AND field_id = 'field-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let history_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM project_task_change_events
                 WHERE task_id = 'task-a'
                   AND event_type = 'updated'
                   AND field_name = 'custom_field:Risk'
                   AND old_value IS NULL
                   AND new_value = 'High'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(stored_value, "High");
            assert_eq!(history_count, 1);
        });
    }

    #[test]
    fn sql_like_contains_pattern_escapes_wildcards() {
        assert_eq!(sql_like_contains_pattern("100% done"), "%100\\% done%");
        assert_eq!(sql_like_contains_pattern("task_1"), "%task\\_1%");
        assert_eq!(sql_like_contains_pattern(r"c:\work"), r"%c:\\work%");
    }

    #[test]
    fn normalize_optional_date_filter_accepts_blank_and_iso_dates() {
        assert_eq!(normalize_optional_date_filter(None, "start_date"), Ok(None));
        assert_eq!(
            normalize_optional_date_filter(Some(" ".to_string()), "start_date"),
            Ok(None)
        );
        assert_eq!(
            normalize_optional_date_filter(Some("2026-06-12".to_string()), "start_date"),
            Ok(Some("2026-06-12".to_string()))
        );
    }

    #[test]
    fn normalize_optional_date_filter_rejects_malformed_dates() {
        assert_eq!(
            normalize_optional_date_filter(Some("2026-6-12".to_string()), "start_date"),
            Err("start_date must use YYYY-MM-DD".to_string())
        );
        assert_eq!(
            normalize_optional_date_filter(Some("2026/06/12".to_string()), "end_date"),
            Err("end_date must use YYYY-MM-DD".to_string())
        );
    }
}
