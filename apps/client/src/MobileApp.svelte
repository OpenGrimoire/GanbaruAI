<script lang="ts">
  import { onBackButtonPress } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import MobileNavigation from "$lib/components/mobile/MobileNavigation.svelte";
  import MobilePomodoroSheet from "$lib/components/mobile/MobilePomodoroSheet.svelte";
  import MobileTopBar from "$lib/components/mobile/MobileTopBar.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { ensureDbUrl } from "$lib/api/db";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    MobileBackListenerController,
    resolveMobileBackAction,
  } from "$lib/mobile-back";
  import { mobileNavigationPresentation } from "$lib/mobile-layout";
  import { MobilePersistenceLifecycleController } from "$lib/mobile-persistence-lifecycle";
  import { activateModalFocus } from "$lib/modal-focus";
  import type { View } from "$lib/navigation";
  import { BUILD_PLATFORM_PROFILE, platformHasCapability } from "$lib/platform";
  import { flushQuickNoteEditors } from "$lib/quick-notes/persistence";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getCalendars } from "$lib/stores/calendars.svelte";
  import { getNavigation } from "$lib/stores/navigation.svelte";
  import { getPomodoro } from "$lib/stores/pomodoro.svelte";
  import { getMobileBackStack } from "$lib/stores/mobile-back-stack.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { getZoom } from "$lib/stores/zoom.svelte";
  import { flushConfig } from "$lib/vault/config";

  type CalendarComponent = typeof import("$lib/components/calendar/CalendarView.svelte").default;
  type ProjectsComponent = typeof import("$lib/components/projects/ProjectsView.svelte").default;
  type ProjectMobileListComponent = typeof import("$lib/components/projects/ProjectMobileListView.svelte").default;
  type NotesComponent = typeof import("$lib/components/notes/NotesView.svelte").default;
  type ChatComponent = typeof import("$lib/components/chat/ChatWorkspace.svelte").default;
  type QuickNotesComponent = typeof import("$lib/components/quick-notes/QuickNotesPanel.svelte").default;
  type SettingsComponent = typeof import("$lib/components/settings/SettingsModal.svelte").default;
  type MusicComponent = typeof import("$lib/components/music/MusicPanel.svelte").default;
  type MusicPlaybackHostComponent = typeof import("$lib/components/music/MusicPlaybackHost.svelte").default;
  type NotesStore = ReturnType<typeof import("$lib/stores/notes.svelte").getNotes>;

  const nav = getNavigation();
  const viewport = getViewport();
  const calendar = getCalendar();
  const calendars = getCalendars();
  const pomodoro = getPomodoro();
  const mobileBackStack = getMobileBackStack();
  getZoom().reapply();
  const { t } = getLocalization();
  const androidSystemBackAvailable = platformHasCapability(
    BUILD_PLATFORM_PROFILE,
    "system.android-back",
  );
  const musicAvailable = platformHasCapability(BUILD_PLATFORM_PROFILE, "view.music");

  let showPomodoro = $state(false);
  let showSettings = $state(false);
  let showQuickNotes = $state(false);
  let showMusic = $state(false);
  let musicLoading = $state(false);
  let musicLoadError = $state("");
  let musicLoadDialog = $state<HTMLDivElement | null>(null);
  let quickNotesLoading = $state(false);
  let quickNotesLoadError = $state("");
  let quickNotesLoadDialog = $state<HTMLDivElement | null>(null);
  let settingsLoadDialog = $state<HTMLDivElement | null>(null);
  let nestedRouteOpen = $state(false);
  let loadError = $state<string | null>(null);
  let backendReady = $state(false);
  let initializingWorkspace = $state(false);
  let CalendarSurface = $state<CalendarComponent | null>(null);
  let ProjectsSurface = $state<ProjectsComponent | null>(null);
  let ProjectMobileListSurface = $state<ProjectMobileListComponent | null>(null);
  let NotesSurface = $state<NotesComponent | null>(null);
  let ChatSurface = $state<ChatComponent | null>(null);
  let QuickNotesSurface = $state<QuickNotesComponent | null>(null);
  let SettingsSurface = $state<SettingsComponent | null>(null);
  let MusicSurface = $state<MusicComponent | null>(null);
  let MusicPlaybackHostSurface = $state<MusicPlaybackHostComponent | null>(null);
  let notesStore = $state.raw<NotesStore | null>(null);
  let notesSurfaceMounted = $state(false);
  let surfaceLoadGeneration = 0;
  let quickNotesLoadGeneration = 0;
  let settingsLoadGeneration = 0;
  let musicLoadGeneration = 0;
  let settingsLoading = $state(false);
  let settingsLoadError = $state("");
  let removePomodoroBackLayer = (): void => undefined;
  let removeSettingsBackLayer = (): void => undefined;
  let removeQuickNotesBackLayer = (): void => undefined;
  let removeMusicBackLayer = (): void => undefined;

  const navigationPresentation = $derived(
    mobileNavigationPresentation(viewport.layoutWidth),
  );
  const useNavigationRail = $derived(navigationPresentation === "rail");
  const suspendInfo = $derived(pomodoro.suspendedAway);
  const suspendDecisionOpen = $derived(suspendInfo !== null);
  const modalOpen = $derived(
    showPomodoro || showSettings || showQuickNotes || showMusic || suspendDecisionOpen,
  );
  const currentTitle = $derived(t(`titleBar.tab.${nav.current}`));
  const shouldInterceptSystemBack = $derived(
    androidSystemBackAvailable && (nestedRouteOpen
      || mobileBackStack.hasActiveLayer
      || nav.current !== "calendar"),
  );

  const backListenerController = new MobileBackListenerController(
    (handler) => onBackButtonPress(handler),
    handleSystemBack,
    (error) => {
      console.error("Failed to update Android back handling", error);
    },
  );
  const persistenceLifecycle = new MobilePersistenceLifecycleController({
    documentTarget: document,
    windowTarget: window,
    flushers: {
      config: flushConfig,
      notes: flushMountedNotes,
      quickNotes: flushQuickNoteEditors,
    },
    onError: (label, error) => {
      console.error(`Failed to flush mobile ${label} persistence`, error);
    },
  });

  function flushMountedNotes(): Promise<void> {
    if (!notesSurfaceMounted || !notesStore) return Promise.resolve();
    return notesStore.flushPendingWrites();
  }

  async function loadSurface(view: View): Promise<void> {
    const generation = ++surfaceLoadGeneration;
    loadError = null;
    try {
      if (view === "calendar" && !CalendarSurface) {
        const module = await import("$lib/components/calendar/CalendarView.svelte");
        if (generation === surfaceLoadGeneration) CalendarSurface = module.default;
      } else if (view === "projects" && !ProjectsSurface) {
        const [storeModule, module, mobileListModule] = await Promise.all([
          import("$lib/stores/projects.svelte"),
          import("$lib/components/projects/ProjectsView.svelte"),
          import("$lib/components/projects/ProjectMobileListView.svelte"),
        ]);
        const projects = storeModule.getProjects();
        projects.activeView = "list";
        await projects.ensureLoaded();
        if (generation === surfaceLoadGeneration) {
          ProjectMobileListSurface = mobileListModule.default;
          ProjectsSurface = module.default;
        }
      } else if (view === "notes" && !NotesSurface) {
        const [storeModule, module] = await Promise.all([
          import("$lib/stores/notes.svelte"),
          import("$lib/components/notes/NotesView.svelte"),
        ]);
        const nextNotesStore = storeModule.getNotes();
        await nextNotesStore.ensureLoaded();
        if (generation === surfaceLoadGeneration) {
          notesStore = nextNotesStore;
          NotesSurface = module.default;
        }
      } else if (view === "chat" && !ChatSurface) {
        const module = await import("$lib/components/chat/ChatWorkspace.svelte");
        if (generation === surfaceLoadGeneration) ChatSurface = module.default;
      }
    } catch (error) {
      if (generation !== surfaceLoadGeneration) return;
      loadError = error instanceof Error ? error.message : String(error);
      console.error(`Failed to load mobile ${view} surface`, error);
    }
  }

  async function initializeWorkspace(): Promise<void> {
    if (initializingWorkspace) return;
    initializingWorkspace = true;
    loadError = null;
    try {
      await ensureDbUrl();
      await pomodoro.recoverMobileRun();
      await Promise.all([
        calendars.load(),
        calendar.load(),
      ]);
      backendReady = true;
      await loadSurface(nav.current);
    } catch (error) {
      loadError = error instanceof Error ? error.message : String(error);
      console.error("Failed to initialize the mobile workspace", error);
    } finally {
      initializingWorkspace = false;
    }
  }

  function clearNestedRoute(): void {
    if (window.location.hash.length === 0) {
      nestedRouteOpen = false;
      return;
    }
    const previousUrl = window.location.href;
    const nextUrl = new URL(previousUrl);
    nextUrl.hash = "";
    window.history.replaceState(window.history.state, "", nextUrl);
    nestedRouteOpen = false;
    window.dispatchEvent(new HashChangeEvent("hashchange", {
      oldURL: previousUrl,
      newURL: nextUrl.href,
    }));
  }

  function navigate(view: View): void {
    if (view !== "notes") clearNestedRoute();
    if (!nav.navigate(view)) return;
    void loadSurface(view);
  }

  function formatAwayDuration(totalSeconds: number): string {
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    if (hours > 0 && minutes > 0) return t("focusDialog.awayHoursMinutes", hours, minutes);
    if (hours > 0) return t("focusDialog.awayHours", hours);
    if (minutes > 0) return t("focusDialog.awayMinutes", minutes);
    return t("focusDialog.awaySeconds", totalSeconds);
  }

  function stopSuspendedSession(): void {
    pomodoro.dismissedBlockId = pomodoro.activeBlockId;
    void pomodoro.dismissSuspend(false);
  }

  function closePomodoro(): void {
    removePomodoroBackLayer();
    removePomodoroBackLayer = () => undefined;
    showPomodoro = false;
  }

  function openPomodoro(): void {
    closeSettings();
    closeQuickNotes();
    closeMusic();
    showPomodoro = true;
    removePomodoroBackLayer();
    removePomodoroBackLayer = mobileBackStack.activate({ handle: closePomodoro });
  }

  function closeSettings(): void {
    settingsLoadGeneration += 1;
    removeSettingsBackLayer();
    removeSettingsBackLayer = () => undefined;
    showSettings = false;
    settingsLoading = false;
    settingsLoadError = "";
  }

  async function loadSettingsSurface(): Promise<void> {
    if (SettingsSurface || settingsLoading) return;
    const generation = ++settingsLoadGeneration;
    settingsLoading = true;
    settingsLoadError = "";
    try {
      const module = await import("$lib/components/settings/SettingsModal.svelte");
      if (generation === settingsLoadGeneration) SettingsSurface = module.default;
    } catch (error) {
      if (generation !== settingsLoadGeneration) return;
      settingsLoadError = error instanceof Error ? error.message : String(error);
      console.error("Failed to load the mobile settings surface", error);
    } finally {
      if (generation === settingsLoadGeneration) settingsLoading = false;
    }
  }

  function openSettings(): void {
    closePomodoro();
    closeQuickNotes();
    closeMusic();
    showSettings = true;
    removeSettingsBackLayer();
    removeSettingsBackLayer = mobileBackStack.activate({ handle: closeSettings });
    void loadSettingsSurface();
  }

  function closeQuickNotes(): void {
    quickNotesLoadGeneration += 1;
    removeQuickNotesBackLayer();
    removeQuickNotesBackLayer = () => undefined;
    showQuickNotes = false;
    quickNotesLoading = false;
    quickNotesLoadError = "";
  }

  async function loadQuickNotesSurface(): Promise<void> {
    if (QuickNotesSurface || quickNotesLoading) return;
    const generation = ++quickNotesLoadGeneration;
    quickNotesLoading = true;
    quickNotesLoadError = "";
    try {
      const module = await import("$lib/components/quick-notes/QuickNotesPanel.svelte");
      if (generation === quickNotesLoadGeneration) QuickNotesSurface = module.default;
    } catch (error) {
      if (generation !== quickNotesLoadGeneration) return;
      quickNotesLoadError = error instanceof Error ? error.message : String(error);
      console.error("Failed to load the mobile Quick notes surface", error);
    } finally {
      if (generation === quickNotesLoadGeneration) quickNotesLoading = false;
    }
  }

  function openQuickNotes(): void {
    if (showQuickNotes || !backendReady) return;
    closePomodoro();
    closeSettings();
    closeMusic();
    showQuickNotes = true;
    removeQuickNotesBackLayer();
    removeQuickNotesBackLayer = mobileBackStack.activate({ handle: closeQuickNotes });
    void loadQuickNotesSurface();
  }

  function closeMusic(): void {
    musicLoadGeneration += 1;
    removeMusicBackLayer();
    removeMusicBackLayer = () => undefined;
    showMusic = false;
    musicLoading = false;
    musicLoadError = "";
  }

  async function loadMusicSurface(): Promise<void> {
    if ((MusicSurface && MusicPlaybackHostSurface) || musicLoading) return;
    const generation = ++musicLoadGeneration;
    musicLoading = true;
    musicLoadError = "";
    try {
      const [panelModule, hostModule] = await Promise.all([
        import("$lib/components/music/MusicPanel.svelte"),
        import("$lib/components/music/MusicPlaybackHost.svelte"),
      ]);
      if (generation === musicLoadGeneration) {
        MusicSurface = panelModule.default;
        MusicPlaybackHostSurface = hostModule.default;
      }
    } catch (error) {
      if (generation !== musicLoadGeneration) return;
      musicLoadError = error instanceof Error ? error.message : String(error);
      console.error("Failed to load the mobile Music surface", error);
    } finally {
      if (generation === musicLoadGeneration) musicLoading = false;
    }
  }

  function openMusic(): void {
    if (!musicAvailable || showMusic || !backendReady) return;
    closePomodoro();
    closeSettings();
    closeQuickNotes();
    showMusic = true;
    removeMusicBackLayer();
    removeMusicBackLayer = mobileBackStack.activate({ handle: closeMusic });
    void loadMusicSurface();
  }

  function handleSystemBack(): void {
    const action = resolveMobileBackAction({
      featureLayerOpen: mobileBackStack.hasActiveLayer,
      nestedRouteOpen,
      currentView: nav.current,
    });
    if (action === "consume-feature-layer") {
      mobileBackStack.consume();
    } else if (action === "close-nested-route") {
      clearNestedRoute();
    } else if (action === "navigate-calendar") {
      navigate("calendar");
    } else {
      void backListenerController.setEnabled(false);
    }
  }

  onMount(() => {
    const detachPersistenceLifecycle = persistenceLifecycle.attach();
    const syncNestedRoute = (): void => {
      nestedRouteOpen = window.location.hash.length > 0;
    };
    syncNestedRoute();
    window.addEventListener("hashchange", syncNestedRoute);
    window.addEventListener("popstate", syncNestedRoute);

    void initializeWorkspace();

    return () => {
      window.removeEventListener("hashchange", syncNestedRoute);
      window.removeEventListener("popstate", syncNestedRoute);
      removePomodoroBackLayer();
      removeSettingsBackLayer();
      removeQuickNotesBackLayer();
      removeMusicBackLayer();
      detachPersistenceLifecycle();
      void persistenceLifecycle.flush();
      void backListenerController.dispose();
    };
  });

  $effect(() => {
    void backListenerController.setEnabled(shouldInterceptSystemBack);
  });

  $effect(() => {
    const view = nav.current;
    if (!backendReady) return;
    void loadSurface(view);
  });

  $effect(() => {
    if (nav.current !== "notes" || !NotesSurface || !notesStore) return;
    notesSurfaceMounted = true;
  });

  $effect(() => {
    if (!showQuickNotes || QuickNotesSurface || !quickNotesLoadDialog) return;
    return activateModalFocus(quickNotesLoadDialog);
  });

  $effect(() => {
    if (!showSettings || SettingsSurface || !settingsLoadDialog) return;
    return activateModalFocus(settingsLoadDialog);
  });

  $effect(() => {
    if (!showMusic || MusicSurface || !musicLoadDialog) return;
    return activateModalFocus(musicLoadDialog);
  });
</script>

<div
  class="mobile-app-shell mobile-viewport-height flex w-screen flex-col overflow-hidden bg-background text-foreground"
  data-size-class={viewport.sizeClass}
  style="width: calc(100vw * var(--mobile-interface-scale-inverse, 1)); height: calc(100vh * var(--mobile-interface-scale-inverse, 1)); height: calc(100dvh * var(--mobile-interface-scale-inverse, 1)); padding: var(--safe-area-top) var(--safe-area-right) {useNavigationRail ? 'var(--safe-area-bottom)' : '0'} var(--safe-area-left);"
>
  <div
    class="flex min-h-0 flex-1 flex-col"
    inert={modalOpen}
    aria-hidden={modalOpen ? "true" : undefined}
  >
    <MobileTopBar
      title={currentTitle}
      pomodoroTime={pomodoro.formattedTime}
      pomodoroActive={pomodoro.isActive}
      quickNotesOpen={showQuickNotes}
      quickNotesLoading={quickNotesLoading}
      quickNotesDisabled={!backendReady}
      musicOpen={showMusic}
      musicLoading={musicLoading}
      musicDisabled={!backendReady}
      musicVisible={musicAvailable}
      onOpenPomodoro={openPomodoro}
      onOpenQuickNotes={openQuickNotes}
      onOpenMusic={openMusic}
      onOpenSettings={openSettings}
    />

    <div class="flex min-h-0 flex-1 overflow-hidden">
      {#if useNavigationRail}
        <MobileNavigation current={nav.current} presentation="rail" onNavigate={navigate} />
      {/if}

      <main class="relative min-h-0 min-w-0 flex-1 overflow-hidden bg-background">
        {#if loadError}
          <div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center" role="alert">
            <p class="text-sm font-medium">{t("common.viewLoadFailed", currentTitle)}</p>
            <p class="max-w-md text-xs text-muted-foreground">{loadError}</p>
            <button
              type="button"
              disabled={initializingWorkspace}
              class="min-h-12 rounded-xl border border-border bg-card px-5 text-sm font-medium active:bg-accent"
              onclick={() => {
                if (backendReady) void loadSurface(nav.current);
                else void initializeWorkspace();
              }}
            >
              {initializingWorkspace ? t("common.loading") : t("common.retry")}
            </button>
          </div>
        {:else if nav.current === "calendar" && CalendarSurface}
          <CalendarSurface initialViewMode="day" mobileLayout />
        {:else if nav.current === "projects" && ProjectsSurface && ProjectMobileListSurface}
          <ProjectsSurface mobileLayout mobileListComponent={ProjectMobileListSurface} />
        {:else if nav.current === "notes" && NotesSurface}
          <NotesSurface mobileLayout />
        {:else if nav.current === "chat" && ChatSurface}
          <ChatSurface />
        {:else}
          <div class="flex h-full items-center justify-center text-sm text-muted-foreground" aria-busy="true">
            {t("common.loading")}
          </div>
        {/if}
      </main>
    </div>

    {#if !useNavigationRail}
      <div class="shrink-0 bg-sidebar" style="padding-bottom: var(--safe-area-bottom);">
        <MobileNavigation current={nav.current} presentation="bottom" onNavigate={navigate} />
      </div>
    {/if}
  </div>

  {#if MusicPlaybackHostSurface}
    <MusicPlaybackHostSurface />
  {/if}

  {#if showPomodoro}
    <div inert={suspendDecisionOpen} aria-hidden={suspendDecisionOpen ? "true" : undefined}>
      <MobilePomodoroSheet
        onClose={closePomodoro}
        onOpenCalendar={() => {
          closePomodoro();
          navigate("calendar");
        }}
      />
    </div>
  {/if}

  {#if showMusic && MusicSurface}
    <div inert={suspendDecisionOpen} aria-hidden={suspendDecisionOpen ? "true" : undefined}>
      <MusicSurface presentation="mobile" onclose={closeMusic} />
    </div>
  {:else if showMusic}
    <div
      class="fixed z-50 flex items-center justify-center bg-background/95"
      inert={suspendDecisionOpen}
      aria-hidden={suspendDecisionOpen ? "true" : undefined}
      style="left: var(--visual-viewport-offset-left); top: var(--visual-viewport-offset-top); width: var(--visual-viewport-width); height: var(--visual-viewport-height); padding: calc(var(--safe-area-top) + 1rem) calc(var(--safe-area-right) + 1rem) calc(var(--safe-area-bottom) + 1rem) calc(var(--safe-area-left) + 1rem);"
    >
      <div
        bind:this={musicLoadDialog}
        class="flex w-full max-w-sm flex-col items-center gap-3 rounded-2xl border border-border bg-card p-5 text-center text-card-foreground outline-none"
        role="dialog"
        aria-modal="true"
        aria-label={t("music.title")}
        tabindex="-1"
      >
        {#if musicLoadError}
          <p class="text-sm font-medium" role="alert">{t("common.viewLoadFailed", t("music.title"))}</p>
          <p class="max-w-full wrap-break-word text-xs text-muted-foreground">{musicLoadError}</p>
          <button type="button" class="min-h-12 w-full rounded-xl border border-border px-4 text-sm font-medium active:bg-accent" onclick={() => void loadMusicSurface()}>{t("common.retry")}</button>
        {:else}
          <p class="text-sm text-muted-foreground" aria-busy="true">{t("common.loading")}</p>
        {/if}
        <button type="button" class="min-h-12 w-full rounded-xl px-4 text-sm font-medium active:bg-accent" onclick={closeMusic}>{t("common.close")}</button>
      </div>
    </div>
  {/if}

  {#if showSettings && SettingsSurface}
    <div inert={suspendDecisionOpen} aria-hidden={suspendDecisionOpen ? "true" : undefined}>
      <SettingsSurface presentation="mobile" onClose={closeSettings} />
    </div>
  {:else if showSettings}
    <div
      class="fixed z-50 flex items-center justify-center bg-background/95"
      inert={suspendDecisionOpen}
      aria-hidden={suspendDecisionOpen ? "true" : undefined}
      style="left: var(--visual-viewport-offset-left); top: var(--visual-viewport-offset-top); width: var(--visual-viewport-width); height: var(--visual-viewport-height); padding: calc(var(--safe-area-top) + 1rem) calc(var(--safe-area-right) + 1rem) calc(var(--safe-area-bottom) + 1rem) calc(var(--safe-area-left) + 1rem);"
    >
      <div
        bind:this={settingsLoadDialog}
        class="flex w-full max-w-sm flex-col items-center gap-3 rounded-2xl border border-border bg-card p-5 text-center text-card-foreground outline-none"
        role="dialog"
        aria-modal="true"
        aria-label={t("settings.title")}
        tabindex="-1"
      >
        {#if settingsLoadError}
          <p class="text-sm font-medium" role="alert">{t("common.viewLoadFailed", t("settings.title"))}</p>
          <p class="max-w-full wrap-break-word text-xs text-muted-foreground">{settingsLoadError}</p>
          <button
            type="button"
            class="min-h-12 w-full rounded-xl border border-border px-4 text-sm font-medium active:bg-accent"
            onclick={() => void loadSettingsSurface()}
          >{t("common.retry")}</button>
        {:else}
          <p class="text-sm text-muted-foreground" aria-busy="true">{t("common.loading")}</p>
        {/if}
        <button
          type="button"
          class="min-h-12 w-full rounded-xl px-4 text-sm font-medium active:bg-accent"
          onclick={closeSettings}
        >{t("common.close")}</button>
      </div>
    </div>
  {/if}

  {#if showQuickNotes && QuickNotesSurface}
    <div inert={suspendDecisionOpen} aria-hidden={suspendDecisionOpen ? "true" : undefined}>
      <QuickNotesSurface mobileLayout onclose={closeQuickNotes} />
    </div>
  {:else if showQuickNotes}
    <div
      class="fixed z-50 flex items-center justify-center bg-background/95"
      inert={suspendDecisionOpen}
      aria-hidden={suspendDecisionOpen ? "true" : undefined}
      style="left: var(--visual-viewport-offset-left); top: var(--visual-viewport-offset-top); width: var(--visual-viewport-width); height: var(--visual-viewport-height); padding: calc(var(--safe-area-top) + 1rem) calc(var(--safe-area-right) + 1rem) calc(var(--safe-area-bottom) + 1rem) calc(var(--safe-area-left) + 1rem);"
    >
      <div
        bind:this={quickNotesLoadDialog}
        class="flex w-full max-w-sm flex-col items-center gap-3 rounded-2xl border border-border bg-card p-5 text-center text-card-foreground outline-none"
        role="dialog"
        aria-modal="true"
        aria-label={t("quickNotes.title")}
        tabindex="-1"
      >
        {#if quickNotesLoadError}
          <p class="text-sm font-medium" role="alert">{t("quickNotes.loadFailed")}</p>
          <p class="max-w-full wrap-break-word text-xs text-muted-foreground">{quickNotesLoadError}</p>
          <button
            type="button"
            class="min-h-12 w-full rounded-xl border border-border px-4 text-sm font-medium active:bg-accent"
            onclick={() => void loadQuickNotesSurface()}
          >{t("quickNotes.retry")}</button>
        {:else}
          <p class="text-sm text-muted-foreground" aria-busy="true">{t("common.loading")}</p>
        {/if}
        <button
          type="button"
          class="min-h-12 w-full rounded-xl px-4 text-sm font-medium active:bg-accent"
          onclick={closeQuickNotes}
        >{t("common.close")}</button>
      </div>
    </div>
  {/if}

  {#if suspendInfo}
    <ConfirmDialog
      title={t("focusDialog.resumeTitle")}
      message={t("focusDialog.awayMessage", formatAwayDuration(suspendInfo.awaySeconds))}
      confirmLabel={t("focusDialog.resume")}
      cancelLabel={t("focusDialog.stopSessionCancel")}
      danger={false}
      onConfirm={() => { void pomodoro.dismissSuspend(true); }}
      onCancel={stopSuspendedSession}
      onDismiss={() => undefined}
    />
  {/if}
</div>
