package app.ganbaru.mobile_doomscrolling

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
internal class ApplyRulesArgs {
  lateinit var snapshotJson: String
}

@InvokeArg
internal class AcknowledgeEventsArgs {
  var ids: List<String> = listOf()
}

@TauriPlugin
class MobileDoomscrollingPlugin(private val activity: Activity) : Plugin(activity) {
  override fun load(webView: WebView) {
    super.load(webView)
    DoomscrollingRuntimeStore.captureNotificationAction(activity, activity.intent)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    DoomscrollingRuntimeStore.captureNotificationAction(activity, intent)
  }

  @Command
  fun accessStatus(invoke: Invoke) {
    invoke.resolve(JSObject().apply {
      put("usageAccess", DoomscrollingAccess.hasUsageAccess(activity))
      put("accessibility", DoomscrollingAccess.hasAccessibilityAccess(activity))
    })
  }

  @Command
  fun openUsageAccessSettings(invoke: Invoke) {
    val intents = listOf(
      Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS).apply {
        data = Uri.parse("package:${activity.packageName}")
      },
      Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS),
      applicationDetailsIntent(),
    )
    startFirstAvailable(intents, invoke, "Usage Access")
  }

  @Command
  fun openAccessibilitySettings(invoke: Invoke) {
    startFirstAvailable(
      listOf(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS), applicationDetailsIntent()),
      invoke,
      "Accessibility",
    )
  }

  @Command
  fun listLaunchableApps(invoke: Invoke) {
    try {
      val manager = activity.packageManager
      val protected = ProtectedPackages.resolve(activity)
      val intent = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
      val apps = manager.queryIntentActivities(intent, 0)
        .asSequence()
        .mapNotNull { resolved ->
          val packageName = resolved.activityInfo?.packageName?.trim().orEmpty()
          if (packageName.isEmpty() || packageName in protected) return@mapNotNull null
          if (ProtectedPackages.isProtected(activity, packageName)) return@mapNotNull null
          val label = resolved.loadLabel(manager).toString().trim().take(120)
            .ifBlank { packageName }
          label to packageName
        }
        .distinctBy { it.second.lowercase() }
        .sortedWith(compareBy(String.CASE_INSENSITIVE_ORDER) { it.first })
        .take(512)
        .map { (name, packageName) ->
          JSObject().apply {
            put("name", name)
            put("packageName", packageName)
          }
        }
        .toList()
      invoke.resolveObject(JSArray().apply { apps.forEach(::put) })
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Failed to list launchable Android apps")
    }
  }

  @Command
  fun applyRules(invoke: Invoke) {
    val args = invoke.parseArgs(ApplyRulesArgs::class.java)
    try {
      DoomscrollingRuntimeStore.saveRules(activity, args.snapshotJson)
      activity.sendBroadcast(Intent(ACTION_DOOMSCROLLING_RULES_CHANGED).apply {
        setPackage(activity.packageName)
      })
      invoke.resolve()
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Invalid mobile Doomscrolling rule snapshot")
    }
  }

  @Command
  fun pendingEvents(invoke: Invoke) {
    try {
      val events = DoomscrollingJournal(activity).pending().map { event ->
        JSObject().apply {
          put("id", event.id)
          put("kind", event.kind)
          put("packageName", event.packageName)
          put("displayName", event.displayName)
          put("startedAt", event.startedAt)
          put("elapsedSeconds", event.elapsedSeconds)
          put("localDate", event.localDate)
          put("occurredAt", event.occurredAt)
          put("reason", event.reason)
          put("ruleId", event.ruleId)
          put("runId", event.runId)
          put("phase", event.phase)
          put("vaultId", event.vaultId)
        }
      }
      invoke.resolveObject(JSArray().apply { events.forEach(::put) })
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Failed to read mobile Doomscrolling events")
    }
  }

  @Command
  fun acknowledgeEvents(invoke: Invoke) {
    val args = invoke.parseArgs(AcknowledgeEventsArgs::class.java)
    try {
      require(args.ids.size <= 500) { "Too many Doomscrolling event acknowledgements" }
      require(args.ids.all { it.length in 1..120 }) { "Doomscrolling event ID is invalid" }
      DoomscrollingJournal(activity).acknowledge(args.ids)
      invoke.resolve()
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Failed to acknowledge mobile Doomscrolling events")
    }
  }

  @Command
  fun takeNotificationAction(invoke: Invoke) {
    invoke.resolve(JSObject().apply {
      put("target", DoomscrollingRuntimeStore.takeNotificationAction(activity))
    })
  }

  private fun startFirstAvailable(intents: List<Intent>, invoke: Invoke, label: String) {
    var lastError: Exception? = null
    for (intent in intents) {
      if (intent.resolveActivity(activity.packageManager) == null) continue
      try {
        activity.startActivity(intent)
        invoke.resolve()
        return
      } catch (error: Exception) {
        lastError = error
      }
    }
    invoke.reject(lastError?.message ?: "No compatible $label settings screen is available")
  }

  private fun applicationDetailsIntent(): Intent = Intent(
    Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
    Uri.parse("package:${activity.packageName}"),
  )
}
