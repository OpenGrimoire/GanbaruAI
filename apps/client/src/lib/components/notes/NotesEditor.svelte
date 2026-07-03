<script lang="ts">
  import { tick } from "svelte";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { dismissOnOutside } from "$lib/utils/dismiss-on-outside";
  import { addNotesWorkspaceBreadcrumb, buildNotesPageBreadcrumb } from "$lib/notes/page-breadcrumb";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { buildNotesTableOfContents } from "$lib/notes/table-of-contents";
  import type { NotesPageIcon as NotesPageIconValue } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import SmilePlus from "@lucide/svelte/icons/smile-plus";
  import Star from "@lucide/svelte/icons/star";
  import NotesBacklinks from "./NotesBacklinks.svelte";
  import NotesBlockList from "./NotesBlockList.svelte";
  import NotesComments from "./NotesComments.svelte";
  import NotesPageCover from "./NotesPageCover.svelte";
  import NotesPageCoverMenu from "./NotesPageCoverMenu.svelte";
  import NotesPageHistory from "./NotesPageHistory.svelte";
  import NotesPageIcon from "./NotesPageIcon.svelte";
  import NotesPageIconMenu from "./NotesPageIconMenu.svelte";
  import NotesPageLinks from "./NotesPageLinks.svelte";
  import NotesSuggestions from "./NotesSuggestions.svelte";

  const notes = getNotes();
  const localization = getLocalization();
  const { t } = localization;
  let titleDraft = $state("");
  let titleInput: HTMLInputElement | null = $state(null);
  let iconMenuOpen = $state(false);
  let coverMenuOpen = $state(false);
  let blockScrollViewport: HTMLDivElement | null = $state(null);
  let lastTitlePageId = "";
  let lastHandledTitleFocusRequestId = 0;
  const page = $derived(notes.loadedPage);
  const editablePageTitle = $derived(page ? notesPageTitle(page, "") : "");
  const pageIconLabel = $derived(pageIconScreenReaderText(page?.icon ?? null));
  const breadcrumbItems = $derived(
    notes.pageBreadcrumbItems.length > 0
      ? addNotesWorkspaceBreadcrumb(
          notes.pageBreadcrumbItems,
          t("notes.workspace"),
          t("notes.untitled"),
          t("notes.breadcrumbMissingPage"),
        )
      : buildNotesPageBreadcrumb(page, notes.pages, t("notes.workspace"), t("notes.untitled")),
  );
  const tableOfContentsItems = $derived(buildNotesTableOfContents(notes.flatBlocks));
  const pageFavorited = $derived(page ? notes.favoritePageIds.includes(page.id) : false);
  const editedDateLabel = $derived(
    page ? new Date(page.last_edited_time).toLocaleString(localization.locale) : "",
  );

  $effect(() => {
    if (!page || page.id === lastTitlePageId) return;
    lastTitlePageId = page.id;
    titleDraft = editablePageTitle;
  });

  $effect(() => {
    const titleFocusRequestId = notes.titleFocusRequestId;
    if (titleFocusRequestId === lastHandledTitleFocusRequestId) return;
    if (!page || notes.titleFocusPageId !== page.id) return;
    lastHandledTitleFocusRequestId = titleFocusRequestId;
    void tick().then(() => {
      titleInput?.focus();
      const offset = titleInput?.value.length ?? 0;
      titleInput?.setSelectionRange(offset, offset);
    });
  });

  $effect(() => {
    const _focusRequestId = notes.focusRequestId;
    const focusSelection = notes.focusSelection;
    const blockId = notes.focusBlockId;
    if (!blockId) return;
    void tick().then(() => {
      const anchor = blockScrollViewport?.querySelector<HTMLElement>(
        `#${CSS.escape(notesBlockAnchorId(blockId))}`,
      );
      anchor?.scrollIntoView({
        block: focusSelection ? "nearest" : "center",
        inline: "nearest",
      });
    });
  });

  async function saveTitle(): Promise<void> {
    if (!page) return;
    const title = titleDraft.trim();
    titleDraft = title;
    if (title === editablePageTitle) return;
    await notes.renamePage(page.id, title);
  }

  function handleTitleKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      event.currentTarget instanceof HTMLInputElement && event.currentTarget.blur();
    }
  }

  function toggleCoverMenu(): void {
    const nextOpen = !coverMenuOpen;
    coverMenuOpen = nextOpen;
    if (nextOpen) iconMenuOpen = false;
  }

  function toggleIconMenu(): void {
    const nextOpen = !iconMenuOpen;
    iconMenuOpen = nextOpen;
    if (nextOpen) coverMenuOpen = false;
  }

  function closeCoverMenu(): void {
    coverMenuOpen = false;
  }

  function closeIconMenu(): void {
    iconMenuOpen = false;
  }

  function pageIconScreenReaderText(icon: NotesPageIconValue | null): string {
    if (!icon) return t("notes.noPageIcon");
    if (icon.type === "emoji") return icon.emoji;
    if (icon.type === "icon") return icon.icon.name;
    if (icon.type === "custom_emoji") return icon.custom_emoji.name ?? t("notes.pageIconCustomEmoji");
    return icon.type === "external" ? t("notes.pageIconImage") : icon.file.name ?? t("notes.pageIconImage");
  }
</script>

{#if page}
  <section class="flex min-w-0 flex-1 flex-col overflow-hidden">
    {#if page.cover}
      <div class="group relative h-24 shrink-0 overflow-hidden bg-muted sm:h-36">
        <NotesPageCover cover={page.cover} unavailableLabel={t("notes.pageCoverUnavailable")} />
        <div
          class="absolute right-3 top-3"
          use:dismissOnOutside={{ enabled: coverMenuOpen, onDismiss: closeCoverMenu }}
        >
          <button
            class="flex max-w-[calc(100vw-2rem)] items-center gap-1.5 rounded-md bg-background/90 px-2 py-1.5 text-[0.8rem] text-foreground shadow-sm hover:bg-background"
            type="button"
            aria-label={t("notes.changePageCover")}
            onclick={toggleCoverMenu}
          >
            <ImagePlus class="size-3.5" />
            <span class="truncate">{t("notes.changePageCover")}</span>
          </button>
          {#if coverMenuOpen}
            <NotesPageCoverMenu
              cover={page.cover}
              onSelect={(cover) => {
                coverMenuOpen = false;
                void notes.updatePageCover(page.id, cover);
              }}
            />
          {/if}
        </div>
      </div>
    {/if}
    <div class="shrink-0 border-b border-border px-4 py-3 sm:px-6">
      {#if !page.cover}
        <div
          class="relative mb-2"
          use:dismissOnOutside={{ enabled: coverMenuOpen, onDismiss: closeCoverMenu }}
        >
          <button
            class="flex max-w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
            type="button"
            aria-label={t("notes.addPageCover")}
            onclick={toggleCoverMenu}
          >
            <ImagePlus class="size-3.5" />
            <span class="truncate">{t("notes.addPageCover")}</span>
          </button>
          {#if coverMenuOpen}
            <NotesPageCoverMenu
              cover={page.cover}
              onSelect={(cover) => {
                coverMenuOpen = false;
                void notes.updatePageCover(page.id, cover);
              }}
            />
          {/if}
        </div>
      {/if}
      <div class="flex min-w-0 items-center gap-2">
        <div
          class="relative shrink-0"
          use:dismissOnOutside={{ enabled: iconMenuOpen, onDismiss: closeIconMenu }}
        >
          <button
            class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
            type="button"
            aria-label={page?.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}
            data-app-tooltip={page?.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}
            onclick={toggleIconMenu}
          >
            {#if page.icon}
              <NotesPageIcon icon={page.icon} size={20} class="shrink-0" />
            {:else}
              <SmilePlus class="size-4" />
            {/if}
            <span class="sr-only">{pageIconLabel}</span>
          </button>
          {#if iconMenuOpen}
            <NotesPageIconMenu
              icon={page.icon}
              onSelect={(icon) => {
                iconMenuOpen = false;
                void notes.updatePageIcon(page.id, icon);
              }}
            />
          {/if}
        </div>
        <input
          bind:this={titleInput}
          class="min-w-0 flex-1 bg-transparent text-[1.45rem] font-semibold leading-tight text-foreground outline-none placeholder:text-muted-foreground"
          aria-label={t("notes.titleInput")}
          bind:value={titleDraft}
          placeholder={t("notes.titlePlaceholder")}
          onkeydown={handleTitleKeydown}
          onblur={() => {
            void saveTitle();
          }}
        />
        <button
          class={`flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground ${
            pageFavorited ? "text-primary" : ""
          }`}
          type="button"
          aria-label={pageFavorited ? t("notes.removeFromFavorites") : t("notes.addToFavorites")}
          data-app-tooltip={pageFavorited ? t("notes.removeFromFavorites") : t("notes.addToFavorites")}
          onclick={() => {
            notes.setPageFavorited(page.id, !pageFavorited);
          }}
        >
          <Star class={`size-4 ${pageFavorited ? "fill-current" : ""}`} />
        </button>
      </div>
      <div class="mt-1 text-[0.733333rem] text-muted-foreground">
        {t("notes.metadataEdited", editedDateLabel)}
      </div>
      <NotesBacklinks />
      <NotesPageLinks />
      <NotesComments />
      <NotesSuggestions />
      <NotesPageHistory />
    </div>
    <div bind:this={blockScrollViewport} class="min-h-0 flex-1 overflow-auto px-3 py-4 sm:px-5">
      <NotesBlockList
        items={notes.flatBlocks}
        pageId={page.id}
        {breadcrumbItems}
        {tableOfContentsItems}
        onSelectPage={(pageId) => {
          void notes.selectPage(pageId);
        }}
        onFocusBlock={(blockId) => {
          notes.focusBlock(blockId);
        }}
      />
    </div>
  </section>
{:else}
  <div class="flex min-w-0 flex-1 items-center justify-center px-4 text-center text-[0.933333rem] text-muted-foreground">
    {t("notes.emptyEditor")}
  </div>
{/if}
