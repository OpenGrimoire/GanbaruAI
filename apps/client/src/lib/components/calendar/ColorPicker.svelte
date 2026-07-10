<script lang="ts">
  import { tick } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import { FALLBACK_COLOR_INDEX, type EventColor } from "./types";
  import { moveRovingIndex } from "./event-panel-utils";
  import { EVENT_COLOR_OPTIONS, getEventColor } from "./utils";
  import { contrastRatio } from "$lib/components/ui/colorMath";
  import { resolveCalendarTokens, type Theme } from "$lib/stores/themes";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { cn } from "$lib/utils";
  import { portal } from "$lib/utils/portal";

  const { t } = getLocalization();
  const PALETTE_COLUMNS = 4;
  const PALETTE_SWATCH_REM = 1.375;
  const PALETTE_GAP_REM = 0.5;
  const PALETTE_PADDING_REM = 0.625;
  const PALETTE_EDGE_PX = 8;

  let {
    color,
    theme,
    onselect,
    ariaLabel,
    displayLabel = false,
    class: className = "",
    buttonClass = "",
  }: {
    color: EventColor | undefined;
    theme: Theme;
    onselect: (color: EventColor | undefined) => void;
    ariaLabel?: string;
    displayLabel?: boolean;
    class?: string;
    buttonClass?: string;
  } = $props();

  let open = $state(false);
  let buttonEl: HTMLButtonElement | undefined = $state();
  let paletteEl: HTMLDivElement | undefined = $state();
  let activeIndex = $state(0);
  let palettePosition = $state({ left: PALETTE_EDGE_PX, top: PALETTE_EDGE_PX });

  const selectedColor = $derived(color ?? FALLBACK_COLOR_INDEX);
  const colorEntry = $derived(getEventColor(color, theme));
  const buttonLabel = $derived(t("calendar.color.eventColorNumber", selectedColor + 1));
  const calendarTokens = $derived(resolveCalendarTokens(theme));
  const pickerBg = $derived(calendarTokens["--cal-bg"]);
  const pickerText = $derived(calendarTokens["--cal-time-label"]);
  const pickerRing = $derived(calendarTokens["--cal-gridline"]);
  const selectionBorder = $derived(
    contrastRatio(pickerBg, "#000000") >= contrastRatio(pickerBg, "#ffffff")
      ? "#000000"
      : "#ffffff",
  );

  function swatchStyle(bg: string): string {
    return `background-color: ${bg};`;
  }

  function selectedIndex(): number {
    return Math.max(0, EVENT_COLOR_OPTIONS.findIndex((entry) => entry === selectedColor));
  }

  function rootRemPx(): number {
    const fontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
    return Number.isFinite(fontSize) && fontSize > 0 ? fontSize : 16;
  }

  function paletteSizePx(): { width: number; height: number } {
    const rem = rootRemPx();
    const rows = Math.ceil(EVENT_COLOR_OPTIONS.length / PALETTE_COLUMNS);
    return {
      width: PALETTE_COLUMNS * PALETTE_SWATCH_REM * rem
        + (PALETTE_COLUMNS - 1) * PALETTE_GAP_REM * rem
        + PALETTE_PADDING_REM * rem * 2,
      height: rows * PALETTE_SWATCH_REM * rem
        + Math.max(0, rows - 1) * PALETTE_GAP_REM * rem
        + PALETTE_PADDING_REM * rem * 2,
    };
  }

  function computePalettePosition(): void {
    if (!buttonEl) return;
    const rect = buttonEl.getBoundingClientRect();
    const { width, height } = paletteSizePx();
    const rem = rootRemPx();
    const offset = PALETTE_PADDING_REM / 2 * rem;
    const maxLeft = Math.max(PALETTE_EDGE_PX, window.innerWidth - width - PALETTE_EDGE_PX);
    const preferredLeft = displayLabel
      ? rect.left
      : rect.right - width + offset;
    const left = Math.min(Math.max(PALETTE_EDGE_PX, preferredLeft), maxLeft);
    const belowTop = rect.bottom + offset;
    const aboveTop = rect.top - height - offset;
    const top = belowTop + height + PALETTE_EDGE_PX <= window.innerHeight
      ? belowTop
      : Math.max(PALETTE_EDGE_PX, aboveTop);
    palettePosition = { left, top };
  }

  async function focusButton() {
    await tick();
    buttonEl?.focus();
  }

  async function focusSwatch(index: number) {
    await tick();
    paletteEl?.querySelector<HTMLButtonElement>(`[data-color-index="${index}"]`)?.focus();
  }

  function openPalette(source: "keyboard" | "pointer") {
    activeIndex = selectedIndex();
    computePalettePosition();
    open = true;
    void tick().then(() => {
      computePalettePosition();
      if (source === "keyboard") void focusSwatch(activeIndex);
    });
  }

  function closePalette(source: "keyboard" | "pointer") {
    open = false;
    if (source === "keyboard") void focusButton();
  }

  function togglePalette() {
    if (open) closePalette("pointer");
    else openPalette("pointer");
  }

  function selectColor(nextColor: EventColor, source: "keyboard" | "pointer"): void {
    if (color !== nextColor) onselect(nextColor);
    if (source === "keyboard") closePalette("keyboard");
  }

  function handleButtonKeydown(e: KeyboardEvent) {
    if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
    if (e.key === " ") {
      e.preventDefault();
      e.stopPropagation();
      return;
    }
    if (e.key !== "Enter") return;
    e.preventDefault();
    e.stopPropagation();
    openPalette("keyboard");
  }

  function handleSwatchKeydown(e: KeyboardEvent, index: number, nextColor: EventColor) {
    if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;

    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      closePalette("keyboard");
      return;
    }

    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      e.stopPropagation();
      selectColor(nextColor, "keyboard");
      return;
    }

    const nextIndex = moveRovingIndex({
      currentIndex: index,
      itemCount: EVENT_COLOR_OPTIONS.length,
      key: e.key,
      orientation: "grid",
      columns: 4,
    });
    if (nextIndex === index) return;
    e.preventDefault();
    e.stopPropagation();
    activeIndex = nextIndex;
    void focusSwatch(nextIndex);
  }

  function handleBackdropPointerDown(event: PointerEvent): void {
    event.stopPropagation();
    closePalette("pointer");
  }

  $effect(() => {
    if (!open) return;
    function handleResize(): void {
      computePalettePosition();
    }
    function handleScroll(event: Event): void {
      const target = event.target;
      if (target instanceof Node && paletteEl?.contains(target)) return;
      closePalette("pointer");
    }
    window.addEventListener("resize", handleResize);
    window.addEventListener("scroll", handleScroll, true);
    return () => {
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("scroll", handleScroll, true);
    };
  });

  const paletteStyle = $derived(`
    left: ${palettePosition.left}px;
    top: ${palettePosition.top}px;
    grid-template-columns: repeat(${PALETTE_COLUMNS}, 1.375rem);
    background-color: ${pickerBg};
    color: ${pickerText};
    --selection-border: ${selectionBorder};
    --tw-ring-color: ${pickerRing};
  `);
</script>

<div class={cn("relative flex items-center", displayLabel && "min-w-0", className)}>
  <button
    type="button"
    bind:this={buttonEl}
    onclick={togglePalette}
    onkeydown={handleButtonKeydown}
    class={displayLabel
      ? cn("flex h-7 w-full max-w-full items-center justify-between gap-2 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground transition-colors hover:bg-accent/60 dark:bg-transparent", buttonClass)
      : cn("size-4.5 shrink-0 rounded-sm", buttonClass)}
    style={displayLabel ? undefined : `background-color: ${colorEntry.bg};`}
    aria-label={ariaLabel ?? t("calendar.color.eventColor")}
    data-app-tooltip-disabled="true"
  >
    {#if displayLabel}
      <span class="flex min-w-0 items-center gap-2">
        <span
          class="h-3.5 w-3.5 shrink-0 rounded-[3px] border border-transparent"
          style="background-color: {colorEntry.bg};"
          aria-hidden="true"
        ></span>
        <span class="truncate">{buttonLabel}</span>
      </span>
      <ChevronDown
        size={13}
        strokeWidth={2}
        class={cn("shrink-0 text-muted-foreground transition-transform", open && "rotate-180")}
      />
    {/if}
  </button>
  {#if open}
    <div
      use:portal
      role="presentation"
      aria-hidden="true"
      class="fixed inset-0 z-90"
      data-app-floating-surface
      onpointerdown={handleBackdropPointerDown}
    ></div>
    <div
      bind:this={paletteEl}
      use:portal
      data-app-floating-surface
      class={cn(
        "fixed z-100 grid gap-2 rounded-lg p-2.5 shadow-lg ring-1",
      )}
      style={paletteStyle}
    >
      {#each EVENT_COLOR_OPTIONS as c, index}
        {@const entry = getEventColor(c, theme)}
        <button
          type="button"
          data-color-index={index}
          aria-label={ariaLabel ? `${ariaLabel} ${index + 1}` : t("calendar.color.selectEventColor", index + 1)}
          tabindex={activeIndex === index ? 0 : -1}
          onclick={() => { selectColor(c, "pointer"); }}
          onfocus={() => { activeIndex = index; }}
          onkeydown={(e) => handleSwatchKeydown(e, index, c)}
          class="calendar-color-swatch size-5.5 rounded-[3px]"
          class:swatch-selected={selectedColor === c}
          style={swatchStyle(entry.bg)}
          data-app-tooltip-disabled="true"
          data-app-tooltip-focus-disabled="true"
        ></button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .calendar-color-swatch {
    position: relative;
    overflow: hidden;
  }

  .calendar-color-swatch.swatch-selected::after {
    content: "";
    position: absolute;
    inset: 0;
    border: 2px solid var(--selection-border);
    border-radius: inherit;
    pointer-events: none;
  }
</style>
