#[test]
fn mutation_result_preserves_camel_case_wire_shape() {
    let result = super::super::NotesMutationResultDto {
        value: "saved",
        next_history_checkpoint_at: Some("2026-07-12T12:00:00.000Z".to_string()),
    };

    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::json!({
            "value": "saved",
            "nextHistoryCheckpointAt": "2026-07-12T12:00:00.000Z"
        })
    );
}

#[test]
fn project_history_commands_are_registered_at_defining_modules() {
    let handlers = include_str!("../../../lib.rs");
    for command in [
        "notes::project_history::schedule::notes_initialize_project_history",
        "notes::project_history::schedule::notes_flush_due_project_history",
        "notes::project_history::reads::notes_list_project_history_versions",
        "notes::project_history::reads::notes_load_project_history_tree",
        "notes::project_history::reads::notes_load_project_history_page",
        "notes::project_history::retention::notes_get_history_retention_impact",
        "notes::project_history::retention::notes_prune_project_history",
        "notes::project_history::commands::notes_preview_project_history_restore",
        "notes::project_history::commands::notes_restore_project_history_version",
    ] {
        assert!(handlers.contains(command), "missing handler {command}");
    }
}
