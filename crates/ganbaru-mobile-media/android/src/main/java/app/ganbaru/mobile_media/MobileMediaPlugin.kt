package app.ganbaru.mobile_media

import android.app.Activity
import android.content.ComponentName
import android.content.Intent
import android.media.MediaMetadataRetriever
import android.net.Uri
import android.os.Environment
import android.provider.DocumentsContract
import android.util.Base64
import androidx.activity.result.ActivityResult
import androidx.core.content.ContextCompat
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.session.MediaController
import androidx.media3.session.SessionToken
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.google.common.util.concurrent.ListenableFuture
import java.util.Locale
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong

private const val MAX_SCANNED_ENTRIES = 20_000
private const val SAF_LOCATOR_PREFIX = "ganbaru-saf:"
private const val EXTERNAL_STORAGE_DOCUMENTS_AUTHORITY = "com.android.externalstorage.documents"

private val supportedAudioExtensions = setOf(
  "aac", "aif", "aiff", "alac", "flac", "m4a", "mp3", "ogg", "opus", "wav", "wma",
)

private val embeddedArtworkExtensions = setOf(
  "aac", "aif", "aiff", "alac", "flac", "m4a", "mp3", "ogg", "opus", "wav",
)

@InvokeArg
internal class PathArgs {
  lateinit var path: String
}

@InvokeArg
internal class LoadArgs {
  lateinit var source: MediaSourceArgs
  var startMs: Long? = null
  var volume: Double? = null
  var rate: Double? = null
}

internal class MediaSourceArgs {
  lateinit var kind: String
  lateinit var path: String
  lateinit var identity: String
  var title: String? = null
}

@InvokeArg
internal class SeekArgs {
  var positionMs: Long = 0
}

@InvokeArg
internal class VolumeArgs {
  var volume: Double = 0.8
}

@InvokeArg
internal class MutedArgs {
  var muted: Boolean = false
}

@InvokeArg
internal class RateArgs {
  var rate: Double = 1.0
}

@InvokeArg
internal class TreePickArgs {
  var maxFiles: Int = 0
  var maxDepth: Int = 0
}

@InvokeArg
internal class TreeScanArgs {
  lateinit var treeUri: String
  var maxFiles: Int = 0
  var maxDepth: Int = 0
}

@InvokeArg
internal class ArtworkArgs {
  lateinit var path: String
  var embedded: Boolean = false
  var maxBytes: Long = 0
}

private data class PendingTreePick(
  val maxFiles: Int,
  val maxDepth: Int,
)

private data class DocumentEntry(
  val documentId: String,
  val displayName: String,
  val mimeType: String,
  val flags: Int,
  val size: Long?,
  val modifiedAtMs: Long?,
)

private data class ScanBudget(
  val maxFiles: Int,
  val maxDepth: Int,
  var scannedEntries: Int = 0,
  var mediaFiles: Int = 0,
  var truncated: Boolean = false,
)

@TauriPlugin
class MobileMediaPlugin(private val activity: Activity) : Plugin(activity) {
  private val mainExecutor = ContextCompat.getMainExecutor(activity)
  private var controllerFuture: ListenableFuture<MediaController>? = null
  private var pendingTreePick: PendingTreePick? = null
  @Volatile private var pendingTreeCache: Pair<String, JSObject>? = null
  private val pendingArtworkPick = AtomicBoolean(false)
  private val loadGeneration = AtomicLong(0)
  private var muted = false
  private var intendedVolume = 0.8f
  private var sourceIdentity: String? = null
  private var sourceTitle: String? = null
  private var sourceHasVideo = false
  private var lastError: String? = null

  private fun mediaController(): ListenableFuture<MediaController> {
    controllerFuture?.let { return it }
    val token = SessionToken(
      activity,
      ComponentName(activity, GanbaruPlaybackService::class.java),
    )
    return MediaController.Builder(activity, token).buildAsync().also {
      controllerFuture = it
    }
  }

  private fun withController(invoke: Invoke, action: (MediaController) -> Unit) {
    val future = mediaController()
    future.addListener({
      try {
        action(future.get())
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Android media service is unavailable")
      }
    }, mainExecutor)
  }

  @Command
  fun probe(invoke: Invoke) {
    val args = invoke.parseArgs(PathArgs::class.java)
    Thread {
      try {
        val uri = resolveMediaUri(args.path)
        val metadata = documentMetadata(uri)
        val media = readMediaMetadata(
          uri,
          metadata.displayName.substringBeforeLast('.', metadata.displayName),
          extension(metadata.displayName) in embeddedArtworkExtensions,
        )
        invoke.resolve(JSObject().apply {
          put("path", args.path)
          put("title", media.title)
          put("fileSizeBytes", metadata.size ?: 0L)
          put("extension", extension(metadata.displayName))
          put("mediaKind", if (media.hasVideo) "video" else "audio")
          put("playableStartMs", null)
        })
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Failed to inspect selected media")
      }
    }.start()
  }

  @Command
  fun load(invoke: Invoke) {
    val args = invoke.parseArgs(LoadArgs::class.java)
    val generation = loadGeneration.incrementAndGet()
    Thread {
      try {
        require(args.source.kind == "local-file") { "Android Media3 accepts local media sources only" }
        require(args.source.identity.isNotBlank()) { "Media source identity is required" }
        val uri = resolveMediaUri(args.source.path)
        val document = documentMetadata(uri)
        val title = args.source.title?.trim().takeUnless { it.isNullOrEmpty() }
          ?: document.displayName.substringBeforeLast('.', document.displayName)
        val probe = readMediaMetadata(
          uri,
          title,
          extension(document.displayName) in embeddedArtworkExtensions,
        )
        mainExecutor.execute {
          if (generation != loadGeneration.get()) {
            invoke.reject("Media load was superseded by a newer request")
            return@execute
          }
          sourceIdentity = args.source.identity
          sourceTitle = title
          sourceHasVideo = probe.hasVideo
          intendedVolume = args.volume?.coerceIn(0.0, 1.0)?.toFloat() ?: intendedVolume
          muted = false
          lastError = null
          withController(invoke) { controller ->
            if (generation != loadGeneration.get()) {
              invoke.reject("Media load was superseded by a newer request")
              return@withController
            }
            val item = MediaItem.Builder()
              .setUri(uri)
              .setMediaId(args.source.identity)
              .setMediaMetadata(
                MediaMetadata.Builder()
                  .setTitle(title)
                  .setArtist(probe.artist.ifBlank { null })
                  .setAlbumTitle(probe.album.ifBlank { null })
                  .build(),
              )
              .build()
            controller.setMediaItem(item, args.startMs?.coerceAtLeast(0L) ?: 0L)
            controller.volume = intendedVolume
            controller.setPlaybackSpeed(args.rate?.coerceIn(0.25, 2.0)?.toFloat() ?: 1.0f)
            controller.prepare()
            resolveSnapshot(invoke, controller)
          }
        }
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Failed to load selected media")
      }
    }.start()
  }

  @Command
  fun play(invoke: Invoke) = withController(invoke) { controller ->
    controller.play()
    resolveSnapshot(invoke, controller)
  }

  @Command
  fun pause(invoke: Invoke) = withController(invoke) { controller ->
    controller.pause()
    resolveSnapshot(invoke, controller)
  }

  @Command
  fun stop(invoke: Invoke) = withController(invoke) { controller ->
    loadGeneration.incrementAndGet()
    controller.stop()
    controller.clearMediaItems()
    sourceIdentity = null
    sourceTitle = null
    sourceHasVideo = false
    lastError = null
    resolveSnapshot(invoke, controller)
  }

  @Command
  fun seek(invoke: Invoke) {
    val args = invoke.parseArgs(SeekArgs::class.java)
    withController(invoke) { controller ->
      controller.seekTo(args.positionMs.coerceAtLeast(0L))
      resolveSnapshot(invoke, controller)
    }
  }

  @Command
  fun setVolume(invoke: Invoke) {
    val args = invoke.parseArgs(VolumeArgs::class.java)
    intendedVolume = args.volume.coerceIn(0.0, 1.0).toFloat()
    muted = false
    withController(invoke) { controller ->
      controller.volume = intendedVolume
      resolveSnapshot(invoke, controller)
    }
  }

  @Command
  fun setMuted(invoke: Invoke) {
    val args = invoke.parseArgs(MutedArgs::class.java)
    muted = args.muted
    withController(invoke) { controller ->
      controller.volume = if (muted) 0f else intendedVolume
      resolveSnapshot(invoke, controller)
    }
  }

  @Command
  fun setRate(invoke: Invoke) {
    val args = invoke.parseArgs(RateArgs::class.java)
    withController(invoke) { controller ->
      controller.setPlaybackSpeed(args.rate.coerceIn(0.25, 2.0).toFloat())
      resolveSnapshot(invoke, controller)
    }
  }

  @Command
  fun snapshot(invoke: Invoke) = withController(invoke) { controller ->
    resolveSnapshot(invoke, controller)
  }

  @Command
  fun pickMediaTree(invoke: Invoke) {
    try {
      require(pendingTreePick == null) { "A music folder picker is already active" }
      val args = invoke.parseArgs(TreePickArgs::class.java)
      validateScanLimits(args.maxFiles, args.maxDepth)
      pendingTreePick = PendingTreePick(args.maxFiles, args.maxDepth)
      val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        putExtra(DocumentsContract.EXTRA_INITIAL_URI, defaultMusicDirectoryUri())
      }
      startActivityForResult(invoke, intent, "pickMediaTreeResult")
    } catch (error: Exception) {
      pendingTreePick = null
      invoke.reject(error.message ?: "Failed to open music folder picker")
    }
  }

  private fun defaultMusicDirectoryUri(): Uri = DocumentsContract.buildDocumentUri(
    EXTERNAL_STORAGE_DOCUMENTS_AUTHORITY,
    "primary:${Environment.DIRECTORY_MUSIC}",
  )

  @ActivityCallback
  fun pickMediaTreeResult(invoke: Invoke, result: ActivityResult) {
    val pending = pendingTreePick
    pendingTreePick = null
    if (result.resultCode == Activity.RESULT_CANCELED) {
      invoke.resolve(JSObject().apply { put("tree", null) })
      return
    }
    val treeUri = result.data?.data
    if (result.resultCode != Activity.RESULT_OK || pending == null || treeUri == null) {
      invoke.reject("The selected music folder is unavailable")
      return
    }
    try {
      val grantedFlags = result.data?.flags?.and(
        Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION,
      ) ?: Intent.FLAG_GRANT_READ_URI_PERMISSION
      activity.contentResolver.takePersistableUriPermission(treeUri, grantedFlags)
    } catch (error: SecurityException) {
      invoke.reject("Android could not preserve access to the selected music folder")
      return
    }
    Thread {
      try {
        val tree = scanTree(treeUri, pending.maxFiles, pending.maxDepth)
        pendingTreeCache = treeUri.toString() to tree
        invoke.resolve(JSObject().apply {
          put("tree", tree)
        })
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Failed to scan selected music folder")
      }
    }.start()
  }

  @Command
  fun scanMediaTree(invoke: Invoke) {
    val args = invoke.parseArgs(TreeScanArgs::class.java)
    Thread {
      try {
        validateScanLimits(args.maxFiles, args.maxDepth)
        val treeUri = Uri.parse(args.treeUri)
        require(treeUri.scheme == "content") { "Music folder access is invalid" }
        require(hasPersistedReadPermission(treeUri)) { "Music folder access needs to be selected again" }
        val cached = pendingTreeCache?.takeIf { it.first == treeUri.toString() }
        if (cached != null) pendingTreeCache = null
        invoke.resolve(cached?.second ?: scanTree(treeUri, args.maxFiles, args.maxDepth))
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Failed to refresh selected music folder")
      }
    }.start()
  }

  @Command
  fun pickArtworkFile(invoke: Invoke) {
    if (!pendingArtworkPick.compareAndSet(false, true)) {
      invoke.reject("An artwork picker is already active")
      return
    }
    val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
      addCategory(Intent.CATEGORY_OPENABLE)
      type = "image/*"
      addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
      addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
    }
    startActivityForResult(invoke, intent, "pickArtworkFileResult")
  }

  @ActivityCallback
  fun pickArtworkFileResult(invoke: Invoke, result: ActivityResult) {
    pendingArtworkPick.set(false)
    if (result.resultCode == Activity.RESULT_CANCELED) {
      invoke.resolve(JSObject().apply { put("uri", null) })
      return
    }
    val uri = result.data?.data
    if (result.resultCode != Activity.RESULT_OK || uri == null) {
      invoke.reject("The selected artwork is unavailable")
      return
    }
    try {
      activity.contentResolver.takePersistableUriPermission(
        uri,
        Intent.FLAG_GRANT_READ_URI_PERMISSION,
      )
      invoke.resolve(JSObject().apply { put("uri", uri.toString()) })
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Android could not preserve access to the selected artwork")
    }
  }

  @Command
  fun artworkDataUrl(invoke: Invoke) {
    val args = invoke.parseArgs(ArtworkArgs::class.java)
    Thread {
      try {
        require(args.maxBytes in 1..12L * 1024L * 1024L) { "Artwork size limit is invalid" }
        val uri = resolveMediaUri(args.path)
        val bytes = if (args.embedded) {
          val retriever = MediaMetadataRetriever()
          try {
            retriever.setDataSource(activity, uri)
            retriever.embeddedPicture
          } finally {
            retriever.release()
          }
        } else {
          readBoundedBytes(uri, args.maxBytes)
        }
        if (bytes == null) {
          invoke.resolve(JSObject().apply { put("dataUrl", null) })
          return@Thread
        }
        require(bytes.size.toLong() <= args.maxBytes) { "Artwork exceeds the display size limit" }
        val contentType = imageContentType(bytes)
          ?: throw IllegalArgumentException("Selected artwork is not a supported image")
        val encoded = Base64.encodeToString(bytes, Base64.NO_WRAP)
        invoke.resolve(JSObject().apply { put("dataUrl", "data:$contentType;base64,$encoded") })
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Failed to load artwork")
      }
    }.start()
  }

  @Suppress("OVERRIDE_DEPRECATION")
  override fun onDestroy() {
    controllerFuture?.let { MediaController.releaseFuture(it) }
    controllerFuture = null
  }

  private fun resolveSnapshot(invoke: Invoke, controller: MediaController) {
    val playerError = controller.playerError
    if (playerError != null) lastError = playbackErrorMessage(playerError)
    val duration = controller.duration.takeIf { it > 0 && it != C.TIME_UNSET }
    invoke.resolve(JSObject().apply {
      put("status", playbackStatus(controller))
      put("sourceIdentity", controller.currentMediaItem?.mediaId?.takeUnless { it.isBlank() } ?: sourceIdentity)
      put("title", controller.currentMediaItem?.mediaMetadata?.title?.toString() ?: sourceTitle)
      put("positionMs", controller.currentPosition.coerceAtLeast(0L))
      put("durationMs", duration)
      put("volume", intendedVolume.toDouble())
      put("muted", muted)
      put("rate", controller.playbackParameters.speed.toDouble())
      put(
        "hasVideo",
        controller.currentTracks.groups.any { group ->
          group.type == C.TRACK_TYPE_VIDEO && group.isSelected
        } || sourceHasVideo,
      )
      put("backendKind", "media3")
      put("playableStartMs", null)
      put("error", lastError)
    })
  }

  private fun playbackStatus(controller: MediaController): String = when {
    controller.playerError != null -> "error"
    controller.playbackState == Player.STATE_ENDED -> "ended"
    controller.isPlaying -> "playing"
    controller.playbackState == Player.STATE_BUFFERING -> "loading"
    controller.playbackState == Player.STATE_READY && controller.playWhenReady -> "playing"
    controller.playbackState == Player.STATE_READY -> "paused"
    controller.mediaItemCount > 0 -> "ready"
    else -> "idle"
  }

  private fun playbackErrorMessage(error: PlaybackException): String =
    error.message ?: "Android could not play this media file"

  private fun validateScanLimits(maxFiles: Int, maxDepth: Int) {
    require(maxFiles in 1..5_000) { "Music scan file limit must be between 1 and 5000" }
    require(maxDepth in 1..64) { "Music scan depth limit must be between 1 and 64" }
  }

  private fun hasPersistedReadPermission(uri: Uri): Boolean =
    activity.contentResolver.persistedUriPermissions.any { permission ->
      permission.isReadPermission && permission.uri == uri
    }

  private fun scanTree(treeUri: Uri, maxFiles: Int, maxDepth: Int): JSObject {
    val rootId = DocumentsContract.getTreeDocumentId(treeUri)
    val rootUri = DocumentsContract.buildDocumentUriUsingTree(treeUri, rootId)
    val rootName = documentMetadata(rootUri).displayName
    val budget = ScanBudget(maxFiles, maxDepth)
    val tracks = ArrayList<JSObject>()
    scanDirectory(treeUri, rootId, "", 0, budget, tracks)
    tracks.sortBy { it.getString("relativePath") }
    return JSObject().apply {
      put("treeUri", treeUri.toString())
      put("displayName", rootName)
      put("tracks", JSArray(tracks))
      put("truncated", budget.truncated)
    }
  }

  private fun scanDirectory(
    treeUri: Uri,
    documentId: String,
    relativeDirectory: String,
    depth: Int,
    budget: ScanBudget,
    tracks: MutableList<JSObject>,
  ) {
    if (depth > budget.maxDepth || budget.truncated) {
      budget.truncated = true
      return
    }
    val children = queryDocumentChildren(treeUri, documentId)
      .sortedWith(compareBy(String.CASE_INSENSITIVE_ORDER) { it.displayName })
    for (entry in children) {
      budget.scannedEntries += 1
      if (budget.scannedEntries > MAX_SCANNED_ENTRIES) {
        budget.truncated = true
        return
      }
      val relativePath = if (relativeDirectory.isEmpty()) {
        entry.displayName
      } else {
        "$relativeDirectory/${entry.displayName}"
      }
      if (entry.mimeType == DocumentsContract.Document.MIME_TYPE_DIR) {
        scanDirectory(treeUri, entry.documentId, relativePath, depth + 1, budget, tracks)
        if (budget.truncated) return
        continue
      }
      if (entry.flags and DocumentsContract.Document.FLAG_VIRTUAL_DOCUMENT != 0) continue
      if (!isSupportedAudio(entry.displayName, entry.mimeType)) continue
      if (budget.mediaFiles >= budget.maxFiles) {
        budget.truncated = true
        return
      }
      val uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, entry.documentId)
      val metadata = readMediaMetadata(
        uri,
        entry.displayName.substringBeforeLast('.', entry.displayName),
        extension(entry.displayName) in embeddedArtworkExtensions,
      )
      tracks.add(JSObject().apply {
        put("uri", uri.toString())
        put("relativePath", relativePath)
        put("title", metadata.title)
        put("artist", metadata.artist)
        put("album", metadata.album)
        put("trackNumber", metadata.trackNumber)
        put("artworkUri", null)
        put("embeddedArtworkCandidate", metadata.embeddedArtworkCandidate)
        put("durationMs", metadata.durationMs)
        put("fileSizeBytes", entry.size)
        put("modifiedAtMs", entry.modifiedAtMs)
        put("mediaKind", "audio")
        put("mimeType", entry.mimeType)
      })
      budget.mediaFiles += 1
    }
  }

  private fun queryDocumentChildren(treeUri: Uri, parentDocumentId: String): List<DocumentEntry> {
    val childrenUri = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, parentDocumentId)
    val projection = arrayOf(
      DocumentsContract.Document.COLUMN_DOCUMENT_ID,
      DocumentsContract.Document.COLUMN_DISPLAY_NAME,
      DocumentsContract.Document.COLUMN_MIME_TYPE,
      DocumentsContract.Document.COLUMN_FLAGS,
      DocumentsContract.Document.COLUMN_SIZE,
      DocumentsContract.Document.COLUMN_LAST_MODIFIED,
    )
    return activity.contentResolver.query(childrenUri, projection, null, null, null)?.use { cursor ->
      val entries = ArrayList<DocumentEntry>()
      while (cursor.moveToNext()) {
        entries.add(
          DocumentEntry(
            documentId = cursor.getString(0),
            displayName = cursor.getString(1) ?: continue,
            mimeType = cursor.getString(2) ?: "application/octet-stream",
            flags = cursor.getInt(3),
            size = cursor.getLong(4).takeUnless { cursor.isNull(4) },
            modifiedAtMs = cursor.getLong(5).takeUnless { cursor.isNull(5) },
          ),
        )
      }
      entries
    } ?: throw IllegalStateException("Android could not list the selected music folder")
  }

  private fun documentMetadata(uri: Uri): DocumentEntry {
    val projection = arrayOf(
      DocumentsContract.Document.COLUMN_DOCUMENT_ID,
      DocumentsContract.Document.COLUMN_DISPLAY_NAME,
      DocumentsContract.Document.COLUMN_MIME_TYPE,
      DocumentsContract.Document.COLUMN_FLAGS,
      DocumentsContract.Document.COLUMN_SIZE,
      DocumentsContract.Document.COLUMN_LAST_MODIFIED,
    )
    return activity.contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
      check(cursor.moveToFirst()) { "Selected media is unavailable" }
      DocumentEntry(
        documentId = cursor.getString(0),
        displayName = cursor.getString(1) ?: "Media",
        mimeType = cursor.getString(2) ?: "application/octet-stream",
        flags = cursor.getInt(3),
        size = cursor.getLong(4).takeUnless { cursor.isNull(4) },
        modifiedAtMs = cursor.getLong(5).takeUnless { cursor.isNull(5) },
      )
    } ?: throw IllegalStateException("Selected media is unavailable")
  }

  private data class ExtractedMetadata(
    val title: String,
    val artist: String,
    val album: String,
    val trackNumber: Long?,
    val durationMs: Long?,
    val hasVideo: Boolean,
    val embeddedArtworkCandidate: Boolean,
  )

  private fun readMediaMetadata(
    uri: Uri,
    fallbackTitle: String,
    embeddedArtworkCandidate: Boolean,
  ): ExtractedMetadata {
    val retriever = MediaMetadataRetriever()
    return try {
      retriever.setDataSource(activity, uri)
      val title = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_TITLE)
        ?.trim().takeUnless { it.isNullOrEmpty() } ?: fallbackTitle
      val artist = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ARTIST)?.trim().orEmpty()
      val album = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ALBUM)?.trim().orEmpty()
      val trackNumber = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_CD_TRACK_NUMBER)
        ?.substringBefore('/')?.trim()?.toLongOrNull()
      val duration = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)?.toLongOrNull()
      val hasVideo = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_HAS_VIDEO) == "yes"
      ExtractedMetadata(title, artist, album, trackNumber, duration, hasVideo, embeddedArtworkCandidate)
    } catch (_: Exception) {
      ExtractedMetadata(fallbackTitle, "", "", null, null, false, embeddedArtworkCandidate)
    } finally {
      retriever.release()
    }
  }

  private fun isSupportedAudio(displayName: String, mimeType: String): Boolean {
    if (mimeType.startsWith("audio/")) return true
    return extension(displayName)?.lowercase(Locale.ROOT) in supportedAudioExtensions
  }

  private fun extension(displayName: String): String? {
    val index = displayName.lastIndexOf('.')
    if (index < 0 || index == displayName.lastIndex) return null
    return displayName.substring(index + 1).lowercase(Locale.ROOT)
  }

  private fun readBoundedBytes(uri: Uri, maxBytes: Long): ByteArray {
    val stream = activity.contentResolver.openInputStream(uri)
      ?: throw IllegalStateException("Selected artwork could not be opened")
    return stream.use { input ->
      val output = java.io.ByteArrayOutputStream()
      val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
      var total = 0L
      while (true) {
        val count = input.read(buffer)
        if (count < 0) break
        total += count
        require(total <= maxBytes) { "Artwork exceeds the display size limit" }
        output.write(buffer, 0, count)
      }
      output.toByteArray()
    }
  }

  private fun imageContentType(bytes: ByteArray): String? = when {
    bytes.size >= 8 && bytes.copyOfRange(0, 8).contentEquals(
      byteArrayOf(0x89.toByte(), 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a),
    ) -> "image/png"
    bytes.size >= 3 && bytes[0] == 0xff.toByte() && bytes[1] == 0xd8.toByte() && bytes[2] == 0xff.toByte() -> "image/jpeg"
    bytes.size >= 6 && (String(bytes, 0, 6, Charsets.US_ASCII) == "GIF87a" || String(bytes, 0, 6, Charsets.US_ASCII) == "GIF89a") -> "image/gif"
    bytes.size >= 12 && String(bytes, 0, 4, Charsets.US_ASCII) == "RIFF" && String(bytes, 8, 4, Charsets.US_ASCII) == "WEBP" -> "image/webp"
    bytes.size >= 2 && bytes[0] == 'B'.code.toByte() && bytes[1] == 'M'.code.toByte() -> "image/bmp"
    bytes.size >= 12 && String(bytes, 4, 4, Charsets.US_ASCII) == "ftyp" && String(bytes, 8, 4, Charsets.US_ASCII) in setOf("avif", "avis") -> "image/avif"
    else -> null
  }

  private fun resolveMediaUri(locator: String): Uri {
    val direct = Uri.parse(locator)
    if (direct.scheme == "content") return direct
    require(locator.startsWith(SAF_LOCATOR_PREFIX)) { "Android media must come from a selected document" }
    val encoded = locator.removePrefix(SAF_LOCATOR_PREFIX)
    val separator = encoded.indexOf('#')
    require(separator > 0 && separator < encoded.lastIndex) { "Android media locator is invalid" }
    val treeUri = Uri.parse(Uri.decode(encoded.substring(0, separator)))
    val relativePath = Uri.decode(encoded.substring(separator + 1))
    require(treeUri.scheme == "content") { "Android music folder locator is invalid" }
    require(hasPersistedReadPermission(treeUri)) { "Music folder access needs to be selected again" }
    var documentId = DocumentsContract.getTreeDocumentId(treeUri)
    for (segment in relativePath.split('/').filter { it.isNotBlank() }) {
      val child = queryDocumentChildren(treeUri, documentId)
        .firstOrNull { it.displayName == segment }
        ?: throw IllegalStateException("The selected media file has moved or was removed")
      documentId = child.documentId
    }
    return DocumentsContract.buildDocumentUriUsingTree(treeUri, documentId)
  }
}
