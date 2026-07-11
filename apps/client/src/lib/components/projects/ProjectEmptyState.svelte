<script lang="ts">
  import Folder from "@lucide/svelte/icons/folder";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import LazyProjectNavigator from "./LazyProjectNavigator.svelte";

  let {
    selectedProjectId,
    showInactiveProjects = $bindable<boolean>(),
    onProjectSelected,
  }: {
    selectedProjectId: string | null;
    showInactiveProjects: boolean;
    onProjectSelected: () => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();

  let navigatorOpen = $state(false);
</script>

<div
  class="flex h-full flex-col items-center justify-center gap-3 p-4 text-center text-[0.866667rem] text-muted-foreground"
  data-projects-first-use-state
  aria-busy={projects.loading}
>
  <div>{projects.loading ? t("projects.loading") : t("projects.navigator.empty")}</div>
  {#if !projects.loading}
    <button
      type="button"
      class="flex min-h-9 items-center gap-1.5 rounded-md border border-border bg-background px-3 text-[0.8rem] font-medium text-foreground hover:bg-accent"
      onclick={() => {
        navigatorOpen = !navigatorOpen;
      }}
    >
      <Folder size={14} strokeWidth={1.75} />
      <span>{t("projects.navigator.open")}</span>
    </button>
    {#if navigatorOpen}
      <div class="h-[min(24rem,70vh)] w-[min(28rem,100%)] text-left">
        <LazyProjectNavigator
          {selectedProjectId}
          {showInactiveProjects}
          onShowInactiveProjectsChange={(value) => {
            showInactiveProjects = value;
          }}
          onProjectSelected={() => {
            navigatorOpen = false;
            onProjectSelected();
          }}
        />
      </div>
    {/if}
  {/if}
</div>
