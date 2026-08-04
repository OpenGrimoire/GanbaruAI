#![cfg(mobile)]

#[tauri::mobile_entry_point]
pub fn run() {
    ganbaru_tauri_app::run(tauri::generate_context!())
}
