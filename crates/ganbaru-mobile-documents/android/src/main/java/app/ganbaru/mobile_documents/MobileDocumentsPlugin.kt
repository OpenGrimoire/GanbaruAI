package app.ganbaru.mobile_documents

import android.app.Activity
import android.app.AlertDialog
import android.content.ComponentName
import android.content.ContentValues
import android.content.Intent
import android.net.Uri
import android.os.Environment
import android.provider.OpenableColumns
import android.provider.MediaStore
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.nio.ByteBuffer
import java.nio.charset.CodingErrorAction

private const val THEME_JSON_MIME_TYPE = "application/json"

@InvokeArg
internal class PickUtf8DocumentArgs {
  var maxBytes: Long = 0
}

@InvokeArg
internal class SaveUtf8DownloadArgs {
  lateinit var fileName: String
  lateinit var contents: String
  var maxBytes: Long = 0
}

internal object DocumentTextCodec {
  fun isJsonFileName(fileName: String): Boolean =
    fileName.endsWith(".json", ignoreCase = true)

  fun readUtf8(input: InputStream, maxBytes: Long): String {
    require(maxBytes > 0) { "Document size limit must be positive" }
    val output = ByteArrayOutputStream()
    val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
    var total = 0L

    input.use { stream ->
      while (true) {
        val count = stream.read(buffer)
        if (count < 0) break
        total += count
        if (total > maxBytes) {
          throw IllegalArgumentException("Selected document exceeds the size limit")
        }
        output.write(buffer, 0, count)
      }
    }

    return Charsets.UTF_8.newDecoder()
      .onMalformedInput(CodingErrorAction.REPORT)
      .onUnmappableCharacter(CodingErrorAction.REPORT)
      .decode(ByteBuffer.wrap(output.toByteArray()))
      .toString()
  }
}

@TauriPlugin
class MobileDocumentsPlugin(private val activity: Activity) : Plugin(activity) {
  private var pendingReadLimit: Long? = null

  @Command
  fun pickUtf8Document(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(PickUtf8DocumentArgs::class.java)
      require(args.maxBytes > 0) { "Document size limit must be positive" }
      pendingReadLimit = args.maxBytes

      val pickerIntent = Intent(Intent.ACTION_GET_CONTENT).apply {
        addCategory(Intent.CATEGORY_OPENABLE)
        type = THEME_JSON_MIME_TYPE
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
      }
      val providers = activity.packageManager
        .queryIntentActivities(pickerIntent, 0)
        .distinctBy { candidate ->
          candidate.activityInfo.packageName to candidate.activityInfo.name
        }
        .sortedBy { candidate ->
          candidate.loadLabel(activity.packageManager).toString().lowercase()
        }
      require(providers.isNotEmpty()) { "No document provider is installed" }

      if (providers.size == 1) {
        startProvider(invoke, pickerIntent, providers.single())
        return
      }

      val labels = providers
        .map { candidate -> candidate.loadLabel(activity.packageManager).toString() }
        .toTypedArray()
      AlertDialog.Builder(activity)
        .setItems(labels) { dialog, index ->
          try {
            startProvider(invoke, pickerIntent, providers[index])
          } catch (error: Exception) {
            pendingReadLimit = null
            invoke.reject(error.message ?: "Failed to open document provider")
          }
          dialog.dismiss()
        }
        .setOnCancelListener {
          pendingReadLimit = null
          invoke.resolve(JSObject().apply { put("contents", null) })
        }
        .show()
    } catch (error: Exception) {
      pendingReadLimit = null
      invoke.reject(error.message ?: "Failed to open document picker")
    }
  }

  private fun startProvider(
    invoke: Invoke,
    pickerIntent: Intent,
    provider: android.content.pm.ResolveInfo,
  ) {
    pickerIntent.component = ComponentName(
      provider.activityInfo.packageName,
      provider.activityInfo.name,
    )
    startActivityForResult(invoke, pickerIntent, "pickUtf8DocumentResult")
  }

  @ActivityCallback
  fun pickUtf8DocumentResult(invoke: Invoke, result: ActivityResult) {
    val maxBytes = pendingReadLimit
    pendingReadLimit = null

    if (result.resultCode == Activity.RESULT_CANCELED) {
      invoke.resolve(JSObject().apply { put("contents", null) })
      return
    }
    if (result.resultCode != Activity.RESULT_OK || maxBytes == null) {
      invoke.reject("Failed to select document")
      return
    }
    val uri = result.data?.data
    if (uri == null) {
      invoke.reject("The selected document is unavailable")
      return
    }

    Thread {
      try {
        val displayName = resolveOpenableDisplayName(uri)
          ?: throw IllegalArgumentException("The selected document has no file name")
        require(DocumentTextCodec.isJsonFileName(displayName)) {
          "Select a JSON theme file"
        }
        val stream = activity.contentResolver.openInputStream(uri)
          ?: throw IllegalStateException("The selected document could not be opened")
        val contents = DocumentTextCodec.readUtf8(stream, maxBytes)
        invoke.resolve(JSObject().apply { put("contents", contents) })
      } catch (error: Exception) {
        invoke.reject(error.message ?: "Failed to read selected document")
      }
    }.start()
  }

  @Command
  fun saveUtf8Download(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(SaveUtf8DownloadArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Invalid download request")
      return
    }

    Thread {
      var uri: Uri? = null
      try {
        val bytes = args.contents.toByteArray(Charsets.UTF_8)
        require(args.maxBytes > 0) { "Document size limit must be positive" }
        require(bytes.size.toLong() <= args.maxBytes) { "Theme export exceeds the size limit" }
        require(args.fileName.isNotBlank()) { "Download file name is required" }
        require(args.fileName.length <= 255) { "Download file name is too long" }
        require('/' !in args.fileName && '\\' !in args.fileName) {
          "Download file name must not contain path separators"
        }
        require(DocumentTextCodec.isJsonFileName(args.fileName)) {
          "Theme download must use the JSON extension"
        }

        val pendingValues = ContentValues().apply {
          put(MediaStore.MediaColumns.DISPLAY_NAME, args.fileName)
          put(MediaStore.MediaColumns.MIME_TYPE, THEME_JSON_MIME_TYPE)
          put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS)
          put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val createdUri = activity.contentResolver.insert(
          MediaStore.Downloads.EXTERNAL_CONTENT_URI,
          pendingValues,
        ) ?: throw IllegalStateException("Android could not create the download")
        uri = createdUri

        activity.contentResolver.openOutputStream(createdUri, "w").use { output ->
          requireNotNull(output) { "Android could not open the download" }
          output.write(bytes)
          output.flush()
        }

        val published = ContentValues().apply {
          put(MediaStore.MediaColumns.IS_PENDING, 0)
        }
        check(activity.contentResolver.update(createdUri, published, null, null) == 1) {
          "Android could not publish the download"
        }

        val displayName = resolveDisplayName(createdUri) ?: args.fileName
        invoke.resolve(JSObject().apply { put("displayName", displayName) })
      } catch (error: Exception) {
        uri?.let { activity.contentResolver.delete(it, null, null) }
        invoke.reject(error.message ?: "Failed to save download")
      }
    }.start()
  }

  private fun resolveDisplayName(uri: Uri): String? {
    val projection = arrayOf(MediaStore.MediaColumns.DISPLAY_NAME)
    return activity.contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
      if (!cursor.moveToFirst()) return@use null
      cursor.getString(0)
    }
  }

  private fun resolveOpenableDisplayName(uri: Uri): String? {
    val projection = arrayOf(OpenableColumns.DISPLAY_NAME)
    return activity.contentResolver.query(uri, projection, null, null, null)?.use { cursor ->
      if (!cursor.moveToFirst()) return@use null
      cursor.getString(0)
    }
  }
}
