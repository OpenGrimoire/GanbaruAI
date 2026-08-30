package app.ganbaru.mobile_doomscrolling

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class DoomscrollingEvaluatorTest {
  private val schedule = MobileSchedule(
    enabled = true,
    blockDuringFocus = true,
    blockDuringShortBreaks = false,
    blockDuringLongBreaks = true,
    pauseDuringFocusPause = true,
    blockedApps = listOf(
      MobileAppRule("YouTube", "com.google.android.youtube", enabled = true),
    ),
  )

  @Test
  fun blocksSelectedAppDuringEnabledActivePhase() {
    assertTrue(DoomscrollingEvaluator.evaluateSchedule(
      schedule,
      phase(phase = "focus"),
      "com.google.android.youtube",
      NOW,
    ))
  }

  @Test
  fun followsPerPhaseSchedule() {
    assertFalse(DoomscrollingEvaluator.evaluateSchedule(
      schedule,
      phase(phase = "short_break"),
      "com.google.android.youtube",
      NOW,
    ))
    assertTrue(DoomscrollingEvaluator.evaluateSchedule(
      schedule,
      phase(phase = "long_break"),
      "com.google.android.youtube",
      NOW,
    ))
  }

  @Test
  fun failsOpenForPausedOrExpiredProjection() {
    assertFalse(DoomscrollingEvaluator.evaluateSchedule(
      schedule,
      phase(phase = "focus", running = false),
      "com.google.android.youtube",
      NOW,
    ))
    assertFalse(DoomscrollingEvaluator.evaluateSchedule(
      schedule,
      phase(phase = "focus", validUntilEpochMs = NOW),
      "com.google.android.youtube",
      NOW,
    ))
  }

  @Test
  fun ignoresAppsWithoutAnEnabledExactPackageRule() {
    assertFalse(DoomscrollingEvaluator.evaluateSchedule(
      schedule,
      phase(phase = "focus"),
      "com.example.video",
      NOW,
    ))
    assertFalse(DoomscrollingEvaluator.evaluateSchedule(
      schedule.copy(blockedApps = schedule.blockedApps.map { it.copy(enabled = false) }),
      phase(phase = "focus"),
      "com.google.android.youtube",
      NOW,
    ))
  }

  private fun phase(
    phase: String,
    running: Boolean = true,
    validUntilEpochMs: Long = NOW + 60_000L,
  ) = PomodoroPhaseState(
    active = true,
    runId = "run-1",
    phase = phase,
    running = running,
    validUntilEpochMs = validUntilEpochMs,
  )

  private companion object {
    const val NOW = 1_788_041_200_000L
  }
}
