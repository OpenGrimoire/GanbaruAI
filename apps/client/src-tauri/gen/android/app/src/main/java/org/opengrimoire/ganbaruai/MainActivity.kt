package org.opengrimoire.ganbaruai

import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature
import kotlin.math.roundToInt

private const val ANDROID_INSETS_BRIDGE_NAME = "GanbaruAndroidInsets"
private const val ANDROID_APPEARANCE_BRIDGE_NAME = "GanbaruAndroidAppearance"
private const val ANDROID_INSETS_EVENT_NAME = "ganbaru:android-insets"

private class AndroidInsetsBridge(displayDensity: Float) {
  private val density = displayDensity.takeIf { it.isFinite() && it > 0f } ?: 1f
  private var top = 0
  private var right = 0
  private var bottom = 0
  private var left = 0

  @Synchronized
  fun update(top: Int, right: Int, bottom: Int, left: Int) {
    this.top = top
    this.right = right
    this.bottom = bottom
    this.left = left
  }

  @JavascriptInterface
  @Synchronized
  fun systemBars(): String = listOf(top, right, bottom, left)
    .joinToString(",") { value -> (value / density).roundToInt().toString() }
}

private class AndroidAppearanceBridge(private val activity: MainActivity) {
  @JavascriptInterface
  fun setLightTheme(lightTheme: Boolean) {
    activity.runOnUiThread {
      WindowCompat.getInsetsController(activity.window, activity.window.decorView).apply {
        isAppearanceLightStatusBars = lightTheme
        isAppearanceLightNavigationBars = lightTheme
      }
    }
  }
}

class MainActivity : TauriActivity() {
  private val androidInsetsBridge by lazy {
    AndroidInsetsBridge(resources.displayMetrics.density)
  }
  private val androidAppearanceBridge by lazy { AndroidAppearanceBridge(this) }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
      WebSettingsCompat.setAlgorithmicDarkeningAllowed(webView.settings, false)
    }
    webView.addJavascriptInterface(androidInsetsBridge, ANDROID_INSETS_BRIDGE_NAME)
    webView.addJavascriptInterface(androidAppearanceBridge, ANDROID_APPEARANCE_BRIDGE_NAME)
    ViewCompat.setOnApplyWindowInsetsListener(webView) { _, windowInsets ->
      val systemBars = windowInsets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      androidInsetsBridge.update(
        top = systemBars.top,
        right = systemBars.right,
        bottom = systemBars.bottom,
        left = systemBars.left,
      )
      webView.evaluateJavascript(
        "window.dispatchEvent(new Event('$ANDROID_INSETS_EVENT_NAME'))",
        null,
      )
      windowInsets
    }
    ViewCompat.requestApplyInsets(webView)
  }
}
