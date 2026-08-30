package app.ganbaru.mobile_doomscrolling

import android.accessibilityservice.AccessibilityService
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.view.accessibility.AccessibilityEvent

class DoomscrollingAccessibilityService : AccessibilityService() {
  private lateinit var engine: DoomscrollingEngine
  private var receiverRegistered = false
  private val handler = Handler(Looper.getMainLooper())
  private val checkpoint = object : Runnable {
    override fun run() {
      if (engine.onCheckpoint(System.currentTimeMillis())) {
        performGlobalAction(GLOBAL_ACTION_HOME)
      }
      handler.postDelayed(this, CHECKPOINT_INTERVAL_MS)
    }
  }
  private val runtimeReceiver = object : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
      if (intent.action == Intent.ACTION_SCREEN_OFF) {
        engine.stopTracking(System.currentTimeMillis(), countUntilNow = true)
        return
      }
      val runtimeChanged = intent.action == ACTION_DOOMSCROLLING_RULES_CHANGED
        || DoomscrollingRuntimeStore.updatePhaseFromIntent(context, intent)
      if (runtimeChanged
        && engine.onCheckpoint(System.currentTimeMillis())
      ) {
        performGlobalAction(GLOBAL_ACTION_HOME)
      }
    }
  }

  override fun onServiceConnected() {
    super.onServiceConnected()
    engine = DoomscrollingEngine(applicationContext)
    engine.reconcile(System.currentTimeMillis())
    val filter = IntentFilter(Intent.ACTION_SCREEN_OFF).apply {
      addAction(ACTION_DOOMSCROLLING_PHASE)
      addAction(ACTION_DOOMSCROLLING_PHASE_CLEAR)
      addAction(ACTION_DOOMSCROLLING_RULES_CHANGED)
    }
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      registerReceiver(runtimeReceiver, filter, Context.RECEIVER_NOT_EXPORTED)
    } else {
      @Suppress("DEPRECATION")
      registerReceiver(runtimeReceiver, filter)
    }
    receiverRegistered = true
    handler.postDelayed(checkpoint, CHECKPOINT_INTERVAL_MS)
  }

  override fun onAccessibilityEvent(event: AccessibilityEvent?) {
    val packageName = event?.packageName?.toString()?.trim().orEmpty()
    if (packageName.isEmpty()) return
    val blocked = engine.onForegroundPackage(packageName, System.currentTimeMillis())
    if (blocked) performGlobalAction(GLOBAL_ACTION_HOME)
  }

  override fun onInterrupt() = Unit

  override fun onDestroy() {
    handler.removeCallbacks(checkpoint)
    if (::engine.isInitialized) {
      engine.stopTracking(System.currentTimeMillis())
    }
    if (receiverRegistered) {
      try {
        unregisterReceiver(runtimeReceiver)
      } catch (_: IllegalArgumentException) {
        Unit
      }
      receiverRegistered = false
    }
    super.onDestroy()
  }

  companion object {
    private const val CHECKPOINT_INTERVAL_MS = 30_000L
  }
}
