<script lang="ts">
  import { onMount } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import CircleCheck from "@lucide/svelte/icons/circle-check";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderX from "@lucide/svelte/icons/folder-x";
  import Link2 from "@lucide/svelte/icons/link-2";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Unlink from "@lucide/svelte/icons/unlink";
  import type { ChatWorkspaceRead } from "$lib/chat/contracts";
  import { providerInstanceIdFromLabel } from "$lib/chat/provider-setup";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import CustomSelect from "../CustomSelect.svelte";

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const projects = getProjects();
  let creationMode = $state<"project" | "standalone" | null>(null);
  let workspaceName = $state("");
  let projectId = $state<string | null>(null);
  let busyId = $state<string | null>(null);
  let error = $state<string | null>(null);
  let renameId = $state<string | null>(null);
  let renameValue = $state("");
  let mismatchWorkspace = $state<ChatWorkspaceRead | null>(null);
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const projectOptions = $derived([
    { value: "", label: t("settings.chat.workspaces.chooseProject") },
    ...projects.projects
      .filter((project) => project.status === "active")
      .map((project) => ({ value: project.id, label: project.name })),
  ]);
  const providerOptions = $derived([
    { value: "", label: t("settings.chat.workspaces.noPreference") },
    ...providers.map((provider) => ({
      value: provider.configuration.instanceId,
      label: provider.configuration.label,
    })),
  ]);
  const orderedWorkspaces = $derived.by(() => {
    const collator = new Intl.Collator(localization.locale);
    return [...chat.workspaces].sort((left, right) => {
      const leftProject = left.workspace.projectId ? projects.projectById(left.workspace.projectId)?.name ?? "" : "zzzz";
      const rightProject = right.workspace.projectId ? projects.projectById(right.workspace.projectId)?.name ?? "" : "zzzz";
      return collator.compare(leftProject, rightProject) || collator.compare(left.workspace.displayName, right.workspace.displayName);
    });
  });
  const workspaceGroups = $derived.by(() => {
    const grouped = new Map<string, { id: string; label: string; workspaces: ChatWorkspaceRead[] }>();
    for (const workspace of orderedWorkspaces) {
      const id = workspace.workspace.projectId ?? "standalone";
      const label = workspace.workspace.projectId
        ? projects.projectById(workspace.workspace.projectId)?.name ?? workspace.workspace.projectId
        : t("chat.standalone");
      const group = grouped.get(id) ?? { id, label, workspaces: [] };
      group.workspaces.push(workspace);
      grouped.set(id, group);
    }
    return [...grouped.values()];
  });

  onMount(() => {
    void projects.ensureLoaded().catch((cause: unknown) => { error = errorMessage(cause); });
  });

  function startCreate(mode: "project" | "standalone"): void {
    creationMode = mode;
    workspaceName = "";
    projectId = mode === "project" ? projects.projects.find((project) => project.status === "active")?.id ?? null : null;
  }

  async function create(): Promise<void> {
    const name = workspaceName.trim();
    if (!name || (creationMode === "project" && !projectId)) return;
    error = null;
    try {
      await chat.createWorkspace({
        id: `workspace-${providerInstanceIdFromLabel(name)}-${crypto.randomUUID()}`,
        projectId: creationMode === "project" ? projectId : null,
        displayName: name,
      }, t("settings.chat.setup.chooseWorkspaceFolder"));
      creationMode = null;
    } catch (cause: unknown) {
      error = errorMessage(cause);
    }
  }

  async function perform(workspaceId: string, action: () => Promise<void>): Promise<void> {
    busyId = workspaceId;
    error = null;
    try {
      await action();
    } catch (cause: unknown) {
      error = errorMessage(cause);
      mismatchWorkspace = error.includes("different repository")
        ? chat.workspaces.find((entry) => entry.workspace.id === workspaceId) ?? null
        : null;
    } finally {
      busyId = null;
    }
  }

  function createForMismatch(): void {
    const source = mismatchWorkspace;
    if (!source) return;
    creationMode = source.workspace.projectId ? "project" : "standalone";
    projectId = source.workspace.projectId;
    workspaceName = "";
    mismatchWorkspace = null;
  }

  async function commitRename(workspace: ChatWorkspaceRead): Promise<void> {
    const value = renameValue.trim();
    if (!value) return;
    await perform(workspace.workspace.id, () => chat.renameWorkspace(workspace.workspace.id, value));
    renameId = null;
  }

  function bindingLabel(workspace: ChatWorkspaceRead): string {
    if (workspace.bindingStatus === "available") return t("settings.chat.workspaces.available");
    if (workspace.bindingStatus === "missing") return t("settings.chat.workspaces.missing");
    if (workspace.bindingStatus === "repository_mismatch") return t("settings.chat.workspaces.mismatch");
    return t("settings.chat.workspaces.unbound");
  }

  function errorMessage(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }
</script>

<section class="flex flex-col gap-4">
  <div class="flex flex-wrap items-start justify-between gap-3">
    <div class="min-w-0 px-1"><h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.workspaces.heading")}</h2><p class="mt-1 max-w-2xl text-[0.8rem] text-muted-foreground">{t("settings.chat.workspaces.description")}</p></div>
    <div class="flex flex-wrap gap-2"><button type="button" class="chat-settings-button" onclick={() => startCreate("project")}><Plus size={13} />{t("settings.chat.workspaces.addProject")}</button><button type="button" class="chat-settings-button" onclick={() => startCreate("standalone")}><Plus size={13} />{t("settings.chat.workspaces.addStandalone")}</button></div>
  </div>

  {#if creationMode}
    <div class="grid gap-3 rounded-lg border border-border bg-card/40 p-3 sm:grid-cols-2">
      <label class="chat-inline-field"><span>{t("settings.chat.workspaces.name")}</span><input bind:value={workspaceName} /></label>
      {#if creationMode === "project"}
        <div class="flex flex-col gap-1.5 text-[0.733333rem] font-medium text-muted-foreground">
          <span>{t("settings.chat.workspaces.project")}</span>
          <CustomSelect
            inline
            class="w-full"
            value={projectId ?? ""}
            options={projectOptions}
            onChange={(value) => { projectId = value || null; }}
            ariaLabel={t("settings.chat.workspaces.project")}
          />
        </div>
      {/if}
      <div class="flex gap-2 sm:col-span-2"><button type="button" class="setup-secondary-button" onclick={() => { creationMode = null; }}>{t("chat.cancel")}</button><button type="button" class="setup-primary-button" onclick={() => void create()}>{t("settings.chat.workspaces.createAndLocate")}</button></div>
    </div>
  {/if}
  {#if error}<div role="alert" class="flex flex-wrap items-center gap-2 text-sm text-destructive"><span>{error}</span>{#if mismatchWorkspace}<button type="button" class="chat-settings-button" onclick={createForMismatch}>{t("settings.chat.workspaces.createForMismatch")}</button>{/if}</div>{/if}

  {#if orderedWorkspaces.length === 0}
    <p class="rounded-lg border border-dashed border-border p-4 text-sm text-muted-foreground">{t("settings.chat.workspaces.empty")}</p>
  {:else}
    <div class="flex flex-col gap-5">
      {#each workspaceGroups as group (group.id)}
        <section class="flex flex-col gap-2">
          <h3 class="px-1 text-[0.733333rem] font-semibold text-muted-foreground">{group.label}</h3>
      {#each group.workspaces as workspace (workspace.workspace.id)}
        {@const project = workspace.workspace.projectId ? projects.projectById(workspace.workspace.projectId) : null}
        {@const preference = chat.settings?.configuration.workspaceProviderPreferences[workspace.workspace.id] ?? ""}
        <article class="overflow-hidden rounded-lg border border-border bg-card/40 {workspace.workspace.archivedAt ? 'opacity-70' : ''}">
          <div class="flex min-w-0 items-start gap-3 p-3">
            <div class="grid size-8 shrink-0 place-items-center rounded-md border border-border bg-background text-muted-foreground"><FolderOpen size={15} /></div>
            <div class="min-w-0 flex-1">
              <div class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
                {#if renameId === workspace.workspace.id}
                  <div class="flex max-w-sm flex-1 gap-2"><input class="h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-sm" bind:value={renameValue} onkeydown={(event) => { if (event.key === "Enter") void commitRename(workspace); if (event.key === "Escape") renameId = null; }} /><button type="button" class="chat-settings-button" onclick={() => void commitRename(workspace)}>{t("chat.save")}</button></div>
                {:else}
                  <h3 class="truncate text-[0.866667rem] font-semibold text-foreground">{workspace.workspace.displayName}</h3>
                {/if}
                <span class="inline-flex items-center gap-1 text-[0.7rem] {workspace.bindingStatus === 'available' ? 'text-action-confirm' : 'text-status-tentative'}">
                  {#if workspace.bindingStatus === "available"}<CircleCheck size={11} />{:else}<FolderX size={11} />{/if}
                  {bindingLabel(workspace)}
                </span>
              </div>
              <dl class="mt-1.5 grid min-w-0 gap-x-3 gap-y-1 text-[0.733333rem] sm:grid-cols-[7.5rem_minmax(0,1fr)]">
                <dt class="text-muted-foreground">{t("settings.chat.workspaces.project")}</dt>
                <dd class="truncate text-foreground">{project?.name ?? t("chat.standalone")}</dd>
                <dt class="text-muted-foreground">{t("settings.chat.workspaces.localFolder")}</dt>
                <dd class="truncate text-foreground" title={workspace.canonicalPath ?? bindingLabel(workspace)}>{workspace.canonicalPath ?? bindingLabel(workspace)}</dd>
                {#if workspace.workspace.repositoryIdentity}
                  <dt class="text-muted-foreground">{t("settings.chat.workspaces.repository")}</dt>
                  <dd class="truncate text-foreground">{workspace.workspace.repositoryIdentity}</dd>
                {/if}
              </dl>
              {#if workspace.lastVerifiedAt}<div class="mt-1.5 text-[0.666667rem] text-muted-foreground">{t("settings.chat.workspaces.lastVerified", formatDateTime(localization.locale, new Date(workspace.lastVerifiedAt), { dateStyle: "medium", timeStyle: "short" }))}</div>{/if}
            </div>
          </div>
          <div class="border-t border-border/60 px-2 py-2">
            <CustomSelect
              label={t("settings.chat.workspaces.providerPreference")}
              description={t("settings.chat.workspaces.providerPreferenceDescription")}
              value={preference}
              options={providerOptions}
              onChange={(value) => void perform(workspace.workspace.id, () => chat.setWorkspaceProviderPreference(workspace.workspace.id, value || null))}
            />
          </div>
          <div class="flex flex-wrap gap-1 border-t border-border/60 px-3 py-2">
            {#if workspace.bindingStatus === "unbound"}<button type="button" class="chat-settings-button" disabled={busyId === workspace.workspace.id} onclick={() => void perform(workspace.workspace.id, () => chat.bindWorkspace(workspace.workspace.id, t("settings.chat.setup.chooseWorkspaceFolder")))}><Link2 size={13} />{t("settings.chat.workspaces.locate")}</button>{:else}<button type="button" class="chat-settings-button" disabled={busyId === workspace.workspace.id} onclick={() => void perform(workspace.workspace.id, () => chat.rebindWorkspace(workspace.workspace.id, t("settings.chat.setup.chooseWorkspaceFolder")))}><Link2 size={13} />{t("settings.chat.workspaces.rebind")}</button>{/if}
            {#if workspace.bindingStatus === "available"}<button type="button" class="chat-settings-button" onclick={() => void perform(workspace.workspace.id, () => chat.openWorkspaceFolder(workspace.workspace.id))}><FolderOpen size={13} />{t("settings.chat.workspaces.openFolder")}</button><button type="button" class="chat-settings-button" onclick={() => void perform(workspace.workspace.id, () => chat.removeWorkspaceBinding(workspace.workspace.id))}><Unlink size={13} />{t("settings.chat.workspaces.removeBinding")}</button>{/if}
            <button type="button" class="chat-settings-button" onclick={() => { renameId = workspace.workspace.id; renameValue = workspace.workspace.displayName; }}><Pencil size={13} />{t("settings.chat.workspaces.rename")}</button>
            {#if workspace.workspace.archivedAt}<button type="button" class="chat-settings-button" onclick={() => void perform(workspace.workspace.id, () => chat.restoreWorkspace(workspace.workspace.id))}><RotateCcw size={13} />{t("settings.chat.workspaces.restore")}</button>{:else}<button type="button" class="chat-settings-button" onclick={() => void perform(workspace.workspace.id, () => chat.archiveWorkspace(workspace.workspace.id))}><Archive size={13} />{t("settings.chat.workspaces.archive")}</button>{/if}
          </div>
        </article>
      {/each}
        </section>
      {/each}
    </div>
  {/if}
</section>

<style>
  :global(.chat-inline-field) { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.733333rem; color: var(--muted-foreground); }
  :global(.chat-inline-field input) { min-height: 2rem; border-radius: 0.375rem; border: 1px solid var(--border); background: var(--background); padding: 0.25rem 0.5rem; color: var(--foreground); }
</style>
