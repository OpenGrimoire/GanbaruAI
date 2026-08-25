fn main() {
    tauri_plugin::Builder::new(&["pickUtf8Document", "saveUtf8Download"])
        .android_path("android")
        .build();
}
