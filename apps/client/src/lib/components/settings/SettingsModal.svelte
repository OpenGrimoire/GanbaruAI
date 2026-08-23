<script lang="ts">
  import { onMount, untrack, type Component } from "svelte";
  import { cn } from "$lib/utils";
  import SettingsIcon from "@lucide/svelte/icons/settings";
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
    loadSettingsDetail,
    retrySettingsDetail,
    type LoadedSettingsDetail,
  } from "./settings-detail-registry";
  import type {
    DoomscrollingLimitEditorTarget,
    DoomscrollingSettingsTab,
    NotesTransferOperation,
    ChatProviderSetupTarget,
    ChatSettingsSubsection,
    SectionId,
    SettingsDetailKind,
  } from "./types";
  import { SETTINGS_SECTIONS } from "./settings-sections";
  import AppearanceSection from "./AppearanceSection.svelte";
  import ProfileSection from "./ProfileSection.svelte";
  import CalendarsSection from "./CalendarsSection.svelte";
  import ProjectsSection from "./ProjectsSection.svelte";
  import NotesSection from "./NotesSection.svelte";
  import ChatSection from "./ChatSection.svelte";
  import FocusSection from "./FocusSection.svelte";
  import MusicSection from "./MusicSection.svelte";
  import DoomscrollingSection from "./DoomscrollingSection.svelte";
  import DataSection from "./DataSection.svelte";
  import UpdatesSection from "./UpdatesSection.svelte";
  import ShortcutsSection from "./ShortcutsSection.svelte";
  import AboutSection from "./AboutSection.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";

  const SECTION_COMPONENTS = {
    appearance: AppearanceSection,
    profile: ProfileSection,
    calendars: CalendarsSection,
    projects: ProjectsSection,
    notes: NotesSection,
    chat: ChatSection,
    focus: FocusSection,
    music: MusicSection,
    doomscrolling: DoomscrollingSection,
    data: DataSection,
    updates: UpdatesSection,
    shortcuts: ShortcutsSection,
    about: AboutSection,
  } satisfies Readonly<Record<SectionId, Component>>;

  type SettingsDetailView =
    | { kind: "doomscrolling-limit"; target: DoomscrollingLimitEditorTarget }
    | { kind: "notes-transfer"; operation: NotesTransferOperation }
    | { kind: "chat-provider"; target: ChatProviderSetupTarget };

  let {
    onClose,
    initialSection,
    initialDoomscrollingTab,
    initialChatSubsection,
    initialChatTeammateId,
    initialChatChannelId,
    initialChatCreateTeammate,
  }: {
    onClose: () => void;
    initialSection?: SectionId;
    initialDoomscrollingTab?: DoomscrollingSettingsTab;
    initialChatSubsection?: ChatSettingsSubsection;
    initialChatTeammateId?: string;
    initialChatChannelId?: string;
    initialChatCreateTeammate?: boolean;
  } = $props();

  const themeEditor = getThemeEditor();
  const viewport = getViewport();
  const { t } = getLocalization();

  // When the user opens a theme in the floating editor, step out of the way
  // so the modal backdrop does not block clicking through to the app.
  $effect(() => {
    if (themeEditor.editingId) requestSettingsClose();
  });

  const SECTIONS = SETTINGS_SECTIONS;

  const initialActiveSection = untrack(() => initialSection ?? "appearance");
  const initialActiveChatSubsection = untrack(() => initialChatSubsection ?? "teammates");
  let activeSection = $state<SectionId>(initialActiveSection);
  let activeChatSubsection = $state<ChatSettingsSubsection>(initialActiveChatSubsection);
  let teammateDraftOpen = $state(false);
  let pendingDraftNavigation = $state<(() => void) | null>(null);
  let detailView = $state<SettingsDetailView | null>(null);
  let detailLoadState = $state<LazyComponentLoadState<
    SettingsDetailKind,
    LoadedSettingsDetail
  > | null>(null);
  const activeSectionComponent = $derived(SECTION_COMPONENTS[activeSection]);
  const activeDetailLoadState = $derived(
    detailView && detailLoadState?.key === detailView.kind ? detailLoadState : null,
  );
  let detailScrollEl: HTMLElement | undefined = $state();
  let modalPanel: HTMLElement | undefined = $state();
  let detailScrollbarInsetTop = $state(0);
  let detailScrollbarInsetBottom = $state(0);
  let settingsScrollEl: HTMLElement | undefined = $state();
  const useTopNav = $derived(viewport.below("compact"));
  const useIconRail = $derived(!useTopNav && viewport.below("regular"));
  const settingsScrollbarInset = $derived(useTopNav ? 12 : useIconRail ? 16 : 24);
  const settingsContentPaddingClass = $derived(useTopNav ? "px-3 py-4" : useIconRail ? "px-5 py-5" : "p-8");
  const chatTeammatesUsesInternalScroll = $derived(
    activeSection === "chat" && activeChatSubsection === "teammates",
  );

  function requestSettingsNavigation(navigate: () => void): void {
    if (!teammateDraftOpen) {
      navigate();
      return;
    }
    pendingDraftNavigation = navigate;
  }

  function requestSettingsClose(): void {
    requestSettingsNavigation(onClose);
  }

  function cancelDraftNavigation(): void {
    pendingDraftNavigation = null;
  }

  function discardDraftAndContinue(): void {
    const navigate = pendingDraftNavigation;
    pendingDraftNavigation = null;
    teammateDraftOpen = false;
    navigate?.();
  }

  function updateTeammateDraftState(open: boolean): void {
    teammateDraftOpen = open;
    if (!open) pendingDraftNavigation = null;
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
    const section = SECTIONS.find((candidate) => candidate.id === activeSection);
    return section ? t(section.labelKey) : t("settings.title");
  }

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

  function focusableElements(container: HTMLElement): HTMLElement[] {
    return [...container.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex='-1'])",
    )].filter((element) => !element.hidden && element.getClientRects().length > 0);
  }

  function trapModalFocus(event: KeyboardEvent): void {
    if (event.key !== "Tab" || !modalPanel) return;
    const focusable = focusableElements(modalPanel);
    if (focusable.length === 0) {
      event.preventDefault();
      modalPanel.focus();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function selectSection(section: SectionId): void {
    if (section === activeSection && !detailView) return;
    requestSettingsNavigation(() => {
      activeSection = section;
      detailView = null;
      detailLoadState = null;
      detailScrollEl = undefined;
      detailScrollbarInsetTop = 0;
      detailScrollbarInsetBottom = 0;
      scrollSettingsToTop();
    });
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

  function openChatProviderSetup(target: ChatProviderSetupTarget): void {
    activeSection = "chat";
    activeChatSubsection = "providers";
    detailView = { kind: "chat-provider", target };
    detailScrollEl = undefined;
    detailScrollbarInsetTop = 0;
    detailScrollbarInsetBottom = 0;
    requestSettingsDetail("chat-provider");
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
    const returnFocus = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    queueMicrotask(() => {
      const first = modalPanel ? focusableElements(modalPanel)[0] : undefined;
      (first ?? modalPanel)?.focus();
    });
    function handleKeydown(e: KeyboardEvent) {
      if (pendingDraftNavigation) return;
      trapModalFocus(e);
      if (hasOnlyShortcutModifier(e) && e.key === ",") {
        e.preventDefault();
        e.stopPropagation();
        requestSettingsClose();
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
        requestSettingsClose();
        return;
      }
      // Keep the modal from leaking shortcuts to underlying panels
      e.stopPropagation();
    }
    window.addEventListener("keydown", handleKeydown, true);
    return () => {
      window.removeEventListener("keydown", handleKeydown, true);
      queueMicrotask(() => {
        if (returnFocus?.isConnected) returnFocus.focus();
      });
    };
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
    requestSettingsClose();
  }}
>
  <div class="absolute inset-0 bg-black/50"></div>
  <div
    bind:this={modalPanel}
    data-settings-modal-panel
    role="dialog"
    aria-modal="true"
    aria-label={t("settings.title")}
    tabindex="-1"
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
              <span>{t(section.labelKey)}</span>
            </button>
          {/each}
        </nav>
        <button
          type="button"
          onclick={requestSettingsClose}
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
            onclick={requestSettingsClose}
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
              aria-label={t(section.labelKey)}
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
                <span>{t(section.labelKey)}</span>
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
          detailView || activeSection === "shortcuts" || chatTeammatesUsesInternalScroll
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
            {:else if loadedDetail.kind === "chat-provider" && detailView.kind === "chat-provider"}
              {@const DetailComponent = loadedDetail.component}
              <DetailComponent
                target={detailView.target}
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
        {:else if activeSection === "notes"}
            <NotesSection onOpenTransferPanel={openNotesTransferPanel} />
        {:else if activeSection === "doomscrolling"}
            <DoomscrollingSection
              initialTab={initialDoomscrollingTab}
              onOpenLimitEditor={openDoomscrollingLimitEditor}
            />
        {:else if activeSection === "chat"}
            <ChatSection
              initialSubsection={activeChatSubsection}
              {initialChatTeammateId}
              {initialChatChannelId}
              {initialChatCreateTeammate}
              onOpenProviderSetup={openChatProviderSetup}
              onSubsectionChange={(subsection) => {
                activeChatSubsection = subsection;
                scrollSettingsToTop();
              }}
              onRequestNavigation={requestSettingsNavigation}
              onTeammateDraftStateChange={updateTeammateDraftState}
            />
        {:else}
          {@const SectionComponent = activeSectionComponent}
          <SectionComponent />
        {/if}
      </section>
      {#if detailView}
        <CalendarScrollbar
          scrollContainer={detailScrollEl}
          stickyTop={detailScrollbarInsetTop}
          stickyBottom={detailScrollbarInsetBottom}
          wheelPassthrough
        />
      {:else if activeSection !== "shortcuts" && !chatTeammatesUsesInternalScroll}
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

{#if pendingDraftNavigation}
  <ConfirmDialog
    title={t("settings.chat.teammates.discardDraftTitle")}
    message={t("settings.chat.teammates.discardDraftMessage")}
    confirmLabel={t("settings.chat.teammates.discardDraft")}
    cancelLabel={t("settings.chat.teammates.keepEditing")}
    onConfirm={discardDraftAndContinue}
    onCancel={cancelDraftNavigation}
  />
{/if}
