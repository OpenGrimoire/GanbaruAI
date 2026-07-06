<script lang="ts">
  import type { Component } from "svelte";
  import CloudDownload from "@lucide/svelte/icons/cloud-download";
  import DatabaseBackup from "@lucide/svelte/icons/database-backup";
  import FileText from "@lucide/svelte/icons/file-text";
  import Upload from "@lucide/svelte/icons/upload";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import ToggleSetting from "./ToggleSetting.svelte";
  import type { NotesTransferOperation } from "./types";

  let {
    onOpenTransferPanel = () => {},
  }: {
    onOpenTransferPanel?: (operation: NotesTransferOperation) => void;
  } = $props();

  const preferences = getPreferences();
  const { t } = getLocalization();

  interface TransferAction {
    id: NotesTransferOperation;
    label: () => string;
    description: () => string;
    buttonLabel: () => string;
    icon: Component;
  }

  const importActions: readonly TransferAction[] = [
    {
      id: "html-import",
      label: () => t("notes.htmlImportOpen"),
      description: () => t("settings.notesTransfers.htmlImportSummary"),
      buttonLabel: () => t("notes.htmlImportSubmit"),
      icon: Upload,
    },
    {
      id: "notion-api-import",
      label: () => t("notes.notionApiImportOpen"),
      description: () => t("settings.notesTransfers.notionApiImportSummary"),
      buttonLabel: () => t("notes.notionApiImportSubmit"),
      icon: CloudDownload,
    },
    {
      id: "notion-export-import",
      label: () => t("notes.notionExportImportOpen"),
      description: () => t("settings.notesTransfers.notionExportImportSummary"),
      buttonLabel: () => t("notes.notionExportImportSubmit"),
      icon: FileText,
    },
  ];

  const exportActions: readonly TransferAction[] = [
    {
      id: "json-graph-export",
      label: () => t("notes.jsonGraphExportOpen"),
      description: () => t("settings.notesTransfers.jsonGraphExportSummary"),
      buttonLabel: () => t("notes.jsonGraphExportSubmit"),
      icon: DatabaseBackup,
    },
  ];
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

  <div class="h-px shrink-0 scale-y-50 bg-border" aria-hidden="true"></div>

  <section class="flex flex-col gap-4">
    <div class="min-w-0 px-1">
      <h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.notesTransfers.importHeading")}</h2>
    </div>

    <div class="flex flex-col gap-3">
      {#each importActions as action (action.id)}
        {@const Icon = action.icon}
        <div class="flex items-center justify-between gap-4 px-1 py-1 max-[520px]:flex-col max-[520px]:items-stretch max-[520px]:gap-2">
          <div class="min-w-0 flex-1">
            <div class="text-[0.866667rem] text-foreground">{action.label()}</div>
            <div class="mt-0.5 text-[0.8rem] text-muted-foreground">{action.description()}</div>
          </div>
          <button
            type="button"
            class="flex h-7 shrink-0 items-center justify-center gap-1.5 rounded-md border border-border bg-card px-2.5 text-[0.8rem] font-medium text-foreground transition-colors hover:bg-accent dark:bg-transparent max-[520px]:w-fit"
            onclick={() => onOpenTransferPanel(action.id)}
          >
            <Icon size={13} strokeWidth={2.25} />
            <span>{action.buttonLabel()}</span>
          </button>
        </div>
      {/each}
    </div>
  </section>

  <div class="h-px shrink-0 scale-y-50 bg-border" aria-hidden="true"></div>

  <section class="flex flex-col gap-4">
    <div class="min-w-0 px-1">
      <h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.notesTransfers.exportHeading")}</h2>
    </div>

    <div class="flex flex-col gap-3">
      {#each exportActions as action (action.id)}
        {@const Icon = action.icon}
        <div class="flex items-center justify-between gap-4 px-1 py-1 max-[520px]:flex-col max-[520px]:items-stretch max-[520px]:gap-2">
          <div class="min-w-0 flex-1">
            <div class="text-[0.866667rem] text-foreground">{action.label()}</div>
            <div class="mt-0.5 text-[0.8rem] text-muted-foreground">{action.description()}</div>
          </div>
          <button
            type="button"
            class="flex h-7 shrink-0 items-center justify-center gap-1.5 rounded-md border border-border bg-card px-2.5 text-[0.8rem] font-medium text-foreground transition-colors hover:bg-accent dark:bg-transparent max-[520px]:w-fit"
            onclick={() => onOpenTransferPanel(action.id)}
          >
            <Icon size={13} strokeWidth={2.25} />
            <span>{action.buttonLabel()}</span>
          </button>
        </div>
      {/each}
    </div>
  </section>
</div>
