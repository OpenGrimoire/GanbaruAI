<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectTagColorDotStyle,
    projectTagColorSwatchClass,
  } from "$lib/projects/project-display";
  import type { ProjectTag, ProjectTask } from "$lib/projects/types";
  import type { Theme } from "$lib/stores/themes";
  import { cn } from "$lib/utils";

  type ActionResult = void | Promise<void>;

  let {
    task,
    tags,
    candidates,
    draft,
    canCreate,
    theme,
    onDraftChange,
    onAttachTag,
    onSubmitTag,
    onDetachTag,
  }: {
    task: ProjectTask;
    tags: ProjectTag[];
    candidates: ProjectTag[];
    draft: string;
    canCreate: boolean;
    theme: Theme;
    onDraftChange: (value: string) => void;
    onAttachTag: (task: ProjectTask, tag: ProjectTag) => ActionResult;
    onSubmitTag: (task: ProjectTask) => ActionResult;
    onDetachTag: (task: ProjectTask, tag: ProjectTag) => ActionResult;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="flex items-center justify-between gap-2">
  <h2 class="text-[0.8rem] font-semibold tracking-normal">{t("projects.detail.tags")}</h2>
  <span class="text-[0.733333rem] text-muted-foreground">{tags.length}</span>
</div>
{#if tags.length > 0}
  <div class="flex flex-wrap gap-1">
    {#each tags as tag (tag.id)}
      <span class="inline-flex min-h-7 max-w-full items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.766667rem]">
        <span
          class={cn("h-2 w-2 shrink-0 rounded-full border", projectTagColorSwatchClass(tag.color))}
          style={projectTagColorDotStyle(tag.color, theme)}
        ></span>
        <span class="truncate">{tag.name}</span>
        <button
          type="button"
          class="flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
          aria-label={t("projects.actions.removeTag", tag.name)}
          title={t("projects.actions.removeTag", tag.name)}
          onclick={() => { void onDetachTag(task, tag); }}
        >
          <X size={12} strokeWidth={1.75} />
        </button>
      </span>
    {/each}
  </div>
{:else}
  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
    {t("projects.detail.noTags")}
  </div>
{/if}
<div class="grid gap-1">
  <div class="flex gap-1">
    <input
      value={draft}
      placeholder={t("projects.detail.addTagPlaceholder")}
      class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
      oninput={(event) => onDraftChange(event.currentTarget.value)}
      onkeydown={(event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          void onSubmitTag(task);
        }
      }}
    />
    <button
      type="button"
      class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
      aria-label={t("projects.detail.addTag")}
      onclick={() => { void onSubmitTag(task); }}
    >
      <Plus size={14} strokeWidth={1.75} />
    </button>
  </div>
  <div class="grid gap-1">
    {#each candidates as tag (tag.id)}
      <button
        type="button"
        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
        onclick={() => { void onAttachTag(task, tag); }}
      >
        <span class="flex min-w-0 items-center gap-2">
          <span
            class={cn("h-2 w-2 shrink-0 rounded-full border", projectTagColorSwatchClass(tag.color))}
            style={projectTagColorDotStyle(tag.color, theme)}
          ></span>
          <span class="truncate text-[0.8rem]">{tag.name}</span>
        </span>
        <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
      </button>
    {:else}
      {#if canCreate}
        <button
          type="button"
          class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
          onclick={() => { void onSubmitTag(task); }}
        >
          <span class="truncate text-[0.8rem]">{t("projects.detail.createTag", draft.trim())}</span>
          <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
        </button>
      {:else}
        <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
          {t("projects.detail.noTagCandidates")}
        </div>
      {/if}
    {/each}
  </div>
</div>
