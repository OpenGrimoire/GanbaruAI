use super::*;

pub(super) fn validate_group_create(group: &ProjectGroupCreate) -> Result<(), String> {
    require_non_empty(&group.id, "id")?;
    require_non_empty(&group.name, "name")?;
    require_non_empty(&group.icon, "icon")?;
    validate_color(group.color)?;
    validate_non_negative(group.sort_order, "sort_order")
}

pub(super) fn validate_group_update(group: &ProjectGroupUpdate) -> Result<(), String> {
    require_non_empty(&group.id, "id")?;
    require_non_empty(&group.name, "name")?;
    require_non_empty(&group.icon, "icon")?;
    validate_color(group.color)?;
    validate_non_negative(group.sort_order, "sort_order")
}

pub(super) fn validate_project_create(project: &ProjectCreate) -> Result<(), String> {
    require_non_empty(&project.id, "id")?;
    require_non_empty(&project.group_id, "group_id")?;
    validate_enum(&project.template_id, "template_id", PROJECT_TEMPLATE_IDS)?;
    require_non_empty(&project.name, "name")?;
    require_non_empty(&project.icon, "icon")?;
    validate_color(project.color)?;
    validate_non_negative(project.sort_order, "sort_order")?;
    validate_optional_text(&project.default_event_name, "default_event_name")?;
    validate_project_event_duration(project.default_event_duration_minutes)?;
    validate_project_default_pomodoro(
        &project.default_pomodoro_mode,
        project.default_pomodoro_preset_key.as_deref(),
        project.default_pomodoro_focus_minutes,
        project.default_pomodoro_short_break_minutes,
        project.default_pomodoro_long_break_minutes,
        project.default_pomodoro_long_break_after_focus_count,
    )?;
    validate_project_default_idle_settings(
        &project.default_idle_settings_source,
        project.default_idle_threshold_minutes,
    )?;
    Ok(())
}

pub(super) fn validate_project_update(project: &ProjectUpdate) -> Result<(), String> {
    require_non_empty(&project.id, "id")?;
    require_non_empty(&project.group_id, "group_id")?;
    require_non_empty(&project.name, "name")?;
    require_non_empty(&project.icon, "icon")?;
    validate_color(project.color)?;
    validate_non_negative(project.sort_order, "sort_order")?;
    validate_enum(&project.status, "status", &["active", "hidden", "archived"])?;
    validate_optional_text(&project.default_event_name, "default_event_name")?;
    validate_project_event_duration(project.default_event_duration_minutes)?;
    validate_project_default_pomodoro(
        &project.default_pomodoro_mode,
        project.default_pomodoro_preset_key.as_deref(),
        project.default_pomodoro_focus_minutes,
        project.default_pomodoro_short_break_minutes,
        project.default_pomodoro_long_break_minutes,
        project.default_pomodoro_long_break_after_focus_count,
    )?;
    validate_project_default_idle_settings(
        &project.default_idle_settings_source,
        project.default_idle_threshold_minutes,
    )?;
    validate_optional_identifier(&project.focus_playlist_id, "focus_playlist_id")?;
    validate_optional_identifier(&project.break_playlist_id, "break_playlist_id")?;
    validate_optional_identifier(&project.work_environment_id, "work_environment_id")?;
    validate_optional_identifier(&project.blocker_ruleset_id, "blocker_ruleset_id")?;
    Ok(())
}

pub(super) fn validate_section_create(section: &ProjectSectionCreate) -> Result<(), String> {
    require_non_empty(&section.id, "id")?;
    require_non_empty(&section.project_id, "project_id")?;
    require_non_empty(&section.name, "name")?;
    validate_non_negative(section.sort_order, "sort_order")
}

pub(super) fn validate_section_update(section: &ProjectSectionUpdate) -> Result<(), String> {
    require_non_empty(&section.id, "id")?;
    require_non_empty(&section.name, "name")?;
    validate_non_negative(section.sort_order, "sort_order")
}

pub(super) fn validate_status_create(status: &ProjectStatusCreate) -> Result<(), String> {
    require_non_empty(&status.id, "id")?;
    require_non_empty(&status.project_id, "project_id")?;
    validate_status_fields(
        &status.name,
        &status.category,
        status.sort_order,
        status.terminal,
    )
}

pub(super) fn validate_status_update(status: &ProjectStatusUpdate) -> Result<(), String> {
    require_non_empty(&status.id, "id")?;
    validate_status_fields(
        &status.name,
        &status.category,
        status.sort_order,
        status.terminal,
    )
}

pub(super) fn validate_status_fields(
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

pub(super) fn validate_task_create(task: &ProjectTaskCreate) -> Result<(), String> {
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

pub(super) fn validate_optional_identifier(
    value: &Option<String>,
    field: &str,
) -> Result<(), String> {
    validate_optional_text(value, field)
}

pub(super) fn validate_optional_text(value: &Option<String>, field: &str) -> Result<(), String> {
    if let Some(text) = value {
        require_non_empty(text, field)?;
    }
    Ok(())
}

pub(super) fn validate_task_update(task: &ProjectTaskUpdate) -> Result<(), String> {
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
    validate_optional_task_time(&task.start_time, "start_time")?;
    validate_optional_task_time(&task.due_time, "due_time")?;
    if task.start_time.is_some() && task.start_date.is_none() {
        return Err("start_time requires start_date".to_string());
    }
    if task.due_time.is_some() && task.due_date.is_none() {
        return Err("due_time requires due_date".to_string());
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

fn validate_optional_task_time(value: &Option<String>, field: &str) -> Result<(), String> {
    let Some(time) = value else {
        return Ok(());
    };
    let bytes = time.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || !bytes[0].is_ascii_digit()
        || !bytes[1].is_ascii_digit()
        || !bytes[3].is_ascii_digit()
        || !bytes[4].is_ascii_digit()
    {
        return Err(format!("{field} must use HH:MM"));
    }
    let hour = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    let minute = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');
    if hour > 23 || minute > 59 {
        return Err(format!("{field} must use HH:MM"));
    }
    Ok(())
}

pub(super) fn validate_task_dependency_create(
    dependency: &ProjectTaskDependencyCreate,
) -> Result<(), String> {
    require_non_empty(&dependency.id, "id")?;
    require_non_empty(&dependency.blocking_task_id, "blocking_task_id")?;
    require_non_empty(&dependency.blocked_task_id, "blocked_task_id")?;
    validate_enum(&dependency.dependency_type, "dependency_type", &["blocks"])?;
    if dependency.blocking_task_id == dependency.blocked_task_id {
        return Err("task cannot depend on itself".to_string());
    }
    Ok(())
}

pub(super) fn validate_checklist_item_create(
    item: &ProjectChecklistItemCreate,
) -> Result<(), String> {
    require_non_empty(&item.id, "id")?;
    require_non_empty(&item.task_id, "task_id")?;
    require_non_empty(&item.title, "title")?;
    validate_non_negative(item.sort_order, "sort_order")
}

pub(super) fn validate_checklist_item_update(
    item: &ProjectChecklistItemUpdate,
) -> Result<(), String> {
    require_non_empty(&item.id, "id")?;
    require_non_empty(&item.title, "title")?;
    validate_non_negative(item.sort_order, "sort_order")
}

pub(super) fn validate_label_create(label: &ProjectLabelCreate) -> Result<(), String> {
    require_non_empty(&label.id, "id")?;
    require_non_empty(&label.project_id, "project_id")?;
    require_non_empty(&label.name, "name")?;
    validate_color(label.color)?;
    validate_non_negative(label.sort_order, "sort_order")
}

pub(super) fn validate_label_update(label: &ProjectLabelUpdate) -> Result<(), String> {
    require_non_empty(&label.id, "id")?;
    require_non_empty(&label.name, "name")?;
    validate_color(label.color)?;
    validate_non_negative(label.sort_order, "sort_order")
}

pub(super) fn validate_task_label_link_create(
    link: &ProjectTaskLabelLinkCreate,
) -> Result<(), String> {
    require_non_empty(&link.task_id, "task_id")?;
    require_non_empty(&link.label_id, "label_id")
}

pub(super) fn validate_custom_field_create(field: &ProjectCustomFieldCreate) -> Result<(), String> {
    require_non_empty(&field.id, "id")?;
    require_non_empty(&field.project_id, "project_id")?;
    require_non_empty(&field.name, "name")?;
    validate_custom_field_type(&field.field_type)?;
    validate_non_negative(field.sort_order, "sort_order")
}

pub(super) fn validate_custom_field_update(field: &ProjectCustomFieldUpdate) -> Result<(), String> {
    require_non_empty(&field.id, "id")?;
    require_non_empty(&field.name, "name")?;
    validate_non_negative(field.sort_order, "sort_order")
}

pub(super) fn validate_custom_field_option_create(
    option: &ProjectCustomFieldOptionCreate,
) -> Result<(), String> {
    require_non_empty(&option.id, "id")?;
    require_non_empty(&option.field_id, "field_id")?;
    require_non_empty(&option.name, "name")?;
    validate_non_negative(option.sort_order, "sort_order")
}

pub(super) fn validate_custom_field_option_update(
    option: &ProjectCustomFieldOptionUpdate,
) -> Result<(), String> {
    require_non_empty(&option.id, "id")?;
    require_non_empty(&option.name, "name")?;
    validate_non_negative(option.sort_order, "sort_order")
}

pub(super) fn validate_custom_field_value_update(
    value: &ProjectCustomFieldValueUpdate,
) -> Result<(), String> {
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

pub(super) fn validate_custom_field_type(field_type: &str) -> Result<(), String> {
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

pub(super) fn validate_task_event_link_create(
    link: &ProjectTaskEventLinkCreate,
) -> Result<(), String> {
    require_non_empty(&link.task_id, "task_id")?;
    require_non_empty(&link.event_id, "event_id")?;
    validate_enum(&link.link_kind, "link_kind", &["scheduled", "reference"])
}

pub(super) fn validate_view_preference(
    preference: &ProjectViewPreferenceUpsert,
) -> Result<(), String> {
    require_non_empty(&preference.project_id, "project_id")?;
    validate_enum(
        &preference.view_id,
        "view_id",
        &["dashboard", "list", "kanban", "calendar", "gantt"],
    )?;
    require_non_empty(&preference.preference_key, "preference_key")?;
    if preference.preference_value.len() > 20_000 {
        return Err("preference_value is too large".to_string());
    }
    Ok(())
}

pub(super) fn validate_custom_emoji_create(emoji: &ProjectCustomEmojiCreate) -> Result<(), String> {
    require_non_empty(&emoji.id, "id")?;
    require_non_empty(&emoji.name, "name")?;
    validate_project_icon_asset_path(&emoji.asset_path)?;
    validate_non_negative(emoji.sort_order, "sort_order")
}

fn validate_project_icon_asset_path(asset_path: &str) -> Result<(), String> {
    let trimmed = asset_path.trim();
    let Some(file_name) = trimmed.strip_prefix("project-icons/") else {
        return Err("asset_path must stay under project-icons".to_string());
    };
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
    {
        return Err("asset_path cannot contain nested or parent paths".to_string());
    }
    let extension = file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .unwrap_or_default();
    if !["png", "jpg", "jpeg", "webp"]
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err("asset_path must use a PNG, JPG, or WebP image".to_string());
    }
    Ok(())
}

pub(super) fn validate_color(value: Option<i64>) -> Result<(), String> {
    if value.is_some_and(|color| !(0..PALETTE_SIZE).contains(&color)) {
        return Err("color is outside the event palette".to_string());
    }
    Ok(())
}

pub(super) fn validate_project_event_duration(value: Option<i64>) -> Result<(), String> {
    if let Some(minutes) = value {
        if minutes <= 0 {
            return Err("default_event_duration_minutes must be positive".to_string());
        }
        if minutes > MAX_PROJECT_EVENT_DURATION_MINUTES {
            return Err("default_event_duration_minutes must be at most 24 hours".to_string());
        }
    }
    Ok(())
}

pub(super) fn validate_pomodoro_preset(value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        validate_enum(
            value,
            "default_pomodoro_preset_key",
            &["adaptive", "creative", "balanced", "deep", "extended"],
        )?;
    }
    Ok(())
}

fn validate_project_default_pomodoro(
    mode: &str,
    preset_key: Option<&str>,
    focus_minutes: Option<i64>,
    short_break_minutes: Option<i64>,
    long_break_minutes: Option<i64>,
    long_break_after_focus_count: Option<i64>,
) -> Result<(), String> {
    validate_enum(mode, "default_pomodoro_mode", &["none", "preset", "custom"])?;
    match mode {
        "none" => {
            if preset_key.is_some()
                || focus_minutes.is_some()
                || short_break_minutes.is_some()
                || long_break_minutes.is_some()
                || long_break_after_focus_count.is_some()
            {
                return Err("default_pomodoro_mode none cannot include config values".to_string());
            }
        }
        "preset" => {
            validate_pomodoro_preset(preset_key)?;
            if focus_minutes.is_some()
                || short_break_minutes.is_some()
                || long_break_minutes.is_some()
                || long_break_after_focus_count.is_some()
            {
                return Err("default_pomodoro_mode preset cannot include custom values".to_string());
            }
        }
        "custom" => {
            if preset_key.is_some() {
                return Err("default_pomodoro_mode custom cannot include preset_key".to_string());
            }
            validate_range(
                focus_minutes,
                "default_pomodoro_focus_minutes",
                1,
                MAX_PROJECT_POMODORO_FOCUS_MINUTES,
            )?;
            validate_range(
                short_break_minutes,
                "default_pomodoro_short_break_minutes",
                1,
                MAX_PROJECT_POMODORO_SHORT_BREAK_MINUTES,
            )?;
            validate_range(
                long_break_minutes,
                "default_pomodoro_long_break_minutes",
                1,
                MAX_PROJECT_POMODORO_LONG_BREAK_MINUTES,
            )?;
            validate_range(
                long_break_after_focus_count,
                "default_pomodoro_long_break_after_focus_count",
                1,
                MAX_PROJECT_POMODORO_CYCLE_COUNT,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_project_default_idle_settings(
    source: &str,
    threshold_minutes: i64,
) -> Result<(), String> {
    validate_enum(
        source,
        "default_idle_settings_source",
        PROJECT_IDLE_SETTINGS_SOURCES,
    )?;
    if !PROJECT_IDLE_THRESHOLD_MINUTES.contains(&threshold_minutes) {
        return Err("default_idle_threshold_minutes has unsupported value".to_string());
    }
    Ok(())
}

fn validate_range(value: Option<i64>, field: &str, min: i64, max: i64) -> Result<(), String> {
    let Some(value) = value else {
        return Err(format!("{field} is required"));
    };
    if !(min..=max).contains(&value) {
        return Err(format!("{field} must be between {min} and {max}"));
    }
    Ok(())
}

pub(super) fn validate_enum(value: &str, field: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("{field} has unsupported value '{value}'"))
    }
}

pub(super) fn validate_non_negative(value: i64, field: &str) -> Result<(), String> {
    if value < 0 {
        Err(format!("{field} must be non-negative"))
    } else {
        Ok(())
    }
}

pub(super) fn require_non_empty(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} cannot be empty"))
    } else {
        Ok(())
    }
}

pub(in crate::projects) fn validate_date(value: &str, field: &str) -> Result<(), String> {
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

pub(super) fn normalize_optional_date_filter(
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

pub(super) fn sql_like_contains_pattern(value: &str) -> String {
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
