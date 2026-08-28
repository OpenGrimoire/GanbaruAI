fn main() {
    tauri_plugin::Builder::new(&[
        "ensureCalendarChannel",
        "showCalendarTestNotification",
        "calendarChannelStatus",
        "scheduleCalendarNotifications",
        "pendingCalendarNotifications",
        "cancelCalendarNotifications",
        "takeCalendarNotificationAction",
        "exactAlarmStatus",
        "openExactAlarmSettings",
        "openNotificationSettings",
    ])
    .android_path("android")
    .build();
}
