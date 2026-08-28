package app.ganbaru.mobile_notifications

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
}
