package app.ganbaru.mobile_notifications

import android.annotation.SuppressLint
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import org.json.JSONArray
import org.json.JSONObject

internal const val POMODORO_CHANNEL_ID = "pomodoro-active-v1"
internal const val POMODORO_ALERTS_CHANNEL_ID = "pomodoro-alerts-v1"
internal const val POMODORO_NOTIFICATION_ID = 1_500_000_001
internal const val POMODORO_ALERT_NOTIFICATION_ID = 1_500_000_002
private const val POMODORO_NOTIFICATION_STORE = "GANBARU_POMODORO_NOTIFICATION_STORE"
private const val POMODORO_NOTIFICATION_KEY = "activeProjection"
private const val POMODORO_ALARM_REQUEST_CODE = 1_500_000_003
private const val EXTRA_RUN_ID = "ganbaruPomodoroRunId"
private const val EXTRA_PHASE_ID = "ganbaruPomodoroPhaseId"
private const val EXTRA_BOUNDARY_EPOCH_MS = "ganbaruPomodoroBoundaryEpochMs"
private const val MAX_PHASES = 128
private const val ACTION_SYNCHRONIZE =
  "app.ganbaru.mobile_notifications.action.SYNCHRONIZE_POMODORO"
private const val ACTION_DELIVER_BOUNDARY =
  "app.ganbaru.mobile_notifications.action.DELIVER_POMODORO_BOUNDARY"

internal data class PomodoroNotificationPhase(
  val id: String,
  val phase: String,
  val rhythmPosition: Int,
  val startsAtEpochMs: Long,
  val endsAtEpochMs: Long,
)

internal data class PomodoroNotificationCopy(
  val channelName: String,
  val channelDescription: String,
  val alertsChannelName: String,
  val alertsChannelDescription: String,
  val focusTitle: String,
  val shortBreakTitle: String,
  val longBreakTitle: String,
  val pausedText: String,
  val focusCompleteTitle: String,
  val breakCompleteTitle: String,
  val sessionCompleteText: String,
)

internal data class PomodoroNotificationProjection(
  val runId: String,
  val eventId: String,
  val eventTitle: String?,
  val eventDate: String,
  val eventEndsAtEpochMs: Long,
  val generatedAtEpochMs: Long,
  val isRunning: Boolean,
  val remainingSeconds: Int,
  val totalSeconds: Int,
  val configJson: String?,
  val phases: List<PomodoroNotificationPhase>,
  val copy: PomodoroNotificationCopy,
)

internal object PomodoroNotificationScheduler {
  fun update(context: Context, projection: PomodoroNotificationProjection) {
    validate(projection)
    save(context, projection)
    DoomscrollingPhaseBridge.publish(context, projection)
    PomodoroNotificationService.synchronize(context)
  }

  fun current(context: Context): PomodoroNotificationProjection? {
    val encoded = store(context).getString(POMODORO_NOTIFICATION_KEY, null) ?: return null
    return decode(encoded)
  }

  fun cancel(context: Context) {
    PomodoroActivationScheduler.dismissCurrent(context)
    alarmManager(context).cancel(boundaryIntent(context, null, null))
    check(store(context).edit().remove(POMODORO_NOTIFICATION_KEY).commit()) {
      "Pomodoro notification state could not be cleared"
    }
    context.stopService(Intent(context, PomodoroNotificationService::class.java))
    notificationManager(context).cancel(POMODORO_NOTIFICATION_ID)
    notificationManager(context).cancel(POMODORO_ALERT_NOTIFICATION_ID)
    DoomscrollingPhaseBridge.clear(context)
  }

  fun restore(context: Context) {
    val projection = current(context) ?: return
    if (projection.eventEndsAtEpochMs <= System.currentTimeMillis()) return
    PomodoroNotificationService.synchronize(context)
  }

  internal fun clearCurrent(context: Context) {
    check(store(context).edit().remove(POMODORO_NOTIFICATION_KEY).commit()) {
      "Pomodoro notification state could not be cleared"
    }
  }

  fun deliverBoundary(
    service: PomodoroNotificationService,
    intent: Intent,
  ): Boolean {
    val context: Context = service
    val projection = current(context) ?: return false
    val runId = intent.getStringExtra(EXTRA_RUN_ID) ?: return false
    val phaseId = intent.getStringExtra(EXTRA_PHASE_ID) ?: return false
    val boundary = intent.getLongExtra(EXTRA_BOUNDARY_EPOCH_MS, 0L)
    val matchingPhase = projection.phases.firstOrNull {
      it.id == phaseId && it.endsAtEpochMs == boundary
    } ?: return false
    if (projection.runId != runId || boundary <= 0L) return false
    if (System.currentTimeMillis() < matchingPhase.endsAtEpochMs) {
      synchronize(service, projection, alertBoundary = false)
      return true
    }
    synchronize(service, projection, alertBoundary = true, completedPhase = matchingPhase)
    return true
  }

  fun synchronize(
    service: PomodoroNotificationService,
    projection: PomodoroNotificationProjection,
    alertBoundary: Boolean,
    completedPhase: PomodoroNotificationPhase? = null,
  ) {
    val context: Context = service
    val now = System.currentTimeMillis()
    if (projection.eventEndsAtEpochMs <= now) {
      alarmManager(context).cancel(boundaryIntent(context, null, null))
      if (alertBoundary) postBoundaryAlert(context, projection, completedPhase, null)
      clearCurrent(context)
      DoomscrollingPhaseBridge.clear(context)
      if (PomodoroActivationScheduler.activateEligible(context) == null) service.finishSession()
      return
    }

    if (!projection.isRunning) {
      DoomscrollingPhaseBridge.publish(context, projection, projection.phases.first())
      postOngoing(service, projection, projection.phases.first(), now)
      scheduleBoundary(context, projection, projection.phases.first())
      return
    }

    val activePhase = projection.phases.firstOrNull { now < it.endsAtEpochMs }
    if (alertBoundary) postBoundaryAlert(context, projection, completedPhase, activePhase)
    if (activePhase == null) {
      alarmManager(context).cancel(boundaryIntent(context, null, null))
      DoomscrollingPhaseBridge.clear(context)
      service.finishSession()
      return
    }
    DoomscrollingPhaseBridge.publish(context, projection, activePhase)
    postOngoing(service, projection, activePhase, now)
    scheduleBoundary(context, projection, activePhase)
  }

  @SuppressLint("InlinedApi")
  private fun postOngoing(
    service: PomodoroNotificationService,
    projection: PomodoroNotificationProjection,
    phase: PomodoroNotificationPhase,
    now: Long,
  ) {
    val context: Context = service
    val isPublishedPhase = phase.id == projection.phases.first().id
    val phaseDurationSeconds = if (isPublishedPhase) {
      projection.totalSeconds
    } else {
      ((phase.endsAtEpochMs - phase.startsAtEpochMs) / 1_000L)
        .coerceIn(1L, Int.MAX_VALUE.toLong()).toInt()
    }
    val elapsedSeconds = if (isPublishedPhase) {
      projection.totalSeconds - projection.remainingSeconds
    } else {
      ((now - phase.startsAtEpochMs) / 1_000L)
        .coerceIn(0L, phaseDurationSeconds.toLong()).toInt()
    }
    val title = projection.eventTitle ?: phaseTitle(projection.copy, phase.phase)
    val builder = Notification.Builder(context, POMODORO_CHANNEL_ID)
      .setSmallIcon(notificationIcon(context))
      .setContentTitle(title)
      .setCategory(Notification.CATEGORY_STOPWATCH)
      .setVisibility(Notification.VISIBILITY_PRIVATE)
      .setContentIntent(launchIntent(context))
      .setOnlyAlertOnce(true)
      .setOngoing(true)
      .setAutoCancel(false)
      .setProgress(phaseDurationSeconds, elapsedSeconds, false)
      .setShowWhen(projection.isRunning)
    if (!projection.isRunning) {
      builder.setContentText(projection.copy.pausedText)
    }
    if (projection.isRunning) {
      builder
        .setWhen(phase.endsAtEpochMs)
        .setUsesChronometer(true)
        .setChronometerCountDown(true)
    }
    val notification = builder.build().apply {
      flags = flags or Notification.FLAG_NO_CLEAR or Notification.FLAG_ONGOING_EVENT
    }
    service.publish(notification)
  }

  private fun postBoundaryAlert(
    context: Context,
    projection: PomodoroNotificationProjection,
    completedPhase: PomodoroNotificationPhase?,
    nextPhase: PomodoroNotificationPhase?,
  ) {
    val title = if (completedPhase?.phase == "focus") {
      projection.copy.focusCompleteTitle
    } else {
      projection.copy.breakCompleteTitle
    }
    val body = nextPhase?.let { phaseTitle(projection.copy, it.phase) }
      ?: projection.copy.sessionCompleteText
    val notification = Notification.Builder(context, POMODORO_ALERTS_CHANNEL_ID)
      .setSmallIcon(notificationIcon(context))
      .setContentTitle(title)
      .setContentText(body)
      .setCategory(Notification.CATEGORY_ALARM)
      .setVisibility(Notification.VISIBILITY_PRIVATE)
      .setContentIntent(launchIntent(context))
      .setAutoCancel(true)
      .build()
    notificationManager(context).notify(POMODORO_ALERT_NOTIFICATION_ID, notification)
  }

  private fun scheduleBoundary(
    context: Context,
    projection: PomodoroNotificationProjection,
    phase: PomodoroNotificationPhase,
  ) {
    val deadline = if (projection.isRunning) phase.endsAtEpochMs else projection.eventEndsAtEpochMs
    val intent = boundaryIntent(context, projection, phase, deadline)
    val manager = alarmManager(context)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && !manager.canScheduleExactAlarms()) {
      manager.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, deadline, intent)
    } else {
      manager.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, deadline, intent)
    }
  }

  private fun boundaryIntent(
    context: Context,
    projection: PomodoroNotificationProjection?,
    phase: PomodoroNotificationPhase?,
    deadline: Long = 0L,
  ): PendingIntent {
    val intent = Intent(context, PomodoroNotificationReceiver::class.java).apply {
      if (projection != null && phase != null) {
        putExtra(EXTRA_RUN_ID, projection.runId)
        putExtra(EXTRA_PHASE_ID, phase.id)
        putExtra(EXTRA_BOUNDARY_EPOCH_MS, deadline)
      }
    }
    return PendingIntent.getBroadcast(
      context,
      POMODORO_ALARM_REQUEST_CODE,
      intent,
      PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )
  }

  internal fun validate(projection: PomodoroNotificationProjection) {
    require(projection.runId.isNotBlank() && projection.runId.length <= 128) {
      "Pomodoro run ID must contain 1 to 128 characters"
    }
    require(projection.eventId.isNotBlank() && projection.eventId.length <= 256) {
      "Pomodoro event ID must contain 1 to 256 characters"
    }
    require(
      projection.eventTitle == null
        || (projection.eventTitle.isNotBlank() && projection.eventTitle.length <= 160),
    ) {
      "Pomodoro event title must contain 1 to 160 characters when present"
    }
    require(projection.eventDate.length == 10) { "Pomodoro event date is invalid" }
    require(projection.eventEndsAtEpochMs > projection.generatedAtEpochMs) {
      "Pomodoro event deadline must be in the future"
    }
    require(projection.remainingSeconds in 0..projection.totalSeconds && projection.totalSeconds > 0) {
      "Pomodoro remaining and total seconds are invalid"
    }
    projection.configJson?.let(::validateConfig)
    require(projection.phases.isNotEmpty() && projection.phases.size <= MAX_PHASES) {
      "Pomodoro phase projection must contain 1 to $MAX_PHASES phases"
    }
    val ids = mutableSetOf<String>()
    var previousEnd: Long? = null
    for (phase in projection.phases) {
      require(phase.id.isNotBlank() && phase.id.length <= 128 && ids.add(phase.id)) {
        "Pomodoro phase IDs must be bounded and unique"
      }
      require(phase.phase in setOf("focus", "short_break", "long_break")) {
        "Pomodoro phase kind is invalid"
      }
      require(phase.rhythmPosition > 0) { "Pomodoro rhythm position must be positive" }
      require(phase.startsAtEpochMs < phase.endsAtEpochMs) {
        "Pomodoro phase timestamps are invalid"
      }
      require(phase.endsAtEpochMs <= projection.eventEndsAtEpochMs) {
        "Pomodoro phase exceeds the event deadline"
      }
      if (previousEnd != null) {
        require(phase.startsAtEpochMs == previousEnd) {
          "Pomodoro phases must be contiguous"
        }
      }
      previousEnd = phase.endsAtEpochMs
    }
    if (projection.isRunning) {
      require(previousEnd == projection.eventEndsAtEpochMs) {
        "Running Pomodoro projection must cover the event window"
      }
    } else {
      require(projection.phases.size == 1) {
        "Paused Pomodoro projection must contain only its current phase"
      }
    }
    validateCopy(projection.copy)
  }

  private fun validateCopy(copy: PomodoroNotificationCopy) {
    val values = listOf(
      copy.channelName,
      copy.channelDescription,
      copy.alertsChannelName,
      copy.alertsChannelDescription,
      copy.focusTitle,
      copy.shortBreakTitle,
      copy.longBreakTitle,
      copy.pausedText,
      copy.focusCompleteTitle,
      copy.breakCompleteTitle,
      copy.sessionCompleteText,
    )
    require(values.all { it.isNotBlank() && it.length <= 160 }) {
      "Pomodoro notification text must contain 1 to 160 characters"
    }
  }

  private fun validateConfig(encoded: String) {
    require(encoded.length in 2..4_096) { "Pomodoro config must be bounded" }
    val config = JSONObject(encoded)
    val source = config.getString("rhythmSource")
    require(source == "preset" || source == "custom") { "Pomodoro rhythm source is invalid" }
    val idle = config.opt("idleTimeoutMinutes")
    require(idle == null || idle == JSONObject.NULL || (idle is Number && idle.toInt() > 0)) {
      "Pomodoro idle timeout is invalid"
    }
    val rhythm = config.getJSONObject("rhythm")
    when (rhythm.getString("kind")) {
      "count" -> {
        require(rhythm.getInt("focusDurationMinutes") in 1..120)
        require(rhythm.getInt("shortBreakMinutes") in 1..30)
        require(rhythm.getInt("longBreakMinutes") in 1..60)
        require(rhythm.getInt("longBreakAfterFocusCount") in 1..12)
      }
      "sequence" -> {
        val steps = rhythm.getJSONArray("steps")
        require(steps.length() in 1..12)
        for (index in 0 until steps.length()) {
          val step = steps.getJSONObject(index)
          require(step.getInt("focusDurationMinutes") in 1..120)
          require(step.getString("breakPhase") in setOf("short_break", "long_break"))
          require(step.getInt("breakDurationMinutes") in 1..60)
        }
      }
      else -> error("Pomodoro rhythm kind is invalid")
    }
  }

  private fun phaseTitle(copy: PomodoroNotificationCopy, phase: String): String = when (phase) {
    "focus" -> copy.focusTitle
    "short_break" -> copy.shortBreakTitle
    else -> copy.longBreakTitle
  }

  internal fun save(context: Context, projection: PomodoroNotificationProjection) {
    check(store(context).edit()
      .putString(POMODORO_NOTIFICATION_KEY, encode(projection))
      .commit()) { "Pomodoro notification state could not be persisted" }
  }

  internal fun encode(projection: PomodoroNotificationProjection): String = JSONObject()
    .put("runId", projection.runId)
    .put("eventId", projection.eventId)
    .put("eventTitle", projection.eventTitle ?: JSONObject.NULL)
    .put("eventDate", projection.eventDate)
    .put("eventEndsAtEpochMs", projection.eventEndsAtEpochMs)
    .put("generatedAtEpochMs", projection.generatedAtEpochMs)
    .put("isRunning", projection.isRunning)
    .put("remainingSeconds", projection.remainingSeconds)
    .put("totalSeconds", projection.totalSeconds)
    .put("configJson", projection.configJson ?: JSONObject.NULL)
    .put("phases", JSONArray().apply {
      projection.phases.forEach { phase ->
        put(JSONObject()
          .put("id", phase.id)
          .put("phase", phase.phase)
          .put("rhythmPosition", phase.rhythmPosition)
          .put("startsAtEpochMs", phase.startsAtEpochMs)
          .put("endsAtEpochMs", phase.endsAtEpochMs))
      }
    })
    .put("copy", JSONObject()
      .put("channelName", projection.copy.channelName)
      .put("channelDescription", projection.copy.channelDescription)
      .put("alertsChannelName", projection.copy.alertsChannelName)
      .put("alertsChannelDescription", projection.copy.alertsChannelDescription)
      .put("focusTitle", projection.copy.focusTitle)
      .put("shortBreakTitle", projection.copy.shortBreakTitle)
      .put("longBreakTitle", projection.copy.longBreakTitle)
      .put("pausedText", projection.copy.pausedText)
      .put("focusCompleteTitle", projection.copy.focusCompleteTitle)
      .put("breakCompleteTitle", projection.copy.breakCompleteTitle)
      .put("sessionCompleteText", projection.copy.sessionCompleteText))
    .toString()

  internal fun decode(encoded: String): PomodoroNotificationProjection? = try {
    val value = JSONObject(encoded)
    val phasesJson = value.getJSONArray("phases")
    val phases = (0 until phasesJson.length()).map { index ->
      val phase = phasesJson.getJSONObject(index)
      PomodoroNotificationPhase(
        id = phase.getString("id"),
        phase = phase.getString("phase"),
        rhythmPosition = phase.getInt("rhythmPosition"),
        startsAtEpochMs = phase.getLong("startsAtEpochMs"),
        endsAtEpochMs = phase.getLong("endsAtEpochMs"),
      )
    }
    val copy = value.getJSONObject("copy")
    PomodoroNotificationProjection(
      runId = value.getString("runId"),
      eventId = value.getString("eventId"),
      eventTitle = if (value.isNull("eventTitle")) null else value.optString("eventTitle")
        .trim().takeIf(String::isNotEmpty),
      eventDate = value.getString("eventDate"),
      eventEndsAtEpochMs = value.getLong("eventEndsAtEpochMs"),
      generatedAtEpochMs = value.getLong("generatedAtEpochMs"),
      isRunning = value.getBoolean("isRunning"),
      remainingSeconds = value.getInt("remainingSeconds"),
      totalSeconds = value.getInt("totalSeconds"),
      configJson = if (value.isNull("configJson")) null else value.optString("configJson")
        .takeIf(String::isNotBlank),
      phases = phases,
      copy = PomodoroNotificationCopy(
        channelName = copy.getString("channelName"),
        channelDescription = copy.getString("channelDescription"),
        alertsChannelName = copy.getString("alertsChannelName"),
        alertsChannelDescription = copy.getString("alertsChannelDescription"),
        focusTitle = copy.getString("focusTitle"),
        shortBreakTitle = copy.getString("shortBreakTitle"),
        longBreakTitle = copy.getString("longBreakTitle"),
        pausedText = copy.getString("pausedText"),
        focusCompleteTitle = copy.getString("focusCompleteTitle"),
        breakCompleteTitle = copy.getString("breakCompleteTitle"),
        sessionCompleteText = copy.getString("sessionCompleteText"),
      ),
    ).also(::validate)
  } catch (_: Exception) {
    null
  }

  private fun launchIntent(context: Context): PendingIntent? =
    context.packageManager.getLaunchIntentForPackage(context.packageName)?.let { intent ->
      intent.flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
      PendingIntent.getActivity(
        context,
        POMODORO_NOTIFICATION_ID,
        intent,
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
      )
    }

  private fun notificationIcon(context: Context): Int = context.resources.getIdentifier(
    "ic_notification_focus",
    "drawable",
    context.packageName,
  ).takeIf { it != 0 } ?: context.applicationInfo.icon

  private fun store(context: Context) = context.getSharedPreferences(
    POMODORO_NOTIFICATION_STORE,
    Context.MODE_PRIVATE,
  )

  private fun notificationManager(context: Context) =
    context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

  private fun alarmManager(context: Context) =
    context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
}

class PomodoroNotificationReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    PomodoroNotificationService.deliverBoundary(context, intent)
  }
}

class PomodoroNotificationRestoreReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    if (!isNotificationRestoreAction(intent.action)) return
    synchronized(POMODORO_GUARDIAN_LOCK) {
      PomodoroActivationScheduler.restore(context)
      PomodoroNotificationScheduler.restore(context)
    }
  }
}

class PomodoroNotificationService : Service() {
  override fun onBind(intent: Intent?): IBinder? = null

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int =
    synchronized(POMODORO_GUARDIAN_LOCK) {
      val projection = PomodoroNotificationScheduler.current(this)
      if (projection == null || projection.eventEndsAtEpochMs <= System.currentTimeMillis()) {
        finishSession()
        return@synchronized START_NOT_STICKY
      }
      if (intent?.action == ACTION_DELIVER_BOUNDARY) {
        if (!PomodoroNotificationScheduler.deliverBoundary(this, intent)) {
          PomodoroNotificationScheduler.synchronize(this, projection, alertBoundary = false)
        }
      } else {
        PomodoroNotificationScheduler.synchronize(this, projection, alertBoundary = false)
      }
      START_STICKY
    }

  internal fun publish(notification: Notification) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
      startForeground(
        POMODORO_NOTIFICATION_ID,
        notification,
        ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE,
      )
    } else {
      startForeground(POMODORO_NOTIFICATION_ID, notification)
    }
  }

  internal fun finishSession() {
    stopForeground(STOP_FOREGROUND_REMOVE)
    stopSelf()
  }

  companion object {
    fun synchronize(context: Context) {
      start(context, Intent(context, PomodoroNotificationService::class.java).apply {
        action = ACTION_SYNCHRONIZE
      })
    }

    fun deliverBoundary(context: Context, boundaryIntent: Intent) {
      start(context, Intent(context, PomodoroNotificationService::class.java).apply {
        action = ACTION_DELIVER_BOUNDARY
        putExtra(EXTRA_RUN_ID, boundaryIntent.getStringExtra(EXTRA_RUN_ID))
        putExtra(EXTRA_PHASE_ID, boundaryIntent.getStringExtra(EXTRA_PHASE_ID))
        putExtra(
          EXTRA_BOUNDARY_EPOCH_MS,
          boundaryIntent.getLongExtra(EXTRA_BOUNDARY_EPOCH_MS, 0L),
        )
      })
    }

    private fun start(context: Context, intent: Intent) {
      context.startForegroundService(intent)
    }
  }
}
