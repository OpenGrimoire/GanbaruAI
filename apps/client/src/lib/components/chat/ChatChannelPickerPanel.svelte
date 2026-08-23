<script lang="ts">
  import Hash from "@lucide/svelte/icons/hash";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import type { ChatChannelRead } from "$lib/chat/contracts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { cn } from "$lib/utils";

  type MaybePromise<T> = T | Promise<T>;

  let {
    rootElement = $bindable<HTMLDivElement | undefined>(),
    channels,
    selectedChannelId = null,
    frameStyle,
    className = "relative",
    zIndexClass = "",
    iconStrokeWidth = 1.6,
    onChannelSelected,
    onCreateChannel,
    onLayoutChange = undefined,
    onPointerLeave = undefined,
  }: {
    rootElement?: HTMLDivElement;
    channels: readonly ChatChannelRead[];
    selectedChannelId?: string | null;
    frameStyle: string;
    className?: string;
    zIndexClass?: string;
    iconStrokeWidth?: number;
    onChannelSelected: (channel: ChatChannelRead) => MaybePromise<void>;
    onCreateChannel: () => MaybePromise<void>;
    onLayoutChange?: () => void;
    onPointerLeave?: (event: PointerEvent) => void;
  } = $props();

  const { t } = getLocalization();
  const iconSize = 13;
  let search = $state("");
  let scrollElement = $state<HTMLElement | undefined>();
  const normalizedSearch = $derived(search.trim().toLocaleLowerCase());
  const visibleChannels = $derived(channels.filter((channel) => (
    !normalizedSearch
    || channel.name.toLocaleLowerCase().includes(normalizedSearch)
    || channel.topic.toLocaleLowerCase().includes(normalizedSearch)
  )));

  $effect(() => {
    void visibleChannels.length;
    requestAnimationFrame(() => onLayoutChange?.());
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={rootElement}
  class={cn(
    "chat-channel-picker-panel flex min-h-0 flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60",
    className,
    zIndexClass,
  )}
  style={frameStyle}
  data-chat-channel-picker
  onpointerleave={onPointerLeave}
>
  <div class="shrink-0 px-1.5 pb-0.5 pt-1.5">
    <label class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 px-2">
      <Search size={iconSize} strokeWidth={iconStrokeWidth} class="shrink-0 text-popover-foreground/60" />
      <input
        bind:value={search}
        placeholder={t("chat.channels.search")}
        aria-label={t("chat.channels.search")}
        class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
      />
    </label>
  </div>

  <div class="relative min-h-0 flex-1">
    <div bind:this={scrollElement} class="hide-scrollbar h-full min-h-0 overflow-y-auto p-1">
      {#if visibleChannels.length === 0}
        <p class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
          {normalizedSearch ? t("chat.channels.noResults") : t("chat.channels.empty")}
        </p>
      {:else}
        <div class="grid">
          {#each visibleChannels as channel (channel.id)}
            <button
              type="button"
              class={cn(
                "flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground",
                selectedChannelId === channel.id && "font-medium",
              )}
              aria-current={selectedChannelId === channel.id ? "page" : undefined}
              data-chat-channel-option={channel.id}
              onclick={() => { void onChannelSelected(channel); }}
            >
              <Hash size={iconSize} strokeWidth={iconStrokeWidth} class="shrink-0" />
              <span class="min-w-0 flex-1 truncate">{channel.name}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
    <CalendarScrollbar scrollContainer={scrollElement} stickyTop={4} stickyBottom={4} wheelPassthrough />
  </div>

  <div class="relative z-10 shrink-0 bg-popover p-1.5">
    <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
    <button
      type="button"
      class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
      onclick={() => { void onCreateChannel(); }}
    >
      <Plus size={iconSize} strokeWidth={iconStrokeWidth} />
      <span>{t("chat.channels.createTitle")}</span>
    </button>
  </div>
</div>

<style>
  .chat-channel-picker-panel {
    --cal-scrollbar-thumb: color-mix(in srgb, var(--popover-foreground) 18%, var(--popover));
    --cal-scrollbar-thumb-hover: color-mix(in srgb, var(--popover-foreground) 36%, var(--popover));
  }
</style>
