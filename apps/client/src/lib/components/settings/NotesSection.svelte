<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import ToggleSetting from "./ToggleSetting.svelte";

  const preferences = getPreferences();
  const { t } = getLocalization();
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-4">
    <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("settings.notesNotifications.heading")}</h2>

    <div class="flex flex-col gap-3">
      <ToggleSetting
        label={t("settings.notesNotifications.enable")}
        description={t("settings.notesNotifications.enableDescription")}
        checked={preferences.notesMentionNotificationsEnabled}
        onChange={preferences.setNotesMentionNotificationsEnabled}
      />
      <ToggleSetting
        label={t("settings.notesNotifications.reminders")}
        description={t("settings.notesNotifications.remindersDescription")}
        checked={preferences.notesReminderNotificationsEnabled}
        disabled={!preferences.notesMentionNotificationsEnabled}
        onChange={preferences.setNotesReminderNotificationsEnabled}
      />
      <ToggleSetting
        label={t("settings.notesNotifications.userMentions")}
        description={t("settings.notesNotifications.userMentionsDescription")}
        checked={preferences.notesUserMentionNotificationsEnabled}
        disabled={!preferences.notesMentionNotificationsEnabled}
        onChange={preferences.setNotesUserMentionNotificationsEnabled}
      />
      <ToggleSetting
        label={t("settings.notesNotifications.taskMentions")}
        description={t("settings.notesNotifications.taskMentionsDescription")}
        checked={preferences.notesTaskMentionNotificationsEnabled}
        disabled={!preferences.notesMentionNotificationsEnabled}
        onChange={preferences.setNotesTaskMentionNotificationsEnabled}
      />
      <ToggleSetting
        label={t("settings.notesNotifications.includeContent")}
        description={t("settings.notesNotifications.includeContentDescription")}
        checked={preferences.notesNotificationIncludeContent}
        disabled={!preferences.notesMentionNotificationsEnabled}
        onChange={preferences.setNotesNotificationIncludeContent}
      />
    </div>
  </section>
</div>
