import { invoke } from "@tauri-apps/api/core";

export interface MobileNotificationAccessStatus {
  permission: "granted" | "denied" | "prompt";
  channel: {
    exists: boolean;
    enabled: boolean;
    soundConfigured: boolean;
  };
  exactAlarm: {
    apiLevel: number;
    required: boolean;
    granted: boolean;
  };
}

/** Read Android notification permission and exact-alarm access together. */
export async function mobileNotificationAccessStatus(): Promise<MobileNotificationAccessStatus> {
  const [permissionGranted, channel, exactAlarm] = await Promise.all([
    invoke<boolean | null>("plugin:notification|is_permission_granted"),
    invoke<MobileNotificationAccessStatus["channel"]>(
      "plugin:ganbaru-mobile-notifications|calendarChannelStatus",
    ),
    invoke<MobileNotificationAccessStatus["exactAlarm"]>(
      "mobile_notification_exact_alarm_status",
    ),
  ]);
  return {
    permission: permissionGranted === true
      ? "granted"
      : permissionGranted === false ? "denied" : "prompt",
    channel,
    exactAlarm,
  };
}

/** Request Android notification permission in response to a user action. */
export async function requestMobileNotificationPermission(): Promise<MobileNotificationAccessStatus> {
  await invoke("plugin:notification|request_permission");
  return mobileNotificationAccessStatus();
}

/** Open the relevant Android system settings for the current delivery limitation. */
export async function resolveMobileNotificationAccess(
  status: MobileNotificationAccessStatus,
): Promise<void> {
  if (status.permission !== "granted") {
    await invoke("mobile_notification_open_settings");
  } else if (
    status.channel.exists
    && (!status.channel.enabled || !status.channel.soundConfigured)
  ) {
    await invoke("mobile_notification_open_settings");
  } else if (status.exactAlarm.required && !status.exactAlarm.granted) {
    await invoke("mobile_notification_open_exact_alarm_settings");
  }
}

/** Open Android's app-specific Alarms and reminders access screen. */
export async function openMobileExactAlarmSettings(): Promise<void> {
  await invoke("mobile_notification_open_exact_alarm_settings");
}
