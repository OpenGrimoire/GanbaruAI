<script lang="ts">
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Save from "@lucide/svelte/icons/save";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import ToggleSetting from "./ToggleSetting.svelte";

  const notes = getNotes();
  const preferences = getPreferences();
  const { t } = getLocalization();

  const actionButtonClass =
    "inline-flex h-7 shrink-0 items-center justify-center gap-1.5 rounded-md border border-border px-2.5 text-[0.8rem] font-medium transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-55";
  const primaryButtonClass = `${actionButtonClass} border-primary bg-primary text-primary-foreground hover:bg-primary/90`;

  let draftDisplayName = $state("");
  let initializedUserId = $state<string | null>(null);
  let requestedLoad = $state(false);
  let saveError = $state<string | null>(null);
  let saved = $state(false);

  const trimmedDisplayName = $derived(draftDisplayName.trim());
  const unchanged = $derived(trimmedDisplayName === (notes.localUser?.display_name ?? ""));
  const canSave = $derived(
    trimmedDisplayName.length > 0 && !unchanged && !notes.localUserLoading,
  );

  $effect(() => {
    if (requestedLoad || notes.localUser || notes.localUserLoading) return;
    requestedLoad = true;
    void notes.loadLocalUser();
  });

  $effect(() => {
    if (!notes.localUser || initializedUserId === notes.localUser.id) return;
    initializedUserId = notes.localUser.id;
    draftDisplayName = notes.localUser.display_name;
  });

  async function saveIdentity(): Promise<void> {
    saveError = null;
    saved = false;
    if (!trimmedDisplayName) {
      saveError = t("settings.notesIdentity.emptyName");
      return;
    }
    const updated = await notes.updateLocalUserDisplayName(trimmedDisplayName);
    if (!updated) {
      saveError = notes.localUserError ?? t("settings.notesIdentity.emptyName");
      return;
    }
    draftDisplayName = updated.display_name;
    saved = true;
  }
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-4">
    <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("settings.notesIdentity.heading")}</h2>

    <div class="flex flex-col gap-3">
      <div class="flex items-start justify-between gap-4 px-1 py-1 max-[520px]:flex-col max-[520px]:items-stretch max-[520px]:gap-2">
        <div class="min-w-0 flex-1">
          <label for="notes-display-name" class="text-[0.866667rem] text-foreground">
            {t("settings.notesIdentity.displayName")}
          </label>
          <div class="mt-0.5 text-[0.8rem] leading-5 text-muted-foreground">
            {t("settings.notesIdentity.description")}
          </div>
        </div>

        <div class="flex w-80 max-w-full shrink-0 items-center gap-2 max-[520px]:w-full">
          <input
            id="notes-display-name"
            class="h-8 min-w-0 flex-1 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
            value={draftDisplayName}
            maxlength="80"
            disabled={notes.localUserLoading && !notes.localUser}
            oninput={(event) => {
              draftDisplayName = event.currentTarget.value;
              saveError = null;
              saved = false;
            }}
          />

          <button
            type="button"
            class={primaryButtonClass}
            disabled={!canSave}
            onclick={() => void saveIdentity()}
          >
            {#if notes.localUserLoading && notes.localUser}
              <LoaderCircle size={14} strokeWidth={2.1} class="shrink-0 animate-spin" />
              <span>{t("settings.notesIdentity.saving")}</span>
            {:else}
              <Save size={14} strokeWidth={1.9} class="shrink-0" />
              <span>{t("settings.notesIdentity.save")}</span>
            {/if}
          </button>
        </div>
      </div>

      {#if notes.localUserLoading && !notes.localUser}
        <div class="flex items-center gap-1.5 px-1 text-[0.8rem] leading-5 text-muted-foreground">
          <LoaderCircle size={13} strokeWidth={2.1} class="shrink-0 animate-spin" />
          <span>{t("settings.notesIdentity.loading")}</span>
        </div>
      {/if}

      {#if notes.localUserError && !notes.localUser}
        <div role="alert" class="px-1 text-[0.8rem] leading-5 text-destructive">
          {t("settings.notesIdentity.loadFailed", notes.localUserError)}
        </div>
      {/if}

      {#if saveError}
        <div role="alert" class="px-1 text-[0.8rem] leading-5 text-destructive">
          {t("settings.notesIdentity.saveFailed", saveError)}
        </div>
      {:else if saved}
        <div class="px-1 text-[0.8rem] leading-5 text-muted-foreground">
          {t("settings.notesIdentity.saved")}
        </div>
      {/if}
    </div>
  </section>

  <div class="h-px bg-border/70" aria-hidden="true"></div>

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
