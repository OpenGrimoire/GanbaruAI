<script lang="ts">
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import Archive from "@lucide/svelte/icons/archive";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FolderGit2 from "@lucide/svelte/icons/folder-git-2";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import HardDrive from "@lucide/svelte/icons/hard-drive";
  import Link2 from "@lucide/svelte/icons/link-2";
  import Pencil from "@lucide/svelte/icons/pencil";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { ProjectWorkingFolderRead } from "$lib/chat/contracts";
  import { getChat } from "$lib/stores/chat.svelte";
  import { orderProjectWorkingFolders } from "$lib/projects/working-folder-order";
  import { cn } from "$lib/utils";
  import ProjectSettingsSectionHeading from "./ProjectSettingsSectionHeading.svelte";

  let { projectId }: { projectId: string } = $props();
  const chat = getChat();
  const { t } = getLocalization();
  let operationId = $state<string | null>(null);
  let error = $state<string | null>(null);
  const folders = $derived(orderProjectWorkingFolders(
    chat.workingFolders.filter((entry) => entry.workingFolder.projectId === projectId),
  ));
  const providers = $derived(chat.settings?.providerInstances ?? []);

  $effect(() => {
    const selectedProjectId = projectId;
    void selectedProjectId;
    void chat.ensureLoaded().catch((reason) => {
      error = errorMessage(reason);
    });
  });

  async function run(id: string, operation: () => Promise<void>): Promise<void> {
    operationId = id;
    error = null;
    try {
      await operation();
    } catch (reason) {
      error = errorMessage(reason);
    } finally {
      operationId = null;
    }
  }

  function statusLabel(folder: ProjectWorkingFolderRead): string {
    if (folder.bindingStatus === "available") return t("projects.settings.workingFolders.statusAvailable");
    if (folder.bindingStatus === "missing") return t("projects.settings.workingFolders.statusMissing");
    if (folder.bindingStatus === "repository_mismatch") return t("projects.settings.workingFolders.statusRepositoryMismatch");
    return t("projects.settings.workingFolders.statusUnbound");
  }

  function statusClass(folder: ProjectWorkingFolderRead): string {
    if (folder.bindingStatus === "available") return "text-success";
    if (folder.bindingStatus === "repository_mismatch") return "text-destructive";
    return "text-warning";
  }

  function providerPreference(folder: ProjectWorkingFolderRead): string {
    return chat.settings?.configuration.workingFolderProviderPreferences[folder.workingFolder.id] ?? "";
  }

  function pathLabel(folder: ProjectWorkingFolderRead): string {
    return folder.workingFolder.managedRelativePath
      ?? folder.canonicalPath
      ?? t("projects.settings.workingFolders.noLocalPath");
  }

  function rename(folder: ProjectWorkingFolderRead): void {
    const name = window.prompt(
      t("projects.settings.workingFolders.renamePrompt"),
      folder.workingFolder.displayName,
    );
    if (!name?.trim() || name.trim() === folder.workingFolder.displayName) return;
    void run(folder.workingFolder.id, () => chat.renameWorkingFolder(folder.workingFolder.id, name.trim()));
  }

  function remove(folder: ProjectWorkingFolderRead): void {
    if (!window.confirm(t("projects.settings.workingFolders.removeConfirm", folder.workingFolder.displayName))) return;
    void run(folder.workingFolder.id, () => chat.removeWorkingFolder(folder.workingFolder.id));
  }

  function errorMessage(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<section class="flex flex-col gap-2" data-project-working-folders>
  <ProjectSettingsSectionHeading label={t("projects.settings.workingFolders.title")} />
  <p class="px-1 text-[0.75rem] leading-5 text-muted-foreground">
    {t("projects.settings.workingFolders.description")}
  </p>
  {#if error}
    <div class="rounded-md border border-destructive/30 px-2 py-1.5 text-[0.75rem] text-destructive" role="alert">{error}</div>
  {/if}
  <div class="flex flex-col gap-2">
    {#each folders as folder (folder.workingFolder.id)}
      {@const busy = operationId === folder.workingFolder.id}
      <article class="rounded-lg border border-border bg-card/40 p-2.5">
        <div class="flex min-w-0 items-start gap-2">
          {#if folder.workingFolder.kind === "managed"}
            <HardDrive class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
          {:else}
            <FolderGit2 class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
          {/if}
          <div class="min-w-0 flex-1">
            <div class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
              <strong class="truncate text-[0.82rem]">{folder.workingFolder.displayName}</strong>
              <span class="rounded border border-border px-1.5 py-0.5 text-[0.65rem] text-muted-foreground">
                {folder.workingFolder.kind === "managed"
                  ? t("projects.settings.workingFolders.managed")
                  : t("projects.settings.workingFolders.external")}
              </span>
              {#if folder.workingFolder.archivedAt}
                <span class="rounded border border-border px-1.5 py-0.5 text-[0.65rem] text-muted-foreground">{t("projects.settings.workingFolders.archived")}</span>
              {/if}
            </div>
            <p class="mt-1 truncate font-mono text-[0.68rem] text-muted-foreground" title={pathLabel(folder)}>{pathLabel(folder)}</p>
            <div class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-[0.68rem] text-muted-foreground">
              <span class={cn("font-medium", statusClass(folder))}>{statusLabel(folder)}</span>
              <span>{folder.workingFolder.repositoryKind === "git"
                ? folder.currentBranch
                  ? t("projects.settings.workingFolders.gitBranch", folder.currentBranch)
                  : t("projects.settings.workingFolders.gitRepository")
                : t("projects.settings.workingFolders.noRepository")}</span>
              {#if folder.lastVerifiedAt}<span>{t("projects.settings.workingFolders.verified")}</span>{/if}
            </div>
          </div>
        </div>
        <div class="mt-2 flex flex-wrap items-center gap-1.5">
          <label class="mr-auto flex min-w-44 items-center gap-1.5 text-[0.7rem] text-muted-foreground">
            <span>{t("projects.settings.workingFolders.provider")}</span>
            <select
              class="h-7 min-w-0 flex-1 rounded border border-border bg-background px-1.5 text-[0.7rem] text-foreground"
              value={providerPreference(folder)}
              disabled={busy}
              onchange={(event) => {
                const value = event.currentTarget.value;
                void run(folder.workingFolder.id, () => chat.setWorkingFolderProviderPreference(folder.workingFolder.id, value || null));
              }}
            >
              <option value="">{t("projects.settings.workingFolders.providerAutomatic")}</option>
              {#each providers as provider (provider.configuration.instanceId)}
                <option value={provider.configuration.instanceId}>{provider.configuration.label}</option>
              {/each}
            </select>
          </label>
          {#if folder.bindingStatus === "available"}
            <button type="button" class="working-folder-action" disabled={busy} onclick={() => { void run(folder.workingFolder.id, () => chat.openWorkingFolder(folder.workingFolder.id)); }}><ExternalLink class="size-3.5" />{t("projects.settings.workingFolders.open")}</button>
          {/if}
          {#if folder.workingFolder.kind === "managed" && folder.bindingStatus !== "available"}
            <button type="button" class="working-folder-action" disabled={busy} onclick={() => { void run(folder.workingFolder.id, () => chat.recreateManagedWorkingFolder(folder.workingFolder.id)); }}><RefreshCw class="size-3.5" />{t("projects.settings.workingFolders.recreate")}</button>
          {:else if folder.workingFolder.kind === "external" && !folder.workingFolder.archivedAt}
            <button type="button" class="working-folder-action" disabled={busy} onclick={() => { void run(folder.workingFolder.id, () => folder.bindingStatus === "unbound" || folder.bindingStatus === "missing" ? chat.locateWorkingFolder(folder.workingFolder.id, t("projects.settings.workingFolders.pickerTitle")) : chat.rebindWorkingFolder(folder.workingFolder.id, t("projects.settings.workingFolders.pickerTitle"))); }}><Link2 class="size-3.5" />{folder.bindingStatus === "unbound" || folder.bindingStatus === "missing" ? t("projects.settings.workingFolders.locate") : t("projects.settings.workingFolders.rebind")}</button>
          {/if}
          {#if folder.workingFolder.kind === "external"}
            <button type="button" class="working-folder-action" disabled={busy} onclick={() => rename(folder)}><Pencil class="size-3.5" />{t("projects.settings.workingFolders.rename")}</button>
            {#if folder.workingFolder.archivedAt}
              <button type="button" class="working-folder-action" disabled={busy} onclick={() => { void run(folder.workingFolder.id, () => chat.restoreWorkingFolder(folder.workingFolder.id)); }}><ArchiveRestore class="size-3.5" />{t("projects.settings.workingFolders.restore")}</button>
            {:else}
              <button type="button" class="working-folder-action" disabled={busy} onclick={() => { void run(folder.workingFolder.id, () => chat.archiveWorkingFolder(folder.workingFolder.id)); }}><Archive class="size-3.5" />{t("projects.settings.workingFolders.archive")}</button>
            {/if}
            <button type="button" class="working-folder-action text-destructive" disabled={busy} onclick={() => remove(folder)}><Trash2 class="size-3.5" />{t("projects.settings.workingFolders.remove")}</button>
          {/if}
        </div>
      </article>
    {/each}
  </div>
  <button
    type="button"
    class="mt-1 flex min-h-8 items-center justify-center gap-1.5 rounded-md border border-dashed border-border text-[0.78rem] text-muted-foreground hover:bg-accent hover:text-foreground"
    disabled={operationId !== null || chat.loading}
    onclick={() => {
      void run("add", async () => {
        await chat.addExternalWorkingFolder({
          id: `working-folder:${crypto.randomUUID()}`,
          projectId,
          displayName: "",
        }, t("projects.settings.workingFolders.pickerTitle"));
      });
    }}
  >
    <FolderPlus class="size-4" />{t("projects.settings.workingFolders.addExisting")}
  </button>
</section>

<style>
  .working-folder-action {
    display: inline-flex;
    min-height: 1.75rem;
    align-items: center;
    gap: 0.25rem;
    border-radius: 0.375rem;
    padding-inline: 0.45rem;
    font-size: 0.7rem;
    color: var(--muted-foreground);
  }

  .working-folder-action:hover:not(:disabled) {
    background: var(--accent);
    color: var(--foreground);
  }

  .working-folder-action:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
