package app.ganbaru.mobile_notifications

import android.app.Activity
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.media.AudioAttributes
import android.media.RingtoneManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
internal class CalendarChannelArgs {
  lateinit var name: String
  lateinit var description: String
}

@InvokeArg
internal class CalendarTestNotificationArgs {
  lateinit var title: String
  var body: String = ""
}

@InvokeArg
internal class CalendarNotificationArgs {
  var id: Int = 0
  lateinit var title: String
  var body: String = ""
  lateinit var eventId: String
  var scheduledAtEpochMs: Long = 0
}

@InvokeArg
internal class CalendarNotificationBatchArgs {
  var notifications: List<CalendarNotificationArgs> = listOf()
}

@InvokeArg
internal class CalendarNotificationCancelArgs {
  var ids: List<Int> = listOf()
}

internal object ExactAlarmCapability {
  fun isRequired(apiLevel: Int): Boolean = apiLevel >= Build.VERSION_CODES.S

  fun isGranted(apiLevel: Int, alarmManager: AlarmManager): Boolean =
    !isRequired(apiLevel) || alarmManager.canScheduleExactAlarms()
}

@TauriPlugin
class MobileNotificationsPlugin(private val activity: Activity) : Plugin(activity) {
  override fun load(webView: WebView) {
    super.load(webView)
    CalendarNotificationScheduler.captureAction(activity, activity.intent)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    CalendarNotificationScheduler.captureAction(activity, intent)
  }

  @Command
  fun ensureCalendarChannel(invoke: Invoke) {
    val args = invoke.parseArgs(CalendarChannelArgs::class.java)
    if (args.name.isBlank()) {
      invoke.reject("Calendar notification channel name cannot be blank")
      return
    }

    try {
      val notificationManager =
        activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
      val audioAttributes = AudioAttributes.Builder()
        .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
        .setUsage(AudioAttributes.USAGE_NOTIFICATION_EVENT)
        .build()
      val channel = NotificationChannel(
        CALENDAR_CHANNEL_ID,
        args.name,
        NotificationManager.IMPORTANCE_HIGH,
      ).apply {
        description = args.description
        lockscreenVisibility = Notification.VISIBILITY_PRIVATE
        enableLights(true)
        enableVibration(true)
        setSound(
          RingtoneManager.getDefaultUri(RingtoneManager.TYPE_NOTIFICATION),
          audioAttributes,
        )
      }
      notificationManager.createNotificationChannel(channel)
      invoke.resolve()
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Failed to create Calendar notification channel")
    }
  }

  @Command
  fun showCalendarTestNotification(invoke: Invoke) {
    val args = invoke.parseArgs(CalendarTestNotificationArgs::class.java)
    try {
      CalendarNotificationScheduler.showTest(activity, args.title, args.body)
      invoke.resolve()
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Failed to show Calendar test notification")
    }
  }

  @Command
  fun calendarChannelStatus(invoke: Invoke) {
    val notificationManager =
      activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    val channel = notificationManager.getNotificationChannel(CALENDAR_CHANNEL_ID)
    invoke.resolve(JSObject().apply {
      put("exists", channel != null)
      put("enabled", channel != null && channel.importance != NotificationManager.IMPORTANCE_NONE)
      put("soundConfigured", channel?.sound != null)
    })
  }

  @Command
  fun scheduleCalendarNotifications(invoke: Invoke) {
    val args = invoke.parseArgs(CalendarNotificationBatchArgs::class.java)
    try {
      for (notification in args.notifications) {
        CalendarNotificationScheduler.schedule(
          activity,
          CalendarNotificationDelivery(
            id = notification.id,
            title = notification.title,
            body = notification.body,
            eventId = notification.eventId,
            scheduledAtEpochMs = notification.scheduledAtEpochMs,
          ),
        )
      }
      invoke.resolveObject(args.notifications.map { it.id })
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Failed to schedule Calendar notifications")
    }
  }

  @Command
  fun pendingCalendarNotifications(invoke: Invoke) {
    val pending = CalendarNotificationScheduler.pending(activity).map { notification ->
      JSObject().apply {
        put("id", notification.id)
        put("title", notification.title)
        put("body", notification.body)
        put("eventId", notification.eventId)
        put("scheduledAtEpochMs", notification.scheduledAtEpochMs)
      }
    }
    invoke.resolveObject(pending)
  }

  @Command
  fun cancelCalendarNotifications(invoke: Invoke) {
    val args = invoke.parseArgs(CalendarNotificationCancelArgs::class.java)
    CalendarNotificationScheduler.cancel(activity, args.ids)
    invoke.resolve()
  }

  @Command
  fun takeCalendarNotificationAction(invoke: Invoke) {
    invoke.resolveObject(JSObject().apply {
      put("eventId", CalendarNotificationScheduler.takeAction(activity))
    })
  }

  @Command
  fun exactAlarmStatus(invoke: Invoke) {
    val apiLevel = Build.VERSION.SDK_INT
    val alarmManager = activity.getSystemService(Context.ALARM_SERVICE) as AlarmManager
    invoke.resolve(JSObject().apply {
      put("apiLevel", apiLevel)
      put("required", ExactAlarmCapability.isRequired(apiLevel))
      put("granted", ExactAlarmCapability.isGranted(apiLevel, alarmManager))
    })
  }

  @Command
  fun openExactAlarmSettings(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) {
      invoke.resolve()
      return
    }

    try {
      activity.startActivity(
        Intent(
          Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM,
          Uri.parse("package:${activity.packageName}"),
        ),
      )
      invoke.resolve()
    } catch (error: Exception) {
      try {
        activity.startActivity(
          Intent(
            Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
            Uri.parse("package:${activity.packageName}"),
          ),
        )
        invoke.resolve()
      } catch (fallbackError: Exception) {
        invoke.reject(
          fallbackError.message ?: error.message ?: "Failed to open Android settings",
        )
      }
    }
  }

  @Command
  fun openNotificationSettings(invoke: Invoke) {
    try {
      activity.startActivity(
        Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
          putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName)
        },
      )
      invoke.resolve()
    } catch (error: Exception) {
      try {
        activity.startActivity(
          Intent(
            Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
            Uri.parse("package:${activity.packageName}"),
          ),
        )
        invoke.resolve()
      } catch (fallbackError: Exception) {
        invoke.reject(
          fallbackError.message ?: error.message ?: "Failed to open Android settings",
        )
      }
    }
  }
}
