<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import EllipsisVertical from "@lucide/svelte/icons/ellipsis-vertical";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import PanelBottom from "@lucide/svelte/icons/panel-bottom";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Settings from "@lucide/svelte/icons/settings";
  import SquareArrowOutUpRight from "@lucide/svelte/icons/square-arrow-out-up-right";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import { cn } from "$lib/utils";
  import { BUILD_PLATFORM_PROFILE, platformHasCapability } from "$lib/platform";
  import { openDetachedChatWindow } from "$lib/chat/local-execution-ui";

  let {
    bottomPanelOpen,
    onToggleBottomPanel,
    onRename,
  }: {
    bottomPanelOpen: boolean;
    onToggleBottomPanel: () => void;
    onRename: () => void;
  } = $props();

  const chat = getChat();
  const settings = getSettingsLauncher();
  const { t } = getLocalization();
  const localExecutionAvailable = platformHasCapability(
    BUILD_PLATFORM_PROFILE,
    "chat.local-execution",
  );
  const mobilePresentation = BUILD_PLATFORM_PROFILE.shell === "mobile";
  let actionError = $state<string | null>(null);
  let moreActions = $state<HTMLDetailsElement | null>(null);
  const selectedFolder = $derived(chat.selectedWorkingFolder);
  const selectedThread = $derived(chat.selectedThread);
  const selectedChannel = $derived(chat.selectedChannel);

  function run(action: () => Promise<unknown>): void {
    actionError = null;
    void action().catch((error: unknown) => {
      actionError = error instanceof Error ? error.message : String(error);
    });
  }

  function archiveSelectedChannel(): void {
    const channel = selectedChannel;
    if (!channel || !window.confirm(
      `${t("chat.channels.archiveConfirmTitle", channel.name)}\n\n${t("chat.channels.archiveConfirmMessage")}`,
    )) return;
    run(() => chat.archiveChannel(channel));
  }

  function closeMoreActionsFromOutside(event: PointerEvent): void {
    const target = event.target;
    if (moreActions?.open && target instanceof Node && !moreActions.contains(target)) moreActions.open = false;
  }
</script>

<svelte:window onpointerdown={closeMoreActionsFromOutside} />

<div class="chat-header-actions flex min-w-0 shrink-0 items-center gap-1" class:mobilePresentation data-chat-header-actions>
  {#if actionError}<p role="alert" class="max-w-40 truncate text-[0.666667rem] text-destructive">{actionError}</p>{/if}
  {#if localExecutionAvailable}
    <button type="button" class={cn("chat-header-icon-button", bottomPanelOpen && "bg-accent text-foreground")} aria-label={bottomPanelOpen ? t("chat.closeBottomPanel") : t("chat.openBottomPanel")} aria-pressed={bottomPanelOpen} data-chat-bottom-panel-action onclick={onToggleBottomPanel}>
      <PanelBottom size={14} strokeWidth={1.75} />
    </button>
    <button type="button" class={cn("chat-header-icon-button", chat.inspectorOpen && "bg-accent text-foreground")} aria-label={chat.inspectorOpen ? t("chat.closeInspector") : t("chat.openInspector")} aria-pressed={chat.inspectorOpen} data-chat-inspector-action onclick={() => { chat.inspectorOpen = !chat.inspectorOpen; }}>
      <PanelRight size={14} strokeWidth={1.75} />
    </button>
  {/if}
  <details bind:this={moreActions} class="relative">
    <summary class="chat-header-icon-button list-none" aria-label={t("chat.moreActions")}><EllipsisVertical size={14} strokeWidth={1.75} /></summary>
    <div class="chat-actions-menu right-0 top-8">
      {#if selectedChannel && !selectedChannel.isDefault}<button type="button" onclick={onRename}><Pencil size={13} />{t("chat.rename")}</button>{/if}
      {#if localExecutionAvailable && selectedFolder?.bindingStatus === "available"}<button type="button" onclick={() => run(() => chat.openWorkingFolder(selectedFolder.workingFolder.id))}><FolderOpen size={13} />{t("chat.openFolder")}</button>{/if}
      {#if localExecutionAvailable && selectedThread}<button type="button" onclick={() => run(openDetachedChatWindow)}><SquareArrowOutUpRight size={13} />{t("chat.detach")}</button>{/if}
      {#if selectedChannel && !selectedChannel.isDefault && !selectedChannel.archivedAt}<button type="button" onclick={archiveSelectedChannel}><Archive size={13} />{t("chat.archive")}</button>{/if}
      <button type="button" onclick={() => settings.open("chat")}><Settings size={13} />{t("chat.settings")}</button>
    </div>
  </details>
</div>

<style>
  .chat-header-icon-button { display: flex; height: 1.75rem; width: 1.75rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--foreground); transition: background-color 120ms ease; }
  .chat-header-icon-button:hover { background: var(--accent); }
  .chat-actions-menu { position: absolute; z-index: 90; display: grid; min-width: 11rem; overflow: hidden; border: 1px solid var(--border); border-radius: 0.5rem; background: var(--popover); padding: 0.25rem; color: var(--popover-foreground); box-shadow: 0 12px 30px rgb(0 0 0 / 0.2); }
  .chat-actions-menu button { display: flex; min-height: 2rem; align-items: center; gap: 0.5rem; border-radius: 0.375rem; padding-inline: 0.5rem; text-align: left; font-size: calc(0.733333rem * var(--type-scale)); }
  .chat-actions-menu button:hover { background: var(--accent); color: var(--accent-foreground); }
  .chat-header-actions.mobilePresentation .chat-header-icon-button { width: 3rem; height: 3rem; }
  .chat-header-actions.mobilePresentation .chat-actions-menu { top: 3rem; }
  .chat-header-actions.mobilePresentation .chat-actions-menu button { min-height: 3rem; }
  @media (prefers-reduced-motion: reduce) { .chat-header-icon-button { transition: none; } }
</style>
