fn main() {
    tauri_plugin::Builder::new(&[
        "accessStatus",
        "openUsageAccessSettings",
        "openAccessibilitySettings",
        "listLaunchableApps",
        "applyRules",
        "pendingEvents",
        "acknowledgeEvents",
        "takeNotificationAction",
    ])
    .android_path("android")
    .build();
}
