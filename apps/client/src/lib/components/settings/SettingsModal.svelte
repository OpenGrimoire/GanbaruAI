<script lang="ts">
  import { onMount, untrack, type Component } from "svelte";
  import { cn } from "$lib/utils";
  import Palette from "@lucide/svelte/icons/palette";
  import UserRound from "@lucide/svelte/icons/user-round";
  import Calendar from "@lucide/svelte/icons/calendar";
  import Folder from "@lucide/svelte/icons/folder";
  import Book from "@lucide/svelte/icons/book";
  import GlobeOff from "@lucide/svelte/icons/globe-off";
  import Info from "@lucide/svelte/icons/info";
  import Keyboard from "@lucide/svelte/icons/keyboard";
  import Music from "@lucide/svelte/icons/music";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import Timer from "@lucide/svelte/icons/timer";
  import DownloadCloud from "@lucide/svelte/icons/download-cloud";
  import HardDrive from "@lucide/svelte/icons/hard-drive";
  import X from "@lucide/svelte/icons/x";
  import CalendarScrollbar from "../calendar/CalendarScrollbar.svelte";
  import { getThemeEditor } from "$lib/stores/themeEditor.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import {
    loadSettingsSection,
    retrySettingsSection,
    type LoadedSettingsSection,
  } from "./settings-section-registry";
  import {
    loadSettingsDetail,
    retrySettingsDetail,
    type LoadedSettingsDetail,
  } from "./settings-detail-registry";
  import type {
    DoomscrollingLimitEditorTarget,
    DoomscrollingSettingsTab,
    NotesTransferOperation,
    SectionId,
    SettingsDetailKind,
  } from "./types";

  type SettingsDetailView =
    | { kind: "doomscrolling-limit"; target: DoomscrollingLimitEditorTarget }
    | { kind: "notes-transfer"; operation: NotesTransferOperation };

  let {
    onClose,
    initialSection,
    initialDoomscrollingTab,
  }: {
    onClose: () => void;
    initialSection?: SectionId;
    initialDoomscrollingTab?: DoomscrollingSettingsTab;
  } = $props();

  const themeEditor = getThemeEditor();
  const viewport = getViewport();
  const { t } = getLocalization();

  // When the user opens a theme in the floating editor, step out of the way
  // so the modal backdrop does not block clicking through to the app.
  $effect(() => {
    if (themeEditor.editingId) onClose();
  });

  interface SectionMeta {
    id: SectionId;
    label: () => string;
    icon: Component;
  }

  // Keep labels and icons aligned with the typed lazy registry.
  const SECTIONS: SectionMeta[] = [
    { id: "appearance", label: () => t("settings.section.appearance"), icon: Palette },
    { id: "profile", label: () => t("settings.section.profile"), icon: UserRound },
    { id: "calendars", label: () => t("settings.section.calendars"), icon: Calendar },
    { id: "projects", label: () => t("settings.section.projects"), icon: Folder },
    { id: "notes", label: () => t("settings.section.notes"), icon: Book },
    { id: "focus", label: () => t("settings.section.focus"), icon: Timer },
    { id: "music", label: () => t("settings.section.music"), icon: Music },
    { id: "doomscrolling", label: () => t("settings.section.doomscrolling"), icon: GlobeOff },
    { id: "data", label: () => t("settings.section.data"), icon: HardDrive },
    { id: "updates", label: () => t("settings.section.updates"), icon: DownloadCloud },
    { id: "shortcuts", label: () => t("settings.section.shortcuts"), icon: Keyboard },
    { id: "about", label: () => t("settings.section.about"), icon: Info },
  ];

  const initialActiveSection = untrack(() => initialSection ?? "appearance");
  let activeSection = $state<SectionId>(initialActiveSection);
  let detailView = $state<SettingsDetailView | null>(null);
  let sectionLoadState = $state<LazyComponentLoadState<
    SectionId,
    LoadedSettingsSection
  > | null>(null);
  let detailLoadState = $state<LazyComponentLoadState<
    SettingsDetailKind,
    LoadedSettingsDetail
  > | null>(null);
  const activeSectionLoadState = $derived(
    sectionLoadState?.key === activeSection ? sectionLoadState : null,
  );
  const activeDetailLoadState = $derived(
    detailView && detailLoadState?.key === detailView.kind ? detailLoadState : null,
  );
  let detailScrollEl: HTMLElement | undefined = $state();
  let detailScrollbarInsetTop = $state(0);
  let detailScrollbarInsetBottom = $state(0);
  let settingsScrollEl: HTMLElement | undefined = $state();
  const useTopNav = $derived(viewport.below("compact"));
  const useIconRail = $derived(!useTopNav && viewport.below("regular"));
  const settingsScrollbarInset = $derived(useTopNav ? 12 : useIconRail ? 16 : 24);
  const settingsContentPaddingClass = $derived(useTopNav ? "px-3 py-4" : useIconRail ? "px-5 py-5" : "p-8");
  function requestSettingsSection(section: SectionId, retry = false): void {
    if (!retry && sectionLoadState?.key === section) return;
    const loadingState = beginLazyComponentLoad(sectionLoadState, section);
    sectionLoadState = loadingState;
    const request = retry ? retrySettingsSection(section) : loadSettingsSection(section);
    void request
      .then((component) => {
        if (!sectionLoadState) return;
        const nextState = resolveLazyComponentLoad(
          sectionLoadState,
          section,
          loadingState.requestId,
          component,
        );
        if (nextState !== sectionLoadState) sectionLoadState = nextState;
      })
      .catch((error: unknown) => {
        if (!sectionLoadState) return;
        const nextState = rejectLazyComponentLoad(
          sectionLoadState,
          section,
          loadingState.requestId,
          error,
        );
        if (nextState === sectionLoadState) return;
        sectionLoadState = nextState;
        console.error(`Failed to load ${section} Settings section:`, error);
      });
  }

  function requestSettingsDetail(kind: SettingsDetailKind, retry = false): void {
    if (!retry && detailLoadState?.key === kind) return;
    const loadingState = beginLazyComponentLoad(detailLoadState, kind);
    detailLoadState = loadingState;
    const request = retry ? retrySettingsDetail(kind) : loadSettingsDetail(kind);
    void request
      .then((component) => {
        if (!detailLoadState) return;
        const nextState = resolveLazyComponentLoad(
          detailLoadState,
          kind,
          loadingState.requestId,
          component,
        );
        if (nextState !== detailLoadState) detailLoadState = nextState;
      })
      .catch((error: unknown) => {
        if (!detailLoadState) return;
        const nextState = rejectLazyComponentLoad(
          detailLoadState,
          kind,
          loadingState.requestId,
          error,
        );
        if (nextState === detailLoadState) return;
        detailLoadState = nextState;
        console.error(`Failed to load ${kind} Settings detail:`, error);
      });
  }

  function activeSectionLabel(): string {
    return SECTIONS.find((section) => section.id === activeSection)?.label()
      ?? t("settings.title");
  }

  requestSettingsSection(initialActiveSection);

  function focusShortcutsSearch(): void {
    const input = document.querySelector<HTMLInputElement>(
      "[data-shortcuts-search-input]",
    );
    input?.focus();
    input?.select();
  }

  function scrollSettingsToTop(): void {
    queueMicrotask(() => {
      settingsScrollEl?.scrollTo({ top: 0 });
    });
  }

  function selectSection(section: SectionId): void {
    activeSection = section;
    detailView = null;
    detailLoadState = null;
    detailScrollEl = undefined;
    detailScrollbarInsetTop = 0;
    detailScrollbarInsetBottom = 0;
    requestSettingsSection(section);
    scrollSettingsToTop();
  }

  function openDoomscrollingLimitEditor(target: DoomscrollingLimitEditorTarget): void {
    activeSection = "doomscrolling";
    detailView = { kind: "doomscrolling-limit", target };
    detailScrollEl = undefined;
    detailScrollbarInsetTop = 0;
    detailScrollbarInsetBottom = 0;
    requestSettingsDetail("doomscrolling-limit");
    scrollSettingsToTop();
  }

  function openNotesTransferPanel(operation: NotesTransferOperation): void {
    activeSection = "notes";
    detailView = { kind: "notes-transfer", operation };
    detailScrollEl = undefined;
    detailScrollbarInsetTop = 0;
    detailScrollbarInsetBottom = 0;
    requestSettingsDetail("notes-transfer");
    scrollSettingsToTop();
  }

  function closeDetailView(): void {
    detailView = null;
    detailLoadState = null;
    detailScrollEl = undefined;
    detailScrollbarInsetTop = 0;
    detailScrollbarInsetBottom = 0;
    scrollSettingsToTop();
  }

  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (hasOnlyShortcutModifier(e) && e.key === ",") {
        e.preventDefault();
        e.stopPropagation();
        onClose();
        return;
      }
      if (activeSection === "shortcuts" && hasOnlyShortcutModifier(e) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        e.stopPropagation();
        focusShortcutsSearch();
        return;
      }
      if (e.key === "F1") {
        e.preventDefault();
        e.stopPropagation();
        selectSection("shortcuts");
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        if (detailView) {
          closeDetailView();
          return;
        }
        onClose();
        return;
      }
      // Keep the modal from leaking shortcuts to underlying panels
      e.stopPropagation();
    }
    window.addEventListener("keydown", handleKeydown, true);
    return () => window.removeEventListener("keydown", handleKeydown, true);
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class={cn(
    "fixed inset-0 z-70 flex",
    useTopNav ? "items-stretch justify-center p-1" : "items-center justify-center p-2",
  )}
  onclick={(e) => {
    e.stopPropagation();
    onClose();
  }}
>
  <div class="absolute inset-0 bg-black/50"></div>
  <div
    data-settings-modal-panel
    data-settings-section={activeSection}
    class={cn(
      "relative z-10 flex overflow-hidden border border-border bg-card shadow-2xl dark:bg-background",
      useTopNav
        ? "h-[calc(100dvh-0.5rem)] w-full flex-col rounded-md"
        : "h-[80vh] rounded-lg",
      !useTopNav && useIconRail ? "w-[min(760px,94vw)]" : "",
      !useTopNav && !useIconRail ? "w-[min(900px,90vw)]" : "",
    )}
    onclick={(e) => e.stopPropagation()}
  >
    {#if useTopNav}
      <header class="flex shrink-0 items-center gap-2 border-b border-border/70 bg-background/40 px-2 py-2 dark:bg-black/20">
        <nav class="flex min-w-0 flex-1 gap-1 overflow-x-auto rounded-md bg-card/60 p-0.5 dark:bg-background/60">
          {#each SECTIONS as section}
            {@const Icon = section.icon}
            <button
              onclick={() => {
                selectSection(section.id);
              }}
              class={cn(
                "flex h-8 shrink-0 items-center gap-1.5 rounded px-2.5 text-[0.8rem] font-medium",
                activeSection === section.id
                  ? "bg-accent text-accent-foreground"
                  : "text-foreground hover:bg-accent/60",
              )}
            >
              <Icon size={14} strokeWidth={1.75} class="shrink-0" />
              <span>{section.label()}</span>
            </button>
          {/each}
        </nav>
        <button
          type="button"
          onclick={onClose}
          aria-label={t("settings.close")}
          data-app-tooltip-disabled="true"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          <X size={15} strokeWidth={2} />
        </button>
      </header>
    {:else}
      <!-- Sidebar -->
      <aside
        class={cn(
          "flex shrink-0 flex-col gap-3 bg-background/40 dark:bg-black/20",
          useIconRail ? "w-14 px-1 py-3" : "w-58 py-4 pl-2 pr-0",
        )}
      >
        <div class={cn("flex items-center", useIconRail ? "justify-center" : "justify-between")}>
          {#if !useIconRail}
            <span class="flex h-7 min-w-0 items-center gap-2.5 px-3 text-[0.866667rem] font-semibold text-muted-foreground">
              <SettingsIcon size={15} strokeWidth={1.75} class="shrink-0" />
              <span class="truncate">{t("settings.title")}</span>
            </span>
          {/if}
          <button
            type="button"
            onclick={onClose}
            aria-label={t("settings.close")}
            data-app-tooltip-disabled="true"
            class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
          >
            <X size={14} strokeWidth={2} />
          </button>
        </div>
        <nav class="flex flex-col">
          {#each SECTIONS as section}
            {@const Icon = section.icon}
            <button
              onclick={() => {
                selectSection(section.id);
              }}
              aria-label={section.label()}
              data-app-tooltip-disabled="true"
              class={cn(
                "flex items-center rounded-md text-left text-[0.866667rem] font-medium",
                useIconRail ? "h-9 justify-center px-0" : "gap-2.5 px-3 py-1.5",
                activeSection === section.id
                  ? "bg-accent text-accent-foreground"
                  : "text-foreground hover:bg-accent/60",
              )}
            >
              <Icon size={15} strokeWidth={1.75} class="shrink-0" />
              {#if !useIconRail}
                <span>{section.label()}</span>
              {/if}
            </button>
          {/each}
        </nav>
      </aside>
    {/if}

    <div class="relative min-h-0 flex-1">
      <!-- Content -->
      <section
        bind:this={settingsScrollEl}
        data-settings-content
        class={cn(
          "h-full min-h-0 bg-background/40 dark:bg-black/20",
          detailView || activeSection === "shortcuts"
            ? "overflow-hidden"
            : "hide-scrollbar overflow-y-auto",
          detailView ? "p-0" : settingsContentPaddingClass,
        )}
      >
        {#if detailView}
          {#if activeDetailLoadState?.status === "ready"}
            {@const loadedDetail = activeDetailLoadState.component}
            {#if loadedDetail.kind === "doomscrolling-limit" && detailView.kind === "doomscrolling-limit"}
              {@const DetailComponent = loadedDetail.component}
              <DetailComponent
                target={detailView.target}
                onDone={closeDetailView}
                onCancel={closeDetailView}
                compactLayout={useTopNav}
                iconRailLayout={useIconRail}
                onScrollContainerChange={(scrollContainer: HTMLElement | undefined) => {
                  detailScrollEl = scrollContainer;
                }}
                onScrollbarInsetsChange={(insets: { top: number; bottom: number }) => {
                  detailScrollbarInsetTop = insets.top;
                  detailScrollbarInsetBottom = insets.bottom;
                }}
              />
            {:else if loadedDetail.kind === "notes-transfer" && detailView.kind === "notes-transfer"}
              {@const DetailComponent = loadedDetail.component}
              <DetailComponent
                operation={detailView.operation}
                onCancel={closeDetailView}
                compactLayout={useTopNav}
                iconRailLayout={useIconRail}
                onScrollContainerChange={(scrollContainer: HTMLElement | undefined) => {
                  detailScrollEl = scrollContainer;
                }}
                onScrollbarInsetsChange={(insets: { top: number; bottom: number }) => {
                  detailScrollbarInsetTop = insets.top;
                  detailScrollbarInsetBottom = insets.bottom;
                }}
              />
            {/if}
          {:else if activeDetailLoadState?.status === "failed"}
            {@const failedDetailKind = activeDetailLoadState.key}
            <div
              class="flex h-full flex-col items-center justify-center gap-3 p-4 text-center text-sm text-muted-foreground"
              role="alert"
            >
              <p>{t("common.viewLoadFailed", activeSectionLabel())}</p>
              <button
                type="button"
                class="min-h-9 rounded-md border border-border bg-background px-3 font-medium text-foreground hover:bg-accent"
                onclick={() => requestSettingsDetail(failedDetailKind, true)}
              >
                {t("common.retry")}
              </button>
            </div>
          {:else}
            <div
              class="flex h-full items-center justify-center p-4 text-sm text-muted-foreground"
              aria-busy="true"
            >
              {t("common.loading")}
            </div>
          {/if}
        {:else if activeSectionLoadState?.status === "ready"}
          {@const loadedSection = activeSectionLoadState.component}
          {#if loadedSection.section === "notes"}
            {@const SectionComponent = loadedSection.component}
            <SectionComponent onOpenTransferPanel={openNotesTransferPanel} />
          {:else if loadedSection.section === "doomscrolling"}
            {@const SectionComponent = loadedSection.component}
            <SectionComponent
              initialTab={initialDoomscrollingTab}
              onOpenLimitEditor={openDoomscrollingLimitEditor}
            />
          {:else}
            {@const SectionComponent = loadedSection.component}
            <SectionComponent />
          {/if}
        {:else if activeSectionLoadState?.status === "failed"}
          <div
            class="flex min-h-40 flex-col items-center justify-center gap-3 text-center text-sm text-muted-foreground"
            role="alert"
          >
            <p>{t("common.viewLoadFailed", activeSectionLabel())}</p>
            <button
              type="button"
              class="min-h-9 rounded-md border border-border bg-background px-3 font-medium text-foreground hover:bg-accent"
              onclick={() => requestSettingsSection(activeSection, true)}
            >
              {t("common.retry")}
            </button>
          </div>
        {:else}
          <div
            class="flex min-h-40 items-center justify-center text-sm text-muted-foreground"
            aria-busy="true"
          >
            {t("common.loading")}
          </div>
        {/if}
      </section>
      {#if detailView}
        <CalendarScrollbar
          scrollContainer={detailScrollEl}
          stickyTop={detailScrollbarInsetTop}
          stickyBottom={detailScrollbarInsetBottom}
          wheelPassthrough
        />
      {:else if activeSection !== "shortcuts"}
        <CalendarScrollbar
          scrollContainer={settingsScrollEl}
          stickyTop={settingsScrollbarInset}
          stickyBottom={settingsScrollbarInset}
          wheelPassthrough
        />
      {/if}
    </div>
  </div>
</div>
