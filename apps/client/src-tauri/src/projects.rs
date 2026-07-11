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
const PROJECT_EVENT_TIME_MODES: &[&str] = &["timed", "all_day"];
const PROJECT_IDLE_SETTINGS_SOURCES: &[&str] = &["global", "custom"];
const PROJECT_IDLE_THRESHOLD_MINUTES: &[i64] = &[1, 2, 3, 4, 5, 10, 15];
const NOTES_PAGE_OPEN_MODES: &[&str] = &["center", "side", "full"];
const NOTES_HISTORY_RETENTION_DAYS: &[i64] = &[0, 7, 30, 90, 180, 365];
const MAX_TASK_CHANGE_REASON_LENGTH: usize = 1000;
const ROUTINE_GROUP_ID: &str = "group-routine";

#[derive(Clone, Copy)]
struct BuiltInRoutineProject {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    color: i64,
    sort_order: i64,
    default_pomodoro_mode: &'static str,
    default_pomodoro_preset_key: Option<&'static str>,
}

const BUILT_IN_ROUTINE_PROJECTS: &[BuiltInRoutineProject] = &[
    BuiltInRoutineProject {
        id: "project-routine-learning",
        name: "Learning",
        icon: "graduation-cap",
        color: 8,
        sort_order: 0,
        default_pomodoro_mode: "preset",
        default_pomodoro_preset_key: Some("adaptive"),
    },
    BuiltInRoutineProject {
        id: "project-routine-reading",
        name: "Reading",
        icon: "book-open",
        color: 25,
        sort_order: 10,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-exercise",
        name: "Exercise",
        icon: "sport-shoe",
        color: 0,
        sort_order: 20,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-hygiene",
        name: "Hygiene",
        icon: "bath",
        color: 15,
        sort_order: 30,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-eat",
        name: "Eating",
        icon: "apple",
        color: 13,
        sort_order: 40,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-commute",
        name: "Commute",
        icon: "bike",
        color: 17,
        sort_order: 50,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-social",
        name: "Social",
        icon: "heart",
        color: 21,
        sort_order: 60,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-chores",
        name: "Chores",
        icon: "shopping-cart",
        color: 4,
        sort_order: 70,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-leisure",
        name: "Leisure",
        icon: "clapperboard",
        color: 31,
        sort_order: 80,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-meditate",
        name: "Meditate",
        icon: "smile",
        color: 23,
        sort_order: 90,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-health",
        name: "Health",
        icon: "pill",
        color: 3,
        sort_order: 100,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
    BuiltInRoutineProject {
        id: "project-routine-sleep",
        name: "Sleep",
        icon: "bed",
        color: 30,
        sort_order: 110,
        default_pomodoro_mode: "none",
        default_pomodoro_preset_key: None,
    },
];

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
use templates::{
    insert_default_priorities, insert_default_project_graphs, insert_default_statuses,
    insert_template_sections,
};
use validation::*;

fn built_in_routine_project(project_id: &str) -> Option<&'static BuiltInRoutineProject> {
    BUILT_IN_ROUTINE_PROJECTS
        .iter()
        .find(|project| project.id == project_id)
}

async fn built_in_routine_defaults_intact(pool: &sqlx::SqlitePool) -> Result<bool, String> {
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT EXISTS(SELECT 1 FROM project_groups WHERE id = ",
    );
    query
        .push_bind(ROUTINE_GROUP_ID)
        .push(" AND name = 'Routine') AND (SELECT COUNT(*) FROM projects WHERE ");
    for (index, project) in BUILT_IN_ROUTINE_PROJECTS.iter().enumerate() {
        if index > 0 {
            query.push(" OR ");
        }
        query
            .push("(id = ")
            .push_bind(project.id)
            .push(" AND group_id = ")
            .push_bind(ROUTINE_GROUP_ID)
            .push(" AND name = ")
            .push_bind(project.name)
            .push(" AND sort_order = ")
            .push_bind(project.sort_order)
            .push(")");
    }
    query
        .push(") = ")
        .push_bind(BUILT_IN_ROUTINE_PROJECTS.len() as i64);
    query
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await
        .map(|value| value != 0)
        .map_err(|e| format!("check built-in Routine integrity: {e}"))
}

async fn repair_built_in_routine_defaults(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin built-in Routine repair: {e}"))?;

    let mut existing_query =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT id FROM projects WHERE id IN (");
    {
        let mut separated = existing_query.separated(", ");
        for project in BUILT_IN_ROUTINE_PROJECTS {
            separated.push_bind(project.id);
        }
    }
    existing_query.push(")");
    let existing_ids: HashSet<String> = existing_query
        .build_query_scalar()
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| format!("load existing built-in Routine projects: {e}"))?
        .into_iter()
        .collect();
    let missing_project_ids: Vec<&str> = BUILT_IN_ROUTINE_PROJECTS
        .iter()
        .filter(|project| !existing_ids.contains(project.id))
        .map(|project| project.id)
        .collect();

    sqlx::query(
        "INSERT OR IGNORE INTO project_groups (id, name, icon, color, sort_order)
         VALUES (?, 'Routine', 'repeat', 0, 0)",
    )
    .bind(ROUTINE_GROUP_ID)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("restore built-in Routine group: {e}"))?;
    sqlx::query(
        "UPDATE project_groups
         SET name = 'Routine'
         WHERE id = ? AND name <> 'Routine'",
    )
    .bind(ROUTINE_GROUP_ID)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("normalize built-in Routine group: {e}"))?;

    let mut insert_projects = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "INSERT OR IGNORE INTO projects (id, group_id, name, icon, color, sort_order, default_pomodoro_mode, default_pomodoro_preset_key) ",
    );
    insert_projects.push_values(BUILT_IN_ROUTINE_PROJECTS, |mut row, project| {
        row.push_bind(project.id)
            .push_bind(ROUTINE_GROUP_ID)
            .push_bind(project.name)
            .push_bind(project.icon)
            .push_bind(project.color)
            .push_bind(project.sort_order)
            .push_bind(project.default_pomodoro_mode)
            .push_bind(project.default_pomodoro_preset_key);
    });
    insert_projects
        .build()
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("restore built-in Routine projects: {e}"))?;

    let mut normalize_projects =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("UPDATE projects SET group_id = ");
    normalize_projects
        .push_bind(ROUTINE_GROUP_ID)
        .push(", name = CASE id ");
    for project in BUILT_IN_ROUTINE_PROJECTS {
        normalize_projects
            .push("WHEN ")
            .push_bind(project.id)
            .push(" THEN ")
            .push_bind(project.name)
            .push(" ");
    }
    normalize_projects.push("END, sort_order = CASE id ");
    for project in BUILT_IN_ROUTINE_PROJECTS {
        normalize_projects
            .push("WHEN ")
            .push_bind(project.id)
            .push(" THEN ")
            .push_bind(project.sort_order)
            .push(" ");
    }
    normalize_projects.push("END WHERE id IN (");
    {
        let mut separated = normalize_projects.separated(", ");
        for project in BUILT_IN_ROUTINE_PROJECTS {
            separated.push_bind(project.id);
        }
    }
    normalize_projects.push(")");
    normalize_projects
        .build()
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("normalize built-in Routine projects: {e}"))?;

    insert_default_project_graphs(&mut tx, &missing_project_ids).await?;

    tx.commit()
        .await
        .map_err(|e| format!("commit built-in Routine repair: {e}"))
}

async fn ensure_built_in_routine_defaults(pool: &sqlx::SqlitePool) -> Result<(), String> {
    if built_in_routine_defaults_intact(pool).await? {
        return Ok(());
    }
    repair_built_in_routine_defaults(pool).await
}

#[tauri::command]
pub async fn projects_load_workspace<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    preferred_project_id: Option<String>,
    active_view: ProjectViewId,
) -> Result<ProjectsWorkspaceSnapshot, String> {
    let pool = connect_sqlite(app, db_url).await?;
    load_projects_workspace(&pool, preferred_project_id.as_deref(), active_view, true).await
}

#[tauri::command]
pub async fn projects_refresh_workspace<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    preferred_project_id: Option<String>,
    active_view: ProjectViewId,
) -> Result<ProjectsWorkspaceSnapshot, String> {
    let pool = connect_sqlite(app, db_url).await?;
    load_projects_workspace(&pool, preferred_project_id.as_deref(), active_view, false).await
}

fn resolved_project_id(
    projects: &[ProjectRow],
    preferred_project_id: Option<&str>,
) -> Option<String> {
    if let Some(preferred) = preferred_project_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if projects.iter().any(|project| project.id == preferred) {
            return Some(preferred.to_string());
        }
    }
    projects
        .iter()
        .filter(|project| project.status == "active")
        .min_by(|left, right| {
            left.sort_order
                .cmp(&right.sort_order)
                .then_with(|| left.name.cmp(&right.name))
        })
        .or_else(|| projects.first())
        .map(|project| project.id.clone())
}

async fn load_projects_workspace(
    pool: &sqlx::SqlitePool,
    preferred_project_id: Option<&str>,
    active_view: ProjectViewId,
    repair_built_ins: bool,
) -> Result<ProjectsWorkspaceSnapshot, String> {
    if repair_built_ins {
        ensure_built_in_routine_defaults(pool).await?;
    }
    let groups = sqlx::query_as::<_, ProjectGroupRow>(
        "SELECT * FROM project_groups ORDER BY sort_order ASC, name ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load project groups: {e}"))?;
    let projects = sqlx::query_as::<_, ProjectRow>(
        "SELECT * FROM projects ORDER BY group_id ASC, sort_order ASC, name ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load projects: {e}"))?;
    let resolved_project_id = resolved_project_id(&projects, preferred_project_id);
    let mut snapshot = load_projects_core_snapshot(pool, resolved_project_id.as_deref()).await?;
    snapshot.groups = groups;
    snapshot.projects = projects;
    Ok(ProjectsWorkspaceSnapshot {
        resolved_project_id,
        active_view,
        snapshot,
    })
}

async fn load_projects_core_snapshot(
    pool: &sqlx::SqlitePool,
    project_id: Option<&str>,
) -> Result<ProjectsSnapshot, String> {
    let normalized_project_id = project_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("\0");
    let sections = sqlx::query_as::<_, ProjectSectionRow>(
        "SELECT * FROM project_sections
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load project sections: {e}"))?;
    let statuses = sqlx::query_as::<_, ProjectStatusRow>(
        "SELECT * FROM project_statuses
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load project statuses: {e}"))?;
    let priorities = sqlx::query_as::<_, ProjectPriorityRow>(
        "SELECT * FROM project_priorities
         WHERE project_id = ?
         ORDER BY project_id ASC, sort_order ASC, name ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load project priorities: {e}"))?;
    let tasks = sqlx::query_as::<_, ProjectTaskRow>(
        "SELECT * FROM project_tasks
         WHERE project_id = ?
         ORDER BY project_id ASC, section_sort_order ASC, created_at ASC",
    )
    .bind(normalized_project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load project tasks: {e}"))?;
    Ok(ProjectsSnapshot {
        groups: Vec::new(),
        projects: Vec::new(),
        sections,
        statuses,
        priorities,
        tasks,
        checklist_items: Vec::new(),
        tags: Vec::new(),
        task_tag_links: Vec::new(),
        custom_fields: Vec::new(),
        custom_field_options: Vec::new(),
        custom_field_values: Vec::new(),
        custom_field_option_values: Vec::new(),
        dependencies: Vec::new(),
        event_links: Vec::new(),
        task_change_events: Vec::new(),
        view_preferences: Vec::new(),
        custom_emojis: Vec::new(),
    })
}

#[cfg(test)]
pub(crate) async fn load_projects_workspace_for_first_use_contract(
    pool: &sqlx::SqlitePool,
    preferred_project_id: Option<&str>,
    active_view: ProjectViewId,
) -> Result<ProjectsWorkspaceSnapshot, String> {
    load_projects_workspace(pool, preferred_project_id, active_view, true).await
}

#[cfg(test)]
pub(crate) async fn refresh_projects_workspace_for_first_use_contract(
    pool: &sqlx::SqlitePool,
    preferred_project_id: Option<&str>,
    active_view: ProjectViewId,
) -> Result<ProjectsWorkspaceSnapshot, String> {
    load_projects_workspace(pool, preferred_project_id, active_view, false).await
}

#[tauri::command]
pub async fn projects_load_optional_data<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: Option<String>,
    kind: ProjectOptionalDataKind,
) -> Result<ProjectsOptionalData, String> {
    let pool = connect_sqlite(app, db_url).await?;
    load_projects_optional_data(&pool, project_id.as_deref(), kind).await
}

fn required_optional_project_id(
    project_id: Option<&str>,
    kind: ProjectOptionalDataKind,
) -> Result<&str, String> {
    project_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("project id is required for {kind:?} data"))
}

async fn load_projects_optional_data(
    pool: &sqlx::SqlitePool,
    project_id: Option<&str>,
    kind: ProjectOptionalDataKind,
) -> Result<ProjectsOptionalData, String> {
    let mut result = ProjectsOptionalData {
        kind,
        project_id: project_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        checklist_items: Vec::new(),
        tags: Vec::new(),
        task_tag_links: Vec::new(),
        custom_fields: Vec::new(),
        custom_field_options: Vec::new(),
        custom_field_values: Vec::new(),
        custom_field_option_values: Vec::new(),
        dependencies: Vec::new(),
        event_links: Vec::new(),
        task_change_events: Vec::new(),
        view_preferences: Vec::new(),
        custom_emojis: Vec::new(),
    };

    match kind {
        ProjectOptionalDataKind::Relationships => {
            let project_id = required_optional_project_id(project_id, kind)?;
            result.tags = sqlx::query_as::<_, ProjectTagRow>(
                "SELECT * FROM project_tags
                 WHERE project_id = ?
                 ORDER BY project_id ASC, sort_order ASC, name ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project tags: {e}"))?;
            result.task_tag_links = sqlx::query_as::<_, ProjectTaskTagLinkRow>(
                "SELECT project_task_tag_links.*
                 FROM project_task_tag_links
                 JOIN project_tasks ON project_tasks.id = project_task_tag_links.task_id
                 WHERE project_tasks.project_id = ?
                 ORDER BY project_task_tag_links.created_at ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project task tag links: {e}"))?;
            result.dependencies = sqlx::query_as::<_, ProjectTaskDependencyRow>(
                "SELECT project_task_dependencies.*
                 FROM project_task_dependencies
                 JOIN project_tasks ON project_tasks.id = project_task_dependencies.blocked_task_id
                 WHERE project_tasks.project_id = ?
                 ORDER BY project_task_dependencies.created_at ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project task dependencies: {e}"))?;
            result.event_links = sqlx::query_as::<_, ProjectTaskEventLinkRow>(
                "SELECT project_task_event_links.*
                 FROM project_task_event_links
                 JOIN project_tasks ON project_tasks.id = project_task_event_links.task_id
                 WHERE project_tasks.project_id = ?
                 ORDER BY project_task_event_links.created_at ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project task event links: {e}"))?;
        }
        ProjectOptionalDataKind::CustomFields => {
            let project_id = required_optional_project_id(project_id, kind)?;
            result.custom_fields = sqlx::query_as::<_, ProjectCustomFieldRow>(
                "SELECT * FROM project_custom_fields
                 WHERE project_id = ?
                 ORDER BY project_id ASC, sort_order ASC, name ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project custom fields: {e}"))?;
            result.custom_field_options = sqlx::query_as::<_, ProjectCustomFieldOptionRow>(
                "SELECT project_custom_field_options.*
                 FROM project_custom_field_options
                 JOIN project_custom_fields ON project_custom_fields.id = project_custom_field_options.field_id
                 WHERE project_custom_fields.project_id = ?
                 ORDER BY project_custom_field_options.field_id ASC, project_custom_field_options.sort_order ASC, project_custom_field_options.name ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project custom field options: {e}"))?;
            result.custom_field_values = sqlx::query_as::<_, ProjectCustomFieldValueRow>(
                "SELECT project_custom_field_values.*
                 FROM project_custom_field_values
                 JOIN project_tasks ON project_tasks.id = project_custom_field_values.task_id
                 WHERE project_tasks.project_id = ?
                 ORDER BY project_custom_field_values.updated_at ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project custom field values: {e}"))?;
            result.custom_field_option_values =
                sqlx::query_as::<_, ProjectCustomFieldOptionValueRow>(
                    "SELECT project_custom_field_option_values.*
                     FROM project_custom_field_option_values
                     JOIN project_tasks ON project_tasks.id = project_custom_field_option_values.task_id
                     WHERE project_tasks.project_id = ?
                     ORDER BY project_custom_field_option_values.created_at ASC",
                )
                .bind(project_id)
                .fetch_all(pool)
                .await
                .map_err(|e| format!("load project custom field option values: {e}"))?;
        }
        ProjectOptionalDataKind::History => {
            let project_id = required_optional_project_id(project_id, kind)?;
            result.task_change_events = sqlx::query_as::<_, ProjectTaskChangeEventRow>(
                "SELECT project_task_change_events.*
                 FROM project_task_change_events
                 JOIN project_tasks ON project_tasks.id = project_task_change_events.task_id
                 WHERE project_tasks.project_id = ?
                 ORDER BY project_task_change_events.occurred_at DESC
                 LIMIT 500",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project task change events: {e}"))?;
        }
        ProjectOptionalDataKind::Checklist => {
            let project_id = required_optional_project_id(project_id, kind)?;
            result.checklist_items = sqlx::query_as::<_, ProjectChecklistItemRow>(
                "SELECT project_checklist_items.*
                 FROM project_checklist_items
                 JOIN project_tasks ON project_tasks.id = project_checklist_items.task_id
                 WHERE project_tasks.project_id = ?
                 ORDER BY project_checklist_items.task_id ASC, project_checklist_items.sort_order ASC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project checklist items: {e}"))?;
        }
        ProjectOptionalDataKind::SavedViews => {
            let project_id = required_optional_project_id(project_id, kind)?;
            result.view_preferences = sqlx::query_as::<_, ProjectViewPreferenceRow>(
                "SELECT * FROM project_view_preferences
                 WHERE project_id = ?
                 ORDER BY view_id ASC, updated_at DESC",
            )
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("load project view preferences: {e}"))?;
        }
        ProjectOptionalDataKind::CustomEmojis => {
            if result.project_id.is_some() {
                return Err("project id must be omitted for custom emoji data".to_string());
            }
            result.custom_emojis = load_project_custom_emojis(pool).await?;
        }
    }

    Ok(result)
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

macro_rules! define_mutation_loader {
    ($name:ident, $field:ident, $row:ty, $table:literal) => {
        async fn $name(pool: &sqlx::SqlitePool, id: &str) -> Result<ProjectsMutationRows, String> {
            let row = sqlx::query_as::<_, $row>(concat!("SELECT * FROM ", $table, " WHERE id = ?"))
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("load mutation result from {}: {e}", $table))?;
            let mut mutation = ProjectsMutationRows::default();
            mutation.$field.push(row);
            Ok(mutation)
        }
    };
}

define_mutation_loader!(group_mutation, groups, ProjectGroupRow, "project_groups");
define_mutation_loader!(project_mutation, projects, ProjectRow, "projects");
define_mutation_loader!(
    section_mutation,
    sections,
    ProjectSectionRow,
    "project_sections"
);
define_mutation_loader!(
    status_mutation,
    statuses,
    ProjectStatusRow,
    "project_statuses"
);
define_mutation_loader!(
    priority_mutation,
    priorities,
    ProjectPriorityRow,
    "project_priorities"
);
define_mutation_loader!(task_mutation_base, tasks, ProjectTaskRow, "project_tasks");
define_mutation_loader!(
    checklist_item_mutation_base,
    checklist_items,
    ProjectChecklistItemRow,
    "project_checklist_items"
);
define_mutation_loader!(tag_mutation, tags, ProjectTagRow, "project_tags");
define_mutation_loader!(
    custom_field_mutation,
    custom_fields,
    ProjectCustomFieldRow,
    "project_custom_fields"
);
define_mutation_loader!(
    custom_field_option_mutation,
    custom_field_options,
    ProjectCustomFieldOptionRow,
    "project_custom_field_options"
);
define_mutation_loader!(
    dependency_mutation_base,
    dependencies,
    ProjectTaskDependencyRow,
    "project_task_dependencies"
);
define_mutation_loader!(
    custom_emoji_mutation,
    custom_emojis,
    ProjectCustomEmojiRow,
    "project_custom_emojis"
);

async fn latest_task_change_events(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<Vec<ProjectTaskChangeEventRow>, String> {
    sqlx::query_as::<_, ProjectTaskChangeEventRow>(
        "SELECT * FROM project_task_change_events
         WHERE task_id = ?
         ORDER BY occurred_at DESC, id DESC
         LIMIT 32",
    )
    .bind(task_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load mutation task history: {e}"))
}

async fn task_mutation(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<ProjectsMutationRows, String> {
    let mut mutation = task_mutation_base(pool, task_id).await?;
    mutation.task_change_events = latest_task_change_events(pool, task_id).await?;
    Ok(mutation)
}

async fn checklist_item_mutation(
    pool: &sqlx::SqlitePool,
    item_id: &str,
    task_id: &str,
) -> Result<ProjectsMutationRows, String> {
    let mut mutation = checklist_item_mutation_base(pool, item_id).await?;
    mutation.task_change_events = latest_task_change_events(pool, task_id).await?;
    Ok(mutation)
}

async fn dependency_mutation(
    pool: &sqlx::SqlitePool,
    dependency_id: &str,
    task_id: &str,
) -> Result<ProjectsMutationRows, String> {
    let mut mutation = dependency_mutation_base(pool, dependency_id).await?;
    mutation.task_change_events = latest_task_change_events(pool, task_id).await?;
    Ok(mutation)
}

async fn task_tag_link_mutation(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    tag_id: &str,
) -> Result<ProjectsMutationRows, String> {
    let link = sqlx::query_as::<_, ProjectTaskTagLinkRow>(
        "SELECT * FROM project_task_tag_links WHERE task_id = ? AND tag_id = ?",
    )
    .bind(task_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("load task tag mutation result: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.task_tag_links.push(link);
    mutation.task_change_events = latest_task_change_events(pool, task_id).await?;
    Ok(mutation)
}

async fn event_link_mutation(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    event_id: &str,
) -> Result<ProjectsMutationRows, String> {
    let link = sqlx::query_as::<_, ProjectTaskEventLinkRow>(
        "SELECT * FROM project_task_event_links WHERE task_id = ? AND event_id = ?",
    )
    .bind(task_id)
    .bind(event_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("load task event mutation result: {e}"))?;
    let project_id: String =
        sqlx::query_scalar("SELECT project_id FROM calendar_events WHERE id = ?")
            .bind(event_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("load assigned calendar event project: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.event_links.push(link);
    mutation.task_change_events = latest_task_change_events(pool, task_id).await?;
    mutation
        .calendar_event_project_assignments
        .push(CalendarEventProjectAssignment {
            event_id: event_id.to_string(),
            project_id,
        });
    Ok(mutation)
}

async fn view_preference_mutation(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    view_id: &str,
    preference_key: &str,
) -> Result<ProjectsMutationRows, String> {
    let preference = sqlx::query_as::<_, ProjectViewPreferenceRow>(
        "SELECT * FROM project_view_preferences
         WHERE project_id = ? AND view_id = ? AND preference_key = ?",
    )
    .bind(project_id)
    .bind(view_id)
    .bind(preference_key)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("load view preference mutation result: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.view_preferences.push(preference);
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_group<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group: ProjectGroupCreate,
) -> Result<ProjectsMutationRows, String> {
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
    group_mutation(&pool, &group.id).await
}

#[tauri::command]
pub async fn projects_update_group<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group: ProjectGroupUpdate,
) -> Result<ProjectsMutationRows, String> {
    validate_group_update(&group)?;
    let pool = connect_sqlite(app, db_url).await?;
    let group_name = if group.id == ROUTINE_GROUP_ID {
        "Routine"
    } else {
        group.name.trim()
    };
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
    .bind(group_name)
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
    group_mutation(&pool, &group.id).await
}

#[tauri::command]
pub async fn projects_delete_group<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    group_id: String,
) -> Result<ProjectsMutationRows, String> {
    let pool = connect_sqlite(app, db_url).await?;
    let normalized_group_id = group_id.trim();
    let project_ids = sqlx::query_scalar::<_, String>("SELECT id FROM projects WHERE group_id = ?")
        .bind(normalized_group_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("load deleted group projects: {e}"))?;
    delete_project_group(&pool, normalized_group_id).await?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.removals.push(ProjectMutationRemoval::Group {
        id: normalized_group_id.to_string(),
    });
    mutation.removals.extend(
        project_ids
            .into_iter()
            .map(|id| ProjectMutationRemoval::Project { id }),
    );
    Ok(mutation)
}

async fn delete_project_group(pool: &sqlx::SqlitePool, group_id: &str) -> Result<(), String> {
    let normalized_group_id = group_id.trim();
    require_non_empty(normalized_group_id, "group_id")?;
    if normalized_group_id == ROUTINE_GROUP_ID {
        return Err("built-in Routine group cannot be deleted".to_string());
    }
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
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&group_id, "group_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_groups
         SET collapsed = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(if collapsed { 1_i64 } else { 0_i64 })
    .bind(&group_id)
    .execute(&pool)
    .await
    .map_err(|e| format!("set project group collapsed: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project group not found".to_string());
    }
    group_mutation(&pool, group_id.trim()).await
}

#[tauri::command]
pub async fn projects_create_project<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project: ProjectCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_project_create(&project)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    sqlx::query(
        "INSERT INTO projects (
            id, group_id, name, icon, color, sort_order,
            default_event_name, default_event_time_mode,
            default_event_duration_minutes,
            default_pomodoro_mode, default_pomodoro_preset_key,
            default_pomodoro_focus_minutes, default_pomodoro_short_break_minutes,
            default_pomodoro_long_break_minutes, default_pomodoro_long_break_after_focus_count,
            default_idle_settings_source, default_idle_pause_enabled, default_idle_threshold_minutes
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&project.id)
    .bind(&project.group_id)
    .bind(project.name.trim())
    .bind(project.icon.trim())
    .bind(project.color)
    .bind(project.sort_order)
    .bind(normalized_optional_text(
        project.default_event_name.as_deref(),
    ))
    .bind(&project.default_event_time_mode)
    .bind(project.default_event_duration_minutes)
    .bind(&project.default_pomodoro_mode)
    .bind(&project.default_pomodoro_preset_key)
    .bind(project.default_pomodoro_focus_minutes)
    .bind(project.default_pomodoro_short_break_minutes)
    .bind(project.default_pomodoro_long_break_minutes)
    .bind(project.default_pomodoro_long_break_after_focus_count)
    .bind(&project.default_idle_settings_source)
    .bind(if project.default_idle_pause_enabled {
        1_i64
    } else {
        0_i64
    })
    .bind(project.default_idle_threshold_minutes)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create project: {e}"))?;

    insert_template_sections(&mut tx, &project.id, &project.template_id).await?;
    insert_default_statuses(&mut tx, &project.id).await?;
    insert_default_priorities(&mut tx, &project.id).await?;

    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = project_mutation(&pool, &project.id).await?;
    mutation.sections = sqlx::query_as::<_, ProjectSectionRow>(
        "SELECT * FROM project_sections WHERE project_id = ? ORDER BY sort_order, id",
    )
    .bind(&project.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load created project sections: {e}"))?;
    mutation.statuses = sqlx::query_as::<_, ProjectStatusRow>(
        "SELECT * FROM project_statuses WHERE project_id = ? ORDER BY sort_order, id",
    )
    .bind(&project.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load created project statuses: {e}"))?;
    mutation.priorities = sqlx::query_as::<_, ProjectPriorityRow>(
        "SELECT * FROM project_priorities WHERE project_id = ? ORDER BY sort_order, id",
    )
    .bind(&project.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load created project priorities: {e}"))?;
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_update_project<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project: ProjectUpdate,
) -> Result<ProjectsMutationRows, String> {
    validate_project_update(&project)?;
    let pool = connect_sqlite(app, db_url).await?;
    let built_in = built_in_routine_project(&project.id);
    let group_id = built_in
        .map(|_| ROUTINE_GROUP_ID)
        .unwrap_or(project.group_id.as_str());
    let name = built_in
        .map(|default| default.name)
        .unwrap_or_else(|| project.name.trim());
    let sort_order = built_in
        .map(|default| default.sort_order)
        .unwrap_or(project.sort_order);
    let result = sqlx::query(
        "UPDATE projects
         SET group_id = ?,
             name = ?,
             icon = ?,
             color = ?,
             sort_order = ?,
             status = ?,
             default_event_name = ?,
             default_event_time_mode = ?,
             default_event_duration_minutes = ?,
             default_pomodoro_mode = ?,
             default_pomodoro_preset_key = ?,
             default_pomodoro_focus_minutes = ?,
             default_pomodoro_short_break_minutes = ?,
             default_pomodoro_long_break_minutes = ?,
             default_pomodoro_long_break_after_focus_count = ?,
             default_idle_settings_source = ?,
             default_idle_pause_enabled = ?,
             default_idle_threshold_minutes = ?,
             focus_playlist_id = ?,
             break_playlist_id = ?,
             work_environment_id = ?,
             blocker_ruleset_id = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(group_id)
    .bind(name)
    .bind(project.icon.trim())
    .bind(project.color)
    .bind(sort_order)
    .bind(&project.status)
    .bind(normalized_optional_text(
        project.default_event_name.as_deref(),
    ))
    .bind(&project.default_event_time_mode)
    .bind(project.default_event_duration_minutes)
    .bind(&project.default_pomodoro_mode)
    .bind(&project.default_pomodoro_preset_key)
    .bind(project.default_pomodoro_focus_minutes)
    .bind(project.default_pomodoro_short_break_minutes)
    .bind(project.default_pomodoro_long_break_minutes)
    .bind(project.default_pomodoro_long_break_after_focus_count)
    .bind(&project.default_idle_settings_source)
    .bind(if project.default_idle_pause_enabled {
        1_i64
    } else {
        0_i64
    })
    .bind(project.default_idle_threshold_minutes)
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
    project_mutation(&pool, &project.id).await
}

#[tauri::command]
pub async fn projects_update_notes_settings<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    notes_default_open_mode: Option<String>,
    notes_history_retention_days: Option<i64>,
) -> Result<ProjectsMutationRows, String> {
    let project_id = project_id.trim();
    require_non_empty(project_id, "project_id")?;
    if let Some(value) = notes_default_open_mode.as_deref() {
        validate_enum(value, "notes_default_open_mode", NOTES_PAGE_OPEN_MODES)?;
    }
    if let Some(days) = notes_history_retention_days {
        if !NOTES_HISTORY_RETENTION_DAYS.contains(&days) {
            return Err(
                "notes_history_retention_days must be 0, 7, 30, 90, 180, 365, or null".to_string(),
            );
        }
    }
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin project Notes settings update: {e}"))?;
    let result = sqlx::query(
        "UPDATE projects
         SET notes_default_open_mode = ?,
             notes_history_retention_days = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(notes_default_open_mode)
    .bind(notes_history_retention_days)
    .bind(project_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update project Notes settings: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project not found".to_string());
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit project Notes settings update: {e}"))?;
    project_mutation(&pool, project_id).await
}

#[tauri::command]
pub async fn projects_create_section<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    section: ProjectSectionCreate,
) -> Result<ProjectsMutationRows, String> {
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
    section_mutation(&pool, &section.id).await
}

#[tauri::command]
pub async fn projects_update_section<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    section: ProjectSectionUpdate,
) -> Result<ProjectsMutationRows, String> {
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
    section_mutation(&pool, &section.id).await
}

#[tauri::command]
pub async fn projects_create_status<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    status: ProjectStatusCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_status_create(&status)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, &status.project_id).await?;
    sqlx::query(
        "INSERT INTO project_statuses (id, project_id, name, category, color, sort_order, terminal)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&status.id)
    .bind(&status.project_id)
    .bind(status.name.trim())
    .bind(&status.category)
    .bind(status.color)
    .bind(status.sort_order)
    .bind(if status.terminal { 1_i64 } else { 0_i64 })
    .execute(&pool)
    .await
    .map_err(|e| format!("create project status: {e}"))?;
    status_mutation(&pool, &status.id).await
}

#[tauri::command]
pub async fn projects_update_status<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    status: ProjectStatusUpdate,
) -> Result<ProjectsMutationRows, String> {
    validate_status_update(&status)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_statuses
         SET name = ?,
             category = ?,
             color = ?,
             sort_order = ?,
             terminal = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(status.name.trim())
    .bind(&status.category)
    .bind(status.color)
    .bind(status.sort_order)
    .bind(if status.terminal { 1_i64 } else { 0_i64 })
    .bind(&status.id)
    .execute(&pool)
    .await
    .map_err(|e| format!("update project status: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project status not found".to_string());
    }
    status_mutation(&pool, &status.id).await
}

#[tauri::command]
pub async fn projects_delete_status<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    status_id: String,
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&status_id, "status_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_unused_status(&mut tx, status_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.removals.push(ProjectMutationRemoval::Status {
        id: status_id.trim().to_string(),
    });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_priority<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    priority: ProjectPriorityCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_priority_create(&priority)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, &priority.project_id).await?;
    sqlx::query(
        "INSERT INTO project_priorities (id, project_id, name, color, sort_order)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(priority.id.trim())
    .bind(&priority.project_id)
    .bind(priority.name.trim())
    .bind(priority.color)
    .bind(priority.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project priority: {e}"))?;
    priority_mutation(&pool, priority.id.trim()).await
}

#[tauri::command]
pub async fn projects_update_priority<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    priority: ProjectPriorityUpdate,
) -> Result<ProjectsMutationRows, String> {
    validate_priority_update(&priority)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_priorities
         SET name = ?,
             color = ?,
             sort_order = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE project_id = ? AND id = ?",
    )
    .bind(priority.name.trim())
    .bind(priority.color)
    .bind(priority.sort_order)
    .bind(&priority.project_id)
    .bind(priority.id.trim())
    .execute(&pool)
    .await
    .map_err(|e| format!("update project priority: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project priority not found".to_string());
    }
    priority_mutation(&pool, priority.id.trim()).await
}

#[tauri::command]
pub async fn projects_delete_priority<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    priority_id: String,
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&project_id, "project_id")?;
    require_non_empty(&priority_id, "priority_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_unused_priority(&mut tx, project_id.trim(), priority_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.removals.push(ProjectMutationRemoval::Priority {
        id: priority_id.trim().to_string(),
    });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_task<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task: ProjectTaskCreate,
) -> Result<ProjectsMutationRows, String> {
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
    task_mutation(&pool, &task.id).await
}

#[tauri::command]
pub async fn projects_update_task<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task: ProjectTaskUpdate,
) -> Result<ProjectsMutationRows, String> {
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
    ensure_priority_matches_project(&mut tx, &project_id, &task.priority).await?;
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
    task_mutation(&pool, &task.id).await
}

#[tauri::command]
pub async fn projects_create_task_dependency<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    dependency: ProjectTaskDependencyCreate,
) -> Result<ProjectsMutationRows, String> {
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
    dependency_mutation(&pool, dependency.id.trim(), &dependency.blocked_task_id).await
}

#[tauri::command]
pub async fn projects_delete_task_dependency<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    dependency_id: String,
) -> Result<ProjectsMutationRows, String> {
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
    let mut mutation = ProjectsMutationRows {
        task_change_events: latest_task_change_events(&pool, &row.blocked_task_id).await?,
        ..ProjectsMutationRows::default()
    };
    mutation.removals.push(ProjectMutationRemoval::Dependency {
        id: dependency_id.trim().to_string(),
    });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_checklist_item<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    item: ProjectChecklistItemCreate,
) -> Result<ProjectsMutationRows, String> {
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
    checklist_item_mutation(&pool, &item.id, &item.task_id).await
}

#[tauri::command]
pub async fn projects_update_checklist_item<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    item: ProjectChecklistItemUpdate,
) -> Result<ProjectsMutationRows, String> {
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
    checklist_item_mutation(&pool, &item.id, &task_id).await
}

#[tauri::command]
pub async fn projects_delete_checklist_item<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    item_id: String,
) -> Result<ProjectsMutationRows, String> {
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
    let mut mutation = ProjectsMutationRows {
        task_change_events: latest_task_change_events(&pool, &task_id).await?,
        ..ProjectsMutationRows::default()
    };
    mutation
        .removals
        .push(ProjectMutationRemoval::ChecklistItem {
            id: item_id.trim().to_string(),
        });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_tag<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    tag: ProjectTagCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_tag_create(&tag)?;
    let pool = connect_sqlite(app, db_url).await?;
    ensure_project_exists_in_pool(&pool, tag.project_id.trim()).await?;
    sqlx::query(
        "INSERT INTO project_tags (id, project_id, name, color, sort_order)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(tag.id.trim())
    .bind(tag.project_id.trim())
    .bind(tag.name.trim())
    .bind(tag.color)
    .bind(tag.sort_order)
    .execute(&pool)
    .await
    .map_err(|e| format!("create project tag: {e}"))?;
    tag_mutation(&pool, tag.id.trim()).await
}

#[tauri::command]
pub async fn projects_update_tag<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    tag: ProjectTagUpdate,
) -> Result<ProjectsMutationRows, String> {
    validate_tag_update(&tag)?;
    let pool = connect_sqlite(app, db_url).await?;
    let result = sqlx::query(
        "UPDATE project_tags
         SET name = ?,
             color = ?,
             sort_order = ?,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(tag.name.trim())
    .bind(tag.color)
    .bind(tag.sort_order)
    .bind(tag.id.trim())
    .execute(&pool)
    .await
    .map_err(|e| format!("update project tag: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project tag not found".to_string());
    }
    tag_mutation(&pool, tag.id.trim()).await
}

#[tauri::command]
pub async fn projects_delete_tag<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    tag_id: String,
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&tag_id, "tag_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id FROM project_task_tag_links WHERE tag_id = ?",
    )
    .bind(tag_id.trim())
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load tasks affected by tag deletion: {e}"))?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_tag_with_history(&mut tx, tag_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    for task_id in task_ids {
        mutation
            .task_change_events
            .extend(latest_task_change_events(&pool, &task_id).await?);
    }
    mutation.removals.push(ProjectMutationRemoval::Tag {
        id: tag_id.trim().to_string(),
    });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_link_task_tag<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    link: ProjectTaskTagLinkCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_task_tag_link_create(&link)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let tag_name = tag_for_task_tag_link(&mut tx, &link.task_id, &link.tag_id).await?;
    let result = sqlx::query(
        "INSERT INTO project_task_tag_links (task_id, tag_id)
         VALUES (?, ?)
         ON CONFLICT(task_id, tag_id) DO NOTHING",
    )
    .bind(link.task_id.trim())
    .bind(link.tag_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("link project task tag: {e}"))?;
    if result.rows_affected() > 0 {
        insert_task_change_event(
            &mut tx,
            link.task_id.trim(),
            "updated",
            Some("tags"),
            None,
            Some(&tag_name),
        )
        .await?;
    }
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    task_tag_link_mutation(&pool, link.task_id.trim(), link.tag_id.trim()).await
}

#[tauri::command]
pub async fn projects_unlink_task_tag<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    task_id: String,
    tag_id: String,
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&task_id, "task_id")?;
    require_non_empty(&tag_id, "tag_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    let tag_name = tag_for_existing_task_tag_link(&mut tx, task_id.trim(), tag_id.trim()).await?;
    sqlx::query(
        "DELETE FROM project_task_tag_links
         WHERE task_id = ? AND tag_id = ?",
    )
    .bind(task_id.trim())
    .bind(tag_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("unlink project task tag: {e}"))?;
    insert_task_change_event(
        &mut tx,
        task_id.trim(),
        "updated",
        Some("tags"),
        Some(&tag_name),
        None,
    )
    .await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows {
        task_change_events: latest_task_change_events(&pool, task_id.trim()).await?,
        ..ProjectsMutationRows::default()
    };
    mutation.removals.push(ProjectMutationRemoval::TaskTagLink {
        task_id: task_id.trim().to_string(),
        tag_id: tag_id.trim().to_string(),
    });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_custom_field<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    field: ProjectCustomFieldCreate,
) -> Result<ProjectsMutationRows, String> {
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
    custom_field_mutation(&pool, field.id.trim()).await
}

#[tauri::command]
pub async fn projects_update_custom_field<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    field: ProjectCustomFieldUpdate,
) -> Result<ProjectsMutationRows, String> {
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
    custom_field_mutation(&pool, field.id.trim()).await
}

#[tauri::command]
pub async fn projects_delete_custom_field<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    field_id: String,
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&field_id, "field_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id FROM project_custom_field_values WHERE field_id = ?
         UNION
         SELECT task_id FROM project_custom_field_option_values WHERE field_id = ?",
    )
    .bind(field_id.trim())
    .bind(field_id.trim())
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load tasks affected by custom field deletion: {e}"))?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_custom_field_with_history(&mut tx, field_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    for task_id in task_ids {
        mutation
            .task_change_events
            .extend(latest_task_change_events(&pool, &task_id).await?);
    }
    mutation.removals.push(ProjectMutationRemoval::CustomField {
        id: field_id.trim().to_string(),
    });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_custom_field_option<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    option: ProjectCustomFieldOptionCreate,
) -> Result<ProjectsMutationRows, String> {
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
    custom_field_option_mutation(&pool, option.id.trim()).await
}

#[tauri::command]
pub async fn projects_update_custom_field_option<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    option: ProjectCustomFieldOptionUpdate,
) -> Result<ProjectsMutationRows, String> {
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
    custom_field_option_mutation(&pool, option.id.trim()).await
}

#[tauri::command]
pub async fn projects_delete_custom_field_option<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    option_id: String,
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&option_id, "option_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id FROM project_custom_field_option_values WHERE option_id = ?",
    )
    .bind(option_id.trim())
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load tasks affected by custom field option deletion: {e}"))?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    delete_custom_field_option_with_history(&mut tx, option_id.trim()).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    for task_id in task_ids {
        mutation
            .task_change_events
            .extend(latest_task_change_events(&pool, &task_id).await?);
    }
    mutation
        .removals
        .push(ProjectMutationRemoval::CustomFieldOption {
            id: option_id.trim().to_string(),
        });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_update_custom_field_value<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    value: ProjectCustomFieldValueUpdate,
) -> Result<ProjectsMutationRows, String> {
    validate_custom_field_value_update(&value)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    update_custom_field_value_with_history(&mut tx, &value).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    let mut mutation = ProjectsMutationRows::default();
    mutation
        .removals
        .push(ProjectMutationRemoval::CustomFieldValue {
            task_id: value.task_id.trim().to_string(),
            field_id: value.field_id.trim().to_string(),
        });
    if let Some(row) = sqlx::query_as::<_, ProjectCustomFieldValueRow>(
        "SELECT * FROM project_custom_field_values WHERE task_id = ? AND field_id = ?",
    )
    .bind(value.task_id.trim())
    .bind(value.field_id.trim())
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("load custom field value mutation result: {e}"))?
    {
        mutation.custom_field_values.push(row);
    }
    mutation.custom_field_option_values = sqlx::query_as::<_, ProjectCustomFieldOptionValueRow>(
        "SELECT * FROM project_custom_field_option_values WHERE task_id = ? AND field_id = ?",
    )
    .bind(value.task_id.trim())
    .bind(value.field_id.trim())
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load custom field option value mutation result: {e}"))?;
    mutation.task_change_events = latest_task_change_events(&pool, value.task_id.trim()).await?;
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_link_task_event<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    link: ProjectTaskEventLinkCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_task_event_link_create(&link)?;
    let pool = connect_sqlite(app, db_url).await?;
    let mut tx = pool.begin().await.map_err(|e| format!("begin: {e}"))?;
    link_task_event_with_project_assignment(&mut tx, &link).await?;
    tx.commit().await.map_err(|e| format!("commit: {e}"))?;
    event_link_mutation(&pool, link.task_id.trim(), link.event_id.trim()).await
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
) -> Result<ProjectsMutationRows, String> {
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
    let mut mutation = ProjectsMutationRows {
        task_change_events: latest_task_change_events(&pool, task_id.trim()).await?,
        ..ProjectsMutationRows::default()
    };
    mutation.removals.push(ProjectMutationRemoval::EventLink {
        task_id: task_id.trim().to_string(),
        event_id: event_id.trim().to_string(),
    });
    Ok(mutation)
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
) -> Result<ProjectsMutationRows, String> {
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
    view_preference_mutation(
        &pool,
        preference.project_id.trim(),
        preference.view_id.trim(),
        preference.preference_key.trim(),
    )
    .await
}

#[tauri::command]
pub async fn projects_delete_view_preference<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    view_id: String,
    preference_key: String,
) -> Result<ProjectsMutationRows, String> {
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
    let mut mutation = ProjectsMutationRows::default();
    mutation
        .removals
        .push(ProjectMutationRemoval::ViewPreference {
            project_id: project_id.trim().to_string(),
            view_id: view_id.trim().to_string(),
            preference_key: preference_key.trim().to_string(),
        });
    Ok(mutation)
}

#[tauri::command]
pub async fn projects_create_custom_emoji<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    emoji: ProjectCustomEmojiCreate,
) -> Result<ProjectsMutationRows, String> {
    validate_custom_emoji_create(&emoji)?;
    let pool = connect_sqlite(app, db_url).await?;
    insert_project_custom_emoji(&pool, &emoji).await?;
    custom_emoji_mutation(&pool, emoji.id.trim()).await
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
) -> Result<ProjectsMutationRows, String> {
    require_non_empty(&emoji_id, "emoji_id")?;
    let pool = connect_sqlite(app, db_url).await?;
    delete_project_custom_emoji(&pool, &emoji_id).await?;
    let mut mutation = ProjectsMutationRows::default();
    mutation.removals.push(ProjectMutationRemoval::CustomEmoji {
        id: emoji_id.trim().to_string(),
    });
    Ok(mutation)
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

async fn delete_unused_status(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    status_id: &str,
) -> Result<(), String> {
    let project_id =
        sqlx::query_scalar::<_, String>("SELECT project_id FROM project_statuses WHERE id = ?")
            .bind(status_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load project status: {e}"))?
            .ok_or_else(|| "project status not found".to_string())?;

    let status_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_statuses WHERE project_id = ?")
            .bind(&project_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("count project statuses: {e}"))?;
    if status_count <= 1 {
        return Err("project must keep at least one task status".to_string());
    }

    let task_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_tasks WHERE status_id = ?")
            .bind(status_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("count status tasks: {e}"))?;
    if task_count > 0 {
        return Err("move or delete tasks before deleting this task status".to_string());
    }

    let result = sqlx::query("DELETE FROM project_statuses WHERE id = ?")
        .bind(status_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project status: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project status not found".to_string());
    }
    Ok(())
}

async fn delete_unused_priority(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    priority_id: &str,
) -> Result<(), String> {
    let priority_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_priorities WHERE project_id = ? AND id = ?",
    )
    .bind(project_id)
    .bind(priority_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("load project priority: {e}"))?;
    if priority_exists == 0 {
        return Err("project priority not found".to_string());
    }

    let priority_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_priorities WHERE project_id = ?")
            .bind(project_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("count project priorities: {e}"))?;
    if priority_count <= 1 {
        return Err("project must keep at least one task priority".to_string());
    }

    let task_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_tasks WHERE project_id = ? AND priority = ?",
    )
    .bind(project_id)
    .bind(priority_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("count priority tasks: {e}"))?;
    if task_count > 0 {
        return Err("move or delete tasks before deleting this task priority".to_string());
    }

    let result = sqlx::query("DELETE FROM project_priorities WHERE project_id = ? AND id = ?")
        .bind(project_id)
        .bind(priority_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project priority: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("project priority not found".to_string());
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

async fn ensure_priority_matches_project(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    project_id: &str,
    priority_id: &str,
) -> Result<(), String> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_priorities WHERE id = ? AND project_id = ?",
    )
    .bind(priority_id)
    .bind(project_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("check project priority: {e}"))?;
    if count > 0 {
        return Ok(());
    }
    let project_priority_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM project_priorities WHERE project_id = ?")
            .bind(project_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| format!("count project priorities: {e}"))?;
    if project_priority_count == 0 && matches!(priority_id, "low" | "normal" | "high" | "urgent") {
        return Ok(());
    }
    if count == 0 {
        return Err("priority does not belong to project".to_string());
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

async fn tag_for_task_tag_link(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    tag_id: &str,
) -> Result<String, String> {
    sqlx::query_scalar(
        "SELECT pl.name
         FROM project_tags pl
         JOIN project_tasks pt ON pt.project_id = pl.project_id
         WHERE pt.id = ? AND pl.id = ?",
    )
    .bind(task_id.trim())
    .bind(tag_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project task tag: {e}"))?
    .ok_or_else(|| "tag must belong to the task project".to_string())
}

async fn tag_for_existing_task_tag_link(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    tag_id: &str,
) -> Result<String, String> {
    sqlx::query_scalar(
        "SELECT pl.name
         FROM project_task_tag_links ptl
         JOIN project_tags pl ON pl.id = ptl.tag_id
         WHERE ptl.task_id = ? AND ptl.tag_id = ?",
    )
    .bind(task_id.trim())
    .bind(tag_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project task tag link: {e}"))?
    .ok_or_else(|| "project task tag link not found".to_string())
}

async fn delete_tag_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    tag_id: &str,
) -> Result<(), String> {
    let tag = sqlx::query_as::<_, ProjectTagRow>("SELECT * FROM project_tags WHERE id = ?")
        .bind(tag_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load project tag: {e}"))?
        .ok_or_else(|| "project tag not found".to_string())?;
    let linked_task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id
         FROM project_task_tag_links
         WHERE tag_id = ?
         ORDER BY created_at ASC",
    )
    .bind(tag_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load project tag tasks: {e}"))?;
    sqlx::query("DELETE FROM project_tags WHERE id = ?")
        .bind(tag_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project tag: {e}"))?;
    for task_id in linked_task_ids {
        insert_task_change_event(tx, &task_id, "updated", Some("tags"), Some(&tag.name), None)
            .await?;
    }
    Ok(())
}

fn normalized_optional_identifier(value: Option<&str>) -> Option<String> {
    normalized_optional_text(value)
}

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
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
    fn built_in_routine_defaults_are_protected_and_repaired() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;

            assert_eq!(
                delete_project_group(&pool, ROUTINE_GROUP_ID).await,
                Err("built-in Routine group cannot be deleted".to_string()),
            );

            sqlx::query(
                "UPDATE projects
                 SET name = 'Renamed', group_id = 'group-routine', sort_order = 999,
                     icon = 'utensils', status = 'hidden'
                 WHERE id = 'project-routine-eat'",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query("DELETE FROM projects WHERE id = 'project-routine-reading'")
                .execute(&pool)
                .await
                .unwrap();

            ensure_built_in_routine_defaults(&pool).await.unwrap();

            let eating: (String, String, i64, String, String) = sqlx::query_as(
                "SELECT group_id, name, sort_order, icon, status
                 FROM projects WHERE id = 'project-routine-eat'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(
                eating,
                (
                    ROUTINE_GROUP_ID.to_string(),
                    "Eating".to_string(),
                    40,
                    "utensils".to_string(),
                    "hidden".to_string(),
                ),
            );
            let restored_reading: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM projects WHERE id = 'project-routine-reading'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(restored_reading, 1);

            sqlx::query("DELETE FROM project_groups WHERE id = 'group-routine'")
                .execute(&pool)
                .await
                .unwrap();
            ensure_built_in_routine_defaults(&pool).await.unwrap();

            let restored_projects: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM projects WHERE group_id = 'group-routine'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(restored_projects, BUILT_IN_ROUTINE_PROJECTS.len() as i64);
            let reordered_defaults: Vec<(String, String, i64)> = sqlx::query_as(
                "SELECT id, icon, sort_order
                 FROM projects
                 WHERE id IN (
                    'project-routine-exercise',
                    'project-routine-hygiene',
                    'project-routine-eat',
                    'project-routine-commute',
                    'project-routine-chores',
                    'project-routine-health',
                    'project-routine-sleep'
                 )
                 ORDER BY sort_order ASC",
            )
            .fetch_all(&pool)
            .await
            .unwrap();
            assert_eq!(
                reordered_defaults,
                vec![
                    (
                        "project-routine-exercise".to_string(),
                        "sport-shoe".to_string(),
                        20,
                    ),
                    (
                        "project-routine-hygiene".to_string(),
                        "bath".to_string(),
                        30,
                    ),
                    ("project-routine-eat".to_string(), "apple".to_string(), 40,),
                    (
                        "project-routine-commute".to_string(),
                        "bike".to_string(),
                        50,
                    ),
                    (
                        "project-routine-chores".to_string(),
                        "shopping-cart".to_string(),
                        70,
                    ),
                    (
                        "project-routine-health".to_string(),
                        "pill".to_string(),
                        100,
                    ),
                    ("project-routine-sleep".to_string(), "bed".to_string(), 110,),
                ],
            );
        });
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
    fn task_tag_link_rejects_tags_from_another_project() {
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
                "INSERT INTO project_tags (id, project_id, name, sort_order)
                 VALUES ('tag-a', 'project-a', 'Backend', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let result = tag_for_task_tag_link(&mut tx, "task-other", "tag-a").await;

            assert_eq!(
                result,
                Err("tag must belong to the task project".to_string())
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
    fn delete_status_removes_empty_status() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
                 VALUES ('status-empty', 'project-a', 'Later', 'active', 200, 0)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            delete_unused_status(&mut tx, "status-empty").await.unwrap();
            tx.commit().await.unwrap();

            let status_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM project_statuses WHERE id = 'status-empty'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(status_count, 0);
        });
    }

    #[test]
    fn delete_status_rejects_status_with_tasks() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
                 VALUES ('status-empty', 'project-a', 'Later', 'active', 200, 0)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let result = delete_unused_status(&mut tx, "status-a").await;

            assert_eq!(
                result,
                Err("move or delete tasks before deleting this task status".to_string())
            );
        });
    }

    #[test]
    fn delete_status_rejects_last_project_status() {
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
                "INSERT INTO project_statuses (id, project_id, name, category, sort_order, terminal)
                 VALUES ('status-empty', 'project-empty', 'To do', 'not_started', 100, 0)",
            )
            .execute(&pool)
            .await
            .unwrap();

            let mut tx = pool.begin().await.unwrap();
            let result = delete_unused_status(&mut tx, "status-empty").await;

            assert_eq!(
                result,
                Err("project must keep at least one task status".to_string())
            );
        });
    }

    #[test]
    fn delete_tag_removes_links_and_records_task_history() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            sqlx::query(
                "INSERT INTO project_tags (id, project_id, name, sort_order)
                 VALUES ('tag-a', 'project-a', 'Backend', 100)",
            )
            .execute(&pool)
            .await
            .unwrap();
            for task_id in ["task-a", "task-b"] {
                sqlx::query(
                    "INSERT INTO project_task_tag_links (task_id, tag_id)
                     VALUES (?, 'tag-a')",
                )
                .bind(task_id)
                .execute(&pool)
                .await
                .unwrap();
            }

            let mut tx = pool.begin().await.unwrap();
            delete_tag_with_history(&mut tx, "tag-a").await.unwrap();
            tx.commit().await.unwrap();

            let tag_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM project_tags WHERE id = 'tag-a'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let link_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM project_task_tag_links WHERE tag_id = 'tag-a'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let history_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM project_task_change_events
                 WHERE field_name = 'tags'
                   AND old_value = 'Backend'
                   AND new_value IS NULL",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(tag_count, 0);
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
            default_event_name: None,
            default_event_time_mode: "timed".to_string(),
            default_event_duration_minutes: Some(60),
            default_pomodoro_mode: "none".to_string(),
            default_pomodoro_preset_key: None,
            default_pomodoro_focus_minutes: None,
            default_pomodoro_short_break_minutes: None,
            default_pomodoro_long_break_minutes: None,
            default_pomodoro_long_break_after_focus_count: None,
            default_idle_settings_source: "global".to_string(),
            default_idle_pause_enabled: true,
            default_idle_threshold_minutes: 3,
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
    fn project_update_rejects_blank_default_event_name() {
        let project = ProjectUpdate {
            id: "project-a".to_string(),
            group_id: "group-a".to_string(),
            name: "Project A".to_string(),
            icon: "folder".to_string(),
            color: None,
            sort_order: 100,
            status: "active".to_string(),
            default_event_name: Some(" ".to_string()),
            default_event_time_mode: "timed".to_string(),
            default_event_duration_minutes: Some(60),
            default_pomodoro_mode: "none".to_string(),
            default_pomodoro_preset_key: None,
            default_pomodoro_focus_minutes: None,
            default_pomodoro_short_break_minutes: None,
            default_pomodoro_long_break_minutes: None,
            default_pomodoro_long_break_after_focus_count: None,
            default_idle_settings_source: "global".to_string(),
            default_idle_pause_enabled: true,
            default_idle_threshold_minutes: 3,
            focus_playlist_id: None,
            break_playlist_id: None,
            work_environment_id: None,
            blocker_ruleset_id: None,
        };

        assert_eq!(
            validate_project_update(&project),
            Err("default_event_name cannot be empty".to_string())
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
    fn task_mutation_returns_the_authoritative_row_and_committed_history() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_memory_pool().await;
            insert_project_graph_fixture(&pool).await;
            let mut tx = pool.begin().await.unwrap();
            sqlx::query(
                "UPDATE project_tasks
                 SET title = 'Authoritative', updated_at = '2026-07-11T12:00:00.000Z'
                 WHERE id = 'task-a'",
            )
            .execute(&mut *tx)
            .await
            .unwrap();
            insert_task_change_event(
                &mut tx,
                "task-a",
                "updated",
                Some("title"),
                Some("task-a"),
                Some("Authoritative"),
            )
            .await
            .unwrap();
            tx.commit().await.unwrap();

            let mutation = task_mutation(&pool, "task-a").await.unwrap();

            assert_eq!(mutation.tasks.len(), 1);
            assert_eq!(mutation.tasks[0].title, "Authoritative");
            assert_eq!(mutation.tasks[0].updated_at, "2026-07-11T12:00:00.000Z");
            assert!(mutation.task_change_events.iter().any(|event| {
                event.task_id == "task-a"
                    && event.field_name.as_deref() == Some("title")
                    && event.new_value.as_deref() == Some("Authoritative")
            }));
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
