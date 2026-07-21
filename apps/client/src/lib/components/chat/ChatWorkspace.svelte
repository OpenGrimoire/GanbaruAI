<script lang="ts">
  import { onMount } from "svelte";
  import Command from "@lucide/svelte/icons/command";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import Search from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import X from "@lucide/svelte/icons/x";
  import { nextThreadIndex } from "$lib/chat/shell-model";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import ChatConversationHeader from "./ChatConversationHeader.svelte";
  import ChatFirstUse from "./ChatFirstUse.svelte";
  import ChatThreadRail from "./ChatThreadRail.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  const settings = getSettingsLauncher();
  let rootElement: HTMLDivElement | undefined = $state();
  let commandMenuOpen = $state(false);
  let resizingRail = $state(false);
  let resizingInspector = $state(false);
  let railWidth = $state(260);
  let inspectorWidth = $state(360);
  let loadError = $state<string | null>(null);
  let layoutError = $state<string | null>(null);

  onMount(() => {
    void Promise.all([chat.ensureLoaded(), projects.ensureLoaded()]).catch((error) => {
      loadError = error instanceof Error ? error.message : String(error);
    });
    railWidth = chat.settings?.configuration.panels.railWidthPx ?? 260;
    inspectorWidth = chat.settings?.configuration.panels.inspectorWidthPx ?? 360;
  });

  $effect(() => {
    if (!resizingRail && chat.settings) railWidth = chat.settings.configuration.panels.railWidthPx;
    if (!resizingInspector && chat.settings) inspectorWidth = chat.settings.configuration.panels.inspectorWidthPx;
  });

  function isEditingTarget(target: EventTarget | null): boolean {
    return target instanceof HTMLElement && (
      target.matches("input, textarea, select, [contenteditable='true']")
      || Boolean(target.closest("[role='dialog'], [data-terminal-capture]"))
    );
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (isEditingTarget(event.target)) return;
    if (commandMenuOpen && event.key === "Escape") {
      event.preventDefault();
      commandMenuOpen = false;
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "n") {
      event.preventDefault();
      if (chat.selectedWorkspaceId) chat.newDraft(chat.selectedWorkspaceId);
      else chat.railOpen = true;
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "f") {
      event.preventDefault();
      chat.railOpen = true;
      window.dispatchEvent(new Event("ganbaru-ai:chat-focus-search"));
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "l") {
      event.preventDefault();
      document.querySelector<HTMLElement>("[data-chat-composer]")?.focus();
      return;
    }
    if (event.altKey && !event.ctrlKey && !event.metaKey && ["ArrowDown", "ArrowUp"].includes(event.key)) {
      event.preventDefault();
      const index = chat.activeThreads.findIndex((thread) => thread.id === chat.selectedThreadId);
      const next = nextThreadIndex(index, chat.activeThreads.length, event.key === "ArrowDown" ? "next" : "previous");
      if (next >= 0) chat.selectThread(chat.activeThreads[next].id);
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "b") {
      event.preventDefault();
      chat.railOpen = !chat.railOpen;
      return;
    }
    if (hasOnlyShortcutModifier(event, { shift: true }) && event.key.toLowerCase() === "j") {
      event.preventDefault();
      chat.inspectorOpen = !chat.inspectorOpen;
      return;
    }
    if (hasOnlyShortcutModifier(event, { shift: true }) && event.key.toLowerCase() === "p") {
      event.preventDefault();
      commandMenuOpen = !commandMenuOpen;
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key === ".") {
      event.preventDefault();
      window.dispatchEvent(new Event("ganbaru-ai:chat-stop-requested"));
    }
  }

  function beginRailResize(event: PointerEvent): void {
    resizingRail = true;
    const startX = event.clientX;
    const startWidth = railWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => { railWidth = Math.max(160, Math.min(520, startWidth + moveEvent.clientX - startX)); };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      resizingRail = false;
      void persistPanelWidths();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
  }

  function resizeRailBy(delta: number): void {
    railWidth = Math.max(160, Math.min(520, railWidth + delta));
    void persistPanelWidths();
  }

  function beginInspectorResize(event: PointerEvent): void {
    resizingInspector = true;
    const startX = event.clientX;
    const startWidth = inspectorWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => { inspectorWidth = Math.max(240, Math.min(960, startWidth + startX - moveEvent.clientX)); };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      resizingInspector = false;
      void persistPanelWidths();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
  }

  function resizeInspectorBy(delta: number): void {
    inspectorWidth = Math.max(240, Math.min(960, inspectorWidth + delta));
    void persistPanelWidths();
  }

  async function persistPanelWidths(): Promise<void> {
    if (!chat.settings) return;
    layoutError = null;
    try {
      const { updateChatPanels } = await import("$lib/api/chat");
      await updateChatPanels({ railWidthPx: Math.round(railWidth), inspectorWidthPx: Math.round(inspectorWidth) });
      await chat.refreshSettings();
    } catch (error: unknown) {
      layoutError = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div bind:this={rootElement} class="chat-workspace @container/chat-shell relative flex h-full min-h-0 overflow-hidden" style="background-color:var(--cal-bg);container-type:inline-size;container-name:chat-shell;">
  {#if layoutError}<div role="alert" class="absolute inset-x-2 top-2 z-50 rounded border border-destructive/40 bg-background p-2 text-xs text-destructive">{layoutError}</div>{/if}
  <div class="chat-rail-shell" class:closed={!chat.railOpen} style={`--chat-rail-width:${railWidth}px`}>
    <ChatThreadRail onCollapse={() => { chat.railOpen = false; }} />
  </div>
  <button
    type="button"
    class="chat-rail-separator"
    class:hidden={!chat.railOpen}
    aria-label={t("chat.resizeRail")}
    onpointerdown={beginRailResize}
    onkeydown={(event) => {
      if (event.key === "ArrowLeft") { event.preventDefault(); resizeRailBy(-16); }
      if (event.key === "ArrowRight") { event.preventDefault(); resizeRailBy(16); }
      if (event.key === "Home") { event.preventDefault(); railWidth = 160; void persistPanelWidths(); }
      if (event.key === "End") { event.preventDefault(); railWidth = 520; void persistPanelWidths(); }
      if (event.key === "Enter") { event.preventDefault(); railWidth = 260; void persistPanelWidths(); }
    }}
  ></button>

  <main class="relative flex min-w-0 flex-1 flex-col bg-background/35">
    <ChatConversationHeader onOpenRail={() => { chat.railOpen = true; }} />
    {#if loadError}
      <div role="alert" class="m-auto max-w-md p-4 text-center text-sm text-destructive">{loadError}<div><button type="button" class="chat-secondary-button mt-3" onclick={() => { loadError = null; void chat.reload().catch((error) => { loadError = error instanceof Error ? error.message : String(error); }); }}>{t("common.retry")}</button></div></div>
    {:else if chat.loading}
      <div class="m-auto text-sm text-muted-foreground">{t("common.loading")}</div>
    {:else}
      <ChatFirstUse />
    {/if}
  </main>

  <button type="button" class="chat-inspector-separator" class:hidden={!chat.inspectorOpen} aria-label={t("chat.resizeInspector")} onpointerdown={beginInspectorResize} onkeydown={(event) => { if (event.key === "ArrowLeft") { event.preventDefault(); resizeInspectorBy(16); } if (event.key === "ArrowRight") { event.preventDefault(); resizeInspectorBy(-16); } if (event.key === "Home") { event.preventDefault(); inspectorWidth = 240; void persistPanelWidths(); } if (event.key === "End") { event.preventDefault(); inspectorWidth = 960; void persistPanelWidths(); } if (event.key === "Enter") { event.preventDefault(); inspectorWidth = 360; void persistPanelWidths(); } }}></button>
  <aside class="chat-inspector-shell" class:open={chat.inspectorOpen} aria-label={t("chat.openInspector")} style={`--chat-inspector-width:${inspectorWidth}px`}>
    <header class="flex h-11 items-center border-b border-border px-3"><strong class="min-w-0 flex-1 truncate text-xs">{t("chat.inspector.title")}</strong><button type="button" class="chat-icon-button" aria-label={t("chat.closeInspector")} onclick={() => { chat.inspectorOpen = false; }}><X size={14} /></button></header>
    <div class="p-4 text-xs text-muted-foreground">{t("chat.inspector.placeholder")}</div>
  </aside>

  {#if commandMenuOpen}
    <div class="absolute inset-0 z-50 flex items-start justify-center bg-black/30 p-3 pt-[10vh]">
      <button type="button" class="absolute inset-0" aria-label={t("chat.commandMenu.close")} onclick={() => { commandMenuOpen = false; }}></button>
      <div class="relative w-full max-w-md rounded-lg border border-border bg-popover p-2 shadow-2xl" role="dialog" aria-modal="true" aria-label={t("chat.commandMenu.title")} tabindex="-1">
        <div class="flex items-center gap-2 border-b border-border px-2 py-2 text-xs text-muted-foreground"><Command size={14} />{t("chat.commandMenu.title")}</div>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; if (chat.selectedWorkspaceId) chat.newDraft(chat.selectedWorkspaceId); }}><MessageSquarePlus size={14} />{t("chat.newChat")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.railOpen = true; window.dispatchEvent(new Event("ganbaru-ai:chat-focus-search")); }}><Search size={14} />{t("chat.search")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.inspectorOpen = !chat.inspectorOpen; }}><PanelRight size={14} />{t("chat.openInspector")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; settings.open("chat"); }}><Settings size={14} />{t("chat.settings")}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .chat-rail-shell { width: var(--chat-rail-width); min-width: var(--chat-rail-width); transition: width 140ms ease, min-width 140ms ease, transform 140ms ease; }
  .chat-rail-shell.closed { width: 0; min-width: 0; overflow: hidden; }
  .chat-rail-separator { width: 4px; flex: 0 0 4px; cursor: col-resize; background: transparent; }
  .chat-rail-separator:hover, .chat-rail-separator:focus-visible { background: var(--ring); }
  .chat-inspector-separator { width: 4px; flex: 0 0 4px; cursor: col-resize; background: transparent; }
  .chat-inspector-separator:hover, .chat-inspector-separator:focus-visible { background: var(--ring); }
  .chat-inspector-shell { width: 0; min-width: 0; overflow: hidden; border-left: 0 solid var(--border); background: var(--background); transition: width 140ms ease, min-width 140ms ease; }
  .chat-inspector-shell.open { width: min(var(--chat-inspector-width), 34cqw); min-width: min(240px, 34cqw); border-left-width: 1px; }
  .chat-command { display: flex; width: 100%; min-height: 2.25rem; align-items: center; gap: 0.5rem; border-radius: 0.375rem; padding: 0.375rem 0.5rem; font-size: 0.8rem; }
  .chat-command:hover { background: var(--accent); }
  @container chat-shell (max-width: 719px) {
    .chat-rail-shell { position: absolute; inset-block: 0; left: 0; z-index: 40; width: min(var(--chat-rail-width), 88cqw); min-width: min(var(--chat-rail-width), 88cqw); box-shadow: 8px 0 28px rgb(0 0 0 / 0.22); }
    .chat-rail-shell.closed { width: min(var(--chat-rail-width), 88cqw); min-width: min(var(--chat-rail-width), 88cqw); transform: translateX(-105%); }
    .chat-rail-separator { display: none; }
  }
  @container chat-shell (max-width: 919px) {
    .chat-inspector-separator { display: none; }
    .chat-inspector-shell { position: absolute; inset-block: 0; right: 0; z-index: 35; box-shadow: -8px 0 28px rgb(0 0 0 / 0.22); }
    .chat-inspector-shell.open { width: min(380px, 92cqw); min-width: min(280px, 92cqw); }
  }
  @media (prefers-reduced-motion: reduce) { .chat-rail-shell, .chat-inspector-shell { transition: none; } }
</style>
