# Data architecture

The app stores two categories of data with deliberately different mechanisms. Mixing them, or storing one as the other, creates friction every time. Keeping them separate keeps each tool used for what it is good at.

## The split

**Documents.** Diary entries, project working documents, generated reports, and user attachments. These are files on disk inside the user's Ganbaru AI folder. For markdown documents, the file is the source of truth. SQLite may hold an index for fast search, tag lookups, backlinks, and modified-at queries, but the index is rebuildable from the files.

Why markdown on disk and not in SQLite as text columns:

- Users can open, edit, sync, and back up their documents with any tool they already trust (a text editor, git, rsync, Obsidian, Syncthing).
- The Ganbaru AI folder remains useful if the app stops being maintained. AGPL plus a plain-file format means the user is never trapped.
- Conflict resolution during sync uses the same file-level tools the user already understands.

**Structured data and document graphs.** Calendar events, Notes pages and blocks, kanban tasks, work environment configs, pomodoro runs, segments, pauses, playlist definitions, project metadata. These live in SQLite. The database is the source of truth. There is no authoritative markdown file to fall back to.

Why SQLite and not markdown:

- Structured data needs relational integrity (foreign keys, cascades, atomic transactions). Markdown does not enforce this.
- Aggregations that drive analytics (focus score, break adherence, idle patterns) are SQL queries, not markdown text searches.
- The data model evolves. A schema migration is a known, scoped operation. Re-parsing a thousand markdown files of varying shape is not.

The rule is one-directional: structured data and Notes pages may be exported as markdown for collaborators or AI agents that read repos, but those exports are views, not source. They can be regenerated at any time. The reverse, treating an exported markdown file as authoritative, is forbidden unless an explicit import command converts it back into canonical rows.

## Ganbaru AI folder layout

Everything portable that the app produces lives under one folder. First launch defaults to `Documents/Ganbaru AI` in production and `Documents/Ganbaru AI Dev` in development builds, with secondary actions to choose another folder or import an existing Ganbaru AI folder from another installation. Development setup warns the user to use the dev default or a copied production folder so test data does not mix with real production data. Tauri's platform app config directory stores only device-local bootstrap and runtime state, such as the active folder pointer, benchmark state, and transient doomscrolling snapshots.

Folder setup errors are blocking and remain visible until the user starts another folder action, successfully selects a usable folder, or closes the app. The UI translates backend validation failures into user-facing guidance for non-empty unrelated folders, missing or damaged `vault.json`, unsupported folder schema versions, permission problems, missing folders, and database-open failures for `ganbaru-ai.sqlite`.

```
Ganbaru AI/
  vault.json                         # internal Ganbaru AI folder marker, id, display name, schema version
  config.json                        # user settings, environment definitions, blocker rulesets
  ganbaru-ai.sqlite                  # SQLite source of truth for structured data, Notes, and indexes
  notes/exports/                    # derivative markdown exports for notes (planned)
  diary/morning/, diary/evening/    # dated diary entries (markdown plus indexed fields)
  projects/{project-id}/            # per-project file attachments (PDFs, references)
  reports/                          # generated project status reports (markdown, PDF)
  assets/                           # user assets (images embedded in notes, attachments)
    notes/page-icons/               # copied local Notes page icon images
    notes/page-covers/              # copied local Notes page cover images
    project-icons/                  # copied project and group icon images
  templates/                        # phase templates, methodology templates (SWOT, BMC)
  .yjs/                             # Yjs document state cache (binary)
```

Music files stay wherever the user keeps them. The Ganbaru AI folder stores playlist definitions only, with paths or URIs into the user's music library. This avoids duplicating large audio files into the app folder and respects existing collections.

Backups go to a user-specified path **outside** the Ganbaru AI folder. Backing up the folder into itself defeats the purpose if disk corruption takes the folder.

## Database files

The user database is always `ganbaru-ai.sqlite` at the active Ganbaru AI folder root. Development and production builds keep separate Tauri app config directories and separate `app-state.json` files, so each build can point at a different folder. The benchmark harness uses device-local `benchmark.sqlite` in `app_config_dir`; it is not portable user data.

Lazy initialization: the database connection is opened on first use after a Ganbaru AI folder has been selected, not at process startup. This keeps cold start time low and allows the folder to be on a slower-than-disk path, such as an encrypted volume, without delaying the setup UI.

The Tauri integration owns SQLite in Rust through focused `sqlx` commands. Higher-level ORMs were considered and rejected: they add code to maintain, do not earn enough productivity for an app this small, and obscure the actual queries that show up in performance profiles. Plain SQL with typed command wrappers keeps the call sites direct.

## External tools and the CLI bridge

The app is not the only thing that needs to read this data. AI agents (Codex or another CLI coding agent in the integrated terminal, MCP clients), backup tools, scripts, and human collaborators all interact with the same store.

The bridge is the `ganbaru-ai` CLI (Rust binary, reads the same SQLite). It exposes structured commands (`task list`, `event get`, `export kanban`) that AI agents call via Bash. This keeps three properties:

1. One source of truth. The CLI reads what the app writes. There is no duplicate authoritative store for agents.
2. Markdown exports stay derivative. The CLI can write kanban snapshots or generated reports to a git repo for collaborators who never install the app, but those files are regenerated from the database; editing them by hand is supported only via an explicit import command where the export type supports imports.
3. External readers handle dirty state. If the app crashed and a run is mid-write, the CLI applies the same recovery semantics as the app on startup (see `algorithms/pomodoro-state-machine.md`). Aggregations always operate on a consistent view.

The MCP server is for external clients only (ChatGPT, teammate agents, and other MCP-compatible clients). Internal agent flows use the CLI directly. This keeps MCP a thin, documented surface and avoids two parallel paths to the same data.

## Source-of-truth checks

When designing a new feature, ask:

- Is this content the user would expect to exist as a file they can open without the app? If yes, it is usually a document.
- Does it have relational structure (foreign keys, aggregations, cross-record queries)? If yes, it is structured data.
- Could it be regenerated from another source? If yes, it is a cache (e.g. the `.yjs/` directory, the search index part of `ganbaru-ai.sqlite`).

Notes are the named exception to the openable-file heuristic. Their editable model is a relational page and block graph, so SQLite is canonical and markdown is import, export, or bridge output. Block UI state that affects the local document graph, such as whether a toggle block or toggle heading is open or closed, is persisted with the block payload in SQLite rather than in markdown exports.

Notes block presentation data that belongs to the document, such as a callout icon and Notion-style block color, is persisted as validated block payload data in SQLite. If a later callout icon points at a local file, the asset file belongs in the Ganbaru AI assets folder while the block payload stores only the validated file object reference.

Notes child pages are represented twice because they have two roles. The page row in `notes_pages` owns the child document and its root blocks, while the paired `child_page` block row in the parent document owns the visible page link and block order. The paired page and block share an id so rename, trash, restore, and navigation can stay synchronized without a separate join table.

Notes child databases have a visible block layer and a normalized database layer. The `child_database` block owns the position in the page body and stores the visible title plus local database, data source, and view ids when Ganbaru AI creates it. `notes_databases` stores parent identity, title rich text, icon, cover, timestamps, and optional source provenance. `notes_data_sources` stores the first data source and its validated property schema JSON, including the required title property and supported local schema properties. `notes_database_views` stores the first table view, table property order, hidden property ids, column widths, row open mode, filters, sorts, board grouping, hidden board groups, board card properties, and later independent view settings. Imported title-only child-database blocks remain valid preservation placeholders, but local database state must grow through normalized tables instead of expanding the block payload into an untyped database blob.

Notes database row pages are normal page documents parented by a data source. A row page stores `parent_type = 'data_source_id'` and `parent_data_source_id` in `notes_pages`, keeps its page property values in `notes_pages.properties`, and owns normal body blocks in `notes_blocks`. Table cell editing rewrites only the targeted Notion-shaped page property value, records a recoverable page snapshot before mutation, updates the row title cache when the title property changes, and reloads rows through the table view's filters and sorts. Board card moves use the same targeted property update path for locally writable group properties, then reload grouped rows through the board view's filters, sorts, group order, and hidden group state. Row pages do not have paired `child_page` blocks, because their visible navigation surface is the database view rather than the sidebar tree. Active row pages are searchable, duplicable, trashable, restorable, and permanently deletable through the same page lifecycle commands as other Notes pages, while sidebar root reads exclude them so database rows do not leak into top-level navigation.

Notes page duplication is a graph copy, not a markdown export or a shallow page row clone. The Rust command copies the page row, visible block rows, nested child pages, icons, covers, and child-page block pairings in one transaction. Duplicates get fresh page and block ids and omit source provenance fields so imported pages do not create multiple local rows claiming the same external object identity. Database row page duplicates keep the same data source parent and property values, and they do not create child-page blocks.

Notes page templates are first-class local Notes data. `notes_page_templates` stores the template name, optional source page id, page properties, icon, cover, and timestamps. `notes_page_template_blocks` stores the normalized template block snapshot with payload JSON, plain-text cache, sort order, and parent tree. Applying a template creates new `notes_pages` and `notes_blocks` rows in one transaction, preserving SQLite as the source of truth. Markdown and rendered HTML are never authoritative template formats. Child-page template blocks create paired local child pages with default empty bodies until full subpage template snapshots are implemented.

Notes page history is recoverable SQLite data attached to the canonical page and block graph. `notes_page_history_snapshots` stores compact snapshots of page metadata and canonical block rows before recoverable page or block mutations. `notes_page_history_settings` stores the local retention window, where `NULL` means keep snapshots forever. Restoring a snapshot records the current page first, then replaces page details and page body blocks from the snapshot without moving the page back to an old parent. Copying from history appends fresh block rows to the current page. Markdown and rendered HTML are never authoritative history formats.

Notes page movement updates the page parent row and the paired child-page block in one transaction. Moving under another page creates or restores the paired child-page block at the end of the destination page. Moving to the workspace hides the paired child-page block because the page is now represented by the top-level sidebar tree. The command rejects self moves and descendant moves across both page-parented descendants and block-parented child pages.

Notes page archive state is document metadata stored on `notes_pages.archived`, separate from `notes_pages.in_trash`. Archived pages stay in the graph but are hidden from active page lists, active sidebar search, active page loads, and parent validation. Unarchiving clears only the archive state. Trashing a page clears archive state so deleted content is recoverable through Trash instead of two separate recovery surfaces.

Notes permanent page deletion is a subtree delete, not a single-row delete. The command requires the root page to already be in Trash, walks page-parented descendants, block-parented child pages, and visible child-page blocks, deletes paired child-page blocks outside the subtree, then deletes the page rows so SQLite cascades page body blocks. The command returns deleted page ids so navigation metadata can drop stale favorites and recents.

Notes sidebar navigation metadata is UI state stored in `config.json`. Expanded page state lives under `notes.sidebarExpandedPageIds`, favorites under `notes.favoritePageIds`, and recently opened pages under `notes.recentPageIds`. The older `notes.sidebarCollapsedPageIds` key is cleared when expanded state is saved. These values never change page parent rows or block order. Sidebar page reads load page metadata only: roots, expanded direct children, favorites, recents, and selected-page ancestors. They do not load page block bodies. Selected pages can still reveal their ancestor path in the sidebar without mutating the stored expanded list.

Notes page icons are document metadata stored in the `notes_pages.icon` JSON column, not in navigation config. The editable slice supports emoji icon objects, Notion-style native icon names and colors, reusable custom emoji, external HTTPS image icons, managed local image file icons, and null removal through the page update command. Local page icon image bytes are copied under `assets/notes/page-icons/` using content-hash file names, while `notes_page_icon_assets` stores the managed asset metadata in SQLite. The page icon JSON keeps only the validated document reference and display metadata.

Notes page covers are document metadata stored in the `notes_pages.cover` JSON column. The editable slice supports external HTTPS image file objects, imported Notion-hosted file objects, imported file upload references, managed local image file objects, generated local cover images, and null removal through the page update command. Local and generated page cover image bytes are copied under `assets/notes/page-covers/` using content-hash file names, while `notes_page_cover_assets` stores the managed asset metadata in SQLite. The cover JSON keeps only the validated file object reference and display metadata.

Notes external references that belong to the document, such as bookmark URLs, bookmark captions, link preview URLs, and embed URLs, are persisted as validated block payload data in SQLite. The editor may open a user-saved HTTP or HTTPS bookmark, link preview, or embed on explicit click, but it must not fetch remote preview metadata automatically because Notes must remain private and fully offline.

Notes synced blocks are persisted as validated block payload data while their contents remain normalized child rows. Original synced blocks store `synced_from: null` and can own child rows. Duplicate synced blocks store a source block id reference and stay leaf placeholders until local synced-copy fanout and unsync operations are implemented. This preserves imported Notion data without embedding authoritative children inside payload JSON.

Notes template button blocks are persisted as validated block payload data while their reusable contents remain normalized child rows. The template payload stores only the button title as rich text. Using a template copies the loaded child block subtree into the template block's parent through normal block duplication and movement, so template output is canonical `notes_blocks` data rather than an embedded JSON expansion.

Notes button blocks are persisted as validated local action payloads while inserted content remains normalized child rows. The button payload stores a rich text label, optional icon, and bounded action list. The first action schema is `insert_blocks`, with a target position and `children` as the source. The reusable blocks themselves stay under the button in `notes_blocks`; clicking the button duplicates those child rows into the target location through normal block duplication and movement. External automations, webhook calls, mail, Slack, destructive actions, and database mutations must not be stored as untyped action blobs.

Notes media and file blocks store public file object source shapes in SQLite. New local blocks use external HTTPS URLs until the local file asset model is implemented. Imported Notion-hosted `file` objects and `file_upload` objects are validated and preserved as payload data, but Ganbaru AI should not cache expired Notion-hosted URLs as durable local file content. Future local attachments must add a normalized file metadata table under the Ganbaru AI assets model and keep block payloads as references to those assets.

Notes equation expressions are persisted as validated block payload data in SQLite. Rendering can improve over time, but the canonical value is the stored KaTeX-compatible expression string rather than generated visual output.

Notes unsupported blocks are persisted as visible, validated preservation payloads in SQLite. The `unsupported` payload keeps the imported `block_type` string when available, optional source metadata, optional raw source object data, and optional import warnings. The plain text cache includes the imported type and warnings so search and future diagnostics can find preserved unsupported content even when Ganbaru AI cannot render the original block.

Notes simple tables are persisted as a block subtree in SQLite. The parent `table` block stores table width and header flags, and each `table_row` child stores its cells as rich text arrays. The rendered table is derived from those rows, so row order, duplication, trash, and future sync can use the same block graph rules as the rest of Notes.

Notes column layouts are persisted as a block subtree in SQLite. The parent `column_list` block stores layout identity, each `column` child stores optional width ratio data, and each column owns normal editable child blocks. The editor hides column container blocks from the main document flow while preserving their row order and parent identity for duplication, trash, future resizing, and future sync.

Notes tab layouts are persisted as a block subtree in SQLite. The parent `tab` block stores an empty public payload, each direct paragraph child stores a tab label and optional tab icon, and each label paragraph owns normal editable child blocks for that tab panel. The editor hides label paragraphs from the main document flow while preserving their row order and parent identity for duplication, trash, future reordering, and future sync.

Notes toggle headings are heading 1 through heading 4 blocks with validated `is_toggleable` payload state. A heading can own child rows only when `is_toggleable` is true. The local `ganbaru_open` payload field stores whether those children are visible in the editor. This keeps toggle headings compatible with Notion-shaped heading payloads while preserving local display state offline.

Notes block links are derived from existing page and block ids. They do not add a separate storage table in the first implementation because the block id already anchors the canonical row. Opening a copied block link validates the ids in the URL hash, loads the target page from SQLite, and focuses the target block if it still exists. Page-only Notes links use the same hash format without a block id and load the target page.

Notes inline annotations are canonical rich text `annotations` values inside validated block payloads and comments. The editor can split and merge selected text ranges for bold, italic, underline, strikethrough, inline code, text color, and background color without changing block identity or sibling order. Notes inline equations are canonical rich text `equation.expression` values inside the same validated payloads, and selected text can be converted into a bounded local LaTeX expression. Notes inline hyperlinks are canonical rich text `text.link` and `href` values inside validated payloads. The editor can apply, edit, and remove links over text ranges, and the validators reject unsafe URL schemes before typed use. Local Notes URLs in rich text remain backlink inputs, so editing a link refreshes the derived backlink read model.

Notes clipboard paste is parsed into canonical block updates before it crosses the Tauri command boundary. Multi-line plain text becomes sibling block rows under the same parent, and supported markdown-like line prefixes become typed block payloads. Markdown syntax in clipboard input is an import convenience only; the editable source remains SQLite block rows and validated rich text payloads. Multi-block copy, cut, paste, duplicate, move, and delete operate on selected root subtrees through Rust commands that wrap each operation in one SQLite transaction. Copy also writes plain text to the system clipboard for interoperability, but the internal Notes clipboard stores source block ids and loaded subtree ids so paste can duplicate canonical rows, rich text payloads, and supported block comment threads without making markdown authoritative.

Notes structural block moves are SQLite graph mutations. Keyboard moves, handle actions, drag reorder, drag nesting, drag outdent, and drag-to-page all plan against the loaded block tree, flush pending block saves before the structural command, then persist a new parent plus `after` or `before` anchor through Rust. The command rejects cycles, invalid parent shapes, destination pages inside the moved subtree, and anchors that do not belong to the destination parent.

Notes undo and redo state is stored in `notes_undo_state` as bounded page-local operation snapshots. This state exists only to recover local editor history after a crash or reload. It must remain derived from `notes_pages` and `notes_blocks`; applying undo or redo uses normal block update, trash or restore, and move commands so canonical content still lives in the page and block graph.

Notes mentions are canonical rich text objects inside validated block payloads and comments. The editable mention slices support page mention objects with local page ids and local Notes hrefs, plus date mention objects for common date and reminder inputs. Reminder mentions store local `ganbaru_reminder` metadata inside the validated date mention object, but notification delivery remains a future local feature. Mention display text is part of the rich text object for import compatibility, while backlinks and navigation use stable page ids where a page target exists. Future user, local object, inline equation, and range comment anchors should extend the rich text model with explicit validated variants rather than storing untyped JSON.

Notes backlinks are derived from canonical block payloads and child-page pairings. The first implementation scans visible active block rows on visible active source pages for paired `child_page` references, local Notes URL links, and page mention payloads. A future `notes_backlinks` table may cache the same facts for search and graph views, but it must remain rebuildable from canonical page, block, and rich text rows.

Notes comments are canonical SQLite rows, not page payload annotations. `notes_comment_threads` stores the discussion id, target page, optional target block, open or resolved state, and resolution metadata. `notes_comments` stores individual rich text comments, local author display metadata, attachments metadata, timestamps, and soft-delete state. Page comments use a page parent, block comments use a block parent, and visible comment reads hide block threads when the target block is trashed. Inline text-range anchors, unread state, and future sync metadata should extend these tables or add normalized anchor tables instead of embedding comments into block payload JSON.

Notes workspace search is a read projection over canonical SQLite rows. The first implementation searches active page titles, active block `plain_text`, and non-deleted comment `plain_text` with bounded SQL text matching and returns typed page, block, or comment results. It does not create a cache table yet. Future SQLite FTS tables can cache the same derived text for ranking and large workspaces, but those tables must remain rebuildable from `notes_pages`, `notes_blocks`, and `notes_comments`.

Generated Notes blocks store their public payload shape in SQLite, while rendered text is derived from canonical rows. Breadcrumb paths come from page and parent rows, including unavailable ancestor rows when needed for status display, and table of contents entries come from heading 1 through heading 4 blocks. This avoids stale path or heading copies when a page title, page parent, heading text, or block order changes.

If the answer is unclear, the default is structured data in SQLite. Promoting a value to a markdown file later is easy. Demoting a markdown file with subtle structure to SQLite later is painful.
