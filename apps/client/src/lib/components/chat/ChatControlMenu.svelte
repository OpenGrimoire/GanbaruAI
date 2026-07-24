<script lang="ts">
  import { tick } from "svelte";
  import Bot from "@lucide/svelte/icons/bot";
  import Brain from "@lucide/svelte/icons/brain";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import FilePenLine from "@lucide/svelte/icons/file-pen-line";
  import PencilRuler from "@lucide/svelte/icons/pencil-ruler";
  import Settings from "@lucide/svelte/icons/settings";
  import Shield from "@lucide/svelte/icons/shield";
  import ShieldAlert from "@lucide/svelte/icons/shield-alert";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import ShieldOff from "@lucide/svelte/icons/shield-off";
  import ShieldQuestionMark from "@lucide/svelte/icons/shield-question-mark";
  import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
  import {
    pickSelectPopoverGeometry,
    type SelectPopoverGeometry,
    type SelectPopoverRect,
  } from "$lib/components/settings/customSelectPosition";
  import { portal } from "$lib/utils/portal";

  export type ChatControlIcon =
    | "bot"
    | "brain"
    | "file-pen"
    | "pencil-ruler"
    | "shield"
    | "shield-alert"
    | "shield-check"
    | "shield-off"
    | "shield-question-mark"
    | "settings"
    | "sliders";

  export interface ChatControlOption {
    value: string;
    label: string;
    description?: string;
    icon: ChatControlIcon;
    disabled?: boolean;
  }

  let {
    value,
    options,
    placeholder,
    ariaLabel,
    dataField,
    onChange,
    compact = false,
    minimal = false,
    disabled = false,
  }: {
    value: string;
    options: readonly ChatControlOption[];
    placeholder?: ChatControlOption;
    ariaLabel: string;
    dataField?: string;
    onChange: (value: string) => void;
    compact?: boolean;
    minimal?: boolean;
    disabled?: boolean;
  } = $props();

  const DEFAULT_GEOMETRY: SelectPopoverGeometry = {
    top: 0,
    left: 0,
    width: null,
    minWidth: 0,
    maxWidth: 0,
    maxHeight: 0,
    placement: "above",
  };

  let open = $state(false);
  let ready = $state(false);
  let trigger: HTMLButtonElement | undefined = $state();
  let popover: HTMLDivElement | undefined = $state();
  let geometry = $state<SelectPopoverGeometry>(DEFAULT_GEOMETRY);
  const current = $derived(options.find((option) => option.value === value) ?? placeholder ?? options[0]);

  function rect(value: DOMRect): SelectPopoverRect {
    return {
      top: value.top,
      right: value.right,
      bottom: value.bottom,
      left: value.left,
      width: value.width,
      height: value.height,
    };
  }

  function computePosition(): void {
    if (!trigger) return;
    geometry = pickSelectPopoverGeometry({
      triggerRect: rect(trigger.getBoundingClientRect()),
      boundaryRect: {
        top: 0,
        right: window.innerWidth,
        bottom: window.innerHeight,
        left: 0,
        width: window.innerWidth,
        height: window.innerHeight,
      },
      contentHeight: popover?.scrollHeight ?? 220,
      contentWidth: popover?.scrollWidth ?? 260,
      horizontalAlign: "start",
    });
    ready = true;
  }

  function popoverStyle(): string {
    if (!ready) return "visibility:hidden;top:0;left:0;";
    return `visibility:visible;top:${geometry.top}px;left:${geometry.left}px;min-width:${Math.max(220, geometry.minWidth)}px;max-width:${geometry.maxWidth}px;max-height:${geometry.maxHeight}px;`;
  }

  async function toggle(): Promise<void> {
    if (disabled) return;
    if (open) {
      open = false;
      return;
    }
    ready = false;
    open = true;
    await tick();
    computePosition();
    popover?.querySelector<HTMLButtonElement>("[aria-selected='true']")?.focus();
  }

  function select(option: ChatControlOption): void {
    if (option.disabled) return;
    onChange(option.value);
    open = false;
    queueMicrotask(() => trigger?.focus());
  }

  function handleTriggerKeydown(event: KeyboardEvent): void {
    if (!["ArrowDown", "ArrowUp"].includes(event.key)) return;
    event.preventDefault();
    void toggle();
  }

  function handlePopoverKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      open = false;
      queueMicrotask(() => trigger?.focus());
      return;
    }
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    const buttons = [...(event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>("button:not(:disabled)")];
    if (buttons.length === 0) return;
    event.preventDefault();
    const currentIndex = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const nextIndex = event.key === "Home"
      ? 0
      : event.key === "End"
        ? buttons.length - 1
        : (Math.max(0, currentIndex) + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length;
    buttons[nextIndex]?.focus();
  }

  $effect(() => {
    if (!open) return;
    const handleOutsidePointer = (event: MouseEvent) => {
      if (!(event.target instanceof Node)) return;
      if (trigger?.contains(event.target) || popover?.contains(event.target)) return;
      open = false;
    };
    const handleViewportChange = () => computePosition();
    window.addEventListener("mousedown", handleOutsidePointer, true);
    window.addEventListener("resize", handleViewportChange);
    return () => {
      window.removeEventListener("mousedown", handleOutsidePointer, true);
      window.removeEventListener("resize", handleViewportChange);
    };
  });
</script>

{#snippet controlIcon(icon: ChatControlIcon, size = 14)}
  {#if icon === "bot"}<Bot {size} />
  {:else if icon === "brain"}<Brain {size} />
  {:else if icon === "file-pen"}<FilePenLine {size} />
  {:else if icon === "pencil-ruler"}<PencilRuler {size} />
  {:else if icon === "settings"}<Settings {size} />
  {:else if icon === "shield-alert"}<ShieldAlert {size} />
  {:else if icon === "shield-check"}<ShieldCheck {size} />
  {:else if icon === "shield-off"}<ShieldOff {size} />
  {:else if icon === "shield-question-mark"}<ShieldQuestionMark {size} />
  {:else if icon === "sliders"}<SlidersHorizontal {size} />
  {:else}<Shield {size} />{/if}
{/snippet}

<button
  bind:this={trigger}
  type="button"
  class="control-trigger"
  class:compact
  class:minimal
  {disabled}
  aria-label={ariaLabel}
  aria-haspopup="listbox"
  aria-expanded={open}
  data-chat-field={dataField}
  onclick={() => void toggle()}
  onkeydown={handleTriggerKeydown}
  title={current?.description ?? ariaLabel}
>
  {@render controlIcon(current?.icon ?? "sliders")}
  <span class="control-label">{current?.label ?? ariaLabel}</span>
  {#if !minimal}<ChevronDown size={12} class={open ? "open" : ""} />{/if}
</button>

{#if open}
  <div
    bind:this={popover}
    use:portal
    class="control-popover"
    role="listbox"
    tabindex="-1"
    aria-label={ariaLabel}
    data-app-floating-surface
    style={popoverStyle()}
    onkeydown={handlePopoverKeydown}
  >
    {#each options as option (option.value)}
      <button
        type="button"
        role="option"
        aria-selected={option.value === value}
        disabled={option.disabled}
        onclick={() => select(option)}
      >
        <span class="option-icon">{@render controlIcon(option.icon, 15)}</span>
        <span class="option-copy"><strong>{option.label}</strong>{#if option.description}<small>{option.description}</small>{/if}</span>
        <Check size={13} class={option.value === value ? "visible" : ""} />
      </button>
    {/each}
  </div>
{/if}

<style>
  .control-trigger { display: inline-flex; min-width: 0; height: 1.9rem; max-width: 10rem; flex: 0 1 auto; align-items: center; gap: 0.4rem; border-radius: 0.55rem; padding: 0.25rem 0.45rem; color: var(--muted-foreground); font-size: 0.766667rem; white-space: nowrap; }
  .control-trigger:hover, .control-trigger[aria-expanded="true"] { background: var(--accent); color: var(--foreground); }
  .control-trigger.minimal { padding-inline: 0.35rem; }
  .control-trigger:focus-visible { outline: 2px solid var(--ring); outline-offset: 1px; }
  .control-trigger:disabled { cursor: not-allowed; opacity: 0.45; }
  .control-label { min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .control-trigger :global(svg:last-child) { flex: 0 0 auto; transition: transform 120ms ease; }
  .control-trigger :global(svg.open:last-child) { transform: rotate(180deg); }
  .control-popover { position: fixed; z-index: 80; overflow-y: auto; border: 1px solid var(--border); border-radius: 0.65rem; background: var(--popover); padding: 0.3rem; color: var(--popover-foreground); box-shadow: 0 14px 36px rgb(0 0 0 / 0.2); }
  .control-popover button { display: grid; width: 100%; grid-template-columns: 1.5rem minmax(0, 1fr) 1rem; align-items: center; gap: 0.45rem; border-radius: 0.4rem; padding: 0.45rem 0.5rem; text-align: left; }
  .control-popover button:hover, .control-popover button:focus-visible { background: var(--accent); outline: none; }
  .control-popover button:disabled { cursor: not-allowed; opacity: 0.45; }
  .option-icon { display: grid; place-items: center; color: var(--muted-foreground); }
  .option-copy { min-width: 0; }
  .option-copy strong, .option-copy small { display: block; overflow: hidden; text-overflow: ellipsis; }
  .option-copy strong { font-size: 0.8rem; font-weight: 500; }
  .option-copy small { margin-top: 0.1rem; color: var(--muted-foreground); font-size: 0.733333rem; line-height: 1.05rem; white-space: normal; }
  .control-popover button > :global(svg:last-child) { visibility: hidden; }
  .control-popover button > :global(svg.visible:last-child) { visibility: visible; }
  @container chat-composer (max-width: 520px) { .control-trigger.compact { width: 1.9rem; padding-inline: 0; justify-content: center; } .control-trigger.compact .control-label, .control-trigger.compact :global(svg:last-child) { display: none; } }
</style>
