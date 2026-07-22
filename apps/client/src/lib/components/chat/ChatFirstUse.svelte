<script lang="ts">
  import Folder from "@lucide/svelte/icons/folder";
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import Laptop from "@lucide/svelte/icons/laptop";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Settings from "@lucide/svelte/icons/settings";
  import { resolveChatFirstUseState } from "$lib/chat/shell-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import ChatComposer from "./ChatComposer.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  const settings = getSettingsLauncher();
  let operationError = $state<string | null>(null);
  const firstUse = $derived(resolveChatFirstUseState({ providers: chat.settings?.providerInstances ?? [], workspaces: chat.workspaces, selectedWorkspaceId: chat.selectedWorkspaceId, selectedThreadId: chat.selectedThreadId, threads: [...chat.activeThreads, ...chat.archivedThreads] }));
  const workspace = $derived(chat.selectedWorkspace);

  function run(action: () => Promise<unknown>): void {
    operationError = null;
    void action().catch((error: unknown) => {
      operationError = error instanceof Error ? error.message : String(error);
    });
  }

  function bindingLabel(status: "available" | "unbound" | "missing" | "repository_mismatch"): string {
    switch (status) {
      case "available": return t("settings.chat.workspaces.available");
      case "missing": return t("settings.chat.workspaces.missing");
      case "repository_mismatch": return t("settings.chat.workspaces.mismatch");
      default: return t("settings.chat.workspaces.unbound");
    }
  }
</script>

<div class="first-use-shell" class:new-thread={firstUse.kind === "no_thread"}>
  <section class="first-use-content" class:new-thread={firstUse.kind === "no_thread"}>
    {#if operationError}<p role="alert" class="mb-3 text-sm text-destructive">{operationError}</p>{/if}
    {#if firstUse.kind !== "no_thread"}<div class="mb-4 flex size-12 items-center justify-center rounded-2xl border border-border bg-card"><MessageSquare size={22} /></div>{/if}
    {#if firstUse.kind === "no_provider"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.noProviderTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.noProviderDescription")}</p><div class="mt-5 flex flex-wrap justify-center gap-2"><button type="button" class="chat-primary-button" onclick={() => settings.open("chat", { chatSubsection: "providers" })}><Settings size={15} />{t("chat.firstUse.setUpProvider")}</button></div><div class="mt-5 flex flex-wrap justify-center gap-2 text-xs text-muted-foreground"><span>Codex</span><span>Claude</span><span>Cursor</span><span>OpenCode</span></div>
    {:else if firstUse.kind === "no_workspace"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.noWorkspaceTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.noWorkspaceDescription")}</p><div class="mt-5 flex flex-wrap justify-center gap-2"><button type="button" class="chat-primary-button" onclick={() => settings.open("chat", { chatSubsection: "workspaces" })}>{t("chat.firstUse.bindProject")}</button><button type="button" class="chat-secondary-button" onclick={() => settings.open("chat", { chatSubsection: "workspaces" })}>{t("chat.firstUse.useStandalone")}</button></div>
    {:else if firstUse.kind === "select_workspace"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.selectWorkspaceTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.selectWorkspaceDescription")}</p><div class="mt-5 grid w-full max-w-md gap-2">{#each chat.workspaces.filter((entry) => entry.workspace.archivedAt === null) as workspace}<button type="button" class="rounded-lg border border-border bg-card p-3 text-left hover:bg-accent" onclick={() => chat.selectWorkspace(workspace.workspace.id)}><span class="block text-sm font-medium">{workspace.workspace.displayName}</span><span class="block text-xs text-muted-foreground">{bindingLabel(workspace.bindingStatus)}</span></button>{/each}</div>
    {:else if firstUse.kind === "missing_binding"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.missingBindingTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.missingBindingDescription", firstUse.workspace.workspace.displayName)}</p>{#if firstUse.workspace.workspace.repositoryIdentity}<code class="mt-3 max-w-full truncate text-xs text-muted-foreground">{firstUse.workspace.workspace.repositoryIdentity}</code>{/if}<button type="button" class="chat-primary-button mt-5" onclick={() => run(() => chat.rebindWorkspace(firstUse.workspace.workspace.id, t("chat.firstUse.chooseWorkspaceFolder")))}><FolderSearch size={15} />{t("chat.firstUse.locateFolder")}</button>
    {:else if firstUse.kind === "provider_unavailable"}
      {@const unavailableProvider = firstUse.provider}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.unavailableTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{unavailableProvider?.lastProbe?.detail ?? t("chat.status.providerUnavailable")}</p><div class="mt-5 flex gap-2">{#if unavailableProvider}<button type="button" class="chat-primary-button" onclick={() => run(() => chat.probeProvider(unavailableProvider.configuration.instanceId))}><RefreshCw size={15} />{t("chat.firstUse.retryProvider")}</button>{/if}<button type="button" class="chat-secondary-button" onclick={() => settings.open("chat", { chatSubsection: "providers" })}><Settings size={15} />{t("chat.firstUse.openSettings")}</button></div>
    {:else if firstUse.kind === "archived_thread"}
      <h2 class="text-lg font-semibold">{t("chat.firstUse.archivedTitle")}</h2><p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.firstUse.archivedDescription")}</p><button type="button" class="chat-primary-button mt-5" onclick={() => run(() => chat.restoreThread(firstUse.thread))}>{t("chat.restore")}</button>
    {:else if firstUse.kind === "no_thread"}
      <div class="hero-composer-shell">
        {#if workspace}
          <div class="workspace-context" aria-label={t("chat.hero.workspace")}>
            <button type="button" title={t("chat.openFolder")} onclick={() => run(() => chat.openWorkspaceFolder(workspace.workspace.id))}>
              <Folder size={15} /><span>{workspace.workspace.displayName}</span>
            </button>
            <span><Laptop size={15} />{t("chat.header.local")}</span>
            {#if workspace.currentBranch}<span title={t("chat.header.branch", workspace.currentBranch)}><GitBranch size={15} /><span>{workspace.currentBranch}</span></span>{/if}
          </div>
        {/if}
        <ChatComposer hero />
      </div>
    {:else}
      <h2 class="text-lg font-semibold">{firstUse.thread.title}</h2>
      <p class="mt-2 max-w-lg text-sm text-muted-foreground">{t("chat.hero.conversationPending")}</p>
    {/if}
  </section>
</div>

<style>
  .first-use-shell { display: flex; min-height: 0; flex: 1; align-items: center; justify-content: center; overflow-y: auto; padding: 1rem; }
  .first-use-content { display: flex; width: 100%; max-width: 42rem; flex-direction: column; align-items: center; text-align: center; }
  .first-use-shell.new-thread { align-items: stretch; }
  .first-use-content.new-thread { max-width: none; justify-content: flex-end; padding-bottom: clamp(0.25rem, 2vh, 1.5rem); }
  .hero-composer-shell { width: min(100%, 46rem); text-align: left; }
  .workspace-context { display: flex; min-width: 0; min-height: 3.15rem; align-items: center; gap: 1.2rem; margin-inline: 1.35rem; border-radius: 1.2rem 1.2rem 0 0; background: color-mix(in srgb, var(--muted) 72%, transparent); padding: 0.45rem 1.1rem 0.7rem; color: var(--foreground); font-size: 0.8rem; }
  .workspace-context > button, .workspace-context > span { display: flex; min-width: 0; align-items: center; gap: 0.45rem; }
  .workspace-context > button { max-width: 45%; border-radius: 0.4rem; }
  .workspace-context > button:hover { color: var(--primary); }
  .workspace-context span span, .workspace-context button span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .workspace-context :global(svg) { flex: 0 0 auto; }
  .hero-composer-shell :global(.chat-composer.hero) { margin-top: -0.45rem; }
  :global(.chat-primary-button), :global(.chat-secondary-button) { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.4rem; border-radius: 0.375rem; padding: 0.4rem 0.8rem; font-size: 0.8rem; font-weight: 600; }
  :global(.chat-primary-button) { background: var(--primary); color: var(--primary-foreground); }
  :global(.chat-secondary-button) { border: 1px solid var(--border); background: var(--background); color: var(--foreground); }
  @container chat-shell (max-width: 560px) { .workspace-context { gap: 0.75rem; margin-inline: 0.65rem; } .workspace-context > button { max-width: 55%; } .workspace-context > span:nth-child(2) { display: none; } }
  @media (max-height: 520px) { .first-use-content.new-thread { padding-bottom: 0; } }
</style>
