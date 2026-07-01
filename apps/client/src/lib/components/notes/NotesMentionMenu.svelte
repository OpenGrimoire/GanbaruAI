<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesMentionOptionDomId } from "$lib/notes/editor-accessibility";
  import type { NotesMentionTarget } from "$lib/notes/rich-text";
  import Bell from "@lucide/svelte/icons/bell";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import FileText from "@lucide/svelte/icons/file-text";

  let {
    menuId,
    blockId,
    targets,
    activeIndex,
    onSelect,
  }: {
    menuId: string;
    blockId: string;
    targets: NotesMentionTarget[];
    activeIndex: number;
    onSelect: (target: NotesMentionTarget) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div
  id={menuId}
  class="absolute left-1 top-full z-30 mt-1 w-min min-w-56 max-w-72 overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-lg"
  role="listbox"
  aria-label={t("notes.mentionMenu")}
  aria-live="polite"
>
  {#if targets.length === 0}
    <div class="px-2 py-2 text-[0.8rem] text-muted-foreground" role="status">
      {t("notes.noMentionResults")}
    </div>
  {:else}
    {#each targets as target, index (target.id)}
      <button
        id={notesMentionOptionDomId(blockId, index)}
        type="button"
        class={`flex min-h-9 w-full min-w-0 items-center gap-2 rounded px-2 py-1.5 text-left text-[0.866667rem] ${
          index === activeIndex ? "bg-accent text-accent-foreground" : "hover:bg-accent"
        }`}
        role="option"
        aria-selected={index === activeIndex}
        onclick={() => onSelect(target)}
      >
        <span class="flex size-5 shrink-0 items-center justify-center rounded bg-muted text-[0.8rem] text-muted-foreground">
          {#if target.kind === "date" && target.reminder}
            <Bell class="size-3.5" aria-hidden="true" />
          {:else if target.kind === "date"}
            <CalendarDays class="size-3.5" aria-hidden="true" />
          {:else if target.iconText}
            <span aria-hidden="true">{target.iconText}</span>
          {:else}
            <FileText class="size-3.5" aria-hidden="true" />
          {/if}
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-foreground">{target.title}</span>
          {#if target.subtitle}
            <span class="block truncate text-[0.733333rem] text-muted-foreground">{target.subtitle}</span>
          {/if}
        </span>
      </button>
    {/each}
  {/if}
</div>
