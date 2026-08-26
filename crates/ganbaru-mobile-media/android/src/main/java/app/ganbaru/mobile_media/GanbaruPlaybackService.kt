package app.ganbaru.mobile_media

import android.app.PendingIntent
import android.content.Intent
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService

class GanbaruPlaybackService : MediaSessionService() {
  private var mediaSession: MediaSession? = null

  override fun onCreate() {
    super.onCreate()
    val audioAttributes = AudioAttributes.Builder()
      .setUsage(C.USAGE_MEDIA)
      .setContentType(C.AUDIO_CONTENT_TYPE_MUSIC)
      .build()
    val player = ExoPlayer.Builder(this)
      .setAudioAttributes(audioAttributes, true)
      .setHandleAudioBecomingNoisy(true)
      .setWakeMode(C.WAKE_MODE_LOCAL)
      .build()
    player.repeatMode = Player.REPEAT_MODE_OFF
    val sessionBuilder = MediaSession.Builder(this, player)
    packageManager.getLaunchIntentForPackage(packageName)?.let { launchIntent ->
      launchIntent.addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
      sessionBuilder.setSessionActivity(
        PendingIntent.getActivity(
          this,
          0,
          launchIntent,
          PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        ),
      )
    }
    mediaSession = sessionBuilder.build()
  }

  override fun onGetSession(
    controllerInfo: MediaSession.ControllerInfo,
  ): MediaSession? {
    val session = mediaSession ?: return null
    return if (controllerInfo.packageName == packageName || controllerInfo.isTrusted) session else null
  }

  override fun onDestroy() {
    mediaSession?.run {
      player.release()
      release()
    }
    mediaSession = null
    super.onDestroy()
  }
}
