package app.ganbaru.mobile_documents

import java.io.ByteArrayInputStream
import java.nio.charset.MalformedInputException
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class MobileDocumentsPluginTest {
  @Test
  fun recognizesJsonFileNamesCaseInsensitively() {
    assertEquals(true, DocumentTextCodec.isJsonFileName("theme.JSON"))
    assertEquals(false, DocumentTextCodec.isJsonFileName("theme.json.txt"))
  }

  @Test
  fun readsBoundedUtf8() {
    val input = ByteArrayInputStream("Ganbaru".toByteArray())

    assertEquals("Ganbaru", DocumentTextCodec.readUtf8(input, 7))
  }

  @Test
  fun rejectsDocumentsAboveTheLimit() {
    val input = ByteArrayInputStream("Ganbaru".toByteArray())

    assertThrows(IllegalArgumentException::class.java) {
      DocumentTextCodec.readUtf8(input, 6)
    }
  }

  @Test
  fun rejectsMalformedUtf8() {
    val input = ByteArrayInputStream(byteArrayOf(0xC3.toByte(), 0x28))

    assertThrows(MalformedInputException::class.java) {
      DocumentTextCodec.readUtf8(input, 2)
    }
  }
}
