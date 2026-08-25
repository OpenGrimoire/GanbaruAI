#[macro_use]
extern crate ganbaru_db;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod benchmark_seed;
mod calendar_description;
mod calendar_events;
mod calendar_import;
mod calendar_reads;
mod calendars;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[allow(dead_code)]
mod chat;
#[cfg(any(target_os = "android", target_os = "ios"))]
#[path = "chat_mobile.rs"]
mod chat;
#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod db;
mod db_path;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod doomscrolling;
#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod first_use_contracts;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod media_controls;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod media_player;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod music;
mod music_context;
mod music_error;
mod notes;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod notification;
mod pomodoro;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod pomodoro_enforcement;
mod profile_images;
mod project_icons;
mod projects;
mod quick_notes;
mod recurrence;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod soundscape;
mod themes;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod tray;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod updates;
mod vault;

fn install_default_tls_crypto_provider() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("the default TLS crypto provider must be installed only once during startup");
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod desktop_runtime;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile_runtime;
#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
#[path = "mobile_runtime.rs"]
#[allow(dead_code)]
mod mobile_runtime_typecheck;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use desktop_runtime::run;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile_runtime::run;

#[cfg(test)]
mod composition_tests {
    const MOBILE_RUNTIME: &str = include_str!("mobile_runtime.rs");

    #[test]
    fn mobile_runtime_registers_portable_core_commands() {
        for command in [
            "vault::vault_use_default_folder",
            "calendar_reads::calendar_load_window",
            "projects::workspace::projects_load_workspace",
            "notes::notes_load_workspace_shell",
            "pomodoro::pomodoro_start_run",
            "pomodoro::pomodoro_recover_mobile_run",
            "quick_notes::quick_notes_list",
            "themes::theme_load_all",
            "profile_images::profile_image_asset_data_url",
            "profile_images::profile_image_save_data_url",
            "project_icons::project_icon_asset_data_url",
            "vault::vault_pick_and_read_theme_json",
            "vault::vault_pick_and_write_theme_json",
        ] {
            assert!(
                MOBILE_RUNTIME.contains(command),
                "missing mobile command {command}"
            );
        }
    }

    #[test]
    fn mobile_runtime_excludes_desktop_only_commands() {
        for command in [
            "chat::",
            "doomscrolling::",
            "media_player::",
            "music::",
            "notification::",
            "soundscape::",
            "tray::",
            "updates::",
            "benchmark_seed::",
            "vault::vault_pick_create",
            "vault::vault_pick_open",
            "vault::vault_pick_and_read_ics_import",
            "vault::vault_pick_and_write_ics_export",
            "notes::notes_pick_",
            "notes::working_markdown::",
            "notes::notes_import_notion_export_folder",
            "notes::notes_prepare_import_file_reference",
            "profile_images::profile_image_pick_file",
            "project_icons::project_icon_pick_image_file",
            "project_icons::project_icon_download_image_url",
            "project_icons::project_icon_asset_path",
            "pomodoro::pomodoro_recover_open_runs",
        ] {
            assert!(
                !MOBILE_RUNTIME.contains(command),
                "mobile runtime must exclude {command}"
            );
        }
    }
}
