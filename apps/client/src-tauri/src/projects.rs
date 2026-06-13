use crate::db_path::connect_sqlite;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, Runtime};

const PALETTE_SIZE: i64 = 32;
const DEFAULT_STATUSES: &[(&str, &str, &str, i64, i64)] = &[
    ("backlog", "Backlog", "not_started", 0, 0),
    ("todo", "To do", "not_started", 10, 0),
    ("in-progress", "In progress", "active", 20, 0),
    ("blocked", "Blocked", "blocked", 30, 0),
    ("done", "Done", "done", 40, 1),
];
const PROJECT_TEMPLATE_IDS: &[&str] = &[
    "blank", "software", "course", "routine", "reading", "chores",
];
const MAX_TASK_CHANGE_REASON_LENGTH: usize = 1000;
const BLANK_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[("general", "General", 0)];
const SOFTWARE_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("planning", "Planning", 0),
    ("design", "Design", 10),
    ("implementation", "Implementation", 20),
    ("testing", "Testing", 30),
    ("release", "Release", 40),
];
const COURSE_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("general", "General", 0),
    ("assignments", "Assignments", 10),
    ("exams", "Exams", 20),
    ("reading", "Reading", 30),
];
const ROUTINE_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("morning", "Morning", 0),
    ("afternoon", "Afternoon", 10),
    ("evening", "Evening", 20),
];
const READING_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("to-read", "To read", 0),
    ("reading", "Reading", 10),
    ("finished", "Finished", 20),
];
const CHORES_TEMPLATE_SECTIONS: &[(&str, &str, i64)] = &[
    ("general", "General", 0),
    ("daily", "Daily", 10),
    ("weekly", "Weekly", 20),
    ("monthly", "Monthly", 30),
];

#[derive(Serialize)]
pub struct ProjectGroupRow {
    id: String,
    name: String,
    icon: String,
    color: Option<i64>,
    sort_order: i64,
    collapsed: i64,
    hidden_at: Option<String>,
    archived_at: Option<String>,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectGroupRow {
    id,
    name,
    icon,
    color,
    sort_order,
    collapsed,
    hidden_at,
    archived_at,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectRow {
    id: String,
    group_id: String,
    name: String,
    icon: String,
    color: Option<i64>,
    sort_order: i64,
    status: String,
    default_event_duration_minutes: i64,
    default_pomodoro_preset_key: Option<String>,
    default_idle_timeout_minutes: Option<i64>,
    focus_playlist_id: Option<String>,
    break_playlist_id: Option<String>,
    work_environment_id: Option<String>,
    blocker_ruleset_id: Option<String>,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectRow {
    id,
    group_id,
    name,
    icon,
    color,
    sort_order,
    status,
    default_event_duration_minutes,
    default_pomodoro_preset_key,
    default_idle_timeout_minutes,
    focus_playlist_id,
    break_playlist_id,
    work_environment_id,
    blocker_ruleset_id,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectSectionRow {
    id: String,
    project_id: String,
    name: String,
    sort_order: i64,
    collapsed: i64,
    hidden_at: Option<String>,
    archived_at: Option<String>,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectSectionRow {
    id,
    project_id,
    name,
    sort_order,
    collapsed,
    hidden_at,
    archived_at,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectStatusRow {
    id: String,
    project_id: String,
    name: String,
    category: String,
    sort_order: i64,
    terminal: i64,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectStatusRow {
    id,
    project_id,
    name,
    category,
    sort_order,
    terminal,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectTaskRow {
    id: String,
    project_id: String,
    section_id: String,
    status_id: String,
    parent_task_id: Option<String>,
    title: String,
    description: String,
    priority: String,
    task_type: String,
    section_sort_order: f64,
    status_sort_order: f64,
    estimate_minutes: Option<i64>,
    due_date: Option<String>,
    start_date: Option<String>,
    target_end_date: Option<String>,
    completed_at: Option<String>,
    archived_at: Option<String>,
    blocker_reason: Option<String>,
    milestone: i64,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectTaskRow {
    id,
    project_id,
    section_id,
    status_id,
    parent_task_id,
    title,
    description,
    priority,
    task_type,
    section_sort_order,
    status_sort_order,
    estimate_minutes,
    due_date,
    start_date,
    target_end_date,
    completed_at,
    archived_at,
    blocker_reason,
    milestone,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectChecklistItemRow {
    id: String,
    task_id: String,
    title: String,
    completed_at: Option<String>,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectChecklistItemRow {
    id,
    task_id,
    title,
    completed_at,
    sort_order,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectLabelRow {
    id: String,
    project_id: String,
    name: String,
    color: Option<i64>,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectLabelRow {
    id,
    project_id,
    name,
    color,
    sort_order,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectTaskLabelLinkRow {
    task_id: String,
    label_id: String,
    created_at: String,
}
impl_sqlite_from_row!(ProjectTaskLabelLinkRow {
    task_id,
    label_id,
    created_at,
});

#[derive(Serialize)]
pub struct ProjectCustomFieldRow {
    id: String,
    project_id: String,
    name: String,
    field_type: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectCustomFieldRow {
    id,
    project_id,
    name,
    field_type,
    sort_order,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectCustomFieldOptionRow {
    id: String,
    field_id: String,
    name: String,
    sort_order: i64,
    created_at: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectCustomFieldOptionRow {
    id,
    field_id,
    name,
    sort_order,
    created_at,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectCustomFieldValueRow {
    task_id: String,
    field_id: String,
    text_value: Option<String>,
    number_value: Option<f64>,
    date_value: Option<String>,
    checkbox_value: Option<i64>,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectCustomFieldValueRow {
    task_id,
    field_id,
    text_value,
    number_value,
    date_value,
    checkbox_value,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectCustomFieldOptionValueRow {
    task_id: String,
    field_id: String,
    option_id: String,
    created_at: String,
}
impl_sqlite_from_row!(ProjectCustomFieldOptionValueRow {
    task_id,
    field_id,
    option_id,
    created_at,
});

#[derive(Serialize)]
pub struct ProjectTaskDependencyRow {
    id: String,
    blocking_task_id: String,
    blocked_task_id: String,
    dependency_type: String,
    created_at: String,
}
impl_sqlite_from_row!(ProjectTaskDependencyRow {
    id,
    blocking_task_id,
    blocked_task_id,
    dependency_type,
    created_at,
});

#[derive(Serialize)]
pub struct ProjectTaskEventLinkRow {
    task_id: String,
    event_id: String,
    link_kind: String,
    created_at: String,
}
impl_sqlite_from_row!(ProjectTaskEventLinkRow {
    task_id,
    event_id,
    link_kind,
    created_at,
});

#[derive(Serialize)]
pub struct ProjectTaskChangeEventRow {
    id: String,
    task_id: String,
    event_type: String,
    field_name: Option<String>,
    old_value: Option<String>,
    new_value: Option<String>,
    reason: Option<String>,
    occurred_at: String,
}
impl_sqlite_from_row!(ProjectTaskChangeEventRow {
    id,
    task_id,
    event_type,
    field_name,
    old_value,
    new_value,
    reason,
    occurred_at,
});

#[derive(Serialize)]
pub struct ProjectViewPreferenceRow {
    project_id: String,
    view_id: String,
    preference_key: String,
    preference_value: String,
    updated_at: String,
}
impl_sqlite_from_row!(ProjectViewPreferenceRow {
    project_id,
    view_id,
    preference_key,
    preference_value,
    updated_at,
});

#[derive(Serialize)]
pub struct ProjectLinkableEventTask {
    task_id: String,
    title: String,
    archived_at: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectLinkableEvent {
    id: String,
    project_id: String,
    title: String,
    start_time: String,
    end_time: String,
    timezone: String,
    calendar_id: String,
    color: Option<i64>,
    all_day: i64,
    status: String,
    linked_tasks: Vec<ProjectLinkableEventTask>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLinkableEventSearch {
    project_id: String,
    task_id: Option<String>,
    query: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct ProjectLinkableEventRow {
    id: String,
    project_id: String,
    title: String,
    start_time: String,
    end_time: String,
    timezone: String,
    calendar_id: String,
    color: Option<i64>,
    all_day: i64,
    status: String,
}
impl_sqlite_from_row!(ProjectLinkableEventRow {
    id,
    project_id,
    title,
    start_time,
    end_time,
    timezone,
    calendar_id,
    color,
    all_day,
    status,
});

struct ProjectLinkableEventTaskRow {
    event_id: String,
    task_id: String,
    title: String,
    archived_at: Option<String>,
}
impl_sqlite_from_row!(ProjectLinkableEventTaskRow {
    event_id,
    task_id,
    title,
    archived_at,
});

#[derive(Serialize)]
pub struct ProjectsSnapshot {
    groups: Vec<ProjectGroupRow>,
    projects: Vec<ProjectRow>,
    sections: Vec<ProjectSectionRow>,
    statuses: Vec<ProjectStatusRow>,
    tasks: Vec<ProjectTaskRow>,
    checklist_items: Vec<ProjectChecklistItemRow>,
    labels: Vec<ProjectLabelRow>,
    task_label_links: Vec<ProjectTaskLabelLinkRow>,
    custom_fields: Vec<ProjectCustomFieldRow>,
    custom_field_options: Vec<ProjectCustomFieldOptionRow>,
    custom_field_values: Vec<ProjectCustomFieldValueRow>,
    custom_field_option_values: Vec<ProjectCustomFieldOptionValueRow>,
    dependencies: Vec<ProjectTaskDependencyRow>,
    event_links: Vec<ProjectTaskEventLinkRow>,
    task_change_events: Vec<ProjectTaskChangeEventRow>,
    view_preferences: Vec<ProjectViewPreferenceRow>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectGroupCreate {
    id: String,
    name: String,
    icon: String,
    color: Option<i64>,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectGroupUpdate {
    id: String,
    name: String,
    icon: String,
    color: Option<i64>,
    sort_order: i64,
    collapsed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreate {
    id: String,
    group_id: String,
    template_id: String,
    name: String,
    icon: String,
    color: Option<i64>,
    sort_order: i64,
    default_event_duration_minutes: i64,
    default_pomodoro_preset_key: Option<String>,
    default_idle_timeout_minutes: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdate {
    id: String,
    group_id: String,
    name: String,
    icon: String,
    color: Option<i64>,
    sort_order: i64,
    status: String,
    default_event_duration_minutes: i64,
    default_pomodoro_preset_key: Option<String>,
    default_idle_timeout_minutes: Option<i64>,
    focus_playlist_id: Option<String>,
    break_playlist_id: Option<String>,
    work_environment_id: Option<String>,
    blocker_ruleset_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSectionCreate {
    id: String,
    project_id: String,
    name: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSectionUpdate {
    id: String,
    name: String,
    sort_order: i64,
    collapsed: bool,
    hidden_at: Option<String>,
    archived_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatusCreate {
    id: String,
    project_id: String,
    name: String,
    category: String,
    sort_order: i64,
    terminal: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatusUpdate {
    id: String,
    name: String,
    category: String,
    sort_order: i64,
    terminal: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTaskCreate {
    id: String,
    project_id: String,
    section_id: String,
    status_id: String,
    parent_task_id: Option<String>,
    title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTaskUpdate {
    id: String,
    section_id: String,
    status_id: String,
    parent_task_id: Option<String>,
    title: String,
    description: String,
    priority: String,
    task_type: String,
    section_sort_order: f64,
    status_sort_order: f64,
    estimate_minutes: Option<i64>,
    due_date: Option<String>,
    start_date: Option<String>,
    target_end_date: Option<String>,
    archived_at: Option<String>,
    blocker_reason: Option<String>,
    milestone: bool,
    change_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTaskDependencyCreate {
    id: String,
    blocking_task_id: String,
    blocked_task_id: String,
    dependency_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChecklistItemCreate {
    id: String,
    task_id: String,
    title: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChecklistItemUpdate {
    id: String,
    title: String,
    completed: bool,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLabelCreate {
    id: String,
    project_id: String,
    name: String,
    color: Option<i64>,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLabelUpdate {
    id: String,
    name: String,
    color: Option<i64>,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTaskLabelLinkCreate {
    task_id: String,
    label_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCustomFieldCreate {
    id: String,
    project_id: String,
    name: String,
    field_type: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCustomFieldUpdate {
    id: String,
    name: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCustomFieldOptionCreate {
    id: String,
    field_id: String,
    name: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCustomFieldOptionUpdate {
    id: String,
    name: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCustomFieldValueUpdate {
    task_id: String,
    field_id: String,
    text_value: Option<String>,
    number_value: Option<f64>,
    date_value: Option<String>,
    checkbox_value: Option<bool>,
    option_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTaskEventLinkCreate {
    task_id: String,
    event_id: String,
    link_kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectViewPreferenceUpsert {
    project_id: String,
    view_id: String,
    preference_key: String,
    preference_value: String,
}

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
    })
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
            default_event_duration_minutes, default_pomodoro_preset_key,
            default_idle_timeout_minutes
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&project.id)
    .bind(&project.group_id)
    .bind(project.name.trim())
    .bind(project.icon.trim())
    .bind(project.color)
    .bind(project.sort_order)
    .bind(project.default_event_duration_minutes)
    .bind(&project.default_pomodoro_preset_key)
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
             default_pomodoro_preset_key = ?,
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
    .bind(&project.default_pomodoro_preset_key)
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
             start_date = ?,
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
    .bind(&task.start_date)
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
        &["list", "board", "calendar", "gantt", "summary"],
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

fn project_template_sections(template_id: &str) -> &'static [(&'static str, &'static str, i64)] {
    match template_id {
        "software" => SOFTWARE_TEMPLATE_SECTIONS,
        "course" => COURSE_TEMPLATE_SECTIONS,
        "routine" => ROUTINE_TEMPLATE_SECTIONS,
        "reading" => READING_TEMPLATE_SECTIONS,
        "chores" => CHORES_TEMPLATE_SECTIONS,
        _ => BLANK_TEMPLATE_SECTIONS,
    }
}

async fn insert_template_sections(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    template_id: &str,
) -> Result<(), String> {
    for (slug, name, sort_order) in project_template_sections(template_id.trim()) {
        sqlx::query(
            "INSERT INTO project_sections (id, project_id, name, sort_order)
             VALUES (?, ?, ?, ?)",
        )
        .bind(format!("section-{project_id}-{slug}"))
        .bind(project_id)
        .bind(name)
        .bind(sort_order)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("create project template section: {e}"))?;
    }
    Ok(())
}

async fn insert_default_statuses(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
) -> Result<(), String> {
    for (slug, name, category, sort_order, terminal) in DEFAULT_STATUSES {
        sqlx::query(
            "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("status-{project_id}-{slug}"))
        .bind(project_id)
        .bind(name)
        .bind(category)
        .bind(sort_order)
        .bind(terminal)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("create default project status: {e}"))?;
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
        "SELECT COALESCE(MAX({sort_column}), 0) + 1000
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

async fn custom_field_for_task_value(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    field_id: &str,
) -> Result<ProjectCustomFieldRow, String> {
    sqlx::query_as::<_, ProjectCustomFieldRow>(
        "SELECT cf.*
         FROM project_custom_fields cf
         JOIN project_tasks pt ON pt.project_id = cf.project_id
         WHERE pt.id = ? AND cf.id = ?",
    )
    .bind(task_id.trim())
    .bind(field_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project custom field: {e}"))?
    .ok_or_else(|| "custom field must belong to the task project".to_string())
}

async fn custom_field_by_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    field_id: &str,
) -> Result<ProjectCustomFieldRow, String> {
    sqlx::query_as::<_, ProjectCustomFieldRow>("SELECT * FROM project_custom_fields WHERE id = ?")
        .bind(field_id.trim())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load project custom field: {e}"))?
        .ok_or_else(|| "project custom field not found".to_string())
}

async fn custom_field_option_with_field(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    option_id: &str,
) -> Result<(ProjectCustomFieldOptionRow, ProjectCustomFieldRow), String> {
    let option = sqlx::query_as::<_, ProjectCustomFieldOptionRow>(
        "SELECT * FROM project_custom_field_options WHERE id = ?",
    )
    .bind(option_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project custom field option: {e}"))?
    .ok_or_else(|| "project custom field option not found".to_string())?;
    let field = custom_field_by_id(tx, &option.field_id).await?;
    Ok((option, field))
}

async fn ensure_custom_field_accepts_options_in_pool(
    pool: &sqlx::SqlitePool,
    field_id: &str,
) -> Result<(), String> {
    let field_type = sqlx::query_scalar::<_, String>(
        "SELECT field_type FROM project_custom_fields WHERE id = ?",
    )
    .bind(field_id.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load project custom field: {e}"))?
    .ok_or_else(|| "project custom field not found".to_string())?;
    if field_type != "select" && field_type != "multi_select" {
        return Err("custom field options require select or multi-select field".to_string());
    }
    Ok(())
}

async fn ensure_options_belong_to_field(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    field_id: &str,
    option_ids: &[String],
) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut unique_option_ids = Vec::new();
    for option_id in option_ids {
        let trimmed = option_id.trim().to_string();
        if seen.insert(trimmed.clone()) {
            unique_option_ids.push(trimmed);
        }
    }
    for option_id in &unique_option_ids {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM project_custom_field_options
             WHERE id = ? AND field_id = ?",
        )
        .bind(option_id)
        .bind(field_id.trim())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("load project custom field option: {e}"))?;
        if exists == 0 {
            return Err("custom field option must belong to the field".to_string());
        }
    }
    Ok(unique_option_ids)
}

fn custom_field_history_name(field: &ProjectCustomFieldRow) -> String {
    format!("custom_field:{}", field.name)
}

async fn custom_field_value_label(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    field: &ProjectCustomFieldRow,
) -> Result<Option<String>, String> {
    match field.field_type.as_str() {
        "text" | "url" => sqlx::query_scalar::<_, String>(
            "SELECT text_value
             FROM project_custom_field_values
             WHERE task_id = ? AND field_id = ?",
        )
        .bind(task_id)
        .bind(&field.id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load custom field text value: {e}")),
        "number" => {
            let value = sqlx::query_scalar::<_, f64>(
                "SELECT number_value
                 FROM project_custom_field_values
                 WHERE task_id = ? AND field_id = ?",
            )
            .bind(task_id)
            .bind(&field.id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load custom field number value: {e}"))?;
            Ok(value.map(|number| {
                if number.fract() == 0.0 {
                    format!("{number:.0}")
                } else {
                    number.to_string()
                }
            }))
        }
        "date" => sqlx::query_scalar::<_, String>(
            "SELECT date_value
             FROM project_custom_field_values
             WHERE task_id = ? AND field_id = ?",
        )
        .bind(task_id)
        .bind(&field.id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load custom field date value: {e}")),
        "checkbox" => {
            let value = sqlx::query_scalar::<_, i64>(
                "SELECT checkbox_value
                 FROM project_custom_field_values
                 WHERE task_id = ? AND field_id = ?",
            )
            .bind(task_id)
            .bind(&field.id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load custom field checkbox value: {e}"))?;
            Ok(value.map(|checked| bool_to_string(checked != 0)))
        }
        "select" | "multi_select" => {
            let option_names = sqlx::query_scalar::<_, String>(
                "SELECT opt.name
                 FROM project_custom_field_option_values val
                 JOIN project_custom_field_options opt ON opt.id = val.option_id
                 WHERE val.task_id = ? AND val.field_id = ?
                 ORDER BY opt.sort_order ASC, opt.name ASC",
            )
            .bind(task_id)
            .bind(&field.id)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| format!("load custom field option values: {e}"))?;
            if option_names.is_empty() {
                Ok(None)
            } else {
                Ok(Some(option_names.join(", ")))
            }
        }
        _ => Err("unsupported custom field type".to_string()),
    }
}

fn ensure_custom_field_value_payload_matches_type(
    field_type: &str,
    value: &ProjectCustomFieldValueUpdate,
) -> Result<(), String> {
    let has_text = value
        .text_value
        .as_ref()
        .is_some_and(|text| !text.trim().is_empty());
    let has_date = value
        .date_value
        .as_ref()
        .is_some_and(|date| !date.trim().is_empty());
    match field_type {
        "text" | "url" => {
            if value.number_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("text custom fields only accept text values".to_string());
            }
        }
        "number" => {
            if value.text_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("number custom fields only accept number values".to_string());
            }
        }
        "date" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.checkbox_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("date custom fields only accept date values".to_string());
            }
        }
        "checkbox" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.date_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("checkbox custom fields only accept checkbox values".to_string());
            }
        }
        "select" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
                || value.option_ids.len() > 1
            {
                return Err("select custom fields accept one option".to_string());
            }
        }
        "multi_select" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
            {
                return Err("multi-select custom fields only accept option values".to_string());
            }
        }
        _ => return Err("unsupported custom field type".to_string()),
    }
    if field_type != "text" && field_type != "url" && has_text {
        return Err("text_value is not valid for this custom field".to_string());
    }
    if field_type != "date" && has_date {
        return Err("date_value is not valid for this custom field".to_string());
    }
    Ok(())
}

async fn upsert_scalar_custom_field_value(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    field_id: &str,
    text_value: Option<&str>,
    number_value: Option<f64>,
    date_value: Option<&str>,
    checkbox_value: Option<i64>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO project_custom_field_values (
            task_id, field_id, text_value, number_value, date_value, checkbox_value
         )
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(task_id, field_id) DO UPDATE SET
            text_value = excluded.text_value,
            number_value = excluded.number_value,
            date_value = excluded.date_value,
            checkbox_value = excluded.checkbox_value,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(task_id)
    .bind(field_id)
    .bind(text_value)
    .bind(number_value)
    .bind(date_value)
    .bind(checkbox_value)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("update custom field value: {e}"))?;
    Ok(())
}

async fn update_custom_field_value_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    value: &ProjectCustomFieldValueUpdate,
) -> Result<(), String> {
    let field = custom_field_for_task_value(tx, &value.task_id, &value.field_id).await?;
    ensure_custom_field_value_payload_matches_type(&field.field_type, value)?;
    let old_value = custom_field_value_label(tx, &value.task_id, &field).await?;
    sqlx::query("DELETE FROM project_custom_field_values WHERE task_id = ? AND field_id = ?")
        .bind(value.task_id.trim())
        .bind(value.field_id.trim())
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("clear custom field value: {e}"))?;
    sqlx::query(
        "DELETE FROM project_custom_field_option_values WHERE task_id = ? AND field_id = ?",
    )
    .bind(value.task_id.trim())
    .bind(value.field_id.trim())
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("clear custom field option values: {e}"))?;

    match field.field_type.as_str() {
        "text" | "url" => {
            if let Some(text) = value.text_value.as_ref().map(|text| text.trim()) {
                if !text.is_empty() {
                    upsert_scalar_custom_field_value(
                        tx,
                        value.task_id.trim(),
                        value.field_id.trim(),
                        Some(text),
                        None,
                        None,
                        None,
                    )
                    .await?;
                }
            }
        }
        "number" => {
            if let Some(number) = value.number_value {
                upsert_scalar_custom_field_value(
                    tx,
                    value.task_id.trim(),
                    value.field_id.trim(),
                    None,
                    Some(number),
                    None,
                    None,
                )
                .await?;
            }
        }
        "date" => {
            if let Some(date) = value.date_value.as_ref().map(|date| date.trim()) {
                if !date.is_empty() {
                    validate_date(date, "date_value")?;
                    upsert_scalar_custom_field_value(
                        tx,
                        value.task_id.trim(),
                        value.field_id.trim(),
                        None,
                        None,
                        Some(date),
                        None,
                    )
                    .await?;
                }
            }
        }
        "checkbox" => {
            if let Some(checked) = value.checkbox_value {
                upsert_scalar_custom_field_value(
                    tx,
                    value.task_id.trim(),
                    value.field_id.trim(),
                    None,
                    None,
                    None,
                    Some(if checked { 1 } else { 0 }),
                )
                .await?;
            }
        }
        "select" | "multi_select" => {
            let option_ids =
                ensure_options_belong_to_field(tx, &field.id, &value.option_ids).await?;
            for option_id in option_ids {
                sqlx::query(
                    "INSERT INTO project_custom_field_option_values (task_id, field_id, option_id)
                     VALUES (?, ?, ?)",
                )
                .bind(value.task_id.trim())
                .bind(value.field_id.trim())
                .bind(option_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("update custom field option value: {e}"))?;
            }
        }
        _ => return Err("unsupported custom field type".to_string()),
    }

    let new_value = custom_field_value_label(tx, &value.task_id, &field).await?;
    if old_value != new_value {
        insert_task_change_event_owned(
            tx,
            value.task_id.trim(),
            "updated",
            &custom_field_history_name(&field),
            old_value,
            new_value,
            None,
        )
        .await?;
    }
    Ok(())
}

async fn delete_custom_field_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    field_id: &str,
) -> Result<(), String> {
    let field = custom_field_by_id(tx, field_id).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id FROM project_custom_field_values WHERE field_id = ?
         UNION
         SELECT task_id FROM project_custom_field_option_values WHERE field_id = ?",
    )
    .bind(field_id)
    .bind(field_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load custom field task values: {e}"))?;
    let mut old_values = Vec::new();
    for task_id in task_ids {
        let old_value = custom_field_value_label(tx, &task_id, &field).await?;
        old_values.push((task_id, old_value));
    }
    sqlx::query("DELETE FROM project_custom_fields WHERE id = ?")
        .bind(field_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project custom field: {e}"))?;
    for (task_id, old_value) in old_values {
        if old_value.is_some() {
            insert_task_change_event_owned(
                tx,
                &task_id,
                "updated",
                &custom_field_history_name(&field),
                old_value,
                None,
                None,
            )
            .await?;
        }
    }
    Ok(())
}

async fn delete_custom_field_option_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    option_id: &str,
) -> Result<(), String> {
    let (_option, field) = custom_field_option_with_field(tx, option_id).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id
         FROM project_custom_field_option_values
         WHERE option_id = ?
         ORDER BY created_at ASC",
    )
    .bind(option_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load custom field option tasks: {e}"))?;
    let mut old_values = Vec::new();
    for task_id in task_ids {
        let old_value = custom_field_value_label(tx, &task_id, &field).await?;
        old_values.push((task_id, old_value));
    }
    sqlx::query("DELETE FROM project_custom_field_options WHERE id = ?")
        .bind(option_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project custom field option: {e}"))?;
    for (task_id, old_value) in old_values {
        let new_value = custom_field_value_label(tx, &task_id, &field).await?;
        if old_value != new_value {
            insert_task_change_event_owned(
                tx,
                &task_id,
                "updated",
                &custom_field_history_name(&field),
                old_value,
                new_value,
                None,
            )
            .await?;
        }
    }
    Ok(())
}

async fn status_is_terminal(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    status_id: &str,
) -> Result<bool, String> {
    let terminal: i64 = sqlx::query_scalar("SELECT terminal FROM project_statuses WHERE id = ?")
        .bind(status_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("load project status: {e}"))?;
    Ok(terminal != 0)
}

async fn current_timestamp(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<String, String> {
    sqlx::query_scalar::<_, String>("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')")
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("load timestamp: {e}"))
}

async fn label_for_project_status(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    status_id: &str,
) -> Result<String, String> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT name FROM project_statuses WHERE id = ?")
            .bind(status_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load project status label: {e}"))?
            .unwrap_or_else(|| status_id.to_string()),
    )
}

async fn label_for_project_section(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    section_id: &str,
) -> Result<String, String> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT name FROM project_sections WHERE id = ?")
            .bind(section_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load project section label: {e}"))?
            .unwrap_or_else(|| section_id.to_string()),
    )
}

async fn label_for_project_task(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
) -> Result<String, String> {
    Ok(
        sqlx::query_scalar::<_, String>("SELECT title FROM project_tasks WHERE id = ?")
            .bind(task_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load project task label: {e}"))?
            .unwrap_or_else(|| task_id.to_string()),
    )
}

async fn optional_task_label(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: Option<&str>,
) -> Result<Option<String>, String> {
    match task_id {
        Some(value) => Ok(Some(label_for_project_task(tx, value).await?)),
        None => Ok(None),
    }
}

async fn insert_task_change_event_owned(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    event_type: &str,
    field_name: &str,
    old_value: Option<String>,
    new_value: Option<String>,
    reason: Option<&str>,
) -> Result<(), String> {
    insert_task_change_event_with_reason(
        tx,
        task_id,
        event_type,
        Some(field_name),
        old_value.as_deref(),
        new_value.as_deref(),
        reason,
    )
    .await
}

fn optional_i64_to_string(value: Option<i64>) -> Option<String> {
    value.map(|number| number.to_string())
}

fn bool_to_string(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn normalized_task_change_reason(value: Option<&str>) -> Option<String> {
    value.and_then(|reason| {
        let trimmed = reason.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
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

async fn insert_task_update_change_events(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    previous: &ProjectTaskRow,
    next: &ProjectTaskUpdate,
) -> Result<(), String> {
    let task_id = next.id.as_str();
    let next_title = next.title.trim().to_string();
    let change_reason = normalized_task_change_reason(next.change_reason.as_deref());
    if previous.title != next_title {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "title",
            Some(previous.title.clone()),
            Some(next_title),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.description != next.description {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "description",
            Some(previous.description.clone()),
            Some(next.description.clone()),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.status_id != next.status_id {
        let previous_terminal = status_is_terminal(tx, &previous.status_id).await?;
        let next_terminal = status_is_terminal(tx, &next.status_id).await?;
        let event_type = if !previous_terminal && next_terminal {
            "completed"
        } else if previous_terminal && !next_terminal {
            "reopened"
        } else {
            "status_changed"
        };
        let old_status = label_for_project_status(tx, &previous.status_id).await?;
        let new_status = label_for_project_status(tx, &next.status_id).await?;
        insert_task_change_event_owned(
            tx,
            task_id,
            event_type,
            "status",
            Some(old_status),
            Some(new_status),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.section_id != next.section_id {
        let old_section = label_for_project_section(tx, &previous.section_id).await?;
        let new_section = label_for_project_section(tx, &next.section_id).await?;
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "section",
            Some(old_section),
            Some(new_section),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.parent_task_id != next.parent_task_id {
        let old_parent = optional_task_label(tx, previous.parent_task_id.as_deref()).await?;
        let new_parent = optional_task_label(tx, next.parent_task_id.as_deref()).await?;
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "parent",
            old_parent,
            new_parent,
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.priority != next.priority {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "priority",
            Some(previous.priority.clone()),
            Some(next.priority.clone()),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.task_type != next.task_type {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "type",
            Some(previous.task_type.clone()),
            Some(next.task_type.clone()),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.estimate_minutes != next.estimate_minutes {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "estimate",
            optional_i64_to_string(previous.estimate_minutes),
            optional_i64_to_string(next.estimate_minutes),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.due_date != next.due_date {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "due_date",
            previous.due_date.clone(),
            next.due_date.clone(),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.start_date != next.start_date {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "start_date",
            previous.start_date.clone(),
            next.start_date.clone(),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.target_end_date != next.target_end_date {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "target_date",
            previous.target_end_date.clone(),
            next.target_end_date.clone(),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.archived_at != next.archived_at {
        let event_type = if previous.archived_at.is_none() && next.archived_at.is_some() {
            "archived"
        } else {
            "updated"
        };
        insert_task_change_event_owned(
            tx,
            task_id,
            event_type,
            "archived_at",
            previous.archived_at.clone(),
            next.archived_at.clone(),
            change_reason.as_deref(),
        )
        .await?;
    }
    if previous.blocker_reason != next.blocker_reason {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "blocker_reason",
            previous.blocker_reason.clone(),
            next.blocker_reason.clone(),
            change_reason.as_deref(),
        )
        .await?;
    }
    if (previous.milestone != 0) != next.milestone {
        insert_task_change_event_owned(
            tx,
            task_id,
            "updated",
            "milestone",
            Some(bool_to_string(previous.milestone != 0)),
            Some(bool_to_string(next.milestone)),
            change_reason.as_deref(),
        )
        .await?;
    }
    Ok(())
}

async fn insert_task_change_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    event_type: &str,
    field_name: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
) -> Result<(), String> {
    insert_task_change_event_with_reason(
        tx, task_id, event_type, field_name, old_value, new_value, None,
    )
    .await
}

async fn insert_task_change_event_with_reason(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    event_type: &str,
    field_name: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    reason: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO project_task_change_events
            (id, task_id, event_type, field_name, old_value, new_value, reason)
         VALUES (lower(hex(randomblob(16))), ?, ?, ?, ?, ?, ?)",
    )
    .bind(task_id)
    .bind(event_type)
    .bind(field_name)
    .bind(old_value)
    .bind(new_value)
    .bind(reason)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert project task change event: {e}"))?;
    Ok(())
}

fn validate_group_create(group: &ProjectGroupCreate) -> Result<(), String> {
    require_non_empty(&group.id, "id")?;
    require_non_empty(&group.name, "name")?;
    require_non_empty(&group.icon, "icon")?;
    validate_color(group.color)?;
    validate_non_negative(group.sort_order, "sort_order")
}

fn validate_group_update(group: &ProjectGroupUpdate) -> Result<(), String> {
    require_non_empty(&group.id, "id")?;
    require_non_empty(&group.name, "name")?;
    require_non_empty(&group.icon, "icon")?;
    validate_color(group.color)?;
    validate_non_negative(group.sort_order, "sort_order")
}

fn validate_project_create(project: &ProjectCreate) -> Result<(), String> {
    require_non_empty(&project.id, "id")?;
    require_non_empty(&project.group_id, "group_id")?;
    validate_enum(&project.template_id, "template_id", PROJECT_TEMPLATE_IDS)?;
    require_non_empty(&project.name, "name")?;
    require_non_empty(&project.icon, "icon")?;
    validate_color(project.color)?;
    validate_non_negative(project.sort_order, "sort_order")?;
    if project.default_event_duration_minutes <= 0 {
        return Err("default_event_duration_minutes must be positive".to_string());
    }
    validate_pomodoro_preset(project.default_pomodoro_preset_key.as_deref())?;
    if project
        .default_idle_timeout_minutes
        .is_some_and(|value| value <= 0)
    {
        return Err("default_idle_timeout_minutes must be positive".to_string());
    }
    Ok(())
}

fn validate_project_update(project: &ProjectUpdate) -> Result<(), String> {
    require_non_empty(&project.id, "id")?;
    require_non_empty(&project.group_id, "group_id")?;
    require_non_empty(&project.name, "name")?;
    require_non_empty(&project.icon, "icon")?;
    validate_color(project.color)?;
    validate_non_negative(project.sort_order, "sort_order")?;
    validate_enum(&project.status, "status", &["active", "hidden", "archived"])?;
    if project.default_event_duration_minutes <= 0 {
        return Err("default_event_duration_minutes must be positive".to_string());
    }
    validate_pomodoro_preset(project.default_pomodoro_preset_key.as_deref())?;
    if project
        .default_idle_timeout_minutes
        .is_some_and(|value| value <= 0)
    {
        return Err("default_idle_timeout_minutes must be positive".to_string());
    }
    validate_optional_identifier(&project.focus_playlist_id, "focus_playlist_id")?;
    validate_optional_identifier(&project.break_playlist_id, "break_playlist_id")?;
    validate_optional_identifier(&project.work_environment_id, "work_environment_id")?;
    validate_optional_identifier(&project.blocker_ruleset_id, "blocker_ruleset_id")?;
    Ok(())
}

fn validate_section_create(section: &ProjectSectionCreate) -> Result<(), String> {
    require_non_empty(&section.id, "id")?;
    require_non_empty(&section.project_id, "project_id")?;
    require_non_empty(&section.name, "name")?;
    validate_non_negative(section.sort_order, "sort_order")
}

fn validate_section_update(section: &ProjectSectionUpdate) -> Result<(), String> {
    require_non_empty(&section.id, "id")?;
    require_non_empty(&section.name, "name")?;
    validate_non_negative(section.sort_order, "sort_order")
}

fn validate_status_create(status: &ProjectStatusCreate) -> Result<(), String> {
    require_non_empty(&status.id, "id")?;
    require_non_empty(&status.project_id, "project_id")?;
    validate_status_fields(
        &status.name,
        &status.category,
        status.sort_order,
        status.terminal,
    )
}

fn validate_status_update(status: &ProjectStatusUpdate) -> Result<(), String> {
    require_non_empty(&status.id, "id")?;
    validate_status_fields(
        &status.name,
        &status.category,
        status.sort_order,
        status.terminal,
    )
}

fn validate_status_fields(
    name: &str,
    category: &str,
    sort_order: i64,
    terminal: bool,
) -> Result<(), String> {
    require_non_empty(name, "name")?;
    validate_enum(
        category,
        "category",
        &["not_started", "active", "blocked", "done"],
    )?;
    validate_non_negative(sort_order, "sort_order")?;
    if (category == "done") != terminal {
        return Err(
            "done statuses must be terminal and terminal statuses must use done category"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_task_create(task: &ProjectTaskCreate) -> Result<(), String> {
    require_non_empty(&task.id, "id")?;
    require_non_empty(&task.project_id, "project_id")?;
    require_non_empty(&task.section_id, "section_id")?;
    require_non_empty(&task.status_id, "status_id")?;
    require_non_empty(&task.title, "title")?;
    if let Some(parent_task_id) = &task.parent_task_id {
        require_non_empty(parent_task_id, "parent_task_id")?;
        if parent_task_id == &task.id {
            return Err("task cannot be its own parent".to_string());
        }
    }
    Ok(())
}

fn validate_optional_identifier(value: &Option<String>, field: &str) -> Result<(), String> {
    if let Some(identifier) = value {
        require_non_empty(identifier, field)?;
    }
    Ok(())
}

fn validate_task_update(task: &ProjectTaskUpdate) -> Result<(), String> {
    require_non_empty(&task.id, "id")?;
    require_non_empty(&task.section_id, "section_id")?;
    require_non_empty(&task.status_id, "status_id")?;
    require_non_empty(&task.title, "title")?;
    validate_enum(
        &task.priority,
        "priority",
        &["low", "normal", "high", "urgent"],
    )?;
    validate_enum(
        &task.task_type,
        "task_type",
        &["task", "milestone", "bug", "habit"],
    )?;
    if task.estimate_minutes.is_some_and(|value| value <= 0) {
        return Err("estimate_minutes must be positive".to_string());
    }
    if task.section_sort_order < 0.0 {
        return Err("section_sort_order must be non-negative".to_string());
    }
    if task.status_sort_order < 0.0 {
        return Err("status_sort_order must be non-negative".to_string());
    }
    if let (Some(start_date), Some(target_end_date)) = (&task.start_date, &task.target_end_date) {
        if start_date > target_end_date {
            return Err("start_date must be before target_end_date".to_string());
        }
    }
    if task
        .change_reason
        .as_deref()
        .map(str::trim)
        .is_some_and(|reason| reason.len() > MAX_TASK_CHANGE_REASON_LENGTH)
    {
        return Err("change_reason is too long".to_string());
    }
    Ok(())
}

fn validate_task_dependency_create(dependency: &ProjectTaskDependencyCreate) -> Result<(), String> {
    require_non_empty(&dependency.id, "id")?;
    require_non_empty(&dependency.blocking_task_id, "blocking_task_id")?;
    require_non_empty(&dependency.blocked_task_id, "blocked_task_id")?;
    validate_enum(&dependency.dependency_type, "dependency_type", &["blocks"])?;
    if dependency.blocking_task_id == dependency.blocked_task_id {
        return Err("task cannot depend on itself".to_string());
    }
    Ok(())
}

fn validate_checklist_item_create(item: &ProjectChecklistItemCreate) -> Result<(), String> {
    require_non_empty(&item.id, "id")?;
    require_non_empty(&item.task_id, "task_id")?;
    require_non_empty(&item.title, "title")?;
    validate_non_negative(item.sort_order, "sort_order")
}

fn validate_checklist_item_update(item: &ProjectChecklistItemUpdate) -> Result<(), String> {
    require_non_empty(&item.id, "id")?;
    require_non_empty(&item.title, "title")?;
    validate_non_negative(item.sort_order, "sort_order")
}

fn validate_label_create(label: &ProjectLabelCreate) -> Result<(), String> {
    require_non_empty(&label.id, "id")?;
    require_non_empty(&label.project_id, "project_id")?;
    require_non_empty(&label.name, "name")?;
    validate_color(label.color)?;
    validate_non_negative(label.sort_order, "sort_order")
}

fn validate_label_update(label: &ProjectLabelUpdate) -> Result<(), String> {
    require_non_empty(&label.id, "id")?;
    require_non_empty(&label.name, "name")?;
    validate_color(label.color)?;
    validate_non_negative(label.sort_order, "sort_order")
}

fn validate_task_label_link_create(link: &ProjectTaskLabelLinkCreate) -> Result<(), String> {
    require_non_empty(&link.task_id, "task_id")?;
    require_non_empty(&link.label_id, "label_id")
}

fn validate_custom_field_create(field: &ProjectCustomFieldCreate) -> Result<(), String> {
    require_non_empty(&field.id, "id")?;
    require_non_empty(&field.project_id, "project_id")?;
    require_non_empty(&field.name, "name")?;
    validate_custom_field_type(&field.field_type)?;
    validate_non_negative(field.sort_order, "sort_order")
}

fn validate_custom_field_update(field: &ProjectCustomFieldUpdate) -> Result<(), String> {
    require_non_empty(&field.id, "id")?;
    require_non_empty(&field.name, "name")?;
    validate_non_negative(field.sort_order, "sort_order")
}

fn validate_custom_field_option_create(
    option: &ProjectCustomFieldOptionCreate,
) -> Result<(), String> {
    require_non_empty(&option.id, "id")?;
    require_non_empty(&option.field_id, "field_id")?;
    require_non_empty(&option.name, "name")?;
    validate_non_negative(option.sort_order, "sort_order")
}

fn validate_custom_field_option_update(
    option: &ProjectCustomFieldOptionUpdate,
) -> Result<(), String> {
    require_non_empty(&option.id, "id")?;
    require_non_empty(&option.name, "name")?;
    validate_non_negative(option.sort_order, "sort_order")
}

fn validate_custom_field_value_update(value: &ProjectCustomFieldValueUpdate) -> Result<(), String> {
    require_non_empty(&value.task_id, "task_id")?;
    require_non_empty(&value.field_id, "field_id")?;
    for option_id in &value.option_ids {
        require_non_empty(option_id, "option_id")?;
    }
    if value.number_value.is_some_and(|number| !number.is_finite()) {
        return Err("number_value must be finite".to_string());
    }
    if let Some(date_value) = &value.date_value {
        validate_date(date_value, "date_value")?;
    }
    Ok(())
}

fn validate_custom_field_type(field_type: &str) -> Result<(), String> {
    validate_enum(
        field_type,
        "field_type",
        &[
            "text",
            "number",
            "date",
            "select",
            "multi_select",
            "checkbox",
            "url",
        ],
    )
}

fn validate_task_event_link_create(link: &ProjectTaskEventLinkCreate) -> Result<(), String> {
    require_non_empty(&link.task_id, "task_id")?;
    require_non_empty(&link.event_id, "event_id")?;
    validate_enum(&link.link_kind, "link_kind", &["scheduled", "reference"])
}

fn validate_view_preference(preference: &ProjectViewPreferenceUpsert) -> Result<(), String> {
    require_non_empty(&preference.project_id, "project_id")?;
    validate_enum(
        &preference.view_id,
        "view_id",
        &["list", "board", "calendar", "gantt", "summary"],
    )?;
    require_non_empty(&preference.preference_key, "preference_key")?;
    if preference.preference_value.len() > 20_000 {
        return Err("preference_value is too large".to_string());
    }
    Ok(())
}

fn validate_color(value: Option<i64>) -> Result<(), String> {
    if value.is_some_and(|color| !(0..PALETTE_SIZE).contains(&color)) {
        return Err("color is outside the event palette".to_string());
    }
    Ok(())
}

fn validate_pomodoro_preset(value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        validate_enum(
            value,
            "default_pomodoro_preset_key",
            &["adaptive", "creative", "balanced", "deep", "extended"],
        )?;
    }
    Ok(())
}

fn validate_enum(value: &str, field: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("{field} has unsupported value '{value}'"))
    }
}

fn validate_non_negative(value: i64, field: &str) -> Result<(), String> {
    if value < 0 {
        Err(format!("{field} must be non-negative"))
    } else {
        Ok(())
    }
}

fn require_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} cannot be empty"))
    } else {
        Ok(())
    }
}

fn validate_date(value: &str, field: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.len() != 10 {
        return Err(format!("{field} must use YYYY-MM-DD"));
    }
    let bytes = trimmed.as_bytes();
    let valid_shape = bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !valid_shape {
        return Err(format!("{field} must use YYYY-MM-DD"));
    }
    Ok(())
}

fn normalize_optional_date_filter(
    value: Option<String>,
    field: &str,
) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    validate_date(trimmed, field)?;
    Ok(Some(trimmed.to_string()))
}

fn sql_like_contains_pattern(value: &str) -> String {
    let mut escaped = String::from("%");
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '%' => escaped.push_str("\\%"),
            '_' => escaped.push_str("\\_"),
            _ => escaped.push(character),
        }
    }
    escaped.push('%');
    escaped
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
            start_date: task.start_date.clone(),
            target_end_date: task.target_end_date.clone(),
            archived_at: task.archived_at.clone(),
            blocker_reason: task.blocker_reason.clone(),
            milestone: task.milestone != 0,
            change_reason: None,
        }
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
    fn project_update_rejects_blank_automation_default_ids() {
        let project = ProjectUpdate {
            id: "project-a".to_string(),
            group_id: "group-a".to_string(),
            name: "Project A".to_string(),
            icon: "folder".to_string(),
            color: None,
            sort_order: 100,
            status: "active".to_string(),
            default_event_duration_minutes: 60,
            default_pomodoro_preset_key: None,
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
