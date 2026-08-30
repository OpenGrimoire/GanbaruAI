package app.ganbaru.mobile_doomscrolling

import android.app.AppOpsManager
import android.app.KeyguardManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Process
import android.provider.Settings
import android.telecom.TelecomManager
import android.view.accessibility.AccessibilityManager
import java.time.Instant
import java.time.ZoneId

internal const val ACTION_DOOMSCROLLING_PHASE = "app.ganbaru.intent.action.DOOMSCROLLING_PHASE"
internal const val ACTION_DOOMSCROLLING_PHASE_CLEAR =
  "app.ganbaru.intent.action.DOOMSCROLLING_PHASE_CLEAR"
internal const val ACTION_DOOMSCROLLING_RULES_CHANGED =
  "app.ganbaru.intent.action.DOOMSCROLLING_RULES_CHANGED"
private const val RUNTIME_STORE = "GANBARU_DOOMSCROLLING_RUNTIME"
private const val RULES_KEY = "rules"
private const val RULES_VAULT_ID_KEY = "rulesVaultId"
private const val PHASE_ACTIVE_KEY = "phaseActive"
private const val PHASE_RUN_ID_KEY = "phaseRunId"
private const val PHASE_KIND_KEY = "phaseKind"
private const val PHASE_RUNNING_KEY = "phaseRunning"
private const val PHASE_VALID_UNTIL_KEY = "phaseValidUntil"
private const val ACTION_TARGET_KEY = "notificationAction"
private const val NOTIFICATION_TIMESTAMPS_KEY = "notificationTimestamps"
private const val BLOCK_CHANNEL_ID = "doomscrolling-blocks-v1"
private const val BLOCK_NOTIFICATION_BASE_ID = 1_500_100_000

internal object DoomscrollingRuntimeStore {
  fun saveRules(context: Context, encoded: String) {
    val next = DoomscrollingRuleCodec.decode(encoded)
    val prefs = preferences(context)
    val previousVaultId = prefs.getString(RULES_VAULT_ID_KEY, null) ?: rules(context)?.vaultId
    if (previousVaultId != null && previousVaultId != next.vaultId) {
      DoomscrollingJournal(context).clearTotalsAndCheckpoints()
    }
    check(prefs.edit()
      .putString(RULES_KEY, encoded)
      .putString(RULES_VAULT_ID_KEY, next.vaultId)
      .commit()) { "Doomscrolling rule snapshot could not be persisted" }
  }

  fun rules(context: Context): DoomscrollingRulesSnapshot? {
    val encoded = preferences(context).getString(RULES_KEY, null) ?: return null
    return try {
      DoomscrollingRuleCodec.decode(encoded)
    } catch (_: Exception) {
      null
    }
  }

  fun savePhase(
    context: Context,
    runId: String,
    phase: String,
    running: Boolean,
    validUntilEpochMs: Long,
  ): Boolean {
    if (runId.isBlank() || runId.length > 128
      || phase !in setOf("focus", "short_break", "long_break")
      || validUntilEpochMs <= 0L
    ) {
      clearPhase(context)
      return false
    }
    preferences(context).edit()
      .putBoolean(PHASE_ACTIVE_KEY, true)
      .putString(PHASE_RUN_ID_KEY, runId)
      .putString(PHASE_KIND_KEY, phase)
      .putBoolean(PHASE_RUNNING_KEY, running)
      .putLong(PHASE_VALID_UNTIL_KEY, validUntilEpochMs)
      .apply()
    return true
  }

  fun clearPhase(context: Context) {
    preferences(context).edit()
      .remove(PHASE_ACTIVE_KEY)
      .remove(PHASE_RUN_ID_KEY)
      .remove(PHASE_KIND_KEY)
      .remove(PHASE_RUNNING_KEY)
      .remove(PHASE_VALID_UNTIL_KEY)
      .apply()
  }

  fun updatePhaseFromIntent(context: Context, intent: Intent): Boolean {
    if (intent.action == ACTION_DOOMSCROLLING_PHASE_CLEAR) {
      clearPhase(context)
      return true
    }
    if (intent.action != ACTION_DOOMSCROLLING_PHASE) return false
    return savePhase(
      context = context,
      runId = intent.getStringExtra("runId").orEmpty(),
      phase = intent.getStringExtra("phase").orEmpty(),
      running = intent.getBooleanExtra("running", false),
      validUntilEpochMs = intent.getLongExtra("validUntilEpochMs", 0L),
    )
  }

  fun phase(context: Context): PomodoroPhaseState? {
    val prefs = preferences(context)
    if (!prefs.getBoolean(PHASE_ACTIVE_KEY, false)) return null
    return PomodoroPhaseState(
      active = true,
      runId = prefs.getString(PHASE_RUN_ID_KEY, null),
      phase = prefs.getString(PHASE_KIND_KEY, null),
      running = prefs.getBoolean(PHASE_RUNNING_KEY, false),
      validUntilEpochMs = prefs.getLong(PHASE_VALID_UNTIL_KEY, 0L),
    )
  }

  fun captureNotificationAction(context: Context, intent: Intent?) {
    val target = intent?.getStringExtra(ACTION_TARGET_KEY)
      ?.takeIf { it == "mobile" || it == "limits" }
      ?: return
    preferences(context).edit().putString(ACTION_TARGET_KEY, target).apply()
    intent.removeExtra(ACTION_TARGET_KEY)
  }

  fun takeNotificationAction(context: Context): String? {
    val prefs = preferences(context)
    val target = prefs.getString(ACTION_TARGET_KEY, null)
    prefs.edit().remove(ACTION_TARGET_KEY).apply()
    return target
  }

  fun allowNotification(context: Context, nowEpochMs: Long): Boolean {
    val prefs = preferences(context)
    val recent = prefs.getString(NOTIFICATION_TIMESTAMPS_KEY, "")
      .orEmpty()
      .split(',')
      .mapNotNull(String::toLongOrNull)
      .filter { nowEpochMs - it < 60_000L }
    if (recent.size >= 5) return false
    prefs.edit().putString(
      NOTIFICATION_TIMESTAMPS_KEY,
      (recent + nowEpochMs).joinToString(","),
    ).apply()
    return true
  }

  private fun preferences(context: Context) = context.getSharedPreferences(
    RUNTIME_STORE,
    Context.MODE_PRIVATE,
  )
}

class DoomscrollingPhaseReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    DoomscrollingRuntimeStore.updatePhaseFromIntent(context, intent)
  }
}

internal object DoomscrollingAccess {
  fun hasUsageAccess(context: Context): Boolean {
    val manager = context.getSystemService(Context.APP_OPS_SERVICE) as AppOpsManager
    return manager.checkOpNoThrow(
      AppOpsManager.OPSTR_GET_USAGE_STATS,
      Process.myUid(),
      context.packageName,
    ) == AppOpsManager.MODE_ALLOWED
  }

  fun hasAccessibilityAccess(context: Context): Boolean {
    val manager = context.getSystemService(Context.ACCESSIBILITY_SERVICE) as AccessibilityManager
    return manager.getEnabledAccessibilityServiceList(
      android.accessibilityservice.AccessibilityServiceInfo.FEEDBACK_ALL_MASK,
    ).any { info ->
      val serviceInfo = info.resolveInfo?.serviceInfo ?: return@any false
      serviceInfo.name == DoomscrollingAccessibilityService::class.java.name
        || serviceInfo.name.endsWith(".DoomscrollingAccessibilityService")
    }
  }
}

internal object ProtectedPackages {
  fun resolve(context: Context): Set<String> {
    val manager = context.packageManager
    val packages = mutableSetOf(context.packageName, "android")
    resolve(manager, Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME))?.let(packages::add)
    resolve(manager, Intent(Settings.ACTION_SETTINGS))?.let(packages::add)
    resolve(manager, Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS))?.let(packages::add)
    val telecom = context.getSystemService(Context.TELECOM_SERVICE) as? TelecomManager
    telecom?.defaultDialerPackage?.let(packages::add)
    return packages
  }

  fun isProtected(context: Context, packageName: String): Boolean {
    if (packageName in resolve(context)) return true
    val info = try {
      context.packageManager.getApplicationInfo(packageName, 0)
    } catch (_: PackageManager.NameNotFoundException) {
      return true
    }
    return info.uid < Process.FIRST_APPLICATION_UID
  }

  private fun resolve(manager: PackageManager, intent: Intent): String? =
    manager.resolveActivity(intent, PackageManager.MATCH_DEFAULT_ONLY)?.activityInfo?.packageName
}

internal class DoomscrollingEngine(private val context: Context) {
  private val journal = DoomscrollingJournal(context)
  private var activePackage: String? = null
  private var activeName: String? = null
  private var activeStartedAt: Long = 0L
  private var activeVaultId: String? = null
  private var lastBlockedPackage: String? = null
  private var lastBlockedAt: Long = 0L

  fun onForegroundPackage(packageName: String, nowEpochMs: Long): Boolean {
    checkpoint(nowEpochMs)
    activePackage = packageName
    activeName = appLabel(packageName)
    activeStartedAt = nowEpochMs
    activeVaultId = null
    journal.setMetadata("lastObservedEpochMs", nowEpochMs)

    if (!DoomscrollingAccess.hasUsageAccess(context)) return false
    if (ProtectedPackages.isProtected(context, packageName)) return false
    val rules = DoomscrollingRuntimeStore.rules(context) ?: return false
    if (usageTracked(rules, packageName)) activeVaultId = rules.vaultId
    val decision = evaluate(rules, packageName, nowEpochMs)
    if (!decision.blocked) return false
    return enforce(packageName, rules, decision, nowEpochMs)
  }

  fun onCheckpoint(nowEpochMs: Long): Boolean {
    checkpoint(nowEpochMs)
    val packageName = activePackage ?: return false
    activeVaultId = null
    if (!DoomscrollingAccess.hasUsageAccess(context)) return false
    if (ProtectedPackages.isProtected(context, packageName)) return false
    val rules = DoomscrollingRuntimeStore.rules(context) ?: return false
    if (usageTracked(rules, packageName)) activeVaultId = rules.vaultId
    val decision = evaluate(rules, packageName, nowEpochMs)
    return decision.blocked && enforce(packageName, rules, decision, nowEpochMs)
  }

  private fun enforce(
    packageName: String,
    rules: DoomscrollingRulesSnapshot,
    decision: BlockDecision,
    nowEpochMs: Long,
  ): Boolean {
    if (lastBlockedPackage == packageName && nowEpochMs - lastBlockedAt < 1_500L) return true
    lastBlockedPackage = packageName
    lastBlockedAt = nowEpochMs
    val displayName = activeName ?: packageName
    val phase = DoomscrollingRuntimeStore.phase(context)
    journal.recordBlock(
      packageName,
      displayName,
      nowEpochMs,
      decision.reason ?: "rule",
      decision.ruleId,
      phase?.runId,
      phase?.phase,
      rules.vaultId,
    )
    notifyBlocked(rules, packageName, displayName, decision, nowEpochMs)
    return true
  }

  fun checkpoint(nowEpochMs: Long, countUntilNow: Boolean = false) {
    val packageName = activePackage ?: return
    val startedAt = activeStartedAt
    if (startedAt > 0L
      && nowEpochMs > startedAt
      && (countUntilNow || isInteractiveAndUnlocked())
    ) {
      val vaultId = activeVaultId
      if (vaultId != null) {
        journal.recordUsage(vaultId, packageName, activeName ?: packageName, startedAt, nowEpochMs)
      }
    }
    activeStartedAt = nowEpochMs
    journal.setMetadata("lastObservedEpochMs", nowEpochMs)
  }

  fun stopTracking(nowEpochMs: Long, countUntilNow: Boolean = false) {
    checkpoint(nowEpochMs, countUntilNow)
    activePackage = null
    activeName = null
    activeStartedAt = 0L
    activeVaultId = null
  }

  fun reconcile(nowEpochMs: Long) {
    if (!DoomscrollingAccess.hasUsageAccess(context)) return
    val rules = DoomscrollingRuntimeStore.rules(context) ?: return
    val tracked = if (rules.limitsEnabled) {
      rules.limits.filter { it.enabled }.flatMapTo(mutableSetOf<String>()) { it.packages }
    } else {
      mutableSetOf<String>()
    }
    if (tracked.isEmpty()) return
    val lastObserved = journal.metadata("lastObservedEpochMs") ?: nowEpochMs
    val queryStart = maxOf(nowEpochMs - 12 * 60 * 60 * 1_000L, lastObserved - 12 * 60 * 60 * 1_000L)
    val events = (context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager)
      .queryEvents(queryStart, nowEpochMs)
    val starts = mutableMapOf<String, Long>()
    val event = UsageEvents.Event()
    while (events.hasNextEvent()) {
      events.getNextEvent(event)
      val packageName = event.packageName?.lowercase() ?: continue
      if (packageName !in tracked) continue
      if (event.eventType == UsageEvents.Event.ACTIVITY_RESUMED) {
        starts[packageName] = event.timeStamp
      } else if (event.eventType == UsageEvents.Event.ACTIVITY_PAUSED) {
        val startedAt = starts.remove(packageName) ?: continue
        val clippedStart = maxOf(startedAt, lastObserved)
        if (event.timeStamp > clippedStart) {
          journal.recordUsage(
            rules.vaultId,
            packageName,
            appLabel(packageName),
            clippedStart,
            event.timeStamp,
          )
        }
      }
    }
    starts.maxByOrNull { it.value }?.let { (packageName, startedAt) ->
      val clippedStart = maxOf(startedAt, lastObserved)
      if (nowEpochMs > clippedStart) {
        journal.recordUsage(
          rules.vaultId,
          packageName,
          appLabel(packageName),
          clippedStart,
          nowEpochMs,
        )
      }
      activePackage = packageName
      activeName = appLabel(packageName)
      activeStartedAt = nowEpochMs
      activeVaultId = rules.vaultId
    }
    journal.setMetadata("lastObservedEpochMs", nowEpochMs)
  }

  private fun evaluate(
    rules: DoomscrollingRulesSnapshot,
    packageName: String,
    nowEpochMs: Long,
  ): BlockDecision {
    if (DoomscrollingEvaluator.evaluateSchedule(
        rules.mobile,
        DoomscrollingRuntimeStore.phase(context),
        packageName,
        nowEpochMs,
      )) {
      return BlockDecision(true, "schedule", null)
    }
    if (!rules.limitsEnabled) return BlockDecision(false)
    val localDate = localDate(nowEpochMs)
    val weekStart = Instant.ofEpochMilli(nowEpochMs).atZone(ZoneId.systemDefault())
      .toLocalDate().minusDays((Instant.ofEpochMilli(nowEpochMs).atZone(ZoneId.systemDefault())
        .dayOfWeek.value - 1).toLong()).toString()
    for (limit in rules.limits) {
      if (!limit.enabled || packageName.lowercase() !in limit.packages) continue
      val dailyExhausted = limit.minutesPerDay?.let {
        journal.usedSeconds(limit.packages, localDate, localDate) >= it * 60
      } ?: false
      val weeklyExhausted = limit.minutesPerWeek?.let {
        journal.usedSeconds(limit.packages, weekStart, localDate) >= it * 60
      } ?: false
      if (dailyExhausted || weeklyExhausted) {
        return BlockDecision(true, "usage_limit", limit.id)
      }
    }
    return BlockDecision(false)
  }

  private fun usageTracked(rules: DoomscrollingRulesSnapshot, packageName: String): Boolean {
    if (!rules.limitsEnabled) return false
    val packageKey = packageName.lowercase()
    return rules.limits.any { it.enabled && packageKey in it.packages }
  }

  private fun notifyBlocked(
    rules: DoomscrollingRulesSnapshot,
    packageName: String,
    displayName: String,
    decision: BlockDecision,
    nowEpochMs: Long,
  ) {
    if (!DoomscrollingRuntimeStore.allowNotification(context, nowEpochMs)) return
    val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    manager.createNotificationChannel(
      NotificationChannel(
        BLOCK_CHANNEL_ID,
        rules.copy.channelName,
        NotificationManager.IMPORTANCE_DEFAULT,
      ).apply { description = rules.copy.channelDescription },
    )
    val target = if (decision.reason == "usage_limit") "limits" else "mobile"
    val launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)?.apply {
      flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
      putExtra(ACTION_TARGET_KEY, target)
    }
    val contentIntent = launchIntent?.let {
      PendingIntent.getActivity(
        context,
        BLOCK_NOTIFICATION_BASE_ID + packageName.hashCode(),
        it,
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
      )
    }
    val body = if (decision.reason == "usage_limit") {
      rules.copy.limitMessage
    } else {
      rules.copy.blockedMessage
    }
    manager.notify(
      BLOCK_NOTIFICATION_BASE_ID + (packageName.hashCode() and 0xFFFF),
      Notification.Builder(context, BLOCK_CHANNEL_ID)
        .setSmallIcon(notificationIcon())
        .setContentTitle(displayName)
        .setContentText(body)
        .setCategory(Notification.CATEGORY_STATUS)
        .setVisibility(Notification.VISIBILITY_PRIVATE)
        .setContentIntent(contentIntent)
        .setAutoCancel(true)
        .build(),
    )
  }

  private fun isInteractiveAndUnlocked(): Boolean {
    val power = context.getSystemService(Context.POWER_SERVICE) as android.os.PowerManager
    val keyguard = context.getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager
    return power.isInteractive && !keyguard.isKeyguardLocked
  }

  private fun appLabel(packageName: String): String = try {
    val info = context.packageManager.getApplicationInfo(packageName, 0)
    context.packageManager.getApplicationLabel(info).toString().trim().take(120)
      .ifBlank { packageName }
  } catch (_: PackageManager.NameNotFoundException) {
    packageName
  }

  private fun localDate(epochMs: Long): String = Instant.ofEpochMilli(epochMs)
    .atZone(ZoneId.systemDefault()).toLocalDate().toString()

  private fun notificationIcon(): Int = context.resources.getIdentifier(
    "ic_notification_focus",
    "drawable",
    context.packageName,
  ).takeIf { it != 0 } ?: context.applicationInfo.icon
}
