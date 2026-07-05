<script lang="ts">
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Save from "@lucide/svelte/icons/save";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { NotesLocalUser } from "$lib/notes/types";
  import {
    PROFILE_DISPLAY_NAME_FALLBACK,
    PROFILE_DISPLAY_NAME_MAX_CHARS,
    normalizeProfileDisplayName,
  } from "$lib/stores/preferences";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";

  const notes = getNotes();
  const preferences = getPreferences();
  const { t } = getLocalization();

  let draftDisplayName = $state(preferences.profileDisplayName);
  let initializedDisplayName = $state<string | null>(null);
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let saved = $state(false);

  const normalizedDisplayName = $derived(normalizeProfileDisplayName(draftDisplayName));
  const validationMessage = $derived.by(() => {
    if (normalizedDisplayName.ok) return null;
    if (normalizedDisplayName.reason === "too_long") {
      return t("settings.profileIdentity.nameTooLong", PROFILE_DISPLAY_NAME_MAX_CHARS);
    }
    return t("settings.profileIdentity.invalidName");
  });
  const unchanged = $derived(
    normalizedDisplayName.ok && normalizedDisplayName.value === preferences.profileDisplayName,
  );
  const canSave = $derived(normalizedDisplayName.ok && !unchanged && !saving);

  $effect(() => {
    const current = preferences.profileDisplayName;
    if (initializedDisplayName === current) return;
    initializedDisplayName = current;
    draftDisplayName = current;
  });

  async function saveIdentity(): Promise<void> {
    saveError = null;
    saved = false;
    const normalized = normalizeProfileDisplayName(draftDisplayName);
    if (!normalized.ok) {
      saveError = normalized.reason === "too_long"
        ? t("settings.profileIdentity.nameTooLong", PROFILE_DISPLAY_NAME_MAX_CHARS)
        : t("settings.profileIdentity.invalidName");
      return;
    }

    saving = true;
    const notesDisplayName = normalized.value || PROFILE_DISPLAY_NAME_FALLBACK;
    let updated: NotesLocalUser | null = null;
    try {
      updated = await notes.updateLocalUserDisplayName(notesDisplayName);
    } catch (error) {
      saveError = error instanceof Error ? error.message : String(error);
      return;
    } finally {
      saving = false;
    }
    if (!updated) {
      saveError = notes.localUserError ?? t("settings.profileIdentity.syncFailed");
      return;
    }
    if (!preferences.setProfileDisplayName(normalized.value)) {
      saveError = t("settings.profileIdentity.invalidName");
      return;
    }
    draftDisplayName = normalized.value;
    saved = true;
  }

  function handleDisplayNameKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      if (canSave) void saveIdentity();
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      draftDisplayName = preferences.profileDisplayName;
      saveError = null;
      saved = false;
    }
  }
</script>

<div class="flex min-h-full flex-col">
  <section class="flex flex-1 flex-col gap-4 pb-6">
    <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("settings.profileIdentity.heading")}</h2>

    <div class="flex flex-col gap-3">
      <div class="flex items-center justify-between gap-4 px-1 py-1 max-[520px]:flex-col max-[520px]:items-stretch max-[520px]:gap-2">
        <div class="min-w-0 flex-1">
          <label for="profile-display-name" class="text-[0.866667rem] text-foreground">
            {t("settings.profileIdentity.displayName")}
          </label>
          <div class="mt-0.5 text-[0.8rem] leading-5 text-muted-foreground">
            {t("settings.profileIdentity.description")}
          </div>
        </div>

        <input
          id="profile-display-name"
          class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors focus:border-ring disabled:opacity-60 dark:bg-transparent max-[520px]:w-full"
          value={draftDisplayName}
          maxlength={PROFILE_DISPLAY_NAME_MAX_CHARS}
          disabled={saving}
          onkeydown={handleDisplayNameKeydown}
          oninput={(event) => {
            draftDisplayName = event.currentTarget.value;
            saveError = null;
            saved = false;
          }}
        />
      </div>
    </div>
  </section>

  <footer class="sticky bottom-0 shrink-0 pt-3">
    <div class="flex flex-wrap items-center justify-between gap-2 border-t border-border/70 pt-3">
      <div class="min-w-0 flex-1 px-1 text-[0.8rem] leading-5">
        {#if saveError}
          <span role="alert" class="text-destructive">
            {t("settings.profileIdentity.saveFailed", saveError)}
          </span>
        {:else if validationMessage}
          <span role="alert" class="text-destructive">
            {validationMessage}
          </span>
        {/if}
      </div>
      <button
        type="button"
        onclick={() => void saveIdentity()}
        disabled={!canSave || saving}
        class="flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-md bg-primary px-3 text-[0.8rem] font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-55"
      >
        {#if saving}
          <LoaderCircle size={13} strokeWidth={2.25} class="shrink-0 animate-spin" />
          <span>{t("settings.profileIdentity.saving")}</span>
        {:else}
          <Save size={13} strokeWidth={2.25} class="shrink-0" />
          <span>{t("settings.profileIdentity.save")}</span>
        {/if}
      </button>
    </div>
  </footer>
</div>
