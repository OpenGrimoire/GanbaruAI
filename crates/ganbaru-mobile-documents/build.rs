fn main() {
    tauri_plugin::Builder::new(&[
        "pickUtf8Document",
        "pickDocumentToPath",
        "saveUtf8Download",
        "saveFileDownload",
        "pickVaultTreeToPath",
    ])
    .android_path("android")
    .build();
}
