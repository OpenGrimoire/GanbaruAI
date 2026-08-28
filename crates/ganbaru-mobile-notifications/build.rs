fn main() {
    tauri_plugin::Builder::new(&[
        "ensureCalendarChannel",
        "calendarChannelStatus",
        "scheduleCalendarNotifications",
        "pendingCalendarNotifications",
        "cancelCalendarNotifications",
        "takeCalendarNotificationAction",
        "updatePomodoroNotification",
        "cancelPomodoroNotification",
        "pomodoroNotificationState",
        "exactAlarmStatus",
        "openExactAlarmSettings",
        "openNotificationSettings",
    ])
    .android_path("android")
    .build();
}
