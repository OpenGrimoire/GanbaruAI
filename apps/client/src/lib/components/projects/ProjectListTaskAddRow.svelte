<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { projectListAddRowInputFocus } from "$lib/projects/project-list-add-row-focus";

  type MaybePromise = Promise<void> | void;
  type TaskAddMode = "section" | "group";

  let {
    mode,
    rowId,
    gridTemplate,
    gridMinWidth,
    label,
    draft,
    active,
    pending,
    error = null,
    onDraftChange,
    onActiveChange,
    onSubmit,
  }: {
    mode: TaskAddMode;
    rowId: string;
    gridTemplate: string;
    gridMinWidth: string;
    label: string;
    draft: string;
    active: boolean;
    pending: boolean;
    error?: string | null;
    onDraftChange: (value: string) => void;
    onActiveChange: (active: boolean) => void;
    onSubmit: () => MaybePromise;
  } = $props();

  const { t } = getLocalization();

  const inputSelector = $derived(
    mode === "section" ? "[data-section-task-input]" : "[data-group-task-input]",
  );
</script>

<div
  class="project-list-divider grid cursor-text items-center px-1 py-1.5"
  data-section-task-add-row={mode === "section" ? rowId : undefined}
  data-group-task-add-row={mode === "group" ? rowId : undefined}
  style={`grid-template-columns: ${gridTemplate}; min-width: ${gridMinWidth};`}
  use:projectListAddRowInputFocus={{ selector: inputSelector }}
>
  <div class="absolute inset-0 z-0 cursor-text" aria-hidden="true"></div>
  <div class="relative z-10"></div>
  <div class="relative z-10"></div>
  <form
    class="contents"
    onsubmit={(event) => { event.preventDefault(); void onSubmit(); }}
  >
    <div class="relative z-10 min-w-0 px-2" style="grid-column: 3;">
      {#if !draft.trim() && !active}
        <div
          class="pointer-events-none absolute inset-y-0 left-2 flex items-center gap-2 text-muted-foreground"
          aria-hidden="true"
        >
          <span class="flex h-5 w-5 shrink-0 items-center justify-center">
            <Plus size={15} strokeWidth={1.75} />
          </span>
          <span class="text-[0.866667rem]">{label}</span>
        </div>
      {/if}
      <input
        data-section-task-input={mode === "section" ? rowId : undefined}
        data-group-task-input={mode === "group" ? rowId : undefined}
        aria-label={label}
        value={draft}
        onfocus={() => onActiveChange(true)}
        onblur={() => onActiveChange(false)}
        oninput={(event) => onDraftChange(event.currentTarget.value)}
        class="min-h-8 w-full min-w-0 bg-transparent text-[0.866667rem] text-foreground"
      />
      {#if active && !draft.trim()}
        <span
          class="project-list-add-row-caret pointer-events-none absolute left-2 top-1/2 h-4 w-px -translate-y-1/2 bg-foreground"
          aria-hidden="true"
        ></span>
      {/if}
    </div>
    {#if draft.trim()}
      <div class="relative z-10 flex min-w-0 items-center px-2" style="grid-column: 4;">
        <button
          type="submit"
          disabled={pending}
          class="flex h-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border bg-card px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-60"
        >
          {t("projects.list.saveWithEnter")}
        </button>
      </div>
    {/if}
  </form>
  {#if error}
    <div class="col-span-full rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
      {error}
    </div>
  {/if}
</div>
