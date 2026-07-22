<script lang="ts">
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import type { ChatThreadShellRead } from "$lib/chat/contracts";
  import { filterArchivedThreads } from "$lib/chat/shell-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";

  let { onClose, onDelete }: { onClose: () => void; onDelete: (thread: ChatThreadShellRead) => void } = $props();
  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  let search = $state("");
  let projectId = $state("");
  let error = $state<string | null>(null);
  const threads = $derived(filterArchivedThreads(chat.archivedThreads, search, projectId));
  const projectOptions = $derived([
    { value: "", label: t("chat.archiveBrowser.allProjects") },
    ...projects.projects.map((project) => ({ value: project.id, label: project.name })),
  ]);

  function restore(thread: ChatThreadShellRead): void {
    error = null;
    void chat.restoreThread(thread).catch((cause: unknown) => {
      error = cause instanceof Error ? cause.message : String(cause);
    });
  }
</script>

<div class="absolute inset-0 z-30 flex flex-col bg-background" role="dialog" aria-modal="true" aria-label={t("chat.archiveBrowser.title")}>
  <header class="flex h-12 shrink-0 items-center gap-2 border-b border-border px-3"><h2 class="min-w-0 flex-1 truncate text-sm font-semibold">{t("chat.archiveBrowser.title")}</h2><button type="button" class="chat-icon-button" aria-label={t("chat.archiveBrowser.close")} onclick={onClose}><X size={16} /></button></header>
  <div class="grid gap-2 p-3"><label class="relative"><span class="sr-only">{t("chat.archiveBrowser.search")}</span><Search size={14} class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" /><input class="h-9 w-full rounded-md border border-border bg-background pl-8 pr-2 text-sm" type="search" bind:value={search} placeholder={t("chat.archiveBrowser.search")} /></label><CustomSelect inline class="w-full" value={projectId} options={projectOptions} ariaLabel={t("chat.archiveBrowser.allProjects")} onChange={(value) => { projectId = value; }} /></div>
  {#if error}<p role="alert" class="px-3 pb-2 text-xs text-destructive">{error}</p>{/if}
  <div class="min-h-0 flex-1 overflow-y-auto px-3 pb-3">
    {#if threads.length === 0}<p class="py-8 text-center text-sm text-muted-foreground">{t("chat.archiveBrowser.empty")}</p>{/if}
    <div class="flex flex-col gap-1">
      {#each threads as thread}
        <div class="flex items-center gap-2 rounded-md border border-border/70 p-2"><div class="min-w-0 flex-1"><span class="block truncate text-sm font-medium">{thread.title}</span><span class="block truncate text-xs text-muted-foreground">{chat.workspaces.find((entry) => entry.workspace.id === thread.workspaceId)?.workspace.displayName}</span></div><button type="button" class="chat-small-button" onclick={() => restore(thread)}>{t("chat.restore")}</button><button type="button" class="chat-small-button text-destructive" onclick={() => onDelete(thread)}>{t("chat.deletePermanently")}</button></div>
      {/each}
    </div>
  </div>
</div>
