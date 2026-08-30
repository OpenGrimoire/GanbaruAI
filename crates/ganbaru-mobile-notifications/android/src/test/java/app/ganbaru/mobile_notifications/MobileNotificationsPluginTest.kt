package app.ganbaru.mobile_notifications

import android.content.Intent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class MobileNotificationsPluginTest {
  @Test
  fun exactAlarmAccessIsRequiredFromAndroidTwelve() {
    assertFalse(ExactAlarmCapability.isRequired(30))
    assertTrue(ExactAlarmCapability.isRequired(31))
    assertTrue(ExactAlarmCapability.isRequired(36))
  }

  @Test
  fun backgroundAutostartRegistryCoversMainstreamManufacturerFamilies() {
    val identifiers = listOf(
      "Xiaomi" to "Redmi",
      "Huawei" to "Huawei",
      "Honor" to "HONOR",
      "OPPO" to "OPPO",
      "realme" to "realme",
      "OnePlus" to "OnePlus",
      "vivo" to "iQOO",
      "asus" to "ROG",
      "TECNO" to "TECNO",
      "ZTE" to "nubia",
    )

    identifiers.forEach { (manufacturer, brand) ->
      assertTrue(
        "$manufacturer should have an autostart destination",
        BackgroundExecutionSettingsRegistry.candidates(
          BackgroundSettingsDestination.AUTOSTART,
          manufacturer,
          brand,
        ).isNotEmpty(),
      )
    }
  }

  @Test
  fun samsungUsesItsDocumentedNeverSleepingRouteOnlyForBatteryReview() {
    assertTrue(
      BackgroundExecutionSettingsRegistry.candidates(
        BackgroundSettingsDestination.AUTOSTART,
        " samsung ",
        "SAMSUNG",
      ).isEmpty(),
    )

    val batteryCandidates = BackgroundExecutionSettingsRegistry.candidates(
      BackgroundSettingsDestination.BATTERY,
      "Samsung",
      "Samsung",
    )
    val documentedRoute = batteryCandidates.first() as BackgroundSettingsIntentSpec.Action
    assertEquals(
      "com.samsung.android.sm.ACTION_OPEN_CHECKABLE_LISTACTIVITY",
      documentedRoute.action,
    )
    assertEquals("com.samsung.android.lool", documentedRoute.packageName)
    assertEquals(2, documentedRoute.integerExtras["activity_type"])
  }

  @Test
  fun stockAndroidDoesNotAdvertiseAFalseAutostartControl() {
    assertTrue(
      BackgroundExecutionSettingsRegistry.candidates(
        BackgroundSettingsDestination.AUTOSTART,
        "Google",
        "google",
      ).isEmpty(),
    )
    assertTrue(
      BackgroundExecutionSettingsRegistry.candidates(
        BackgroundSettingsDestination.AUTOSTART,
        "motorola",
        "motorola",
      ).isEmpty(),
    )
  }

  @Test
  fun restoreReceiversAcceptOnlyTheirDeclaredSystemActions() {
    assertTrue(isNotificationRestoreAction(Intent.ACTION_BOOT_COMPLETED))
    assertTrue(isNotificationRestoreAction(Intent.ACTION_MY_PACKAGE_REPLACED))
    assertTrue(isNotificationRestoreAction(Intent.ACTION_TIME_CHANGED))
    assertTrue(isNotificationRestoreAction(Intent.ACTION_TIMEZONE_CHANGED))
    assertFalse(isNotificationRestoreAction(null))
    assertFalse(isNotificationRestoreAction("app.ganbaru.UNTRUSTED_RESTORE"))
  }

  @Test
  fun activationSelectionKeepsCurrentThenChoosesShortestEligibleEvent() {
    val later = projection("later", startsAt = 1_000, endsAt = 9_000)
    val shorter = projection("shorter", startsAt = 2_000, endsAt = 7_000)
    val future = projection("future", startsAt = 6_000, endsAt = 8_000)

    assertTrue(
      PomodoroActivationSelection.select(
        listOf(later, shorter, future),
        emptySet(),
        5_000,
        later,
      )?.runId == "later",
    )
    assertTrue(
      PomodoroActivationSelection.select(
        listOf(later, shorter, future),
        emptySet(),
        5_000,
        null,
      )?.runId == "shorter",
    )
    assertTrue(
      PomodoroActivationSelection.select(
        listOf(later, shorter),
        setOf("shorter"),
        5_000,
        null,
      )?.runId == "later",
    )
  }

  private fun projection(
    runId: String,
    startsAt: Long,
    endsAt: Long,
  ): PomodoroNotificationProjection = PomodoroNotificationProjection(
    runId = runId,
    eventId = runId,
    eventTitle = runId,
    eventDate = "2026-08-28",
    eventEndsAtEpochMs = endsAt,
    generatedAtEpochMs = 0,
    isRunning = true,
    remainingSeconds = 1,
    totalSeconds = 1,
    configJson = """{"rhythm":{"kind":"count","focusDurationMinutes":25,"shortBreakMinutes":5,"longBreakMinutes":15,"longBreakAfterFocusCount":4},"rhythmSource":"preset","presetKey":"creative","idleTimeoutMinutes":null}""",
    phases = listOf(PomodoroNotificationPhase(
      id = "$runId-phase",
      phase = "focus",
      rhythmPosition = 1,
      startsAtEpochMs = startsAt,
      endsAtEpochMs = endsAt,
    )),
    copy = PomodoroNotificationCopy(
      channelName = "Focus",
      channelDescription = "Focus",
      alertsChannelName = "Alerts",
      alertsChannelDescription = "Alerts",
      focusTitle = "Focus",
      shortBreakTitle = "Short break",
      longBreakTitle = "Long break",
      pausedText = "Paused",
      focusCompleteTitle = "Focus complete",
      breakCompleteTitle = "Break complete",
      sessionCompleteText = "Session complete",
    ),
  )
}
