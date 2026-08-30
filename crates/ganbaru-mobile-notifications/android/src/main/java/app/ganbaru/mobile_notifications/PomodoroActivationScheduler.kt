package app.ganbaru.mobile_notifications

import android.app.AlarmManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build

private const val ACTIVATION_STORE = "GANBARU_POMODORO_ACTIVATION_STORE"
private const val DISMISSED_ACTIVATIONS_KEY = "dismissedActivationIds"
private const val EXTRA_ACTIVATION_ID = "ganbaruPomodoroActivationId"
private const val ACTIVATION_REQUEST_CODE_START = 1_600_000_000
private const val ACTIVATION_REQUEST_CODE_SPAN = 100_000_000

internal object PomodoroActivationSelection {
  fun select(
    schedule: List<PomodoroNotificationProjection>,
    dismissed: Set<String>,
    now: Long,
    current: PomodoroNotificationProjection?,
  ): PomodoroNotificationProjection? {
    if (current != null && current.eventEndsAtEpochMs > now) return current
    return schedule
      .filter {
        it.runId !in dismissed
          && it.phases.first().startsAtEpochMs <= now
          && now < it.eventEndsAtEpochMs
      }
      .minWithOrNull(compareBy<PomodoroNotificationProjection> { it.eventEndsAtEpochMs }
        .thenBy { it.runId })
  }
}

internal object PomodoroActivationScheduler {
  fun reconcile(context: Context, schedule: List<PomodoroNotificationProjection>) {
    require(schedule.size <= 128) { "Pomodoro activation schedule must contain at most 128 events" }
    val identifiers = mutableSetOf<String>()
    val requestCodes = mutableSetOf<Int>()
    schedule.forEach { projection ->
      PomodoroNotificationScheduler.validate(projection)
      require(identifiers.add(projection.runId)) { "Pomodoro activation IDs must be unique" }
      require(requestCodes.add(requestCode(projection.runId))) {
        "Pomodoro activation alarm identifier collision"
      }
    }

    val previous = pending(context)
    previous.forEach { cancelAlarm(context, it.runId) }
    val desiredIds = schedule.mapTo(mutableSetOf()) { it.runId }
    val dismissed = dismissed(context).intersect(desiredIds)
    val editor = store(context).edit().clear()
    if (dismissed.isNotEmpty()) editor.putStringSet(DISMISSED_ACTIVATIONS_KEY, dismissed)
    schedule.forEach { projection ->
      editor.putString(projection.runId, PomodoroNotificationScheduler.encode(projection))
    }
    check(editor.commit()) { "Pomodoro activation schedule could not be persisted" }
    schedule.forEach { projection ->
      if (projection.runId !in dismissed) scheduleAlarm(context, projection)
    }
    activateEligible(context)
  }

  fun pending(context: Context): List<PomodoroNotificationProjection> =
    store(context).all.mapNotNull { (key, encoded) ->
      if (key == DISMISSED_ACTIVATIONS_KEY) return@mapNotNull null
      PomodoroNotificationScheduler.decode(encoded as? String ?: return@mapNotNull null)
    }

  fun restore(context: Context) {
    val now = System.currentTimeMillis()
    pending(context).forEach { projection ->
      if (projection.eventEndsAtEpochMs > now && projection.runId !in dismissed(context)) {
        scheduleAlarm(context, projection)
      } else if (projection.eventEndsAtEpochMs <= now) {
        remove(context, projection.runId)
      }
    }
    activateEligible(context)
  }

  fun activate(context: Context, activationId: String?) {
    val desired = pending(context).firstOrNull { it.runId == activationId } ?: return
    val now = System.currentTimeMillis()
    if (
      desired.runId in dismissed(context)
      || now < desired.phases.first().startsAtEpochMs
      || now >= desired.eventEndsAtEpochMs
    ) return
    activateEligible(context)
  }

  fun activateEligible(context: Context): PomodoroNotificationProjection? {
    val now = System.currentTimeMillis()
    val current = PomodoroNotificationScheduler.current(context)
    if (current != null && current.eventEndsAtEpochMs > now) return current
    val selected = PomodoroActivationSelection.select(
      pending(context),
      dismissed(context),
      now,
      current,
    ) ?: return null
    PomodoroNotificationScheduler.save(context, selected)
    PomodoroNotificationService.synchronize(context)
    return selected
  }

  fun nextEligible(context: Context): PomodoroNotificationProjection? {
    PomodoroNotificationScheduler.clearCurrent(context)
    return activateEligible(context)
  }

  fun dismissCurrent(context: Context) {
    val current = PomodoroNotificationScheduler.current(context) ?: return
    val now = System.currentTimeMillis()
    val matching = pending(context).filter {
      it.eventId == current.eventId
        && it.phases.first().startsAtEpochMs <= now
        && now < it.eventEndsAtEpochMs
    }
    if (matching.isEmpty()) return
    val next = dismissed(context).toMutableSet()
    matching.forEach {
      next.add(it.runId)
      cancelAlarm(context, it.runId)
    }
    saveDismissed(context, next)
  }

  private fun scheduleAlarm(context: Context, projection: PomodoroNotificationProjection) {
    val deadline = projection.phases.first().startsAtEpochMs
    if (deadline <= System.currentTimeMillis()) return
    val manager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
    val intent = pendingIntent(context, projection.runId)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && !manager.canScheduleExactAlarms()) {
      manager.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, deadline, intent)
    } else {
      manager.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, deadline, intent)
    }
  }

  private fun cancelAlarm(context: Context, activationId: String) {
    val manager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
    manager.cancel(pendingIntent(context, activationId))
  }

  private fun pendingIntent(context: Context, activationId: String): PendingIntent =
    PendingIntent.getBroadcast(
      context,
      requestCode(activationId),
      Intent(context, PomodoroActivationReceiver::class.java).apply {
        putExtra(EXTRA_ACTIVATION_ID, activationId)
      },
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )

  private fun requestCode(activationId: String): Int =
    ACTIVATION_REQUEST_CODE_START + (activationId.hashCode() and Int.MAX_VALUE) % ACTIVATION_REQUEST_CODE_SPAN

  private fun remove(context: Context, activationId: String) {
    cancelAlarm(context, activationId)
    check(store(context).edit().remove(activationId).commit()) {
      "Pomodoro activation could not be removed"
    }
  }

  private fun dismissed(context: Context): Set<String> =
    store(context).getStringSet(DISMISSED_ACTIVATIONS_KEY, emptySet())?.toSet() ?: emptySet()

  private fun saveDismissed(context: Context, values: Set<String>) {
    check(store(context).edit().putStringSet(DISMISSED_ACTIVATIONS_KEY, values).commit()) {
      "Pomodoro activation dismissal could not be persisted"
    }
  }

  private fun store(context: Context) = context.getSharedPreferences(
    ACTIVATION_STORE,
    Context.MODE_PRIVATE,
  )
}

class PomodoroActivationReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    synchronized(POMODORO_GUARDIAN_LOCK) {
      PomodoroActivationScheduler.activate(context, intent.getStringExtra(EXTRA_ACTIVATION_ID))
    }
  }
}
