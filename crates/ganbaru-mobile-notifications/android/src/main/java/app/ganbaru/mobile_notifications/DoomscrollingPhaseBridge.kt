package app.ganbaru.mobile_notifications

import android.content.Context
import android.content.Intent

private const val ACTION_DOOMSCROLLING_PHASE = "app.ganbaru.intent.action.DOOMSCROLLING_PHASE"
private const val ACTION_DOOMSCROLLING_PHASE_CLEAR =
  "app.ganbaru.intent.action.DOOMSCROLLING_PHASE_CLEAR"

internal object DoomscrollingPhaseBridge {
  fun publish(
    context: Context,
    projection: PomodoroNotificationProjection,
    activePhase: PomodoroNotificationPhase? = activePhase(projection),
  ) {
    val phase = activePhase ?: run {
      clear(context)
      return
    }
    context.sendBroadcast(Intent(ACTION_DOOMSCROLLING_PHASE).apply {
      setPackage(context.packageName)
      putExtra("runId", projection.runId)
      putExtra("phase", phase.phase)
      putExtra("running", projection.isRunning)
      putExtra(
        "validUntilEpochMs",
        if (projection.isRunning) phase.endsAtEpochMs else projection.eventEndsAtEpochMs,
      )
    })
  }

  fun clear(context: Context) {
    context.sendBroadcast(Intent(ACTION_DOOMSCROLLING_PHASE_CLEAR).apply {
      setPackage(context.packageName)
    })
  }

  private fun activePhase(
    projection: PomodoroNotificationProjection,
  ): PomodoroNotificationPhase? {
    if (!projection.isRunning) return projection.phases.firstOrNull()
    val now = System.currentTimeMillis()
    return projection.phases.firstOrNull { phase ->
      now >= phase.startsAtEpochMs && now < phase.endsAtEpochMs
    } ?: projection.phases.firstOrNull { phase -> now < phase.endsAtEpochMs }
  }
}
