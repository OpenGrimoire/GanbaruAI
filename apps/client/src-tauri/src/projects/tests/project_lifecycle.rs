use super::*;

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
