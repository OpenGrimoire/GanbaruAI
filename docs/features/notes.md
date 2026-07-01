# Notes

The notes system is a local page and block editor for capturing thoughts, project context, daily logs, and long-form writing. SQLite is the canonical store. Markdown is derivative only: import, export, preview, or agent bridge output.

The model follows public Notion API concepts where they are useful locally: pages, parents, blocks, rich text arrays, child pagination, timestamps, trash state, and archive state. It does not depend on Notion services, private internals, or hosted infrastructure.

## Source of truth

Notes are stored in `ganbaru-ai.sqlite`:

- `notes_pages` stores page metadata, parent identity, title cache, properties, icon, cover, trash state, archive state, optional external source identity, and timestamps.
- `notes_blocks` stores the canonical block tree. Each row has a page id, parent identity, block type, type payload, rich text, plain text cache, child state, trash state, sort order, optional external source identity, and timestamps.

Markdown exports can be regenerated from SQLite. Markdown imports must be parsed into page and block rows before editing. If an exported markdown file changes outside the app, the app treats that as import input, not as authoritative state.

## Page model

A page has an object type, id, created and edited timestamps, parent, trash state, archive state, properties, icon, and cover. The first local page slice uses a title property named `title`, shaped as a Notion title rich text array, and also keeps a normalized title cache for fast sidebar reads. Page icons use the public Notion icon object shape, with emoji icons and null removal currently editable from the page header. Page covers use the public Notion file object shape, with external HTTPS image covers and null removal currently editable from the top of the page.

Parents are explicit objects:

- `workspace` for top-level pages.
- `page_id` for subpages.
- `block_id` for pages nested under blocks.

Workspace pages appear at the top level of the sidebar. Page-parented child pages appear under their parent page in the sidebar tree, with a local collapse state that does not change the canonical page graph. Block-parented child pages remain addressable through breadcrumbs and visible child-page blocks until the sidebar can resolve loaded block ancestry across the whole workspace.

## Block model

Blocks are ordered children of a page or another block. The first supported block types are:

- Paragraph
- Heading 1, including toggle heading 1
- Heading 2, including toggle heading 2
- Heading 3, including toggle heading 3
- Heading 4, including toggle heading 4
- Bulleted list item
- Numbered list item
- To-do
- Toggle
- Callout
- Quote
- Child page
- Child database
- Breadcrumb
- Table of contents
- Columns
- Table
- Tabs
- Image
- Video
- Audio
- File
- PDF
- Bookmark
- Link preview
- Synced block
- Template button
- Button
- Embed
- Equation
- Divider
- Code
- Unsupported

Text-bearing blocks store public rich text arrays and block-level color in the block type payload. The editor currently supports text rich text, inline annotation rich text, inline equation rich text, safe hyperlink rich text, page mention rich text, and date mention rich text with local reminder metadata. Focused text blocks render stored bold, italic, underline, strikethrough, inline code, links, inline equations, page mentions, date mentions, text colors, and background colors inside the editable surface. Selecting text in a text block exposes a viewport-aware inline toolbar near the real selection for bold, italic, underline, strikethrough, inline code, inline equation, text color, background color, and link editing; the toolbar docks instead of overlapping text when the viewport is too narrow to float safely. Ctrl or Cmd+B, Ctrl or Cmd+I, Ctrl or Cmd+U, Ctrl or Cmd+Shift+S, and Ctrl or Cmd+E toggle bold, italic, underline, strikethrough, and inline code on the selected text, then keep the selected range stable when the rich editor can restore it. Ctrl or Cmd+K opens link editing for the current selection or an existing link under the caret. Users can add, edit, or remove HTTP, HTTPS, and email links without replacing the whole block payload, and invalid schemes are rejected before storage. Editing local Notes links refreshes derived backlinks. Ctrl or Cmd+Shift+E converts selected text into a validated inline equation object, rejects empty or invalid selections with localized feedback, and places the cursor after the inserted equation. Typing `@` in a text block opens a mention picker backed by local page metadata and local date parsing for common inputs such as today, tomorrow, weekdays, ISO dates, and remind queries. Selecting a page inserts a validated page mention object with a local Notes page link. Selecting a date or reminder inserts a validated date mention object. Pasting into the rich editor accepts validated plain text and sanitized rich HTML from browsers, document editors, and other Notes blocks. Rich paste preserves supported inline annotations and safe HTTP, HTTPS, and email links, removes scripts and unsafe link targets before persistence, and splits multiple pasted paragraphs into sibling blocks. Plain text edits around page mentions, date mentions, links, inline equations, and formatted spans preserve each rich text object when its visible label remains present, while edited or deleted labels degrade to plain text without flattening unrelated objects. Unknown supported-by-Notion fields can be preserved inside the payload only after validation.

Nested blocks are represented by parent rows, not by embedding children inside JSON. Child loading uses cursor pagination so large pages can stream in later.

## Editor behavior

The first serious Notes tab includes:

- Page sidebar with favorites, recents, a nested page tree, create, create subpage, select, rename, move, duplicate, archive, collapse, workspace search, and trash actions.
- Archive view with archived-page search and unarchive.
- Trash view with trashed-page search, restore, and permanent delete.
- Backlinks disclosure under the page title for visible pages that reference the current page.
- Comments disclosure under the page title for page discussions and block comments.
- Page header icon picker for setting or removing emoji page icons, mirrored in sidebar rows.
- Page cover banner for setting or removing external HTTPS image covers.
- Block editor with the supported core block types.
- Block handles with add block below, insert block type, turn into, color, copy link, duplicate, move to page, move within page, and delete actions.
- Nested block support through Tab and Shift+Tab.
- Sibling block reordering through handle actions and Ctrl or Cmd+Shift+ArrowUp or ArrowDown.
- Same-sibling drag reordering from the block handle, with a visible drop guide.
- Enter behavior for splitting or creating sibling blocks.
- Backspace behavior for merging with the previous block or deleting empty blocks.
- Slash menu for block conversion, block actions, block colors, filtering, and recent commands.
- Viewport-aware inline formatting toolbar from selected text in editable text blocks, including inline equation conversion.
- Link editor from selected text, toolbar action, or Ctrl+K in editable text blocks.
- Page, date, and reminder mention menu from `@` in editable text blocks.
- Plus menu for inserting a chosen block type without typing a slash command.
- Markdown-like start shortcuts for common block types.
- Multi-line plain-text paste that creates sibling blocks instead of storing several paragraphs inside one block.
- Safe rich HTML paste that preserves supported inline formatting and links while creating sibling blocks for multiple paragraphs.
- Markdown-like paste prefixes for headings, lists, to-dos, toggles, quotes, dividers, and fenced code blocks.

These behaviors are implemented as pure TypeScript planning helpers where possible. Svelte components adapt keyboard and focus events to those helpers, then persist changes through Tauri commands. The editor surface is split between the page shell, block list, block row chrome, text editor controls, card-like block editors, and media or layout-specific child components. The Notes store keeps its public API stable while delegating page preference persistence, focus request tokens, block tree selectors, debounced block persistence, and block actions to focused helper modules.

Duplicating a block clones the selected block subtree through a Rust transaction. The frontend generates the duplicate IDs for the loaded subtree, and the backend validates that the ID map exactly matches the source subtree before inserting rows.

Moving a block up or down reorders it inside its current sibling group through the same Rust move command used by nesting and outdent. The frontend plans the move from the loaded block graph, flushes any pending save for the moved block, persists the new parent and adjacent block position in SQLite, reloads the visible tree, and restores focus to the moved block. This keeps nested block subtrees attached to the moved parent.

Dragging a block from its handle uses the same move planner and command boundary. The editor shows a thin drop guide above or below valid same-parent targets, rejects cross-parent drops for this first drag slice, flushes pending text saves before the structural move, and supports moving before the first sibling through the move request's `before` field. Cross-parent drag nesting, outdent by horizontal drag position, cross-page moves, and multi-block moves remain planned on top of this movement foundation.

The block handle's Move to page action lists other non-trashed Notes pages and moves the selected block subtree to the bottom of the destination page through the Rust move command. The current page is excluded because same-page reordering has dedicated controls, and child-page blocks cannot be moved into their own page or into pages contained by their subtree. After the move, the source page reloads so the removed block disappears and the page list refreshes edited timestamps. A fuller Notion-style destination picker with nested page search and recent destinations remains planned.

Block links are local anchors built from the Notes page id and block id. The block action menu and slash menu can copy a link to the clipboard. Opening that link switches the app to the Notes tab, loads the target page, scrolls the target block into view, and focuses it. Page mention links use the same local Notes hash without a block id and open the target page. Inline text hyperlinks use public rich text `text.link` and `href` fields, validate allowed URL schemes before storage, normalize bare email addresses to `mailto:` links, and refresh derived backlinks when edited. Links use stable SQLite ids when they target local Notes pages or blocks, so renaming a page or editing block content does not invalidate local block links.

Backlinks are derived from canonical block references, not manually maintained. The backlink list includes visible active pages that reference the current page through paired child-page blocks, local Notes URLs in rich text links, or page mention rich text payloads. Trashed or archived source pages are hidden from the backlink list. Selecting a backlink opens the source page and focuses the source block.

Comments are SQLite-backed discussions modeled after public Notion comment objects. A comment thread attaches to either the current page or a visible block, stores a discussion id, rich text body, local author display metadata, status, timestamps, and soft-delete state. The page header comments disclosure lists open threads by default, can include resolved threads, supports replies, edit, soft delete, resolve, and reopen, and block handles can start a block-anchored comment. Comment rich text validates the same text, page mention, date mention, and reminder mention objects supported by the editor. Inline range comments, user mentions, unread state, attachments, notification delivery for reminder metadata, and local notification settings remain planned.

The slash menu uses one command catalog for block conversion, block actions, block colors, filtering, and session-local recent commands. Users can type slash followed by aliases such as duplicate, delete, red background, or table of contents. Action commands reuse the same duplicate, move, delete, and copy-link paths as block handles. Color commands appear only for blocks whose payload can store a Notion color value.

The plus menu uses the same insertable block catalog as the slash menu's block command group. It can insert user-created block types below the current block, including text, headings, lists, to-dos, toggles, callouts, child pages, generated blocks, layout blocks, media, bookmarks, link previews, template buttons, local button blocks, embeds, equations, dividers, and code. Internal structural rows such as table rows and columns are excluded because they are created through their parent table or column-list blocks. Preservation blocks such as child databases, synced block duplicates, and unsupported blocks are excluded until their full creation semantics exist locally.

Toggle blocks use the public Notion `toggle` payload shape for rich text and color, plus a local `ganbaru_open` display flag so closed toggles stay closed after reload. Toggle children remain normal block rows in SQLite. Closing a toggle hides its descendants in the flattened editor view without deleting or moving them. Users can create toggles from the slash menu, convert blocks to toggles, type `>` followed by space, and press Ctrl+Enter while focused in a toggle to open or close it.

Toggle headings use the public heading 1 through heading 4 payload shape with rich text, color, and `is_toggleable`, plus the local `ganbaru_open` display flag. Normal heading blocks cannot own children. A heading can own child block rows only when `is_toggleable` is true, and the backend rejects appends, moves, or updates that would leave children under a non-toggleable heading. Closed toggle headings hide descendants in the flattened editor view without deleting or moving them. Users can create toggle headings from the slash menu and press Ctrl+Enter while focused in a toggle heading to open or close it.

Callout blocks use the public Notion `callout` payload shape for rich text, icon, and color. Ganbaru AI persists the callout icon in the block payload and renders the current default native info icon in the editor. Callout children remain normal block rows in SQLite, so a callout can contain nested notes without embedding child arrays inside JSON. Users can create callouts from the slash menu or convert existing blocks to callouts.

Child-page blocks use the public Notion `child_page` payload shape with a title string. Ganbaru AI stores the child page as a normal page row and stores a paired `child_page` block in the parent document using the same id. Converting a block to a child page creates the nested page, moves existing child blocks into the new page body, opens that page, and keeps future page renames and trash or restore operations synchronized with the parent block. Creating a subpage from the sidebar uses the same Rust page-create command boundary, inserts the paired child-page block into the parent document, opens the new page, and expands the parent row.

Child-database blocks use the public Notion `child_database` payload shape with a title string. Ganbaru AI currently preserves the title, validates it, stores child rows normally in SQLite, derives search text from the title, and renders a visible database placeholder in the editor. Full local database schemas, properties, views, filters, sorts, formulas, relations, rollups, linked database views, and database item pages remain planned as a later database slice.

Favorites and recents are local navigation metadata stored outside the canonical page graph. Favoriting a page pins it in the sidebar and shows a star in the page header and sidebar row. Opening or creating a page records it in recents so frequently used pages stay reachable without changing page parents, page sort order, or block content.

Sidebar search queries canonical SQLite data through the `notes_search` command. Results include active page title matches, active block plain-text matches, and non-deleted comment text matches, with snippets and result type labels. Page results open the page, block results open the page and focus the matching block, and block-comment results focus the commented block when the thread target still exists. The first search slice uses bounded SQLite text matching over canonical columns; a future FTS cache can replace the query internals without changing the result contract.

Duplicating a page copies the source page row, page icon, cover, visible block tree, and nested child pages through one Rust transaction. The duplicate gets fresh page and block ids, does not inherit import provenance, and keeps child-page block ids paired with their duplicated page rows. Workspace pages duplicate as new top-level pages. Nested pages duplicate beside the source child-page block in the same parent.

Moving a page changes the page parent through a Rust transaction and keeps the paired child-page block synchronized with the destination. Moving under another page creates or restores the paired child-page block at the bottom of that destination page. Moving back to the workspace hides the paired child-page block so the page becomes a top-level sidebar item. The command rejects block parents for the sidebar move action, missing or inactive destination pages, moving a page under itself, and moving a page under a descendant reachable through page parents or child-page blocks.

Archived pages keep a separate `archived` state in SQLite and are hidden from the normal page list, favorites, recents, active sidebar search, active page loading, active parent validation, and parent-page document body without entering Trash. The sidebar exposes an Archive view that lists archived page metadata, supports local search, and unarchives a page through the Tauri command boundary. Unarchived pages are selected and loaded immediately. Trashing a page clears its archive state so Trash remains the single recovery surface for deleted pages.

Breadcrumb blocks use the public Notion `breadcrumb` empty payload shape. The rendered path is generated from the current local page graph instead of being stored as duplicated text, so future nested pages can update breadcrumbs automatically when page titles or parents change. Users can create breadcrumbs from the slash menu or convert existing blocks to breadcrumbs.

Table of contents blocks use the public Notion `table_of_contents` payload shape with block-level color. The rendered heading index is generated from the current local block graph instead of being stored as duplicated text, including heading levels 1 through 4. Users can create a table of contents from the slash menu, convert existing blocks to it, and use generated entries to focus the matching heading block.

Column list blocks use the public Notion `column_list` empty payload shape. Column blocks use the public Notion `column` payload shape with optional `width_ratio`. Ganbaru AI stores columns as internal child blocks under the column list, and the editable content inside each column remains normal child blocks in SQLite. The editor renders the column list as one layout surface, hides internal column container blocks from the main page flow, stacks columns when the layout no longer fits, and keeps keyboard deletion and merge behavior scoped to the focused column. Users can create a default two-column layout from the slash menu.

Table blocks use the public Notion `table` payload shape with `table_width`, `has_column_header`, and `has_row_header`. Table row blocks use the public Notion `table_row` payload shape with `cells`, where each cell is a rich text array. Ganbaru AI stores rows as normal child block rows under the table block, but the editor renders them as one table surface instead of exposing row blocks as standalone document rows. Users can create a default local table from the slash menu and edit cells inline.

Tab blocks use the public Notion `tab` empty payload shape. Direct children of a tab block are paragraph rows that act as tab labels; their rich text stores the visible label, their optional paragraph icon stores the tab icon, and their child rows store the tab panel content. The editor renders the tab block as one layout surface, hides label paragraphs from the main page flow, and shows the active tab's child blocks as normal editable Notes content. Users can create a default local tab group from the slash menu.

Image, video, audio, file, and PDF blocks use public Notion file object source shapes. Ganbaru AI currently supports editable external HTTPS file URLs for new local blocks and validates imported Notion-hosted `file` and `file_upload` source objects. Non-empty external URLs must match the block kind, such as image extensions for image blocks, audio extensions for audio blocks, video extensions or supported YouTube URLs for video blocks, `.pdf` for PDF blocks, and any HTTPS URL for file blocks. The editor stores captions as rich text arrays, supports an optional file name on file blocks, derives the plain text cache from captions and source identity, renders native previews for previewable media, and opens media only after explicit user action. Local file attachment storage remains planned and must use the Ganbaru AI assets/file model rather than raw untracked paths.

Bookmark blocks use the public Notion `bookmark` payload shape with a URL string and caption rich text array. Ganbaru AI stores and edits the URL and caption locally, derives the plain text cache from both fields, and does not fetch remote preview metadata automatically. The open button launches only HTTP and HTTPS URLs through the platform opener.

Link preview blocks use the public Notion `link_preview` payload shape with a URL string. Public Notion integrations can retrieve these blocks but cannot create or append them through the public API. Ganbaru AI still supports local creation because Notes are not a hosted Notion connection: it stores and edits the originally pasted URL, derives the plain text cache from that URL, renders a local card from URL parts only, and avoids automatic remote metadata fetches. The open button launches only HTTP and HTTPS URLs through the platform opener.

Synced blocks use the public Notion `synced_block` payload distinction between originals and duplicates. Original synced blocks store `synced_from: null` and can own normal child block rows in SQLite. Duplicate synced blocks store `synced_from.type: "block_id"` with a source block id and are rendered as visible reference placeholders. Ganbaru AI preserves and displays both shapes for import compatibility, but full live fanout editing across synced copies remains planned.

Template button blocks use the public Notion `template` payload shape with a rich text title. Their reusable content remains normal child block rows under the template block, not embedded child JSON in the payload. The editor renders a compact use button on the template block; activating it duplicates the loaded child subtree below the template through the existing duplicate and move commands. Child-page descendants are intentionally not duplicated by this first template slice because page duplication and page link pairing need separate semantics.

Button blocks use a Ganbaru AI local payload because Notion's public block API does not expose a supported `button` payload even though the Notion product supports page buttons. The local payload stores a rich text label, an optional icon, and a bounded action list. The first supported action is `insert_blocks`, which duplicates the loaded child block subtree from under the button to the configured target position. New local buttons default to inserting below the button. Button action children remain normal child block rows in SQLite, and child-page descendants are intentionally not duplicated until page creation and page-link pairing actions are implemented. External actions such as webhooks, mail, Slack, broad database edits, and destructive automations remain unsupported until they have explicit local schemas, user prompts, and permission boundaries.

Embed blocks use the public Notion `embed` payload shape with a URL string. Ganbaru AI stores and edits the URL locally, derives the plain text cache from the URL, and renders a local hostname label instead of fetching remote preview metadata. The open button launches only HTTP and HTTPS URLs through the platform opener.

Equation blocks use the public Notion `equation` payload shape with a KaTeX-compatible expression string. Ganbaru AI stores and edits the expression locally, derives the plain text cache from the expression, and renders a dependency-free formula surface until a vetted math renderer is approved.

Unsupported blocks use the public Notion `unsupported` payload shape and preserve the informational `block_type` string from imports when present. Ganbaru AI also allows validated local provenance fields, raw source metadata, and import warnings in the payload so future importers can avoid silent data loss. The editor renders unsupported blocks as visible placeholders with the imported type and preservation state instead of hiding the row. Users can still duplicate, delete, or deliberately convert the block through normal block handles.

Block colors use the public Notion color names stored in the block payload. Text edits, to-do toggles, toggle open state changes, callout edits, table of contents color changes, conversions between color-capable block types, and duplication preserve the selected color. Rust and TypeScript validators reject unsupported color names before they are treated as typed Notes data.

Trashed pages keep their `in_trash` state in SQLite and are hidden from the normal page list and Archive. Trashing or restoring a page applies to the reachable page subtree so child pages do not remain active under a trashed parent. The sidebar exposes a Trash view that lists trashed page metadata, supports local search, restores a page by setting `in_trash` back to false through the same Tauri command boundary, and permanently deletes a trashed page after confirmation. Permanent delete removes the page subtree, paired child-page blocks, and page body blocks in one Rust transaction. Restored pages are selected and loaded immediately. Automatic retention windows and page history remain planned.

## Backlinks and daily or project notes

Daily notes and project notes remain planned. They should be implemented as page properties, relations, templates, and derived indexes on top of the same page and block graph.

Daily and project note categories are conventions over Notes pages, not separate markdown folders and not separate data models.

## Sync and collaboration

Local editing is SQLite first. Future sync can layer CRDT state over the same page and block graph, but it must preserve SQLite as the local canonical store and keep markdown as a derivative format.
