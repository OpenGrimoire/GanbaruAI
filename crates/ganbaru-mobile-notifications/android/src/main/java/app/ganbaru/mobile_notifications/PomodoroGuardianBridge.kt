package app.ganbaru.mobile_notifications

import android.content.ContentProvider
import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import android.net.Uri
import android.os.Bundle
import org.json.JSONArray
import org.json.JSONObject

private const val GUARDIAN_AUTHORITY_SUFFIX = ".ganbaru.guardian.notifications"
private const val METHOD_UPDATE = "updatePomodoro"
private const val METHOD_CANCEL = "cancelPomodoro"
private const val METHOD_CURRENT = "currentPomodoro"
private const val METHOD_RECONCILE = "reconcilePomodoroSchedule"
private const val KEY_PROJECTION = "projection"
private const val KEY_SCHEDULE = "schedule"
private const val MAX_SCHEDULE_IPC_BYTES = 768 * 1024
internal val POMODORO_GUARDIAN_LOCK = Any()

internal class PomodoroGuardianClient(private val context: Context) {
  fun update(projection: PomodoroNotificationProjection) {
    call(METHOD_UPDATE, Bundle().apply {
      putString(KEY_PROJECTION, PomodoroNotificationScheduler.encode(projection))
    })
  }

  fun cancel() {
    call(METHOD_CANCEL)
  }

  fun current(): PomodoroNotificationProjection? {
    val encoded = call(METHOD_CURRENT).getString(KEY_PROJECTION) ?: return null
    return PomodoroNotificationScheduler.decode(encoded)
      ?: error("Guardian returned an invalid Pomodoro projection")
  }

  fun reconcile(schedule: List<PomodoroReminder>) {
    val encoded = encodeSchedule(schedule)
    require(encoded.toByteArray().size <= MAX_SCHEDULE_IPC_BYTES) {
      "Focus reminder schedule exceeds the Android IPC limit"
    }
    call(METHOD_RECONCILE, Bundle().apply { putString(KEY_SCHEDULE, encoded) })
  }

  private fun call(method: String, extras: Bundle? = null): Bundle =
    context.contentResolver.call(uri(context), method, null, extras)
      ?: error("Pomodoro guardian did not return a response")

  companion object {
    private fun uri(context: Context): Uri = Uri.Builder()
      .scheme("content")
      .authority(context.packageName + GUARDIAN_AUTHORITY_SUFFIX)
      .build()

    internal fun encodeSchedule(schedule: List<PomodoroReminder>): String =
      JSONArray().apply {
        schedule.forEach { projection ->
          put(JSONObject(projection.encode()))
        }
      }.toString()

    internal fun decodeSchedule(encoded: String): List<PomodoroReminder> {
      require(encoded.toByteArray().size <= MAX_SCHEDULE_IPC_BYTES) {
        "Focus reminder schedule exceeds the Android IPC limit"
      }
      val values = JSONArray(encoded)
      require(values.length() <= 128) {
        "Focus reminder schedule must contain at most 128 events"
      }
      return (0 until values.length()).map { index ->
        PomodoroReminder.decode(values.getJSONObject(index).toString())
      }
    }
  }
}

/** Owns Pomodoro runtime state inside Ganbaru AI's private Android guardian process. */
class PomodoroGuardianProvider : ContentProvider() {
  override fun onCreate(): Boolean = true

  override fun call(method: String, arg: String?, extras: Bundle?): Bundle {
    val appContext = requireNotNull(context).applicationContext
    return synchronized(POMODORO_GUARDIAN_LOCK) {
      when (method) {
        METHOD_UPDATE -> {
          val encoded = extras.requireString(KEY_PROJECTION)
          val projection = PomodoroNotificationScheduler.decode(encoded)
            ?: error("Pomodoro projection is invalid")
          PomodoroNotificationScheduler.update(appContext, projection)
          Bundle.EMPTY
        }
        METHOD_CANCEL -> {
          PomodoroNotificationScheduler.cancel(appContext)
          Bundle.EMPTY
        }
        METHOD_CURRENT -> Bundle().apply {
          PomodoroNotificationScheduler.current(appContext)?.let { projection ->
            putString(KEY_PROJECTION, PomodoroNotificationScheduler.encode(projection))
          }
        }
        METHOD_RECONCILE -> {
          val schedule = PomodoroGuardianClient.decodeSchedule(extras.requireString(KEY_SCHEDULE))
          PomodoroReminderScheduler.reconcile(appContext, schedule)
          Bundle.EMPTY
        }
        else -> error("Unknown Pomodoro guardian operation")
      }
    }
  }

  override fun query(
    uri: Uri,
    projection: Array<out String>?,
    selection: String?,
    selectionArgs: Array<out String>?,
    sortOrder: String?,
  ): Cursor? = unsupported()

  override fun getType(uri: Uri): String? = unsupported()

  override fun insert(uri: Uri, values: ContentValues?): Uri? = unsupported()

  override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int =
    unsupported()

  override fun update(
    uri: Uri,
    values: ContentValues?,
    selection: String?,
    selectionArgs: Array<out String>?,
  ): Int = unsupported()

  private fun <T> unsupported(): T = throw UnsupportedOperationException(
    "Pomodoro guardian supports private call operations only",
  )

  private fun Bundle?.requireString(key: String): String =
    this?.getString(key)?.takeIf(String::isNotBlank)
      ?: error("Pomodoro guardian argument is missing")

}
