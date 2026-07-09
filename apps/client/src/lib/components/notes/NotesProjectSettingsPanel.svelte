<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    isNotesPageOpenMode,
    type NotesPageOpenMode,
  } from "$lib/notes/page-open-mode";
  import { projectToolbarPanelGeometry } from "$lib/projects/project-toolbar";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import {
    APP_FLOATING_SURFACE_SELECTOR,
    isAppFloatingSurfaceTarget,
  } from "$lib/utils";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import ProjectSettingsPanelShell from "$lib/components/projects/ProjectSettingsPanelShell.svelte";
  import ProjectSettingsSectionHeading from "$lib/components/projects/ProjectSettingsSectionHeading.svelte";

  type NotesProjectDefaultOpenMode = "default" | NotesPageOpenMode;

  let {
    projectId,
    popoverBoundaryElement,
    onClose,
  }: {
    projectId: string;
    popoverBoundaryElement: HTMLElement | null;
    onClose: () => void;
  } = $props();

  const projects = getProjects();
  const viewport = getViewport();
  const { t } = getLocalization();

  let panelElement = $state<HTMLDivElement | null>(null);
  let scrollElement = $state<HTMLElement | undefined>();
  let panelStyle = $state("");
  let panelGeometryFrame: number | null = null;
  let draftProjectId = $state<string | null>(null);
  let draftProjectUpdatedAt = $state<string | null>(null);
  let defaultOpenModeDraft = $state<NotesProjectDefaultOpenMode>("default");
  let saving = $state(false);
  let error = $state<string | null>(null);

  const selectedProject = $derived(projects.projectById(projectId));
  const draftReady = $derived(Boolean(selectedProject && draftProjectId === selectedProject.id));
  const persistedDefaultOpenMode = $derived<NotesProjectDefaultOpenMode>(
    selectedProject?.notesDefaultOpenMode ?? "default",
  );
  const dirty = $derived(draftReady && defaultOpenModeDraft !== persistedDefaultOpenMode);
  const openModeOptions = $derived([
    { value: "default", label: t("notes.projectSettingsUseGlobal") },
    { value: "center", label: t("notes.centerPeek") },
    { value: "side", label: t("notes.sidePeek") },
    { value: "full", label: t("notes.fullPage") },
  ]);

  function loadDraft(): void {
    if (!selectedProject) return;
    draftProjectId = selectedProject.id;
    draftProjectUpdatedAt = selectedProject.updatedAt;
    defaultOpenModeDraft = selectedProject.notesDefaultOpenMode ?? "default";
    error = null;
  }

  function refreshPanelGeometry(): void {
    panelGeometryFrame = null;
    const trigger = document.querySelector<HTMLElement>("[data-notes-toolbar-trigger='settings']");
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    const geometry = projectToolbarPanelGeometry({
      anchorLeft: rect.left,
      anchorRight: rect.right,
      anchorTop: rect.top,
      anchorBottom: rect.bottom,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
      preferredWidth: 430,
      preferredHeight: 260,
    });
    panelStyle = [
      `left: ${Math.round(geometry.left)}px`,
      `top: ${Math.round(geometry.top)}px`,
      `width: ${Math.round(geometry.width)}px`,
      `height: ${Math.round(geometry.maxHeight)}px`,
      `max-height: ${Math.round(geometry.maxHeight)}px`,
    ].join("; ");
  }

  function requestPanelGeometryRefresh(): void {
    if (panelGeometryFrame !== null) cancelAnimationFrame(panelGeometryFrame);
    panelGeometryFrame = requestAnimationFrame(refreshPanelGeometry);
  }

  function discard(): void {
    loadDraft();
  }

  function close(): void {
    loadDraft();
    onClose();
  }

  function selectDefaultOpenMode(value: string): void {
    if (value === "default" || isNotesPageOpenMode(value)) {
      defaultOpenModeDraft = value;
    }
  }

  async function save(): Promise<void> {
    if (!selectedProject || !dirty || saving) return;
    saving = true;
    error = null;
    try {
      const openMode = defaultOpenModeDraft === "default" ? null : defaultOpenModeDraft;
      await projects.setNotesDefaultOpenMode(selectedProject.id, openMode);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      error = t("notes.projectSettingsSaveFailed", message);
    } finally {
      saving = false;
    }
  }

  function handleWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    const trigger = document.querySelector<HTMLElement>("[data-notes-toolbar-trigger='settings']");
    if (isAppFloatingSurfaceTarget(target) || trigger?.contains(target) || panelElement?.contains(target)) {
      return;
    }
    close();
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape") return;
    if (isAppFloatingSurfaceTarget(event.target) || document.querySelector(APP_FLOATING_SURFACE_SELECTOR)) {
      return;
    }
    event.preventDefault();
    close();
  }

  $effect(() => {
    if (!selectedProject) return;
    if (
      draftProjectId !== selectedProject.id
      || (!dirty && draftProjectUpdatedAt !== selectedProject.updatedAt)
    ) {
      loadDraft();
    }
  });

  $effect(() => {
    const viewportWidth = viewport.width;
    const viewportHeight = viewport.height;
    void viewportWidth;
    void viewportHeight;
    requestPanelGeometryRefresh();
    return () => {
      if (panelGeometryFrame !== null) {
        cancelAnimationFrame(panelGeometryFrame);
        panelGeometryFrame = null;
      }
    };
  });
</script>

<svelte:window onpointerdown={handleWindowPointerDown} onkeydown={handleWindowKeydown} />

{#if selectedProject}
  <div
    bind:this={panelElement}
    class="fixed z-80 flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-card text-[0.8rem] text-foreground shadow-xl"
    style={panelStyle}
    role="dialog"
    tabindex="-1"
    aria-label={t("notes.projectSettingsTitle")}
    data-app-shortcuts="ignore"
  >
    <ProjectSettingsPanelShell
      presentation="popover"
      {draftReady}
      {dirty}
      {saving}
      {error}
      title={t("notes.projectSettingsTitle")}
      discardLabel={t("projects.settings.discard")}
      closeLabel={t("projects.settings.close")}
      saveLabel={t("projects.settings.save")}
      onDiscard={discard}
      onClose={close}
      onSave={() => { void save(); }}
      bind:scrollElement
    >
      <section class="flex flex-col gap-2">
        <div class="h-px bg-border/70" aria-hidden="true"></div>
        <div class="flex flex-col gap-1.5">
          <ProjectSettingsSectionHeading label={t("settings.section.appearance")} />
          <CustomSelect
            label={t("notes.projectSettingsDefaultViewMode")}
            value={defaultOpenModeDraft}
            options={openModeOptions}
            onChange={selectDefaultOpenMode}
            {popoverBoundaryElement}
            class="w-44"
          />
        </div>
      </section>
    </ProjectSettingsPanelShell>
  </div>
{/if}
