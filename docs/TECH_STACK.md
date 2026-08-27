# Tech stack summary

## Overview

A cross-platform productivity app for desktop and mobile built around Calendar, Projects, Pomodoro, project-owned working folders, SQLite-backed Notes, file-backed Markdown working documents, Chat channels and agent coordination, daily diary, sleep alarm, work environments, website and app blocking, music, guided project management, and future collaborative workspaces. Designed as a local-first, privacy-respecting alternative to fragmented planning, knowledge, communication, and AI-assistant tools. A gamification layer is planned for later phases.

Desktop (Windows, Linux) is the primary target. Android via Tauri v2 is the first mobile target, with Android 10, API level 29, as the minimum and API level 36 as the initial target and compile SDK. iOS follows through the same shared contracts later. Mobile shares domain code and adaptive frontend components while offering a focused platform-appropriate capability set. See [features/mobile.md](features/mobile.md).

---

## Languages

### Rust

The backend of the Tauri app. Handles everything that requires OS-level access: process management, global mouse polling, file system operations, native messaging with the browser extension, system tray, local file I/O for markdown files in the Ganbaru AI folder, Rust-backed local audio playback through the internal media player module, the local video loopback fallback, and desktop activity monitoring (active window tracking, idle detection, app-switch counting). Rust is not optional here; it is the layer that makes the OS-level features possible from a web-based frontend.

Short app sound effects are also played from Rust. The packaged app sounds are standardized as 48 kHz stereo 16-bit PCM WAV files and use a dedicated app-sound output path separate from the user media player. See `docs/features/app-sounds.md`.

On mobile, Rust still runs as the Tauri core, but desktop process management, PTYs, global mouse events, tray integration, native browser messaging, arbitrary working folders, and desktop activity monitoring are unavailable. A separate mobile composition root links only shared domain services and supported adapters. Android keeps the canonical vault and SQLite database in app-private storage. Future platform-native adapters will own content URI transfers, notifications, lifecycle events, deep links, permissions, and Media3 audio playback.

### TypeScript

Used throughout the Svelte frontend. Provides type safety across the heavily interconnected state of the app: calendar triggering environment switches, Pomodoro triggering overlays, Yjs document updates propagating across the editor and sync layer simultaneously.

### HTML / CSS

Standard web rendering inside the Tauri webview. CSS handles a significant part of the ambient UI: backdrop blur, transitions on the edge panel and overlays, fullscreen Pomodoro blocked-screen aesthetics, the Notion-like editor layout, visual novel dialogue presentation (planned). Global app tooltips keep the hover delay, then paint at their final opacity and position with no entrance animation.

---

## Core framework

### Tauri v2

The desktop and mobile shell. Wraps the Svelte frontend in a native window and exposes Rust commands to the JS side. Chosen over Electron for lower memory footprint and binary size, which matters for an always-open app. Tauri v2 adds official Android and iOS support, allowing shared Svelte and Rust layers across desktop and mobile with separate platform compositions.

Key desktop Tauri v2 capabilities used in this project:

- **Multi-window management.** Separate windows for the main app, the edge panel, the fullscreen Pomodoro overlay, secondary monitor blockers, and custom notifications. Each independently configured (frameless, transparent, always-on-top, positioned to screen edges or center). On Linux, secondary Pomodoro blockers use native GTK/GDK black windows for more reliable Wayland multi-monitor coverage.
- **Detached module windows.** Primary title-bar tabs can be moved into app-owned secondary windows. While a tab is detached, the main window hides that tab and the detached window locks navigation to that view. The tab can be reattached from the detached window's context menu or by dragging it back onto the main title bar. Detached windows bridge live state through app events: theme changes apply everywhere, calendar mutations reload other windows from SQLite, and the main window coordinates the Pomodoro timer so secondary windows mirror and control the same session.
- **Single-instance enforcement.** Linux desktop bundles declare `SingleMainWindow=true` and `X-GNOME-SingleWindow=true` so GNOME-style docks know Ganbaru AI is a single-main-window app. The Tauri single-instance plugin enforces the runtime rule: a second launch focuses the existing main window and exits the extra process.
- **Startup presentation.** The main window is created hidden and revealed only after folder validation, configuration and theme hydration, and the first Svelte mount. A native timeout fallback reveals the window if the frontend never sends its readiness signal, preventing a permanently hidden process while avoiding normal black, blank, and resize flashes.
- **`set_always_on_top`**. Used for the notification window and fullscreen Pomodoro overlay to appear above all other apps including the Windows taskbar.
- **`set_fullscreen` + `set_decorations(false)` + `set_transparent`**. Combined to produce the fullscreen Pomodoro overlay that covers the taskbar during break and idle blocked states.
- **`setIgnoreCursorEvents`**. Allows the custom notification window to be non-interactive when desired, so it does not interrupt work in other apps.
- **Tray icon API.** The app lives in the system tray when not focused, essential for an always-running productivity tool.
- **Native messaging host binary.** The repo-owned `ganbaru-ai-native-messaging` Rust binary lets Chromium-based browser extensions ask the local app state whether a page should be blocked. Local setup generates separate app and dev native-host launchers so the normal extension and dev extension can be installed at the same time without sharing app config.
- **Rust filesystem services.** Authorized Rust commands and domain services read and write Markdown files and Ganbaru AI folder assets without a broad frontend filesystem plugin.
- **Release and update pipeline.** GitHub Actions builds Linux x64 packages, Windows x64 installers, and signed Android universal APK and AAB artifacts. The protected signing job owns the desktop updater key and Android keystore, while a separate write-token job publishes a draft GitHub Release. Release builds inject a generated updater config from GitHub Actions variables, so the public updater key and GitHub Releases feed are embedded only in desktop release artifacts. Release builds check GitHub Releases at most once per day by default to notify users about available updates; downloads and installs remain user initiated from Settings, Updates or the update prompt. The Tauri self-updater installs AppImage and Windows updates only. Direct Android installs update through APKs carrying the same signing certificate. Linux package-manager installs copy apt, dnf, zypper, yay, or paru commands for the user to run. Published `.deb` and `.rpm` releases register Ganbaru AI package repositories hosted on GitHub Pages, and the package repository metadata is signed by a separate GPG key. Published releases also update the `ganbaru-ai-bin` AUR package for Arch-based users through a dedicated AUR SSH key in the protected release environment. The update prompt can open the matching GitHub Release page in the default browser through Tauri's opener plugin.

**Desktop-only features** (not available on mobile due to OS sandboxing or plugin support): process management, PTYs, native messaging, global mouse position polling, tray, global shortcuts, single-instance handling, self-updater, always-on-top multi-window, edge panel, work environment switching, fullscreen break overlay, and desktop activity monitoring.

**Mobile** is a focused capability set rather than the desktop composition with controls hidden at runtime. The implemented repository foundation includes a stable production identifier, an official `.dev` Android debug application ID suffix with a `Ganbaru AI Dev` launcher label, an API level 29 Android override, separate desktop and mobile Rust roots, target-scoped desktop dependencies, explicit first-use consent for an app-private mobile vault, a least-privilege Android capability, a typed frontend platform profile, an adaptive shell for Calendar, Projects, Notes, Quick notes, Pomodoro, Settings, and Music, and a separate mobile frontend entry with build-time platform adapters. Android Music reuses the shared Svelte player and SQLite library while a native Media3 `MediaSessionService` owns selected-document audio playback. Android production assets exclude the desktop App shell, Chat workspace and store, Rodio playback, native desktop media controls, soundscapes, Doomscrolling runtime, benchmark surfaces, and working-folder panels. Pull requests build an ARM64 development APK. The protected release workflow is configured to compile all four supported ABIs into signed universal APK and AAB artifacts with pinned build versions, 16 KB compatible 64-bit native alignment, privacy-preserving backup rules, a narrow capture provider, and a checked base manifest. Complete backup UX, broader physical-device validation, durable signing-key provisioning, signed release acceptance, and distribution remain roadmap work. The architecture and exact status are defined in [features/mobile.md](features/mobile.md).

---

## Frontend

### Svelte 5

The UI framework for both desktop and mobile. Plain Svelte 5 with Vite, not SvelteKit. File-based routing, SSR infrastructure, and adapter abstractions are unnecessary for a Tauri desktop app that is a single-window SPA with navigation handled by a rune store. Chosen over React for this specific project because:

- **Runes (`$state`, `$derived`, `$effect`)** handle fine-grained reactive state without cascading re-renders. For an app where a single event (Pomodoro ending) must simultaneously update the calendar, trigger the overlay, switch the work environment, update the tray, and push a Yjs update, this matters in terms of code clarity and maintainability, not just raw performance.
- **Less boilerplate.** No `useMemo`, `useCallback`, `useRef` patterns needed to avoid re-render storms. Easier to maintain solo long-term as complexity grows.
- **Svelte 5 is the stable default.** New projects scaffold with Svelte 5 + Vite. No version split to navigate.
- **Shared codebase with adaptive presentation.** The same typed Svelte components and domain controllers run inside Tauri's WebView, while explicit platform capabilities, window-size classes, and mobile navigation select the correct UX.

### Vite

The build tool. Default bundler in the Tauri + Svelte scaffold. Handles HMR during development and production builds. Not a separate choice; it comes with the scaffold.

### pnpm

The package manager. Chosen over npm and yarn for strict dependency isolation (no phantom dependencies, meaning packages can only import what they explicitly declare), disk efficiency via content-addressable storage (shared dependencies are linked, not duplicated across workspaces), and first-class workspace support that Turborepo builds on top of.

Development uses Node.js for the Svelte, Vite, TypeScript, Tailwind, Vitest, pnpm, and Tauri CLI toolchain only. Node is not bundled with the shipped Tauri app. Node 24 LTS is the recommended local version through `.nvmrc` and `.node-version`; the package engine accepts Node 22.12.0 or newer on the Node 22 LTS line while it remains maintained.

### Turborepo

The monorepo task runner. Sits on top of pnpm workspaces and handles build orchestration, task dependency resolution, and local/remote build caching across all packages. Task dependencies are declared in `turbo.json`, and Turborepo ensures the correct execution order and parallelization across the app and shared packages.

Turborepo does not replace Tauri's build pipeline; it invokes `tauri build`/`tauri dev` as a task. The cargo workspace and pnpm workspace coexist at the repo root: Turborepo orchestrates JS/TS tasks across pnpm workspace members, while Cargo handles Rust builds. The Tauri CLI invokes cargo for `apps/client/src-tauri`.

### Rust workspace architecture

The Rust backend is a Cargo workspace with an intentionally small Tauri shell and Tauri-free domain crates. This keeps platform authority at the application boundary, makes domain tests cheaper to compile, and prevents the complete desktop backend from being emitted into every mobile library format during normal desktop development.

| Package | Responsibility |
| --- | --- |
| `apps/client/src-tauri` (`ganbaru-ai`) | Tauri build script, configuration, migrations, capabilities, generated context, desktop entry, and mobile library entry. |
| `apps/client/src-tauri/app` (`ganbaru-tauri-app`) | Tauri command adapters, managed state, setup and exit hooks, active-folder authorization, native credentials, dialogs, webviews, window operations, Projects, and Music. |
| `ganbaru-working-folders` | Working-folder IDs, repository kinds, UTC timestamps, binding DTOs, and pure device-state operations. |
| `ganbaru-db` | SQLite pool registry, connection configuration, embedded migrations, and the shared row conversion macro. |
| `ganbaru-chat-contracts` | Stable Chat errors, IDs, commands, events, configuration, provider-neutral DTOs, and Serde wire contracts. |
| `ganbaru-chat-providers` | Provider processes, transports, drivers, factories, event sinks, cancellation, and registry. |
| `ganbaru-chat` | Chat repositories, canonical event projection, runtime, Git workspaces, checkpoints, review, source control, bounded file operations, and application services. |
| `ganbaru-notes` | Notes domain, persistence, imports, exports, history, assets, validation, and bounded filesystem operations. |
| `ganbaru-native-messaging` | Independent `ganbaru-ai-native-messaging` browser-extension host and its binary-local tests. |

Dependency direction is one way. Chat contracts depend on working-folder contracts. Chat providers depend on Chat contracts. The Chat service depends on contracts, providers, and working-folder contracts. Notes, database services, and native messaging remain independent production packages. Chat and Notes use `ganbaru-db` only as a development dependency for migrated database tests, while production services receive an authorized `&SqlitePool`. `ganbaru-tauri-app` composes the packages and retains all Tauri, active-vault, native keyring, dialog, webview, and managed-state authority.

The desktop `main.rs` calls the desktop composition directly. The root library is mobile-only and retains `staticlib`, `cdylib`, and `rlib` outputs required by Android and iOS. Separate implemented desktop and mobile composition roots register their own commands, managed state, plugins, and lifecycle hooks. The current mobile root exposes the app-private SQLite core for Calendar, Projects, Notes, Pomodoro, Quick notes, Themes, Vault, and managed assets. It excludes local Chat execution, desktop notifications and overlays, Doomscrolling enforcement, desktop media, soundscapes, benchmarks, working-folder Markdown, path pickers, and desktop exit hooks. Desktop-only dependencies and Tauri features are excluded at compile time rather than initialized and hidden at runtime. Shared command contracts remain stable where the capability is genuinely common, while future platform adapters preserve distinct storage, lifecycle, notification, media, and permission semantics.

Core asynchronous work uses Tokio. Tauri runtime wrappers remain at the platform boundary. Chat change delivery and credential access cross that boundary through explicit traits, so the core service does not receive an `AppHandle` or native keyring access. Projects and Music remain in `ganbaru-tauri-app` because their current platform and cross-domain integrations do not yet justify separate packages.

---

## UI component libraries

### shadcn-svelte

The primary component source. Generates component source code directly into the project via CLI rather than installing a package. Requires Tailwind CSS v4, which uses CSS-native `@theme` and `@custom-variant` directives via the `@tailwindcss/vite` plugin instead of a `tailwind.config.js` file. Chosen because:

- Components live in your repo and can be freely modified and interconnected without fighting library abstractions.
- Svelte 5 native.
- Covers standard UI needs: dialogs, dropdowns, popovers, calendars, date pickers, command palette, tabs, and more.

### Calendar widget

The calendar and scheduler component. Handles day/week/month views, drag-and-drop event creation and resizing, overlapping event layout, and multi-day event spanning (the parts that would take months to build correctly from scratch). Has a native Svelte adapter and supports injecting custom Svelte components for event rendering, which is how the calendar integrates with the rest of the app's state. Calendar events ("session blocks") carry references to Pomodoro rhythm settings, work environment configs, and playlist assignments.

### svelte-dnd-action

Drag-and-drop library for the Kanban board. Handles column-to-column card movement, priority reordering, and task organization. Used for both personal Kanban (backlog → to do → in progress → done) and per-project Kanban boards within the project management framework.

### Theming

Registry-based theme system. Each theme is a single frozen object containing the 32-slot event color palette plus optional app and calendar shell token overrides applied via CSS variable injection. Adding a theme (built-in or user-authored) is one object in the registry; no other code needs to change. Events store stable slot IDs, not hex, so switching themes recolors the calendar instantly without migrating data. Theming is intentionally color-deep, not structure-deep: heavier UI edits (layout, typography, new components) are out of scope for themes and route through upstream contributions or a fork. User themes are stored normalized in SQLite alongside calendar events and pomodoro segments (one row per theme, plus per-token rows for sources, app shell, and calendar shell, with seed mirrors for reset and a `derivation_engine_version` stamp); built-in light and dark stay code-pinned. JSON acts as the import/export interchange only. Full design in `docs/features/themes.md`.

---

## Note editor

### Local Notion-shaped block graph

The first Notes implementation uses Svelte 5 components and pure TypeScript editor planning helpers over a Rust and SQLite page and block graph. The public command boundary follows Notion API concepts: pages, parents, blocks, rich text arrays, child pagination, timestamps, trash state, and local archive state. The app never calls Notion and does not depend on private Notion behavior.

SQLite is canonical for Notes. Markdown is import, export, preview, or agent bridge output only. It is never the editable source of truth.

The first editor slice supports page creation, nested sidebar page trees, favorites, recents, workspace search across page titles, blocks, and comments, emoji and managed image page icons, generated and managed image page covers, subpage creation from the sidebar, nested child pages, page duplication, page movement between the workspace and parent pages, selection, rename, archive, unarchive from the local Archive view, trash, restore and permanent delete from the local Trash view, derived backlinks, SQLite-backed page discussions and block comments, page mention, date mention, reminder mention, inline annotation rich text, inline equation rich text, and safe inline hyperlink rich text with local `@`, formatting, and link editors, page-only Notes links, and core block editing with paragraphs, headings 1 through 4, toggle headings 1 through 4, bulleted list items, numbered list items, to-do items, toggles, callouts, quotes, child pages, child database preservation, breadcrumbs, table of contents, columns, simple tables, tabs, media and file blocks, bookmarks, link previews, synced block preservation, template buttons, local button blocks, embeds, equations, dividers, code blocks, unsupported blocks, block colors, block links, nesting, Tab and Shift+Tab, sibling reordering by keyboard, menu, and handle drag, move to page, Enter behavior, Backspace behavior, slash conversion with actions, colors, filtering, session recents, plus-menu insertion, markdown-like start shortcuts, multi-line paste into canonical sibling blocks, and block handle actions for adding, converting, commenting, coloring, copying links, duplicating, moving, and deleting blocks.

Unsupported Notes blocks preserve validated import metadata and render as visible placeholders with their imported type, so future Notion import work can retain data Ganbaru AI cannot yet render.

Tiptap or another rich editing engine may be reconsidered later only if it can target the same canonical page and block graph without taking over persistence.

---

## Sync, collaboration, and backup architecture

This is one of the most important architectural decisions in the app. The system is designed around two deployment tiers plus a permission-aware collaboration model that uses the same underlying primitives.

### Core principle: local-first

Local storage inside the Ganbaru AI folder is always the source of truth. For Notes, that local source is SQLite. For markdown documents, it is the file. The app works fully offline with zero degradation. Sync is additive: it extends the local-first experience to other devices rather than replacing it. This is the fundamental difference from Notion, which requires internet access to function.

### Yjs

A CRDT library that can support parts of sync and collaboration. Future collaborative graphs can use Yjs documents or compatible typed operations whose updates merge from different devices and converge. This means:

- **Fewer generic text and collection conflicts.** Domain rules are still required for protected history, relational invariants, permissions, proposals, deadlines, and revocation.
- **Works offline.** Updates accumulate locally and sync when connection is restored.
- **Real-time collab and async sync use the same primitive.** A Yjs document does not care whether updates arrive 10ms or 10 days later.

Yjs was built specifically for collaborative local-first data and is used in production by Jupyter, several major Notion alternatives, and collaborative editor platforms.

### Hocuspocus

A production-grade Yjs server. Open source and self-hostable. Chosen over the simpler `y-websocket` because:

- Handles **persistent document state**, so new collaborators can load the full document even if they were offline when edits happened.
- Supports **presence and awareness**: who is online, live cursor positions.
- Has **authentication and authorization hooks**, needed for encrypted resource access control.
- Designed for persistent collaborative editing over Yjs.

Users run their own Hocuspocus instance on a cheap VPS, Raspberry Pi, or any cloud provider. The app includes guided setup instructions to make this as painless as possible.

### E2E encryption (libsodium / `@noble/ciphers`)

All data in transit and at rest on the sync server is end-to-end encrypted. The encryption key is derived client-side from the user's credentials and never leaves their device. The server stores and relays only ciphertext; not even you as the operator can read user data. This is a genuine competitive advantage over Notion and ClickUp, and a strong trust signal to privacy-conscious users.

Encryption contexts include:

- **Personal Ganbaru AI folder.** A personal root context known only to the user's authorized devices. The relay is a blind courier.
- **Collaborative resources.** Resource or subtree keys distributed through encrypted member and device envelopes according to group, project, channel, Notes, task-discussion, and working-folder grants. One broad workspace key cannot enforce restricted membership safely.
- **Revocation epochs.** Access reduction rotates the affected future key context and rejects stale offline operations. It cannot erase cleartext legitimately received before revocation.

### The two sync tiers

**Local only (default)**
The app works entirely offline. No sync server involved. The Ganbaru AI folder lives on the user's file system. Automatic local backups are a scheduled encrypted export (a zip of the Ganbaru AI folder) to a user-specified local path. The user can manually copy this to Google Drive, an external drive, or anywhere they want. The app does not manage this for them but makes the export trivial.

**Self-hosted sync**
The user runs their own Hocuspocus server. The app points to their server URL. Real-time E2E encrypted sync across all devices (desktop + mobile), automatic cloud backups, and collaborative workspaces with live presence. The app provides step-by-step setup guides for common cloud providers and local server options, automated where possible.

### Ganbaru AI folder structure on disk

See the Ganbaru AI folder directory tree in `AGENTS.md` for the canonical layout. Everything the app produces or manages lives in one folder, making backup and sync the same operation regardless of data type. First launch creates `Documents/Ganbaru AI` by default in production and `Documents/Ganbaru AI Dev` in development builds, while import lets a user reuse a folder copied from another installation.

---

## Data and state

### SQLite (via Rust commands)

Local database for all structured data and local document graphs in the app. This includes: Notes pages, block trees, rich text payloads, tags, and future bidirectional backlinks; calendar event indexes and session block configurations; Kanban task state, priority tiers, estimated vs. actual Pomodoro counts, and task-to-session-block links; work environment configs and blocker rulesets; Pomodoro session history; requirement version diffs (timestamped changes to task descriptions, scope, and acceptance criteria within the project management framework); diary entry indexes (the entries themselves are markdown files, but mood, energy, sleep quality fields are indexed for trend analysis); and app settings.

SQLite is the source of truth for Notes and structured productivity data. Diary content remains markdown on disk, with SQLite as the fast query layer over indexed fields and productivity metrics.

### Svelte runes (in-memory state)

Module-level `$state` objects exposed through getter functions (e.g. `getPomodoro()`, `getKanban()`, `getNavigation()`) manage live app state: current Pomodoro phase and timer, active work environment, which overlay is visible (break screen), current collaborative session, and presence data. The getter pattern keeps the API surface clean and encapsulates mutations. No external state manager or Svelte stores (`writable`/`readable`) needed; runes handle it natively.

Frontend background work uses lifecycle schedulers backed by injectable clocks. A scheduler owns one concern, runs at most one asynchronous request, coalesces repeated invalidations into one rerun, and ignores deadlines returned by stale work. Calendar and Notes notifications schedule their exact next deadline and catch up when the window resumes or regains focus. Pomodoro, Music, Doomscrolling, and extension status schedulers exist only while their corresponding timer, source, rule, limit, or settings surface is active, so an idle app does not keep high-frequency application polling alive.

---

## Data architecture: documents vs structured data

Ganbaru AI has two fundamentally different categories of data, and each uses the storage format that fits it. Calendar events, Notes page and block graphs, and other relational data belong in SQLite. Diary entries and project working documents belong on disk as user-editable files.

### Documents: markdown on disk, SQLite indexes

Diary entries and project documentation are markdown files stored in the Ganbaru AI folder. SQLite stores metadata such as title, tags, backlinks, and mood or energy fields for fast queries, but the `.md` file is always the source of truth for that content.

Why markdown for documents:

- Human-readable. Users can open their notes in Obsidian, VS Code, or any text editor.
- Git-friendly. Diffs are meaningful, merges are possible, history is inspectable.
- Portable. No lock-in. If the user stops using Ganbaru AI, their writing is intact.
- AI-agent-friendly. Any AI coding agent can read and write markdown natively, with no tools or plugins required.

Why not SQLite for those document files: they need plain-file ownership, meaningful diffs, and direct editing in external tools.

### Notes: SQLite page and block graph

Notes are the deliberate exception to markdown document storage. A Notion-like editor needs stable block ids, nesting, pagination, parent objects, type payloads, trash state, unsupported block preservation, and transactional edits. SQLite stores that graph directly, while markdown remains a derived import, export, preview, or bridge format.

Project working-folder Markdown is not part of that exception because those files already have a canonical file identity. Rust scans each active project folder through the shared authorization boundary, and Notes displays matching `.md` files beside SQLite pages. Reads are UTF-8 and size bounded. Saves require the expected SHA-256 revision and replace the file atomically. No filesystem creation, rename, move, or delete command is exposed by this surface.

### Structured data: SQLite as source of truth

Everything with fields, relationships, and query requirements lives in SQLite. This includes:

- **Calendar events** with start/end times, recurrence rules, pomodoro config, music playlist, color, project ID, workspace association, attendees, alarms, and overrides.
- **Notes pages, blocks, and comments** with parent objects, text, inline annotation, inline equation, inline hyperlink, page mention, date mention, and reminder mention rich text arrays, type payloads, nested child ordering, discussion threads, search projections, trash state, and pagination cursors.
- **Kanban tasks** with status, priority, column position, estimated/actual pomodoro counts, linked calendar events, and project ID.
- **Workspace configurations** defining which browser tabs to open, which terminal to activate, which apps to launch/close, which blocker ruleset to apply, and which project context to load.
- **Pomodoro configs, runs, segments, pauses, and run events** with timestamps, phase history, normalized pause intervals, idle and suspend detection, and future XP computation.
- **Project definitions** linking all of the above: a project references its kanban board, its calendar events, its workspace config, and its notes directory.

Why SQLite for structured data:

- **Relational queries.** "Show all in-progress tasks for project X that have calendar events this week" is a JOIN, not a file-tree traversal.
- **Foreign keys.** A calendar event references a project, a pomodoro config, and a workspace. These relationships are enforced at the schema level.
- **Atomic updates.** Moving a task between columns or updating a recurring event series are transactions that either fully succeed or fully roll back.
- **Fast reads.** Querying today's calendar events or filtering tasks by status is O(index-lookup), not O(parse-every-file).
- **Concurrent access.** The Tauri frontend, background Rust processes (idle detection, file watcher), and the CLI can all query SQLite safely.

### Why NOT markdown (or JSON, or YAML) for structured data

Storing a calendar event as a markdown file would mean:

- **Parsing overhead.** Every query ("what events are today?") requires reading and parsing every event file, extracting YAML frontmatter, and filtering in application code. SQLite does this in microseconds via indexes.
- **No relationships.** A calendar event that references a project, links to kanban tasks, and carries a pomodoro config would need to store IDs as text and resolve them manually. There are no foreign keys, no referential integrity, and no cascading deletes.
- **No atomic updates.** Updating a recurring event series (50+ instances) means writing 50+ files. If the process crashes mid-write, the data is inconsistent. SQLite handles this as a single transaction.
- **No concurrent access.** If the Tauri app and a CLI tool both try to update the same event file simultaneously, one overwrites the other. SQLite handles concurrent writers safely.
- **Query complexity.** "Show all tasks in project X that are in-progress and have a pomodoro session this week" would require reading every task file, every event file, and every session file, parsing each, joining in application code, and filtering. This is a single SQL query with JOINs.

JSON or YAML files have the same fundamental problems. They are slightly more structured than markdown but still lack relationships, transactions, concurrent access, and indexed queries.

A separate SQLite database per project was also rejected: it fragments data, makes cross-project queries impossible (e.g., "show all my events today across all projects"), and complicates backups.

---

## CLI for agent integration

### Why a CLI instead of MCP

All of Ganbaru AI's data operations are local: reading SQLite, writing SQLite, reading and writing files in the Ganbaru AI folder. For local operations, a CLI is strictly simpler than MCP:

- **No running server.** MCP requires a persistent process listening for connections. A CLI starts, runs the query, returns the result, and exits.
- **Zero setup.** Codex and other CLI agents call CLI tools via Bash natively. No plugin installation, no MCP configuration, no `.mcp.json` files.
- **Universal.** The CLI works with any AI agent (Codex, Cursor, Copilot), any script, any automation. MCP is specific to MCP-compatible clients.
- **No protocol overhead.** MCP adds JSON-RPC framing, capability negotiation, and connection lifecycle management. A CLI call is `ganbaru-ai task list`, output to stdout.

MCP becomes the right choice later for features that require persistent connections: pushing real-time notifications to agents ("your calendar event starts in 5 minutes"), streaming progress updates, or bidirectional communication between the running Tauri app and an agent session. That is a post-MVP concern. For all CRUD operations and queries, the CLI is the primary interface.

### CLI design

The `ganbaru-ai` CLI is a Rust binary that links directly to the same SQLite access layer used by the Tauri app. It reads the Ganbaru AI folder path from the app's config and operates on the same database. Output defaults to human-readable text; JSON output mode returns structured JSON for programmatic consumption by agents.

```bash
# Project management
ganbaru-ai project list
ganbaru-ai project info ganbaru-ai
ganbaru-ai project create "my-app" --repo /path/to/repo

# Tasks / Kanban
ganbaru-ai task list --project ganbaru-ai --status in-progress
ganbaru-ai task add "Implement auth module" --project ganbaru-ai --priority high
ganbaru-ai task move 42 --column in-progress
ganbaru-ai task done 42
ganbaru-ai task list --project ganbaru-ai --json  # structured output for agents

# Calendar
ganbaru-ai calendar today
ganbaru-ai calendar week
ganbaru-ai calendar add "Deep work: auth" --start "2026-04-02 10:00" --duration 2h \
  --project ganbaru-ai --pomodoro deep --color indigo
ganbaru-ai calendar next

# Workspace
ganbaru-ai workspace list
ganbaru-ai workspace activate ganbaru-ai-dev
ganbaru-ai workspace current

# Pomodoro
ganbaru-ai pomodoro status
ganbaru-ai pomodoro start --task 42

# Export project state as markdown (for the project's git repo)
ganbaru-ai export kanban --project ganbaru-ai
```

### Markdown export for software project repositories

When a user manages a software project with Ganbaru AI, the structured data (tasks, calendar events, workspace configs) lives in the Ganbaru AI folder's SQLite database. But the project's git repository also needs project context for:

- **Collaborators who don't use Ganbaru AI.** They read exported kanban snapshots and reports to understand what's happening.
- **AI agents without the CLI installed.** They read markdown natively without any tooling.
- **Code review context.** A PR description can reference task numbers from the exported kanban.
- **Onboarding.** New contributors read the project state to orient themselves.

The CLI exports repo-facing views as markdown into the repository:

```bash
# Generate a kanban snapshot
ganbaru-ai export kanban --project ganbaru-ai > KANBAN.md
```

Example output of `ganbaru-ai export kanban`:

```markdown
# Kanban: Ganbaru AI

## In progress
- #42: Notes block editor [high] (assigned calendar: Mon/Wed 10:00-12:00)
- #45: Diary entry forms [medium]

## To do
- #48: Bidirectional backlink index
- #49: Note search and filtering

## Done recently
- #39: Pomodoro segment persistence
- #40: Calendar conflict warnings
```

**This export is a view of the database, not the source of truth.** The flow is:

1. User manages tasks and events in Ganbaru AI's UI (or via CLI). SQLite stores the data.
2. `ganbaru-ai export kanban` writes the current task state as markdown to the project repo.
3. Collaborators and agents read the markdown. It is always up to date because the export runs on commit (via a git hook) or on demand.
4. If a user edits an exported kanban snapshot directly, an explicit import command can validate and apply those changes back to SQLite.

This is the same model as GitHub: the issue database is the source of truth, but issues are viewable as markdown and editable via API. The markdown is portable and useful on its own, but it is derived from the structured data.

### The virtuous cycle: Ganbaru AI developing itself

Ganbaru AI's own development follows this exact workflow across four stages:

**Stage 1 (now, pre-CLI):** ROADMAP.md, AGENTS.md, and feature docs are hand-maintained markdown files in the repo. Codex agents read and write them directly. This works because markdown is the universal baseline that requires no tooling.

**Stage 2 (CLI exists):** Ganbaru AI's project management data moves into its own database. The CLI exports kanban snapshots and generated reports to the repo automatically (via git hook or CI). Agents can use either the CLI's structured output for queries or read the exported markdown for simple context.

**Stage 3 (full UI):** Development sessions are calendar events with pomodoro configs and workspace settings. Starting a "Ganbaru AI dev" calendar event auto-opens VS Code at the project root, switches the terminal, loads project notes, and activates the right blocker rules. The kanban tracks features and bugs across phases. Each PR links to a task.

**Stage 4 (agent integration, post-MVP):** Agents query the database via CLI before starting work, create calendar events for their planned work sessions, update task status as they complete items, and export repo-facing context on commit. The developer reviews agent work from the calendar and kanban views in Ganbaru AI's UI, not by reading raw git logs.

The feedback loop: every workflow friction discovered while building Ganbaru AI with Ganbaru AI becomes a feature improvement. The tool's own development is the primary test case for its project management system, its agent integration, and its markdown export format.

The repo-facing markdown format is a portability layer, not the canonical store. The source of truth stays in SQLite; generated markdown exists to make project state readable in tools that only understand files.

---

## OS-level features (Rust crates)

### `sysinfo`

Cross-platform crate for reading running processes and system info. Used for detecting and killing blocked apps and auto-launching apps when switching work environments. Works on Windows and Linux.

### `std::process::Command`

Rust standard library. Spawns desktop applications as part of work environment switching (e.g. "when switching to deep work mode, open VS Code and close Slack").

### `rdev` (or platform-specific Win32 / X11 APIs)

Global mouse position polling to trigger the edge panel. Tauri does not expose a cross-platform API for this natively. On Windows: Win32 `GetCursorPos`. On Linux: X11 or `rdev` as a cross-platform abstraction.

**Wayland limitation:** Wayland's security model fundamentally blocks global input monitoring and foreign window introspection by design. On Wayland compositors (now the default on most Linux distros), the edge panel trigger and active window tracking may not work without compositor-specific extensions (e.g., `wlr-foreign-toplevel-management` for wlroots-based compositors, which does not cover GNOME's Mutter). This is a known limitation on Linux. X11 and Windows work without issue. The app should detect the display server at startup and degrade gracefully: on Wayland without the required extensions, the edge panel falls back to a keyboard shortcut toggle, and Will system activity tracking is limited to in-app signals only (Pomodoro timer adherence, blocker events, task completions).

### Active window tracking

The Will system requires knowing which application the user is actively working in during Pomodoro focus periods. On Windows: Win32 `GetForegroundWindow`. On Linux: X11 `_NET_ACTIVE_WINDOW` (subject to Wayland limitations above). Tracked data includes app-switch count per session, time spent per application, and idle detection (no input events for a configurable threshold). This data feeds directly into Will Focus (distraction events) and Will Intensity (active creation time vs. passive time) scoring. All tracking is local-only and runs exclusively during active Pomodoro focus periods, never in the background.

### `notify`

Rust crate used by the desktop composition for filesystem watching. The Will Intensity system can detect file saves in authorized project directories during Pomodoro focus periods, and desktop vault observers can react to external file changes that require reindexing. Android does not link or run these arbitrary-path observers; its app-private vault and external content URI boundaries use mobile-specific lifecycle and transfer adapters.

---

## Browser extension and website blocking

### Chrome extension (manifest v3) + Firefox equivalent

Installed once during onboarding. Responsibilities:

- Detect current tab URL and report it to Tauri via native messaging.
- Receive blocklist updates from Tauri and enforce them (redirect to a custom branded block page).
- Open and close specific tabs as part of work environment switching.
- Content-specific blocking where possible. For example, blocking unrelated YouTube videos while allowing task-relevant content via keyword/channel matching logic. More sophisticated content analysis is planned for the AI layer (post-MVP).

The custom block page is a minimal, fully designed HTML page served by the extension, not the default browser block UI. Blocker trigger events during Pomodoro focus periods are logged for productivity analytics.

### Native Messaging API

The protocol connecting the browser extension to the local Rust native host via stdin/stdout JSON messages. The host reads Ganbaru AI's local config and Pomodoro runtime state, then returns allow or block decisions to the extension. This keeps browser data local and avoids adding a separate browser bridge dependency.

---

## Music / media player

A local-first media player integrated directly into the AGPL 3.0 app. Rodio/Symphonia audio playback and local media probing live in the internal Rust media player module at `apps/client/src-tauri/app/src/media_player.rs`; playlist data, media registration, YouTube host URLs, and local video loopback hosting remain in `apps/client/src-tauri/app/src/music.rs`. Supports two sources: local files (primary) and YouTube via the official IFrame API (secondary).

Android keeps those provider-neutral frontend and persistence contracts but replaces desktop local-path and playback integrations at build time. `crates/ganbaru-mobile-media` provides a narrow Tauri mobile plugin backed by ExoPlayer and a `MediaSessionService`; Android's Storage Access Framework provides persistent read grants for selected music trees and artwork. The adapter performs bounded document-tree scanning and metadata work off the main thread. The Android bundle excludes Rodio, desktop tray events, MPRIS, Windows media controls, desktop file revealing, soundscapes, and desktop relink dialogs.

### Current local playback path

Desktop local audio playback is Rust-controlled through `apps/client/src-tauri/app/src/media_player.rs` with Rodio and Symphonia. Desktop local video uses the browser media element inside the Tauri WebView with file access provided by a token-gated Rust loopback media host on `127.0.0.1`. The Rust side validates the file path, scans user-selected folders outside the UI thread, registers only selected files, and streams byte-range responses with media content types.

The WebView path is the intentional local video path. It can play only the formats and codecs supported by the user's platform WebView, but it delegates video decoding and rendering to the media stack that Tauri already ships on each platform. On Linux this is WebKitGTK, which uses GStreamer internally for web media. On Windows this is WebView2. On macOS and iOS this is WKWebView. On Android this is Android WebView.

Local audio volume, mute, pause, seek, rate, duration, and position snapshots are internal Rust media player operations. Local audio, local video, and YouTube volume are capped at normal `100%`. Local video uses the WebView media element directly and avoids Web Audio gain routing so playback can follow default output changes more reliably on Linux Bluetooth setups. When the OS reports an audio device change, the frontend recreates active local video media at the current position and resumes if it was playing.

Hardware and Bluetooth media controls use the existing browser Media Session API where the WebView supports it. Linux desktop builds also expose a lightweight MPRIS bridge through GTK/GIO in `apps/client/src-tauri/app/src/media_controls.rs`, so desktop media widgets, keyboard media keys, and Bluetooth AVRCP controls can call the persistent music player even when local audio is playing through Rodio instead of a WebView media element. Windows desktop builds use the same module to expose System Media Transport Controls from the main native window, keeping Windows media flyouts and hardware controls on the same player action path.

### Rust audio backend target

The production local audio backend is Rust-controlled through `apps/client/src-tauri/app/src/media_player.rs`. Rodio with default features disabled remains the default local audio target, with only playback plus the needed Symphonia decoding features enabled for common local music files. This gives the app a small audio-only path before it initializes any video-capable multimedia framework. The backend owns local audio decoding, transport controls, volume, mute, seeking, rate changes, duration, position snapshots, and audio device output. Metadata and artwork extraction still use the existing Rust music commands.

Rodio is the first target because it is a RustAudio playback crate, uses CPAL for cross-platform audio output, supports player controls such as play, pause, seek, volume, and speed, and uses Symphonia as the default decoder backend for common file types. The approved dependency shape is `rodio` with `default-features = false` and features limited to `playback`, `symphonia-flac`, `symphonia-mp3`, `symphonia-isomp4`, `symphonia-aac`, `symphonia-alac`, `symphonia-ogg`, `symphonia-vorbis`, `symphonia-wav`, and `symphonia-pcm`.

Native video is not part of the shipped local playback path. An app-owned video backend is not clearly better for RAM than the platform WebView stack and increases the platform surface area. A single GStreamer backend is especially poor as a cross-platform default: Linux WebKitGTK already uses GStreamer internally, while Windows, macOS, iOS, and Android are better served by their platform WebView media stacks unless measured evidence proves otherwise. Platform-specific native video should only be revisited for a concrete RAM, speed, compatibility, or stability problem.

### Rodio and Symphonia audio path

Rodio is the selected first Rust-backed local audio playback target. Rodio provides the playback controller and audio output path through CPAL. Symphonia provides decoding for common music containers and codecs. This path replaces the WebView audio element for normal local music playback and is optimized for instant pause, instant resume, bounded buffering, and low audio-only RAM usage.

The Rodio/Symphonia audio feature set supports MP3, FLAC, M4A/MP4 with AAC or ALAC, OGG Vorbis, and WAV/PCM. Opus, WMA, APE, AIFF, and other unsupported audio formats should either fail explicitly or use a later broad multimedia fallback. They should not silently reintroduce the hidden WebView audio element as the normal audio path.

### WebView video path

Local video is hosted through the app's loopback media server and rendered by a normal WebView media element. This keeps the implementation aligned with Tauri's platform media stack, avoids extra video runtime packaging, and avoids a second rendering surface over the app window. The shared Music volume cap matches HTML media element behavior at `100%`. Visualizations and an audio-only video mode remain planned follow-ups.

### FFmpeg or libav fallback role

FFmpeg or libav is a fallback candidate for broader local media support, not part of the current shipped local playback path. If it is added later, it must stay lazy and separate from the Rodio and Symphonia audio-only path so normal local music does not initialize a video-capable backend.

### Symphonia decoder role

A pure-Rust audio decoding library. Used through the first Rust-backed audio backend when only audio playback is needed, such as Pomodoro playlists, background music, and morning alarm audio. Avoids initializing a full video-capable stack for audio-only sessions, reducing the memory footprint during typical productivity use. Broader backends can still handle video or unsupported audio formats later.

### YouTube integration (IFrame API)

YouTube is the secondary media source, integrated via the official IFrame Player API. The IFrame API is officially supported, free with no developer key required, and imposes no per-user limits. It works immediately without user registration or setup friction.

The API allows full programmatic control: loading videos by ID or URL, setting start/end times via URL parameters (`start=`, `end=`), controlling volume and playback speed, seeking to specific timestamps, and switching playlists based on Pomodoro phase.

A transparent `<div>` overlay covers the iframe, intercepting all mouse and touch events before they reach the YouTube controls. The user sees the player normally but cannot interact with it; all control is handled programmatically based on the user's preconfiguration. This is not a hidden or headless player. The player remains fully visible and ads play normally. It is simply input-locked. If the user is logged into YouTube Premium in the WebView session, ads do not appear; this is their account's behavior, not the app's doing.

The IFrame API is stateless. Playback position is saved to SQLite before the app closes and restored on next launch by passing the saved timestamp as the `start` parameter when reinitializing the player.

### Spotify: explicitly not supported

Spotify is not supported. As of 2025-2026, their API policies make third-party indie integration effectively impossible: development mode is capped at 5 users with no viable path to scale, extended quota requires 250,000 monthly active users and a legally registered business, and the gap between 5 users and 250,000 MAU is a deliberate policy to shut out indie developers while grandfathering existing large integrations. This decision will be stated publicly in the project's documentation.

### User preconfiguration

Users configure the following per session block or work environment template: which video, playlist, or local file to load; start and end timestamps; parts to skip (defined as timestamp ranges); volume level; playback speed; and whether to switch to a different source during breaks. This configuration is stored in SQLite alongside the session block data and applied automatically when the session block activates.

### Playlist and workflow integration

The media player is not a standalone feature; it is wired into the session lifecycle. Playlists are assigned to work environment templates and calendar session blocks. The Pomodoro system switches between focus and break playlists automatically. The sleep alarm system triggers the morning playlist on dismissal. Quick controls (play/pause, skip, volume) are accessible from the edge panel without leaving the current context.

---

## Desktop notifications and ambient UI

The implemented desktop ambient surfaces use secondary Tauri windows for fully custom UI rather than operating-system notifications:


| Feature                          | Implementation                                                                                                                                                          |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pomodoro completion screens      | Svelte terminal screens for event, day, and workweek completion using the same Rust/Tauri full-screen enforcement, secondary monitor blockers, monitor reconciliation, Linux shortcut and screensaver guards, Windows topmost and shortcut filtering, and macOS presentation and sleep assertions as break and idle screens |
| Fullscreen Pomodoro blocked screen | Svelte overlay for break, idle, and terminal completion states with Rust/Tauri enforcement: fullscreen always-on-top windows, secondary monitor blockers, monitor reconciliation, Linux shortcut and screensaver guards, Windows `HWND_TOPMOST` plus scoped shell-shortcut filtering and execution-state assertions, and macOS screen-saver-level windows plus presentation options and IOKit sleep assertions |
| Edge panel                       | Narrow always-on-top window anchored to the right screen edge, shown/hidden based on global cursor position polled from Rust                                            |
| Session summary screen           | Displayed during Pomodoro breaks with session completion stats                                                                                                          |

Android does not use these windows. Planned mobile timer boundaries use native notification channels and deadline reconciliation without keeping the Activity or WebView alive. The Android notification, alarm, lifecycle, and permission architecture is defined in [features/mobile.md](features/mobile.md).


---

## Gamification, skill tree, and NPC layer (deferred)

The skill tree visualization, visual novel NPC interactions, Will system, contracts, badges, Skill Capsules, streaks, and XP formula are all planned for later phases. Full specifications remain in PRODUCT_SPEC.md. No gamification code exists in the current codebase.

---

## Mobile (Tauri v2)

Tauri v2 has official Android and iOS support. Android is the first implementation target. The minimum is Android 10, API level 29, while release builds initially target and compile against API level 36. Tauri itself supports API level 24, but older Android versions are outside the Ganbaru AI compatibility objective. See the [Tauri Android distribution guide](https://v2.tauri.app/distribute/google-play/) and the [Tauri 2.11.2 Android template](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/crates/tauri-cli/templates/mobile/android/app/build.gradle.kts).

The Android product is not delivered yet. The existing source foundation includes `org.opengrimoire.ganbaruai` as the production identifier, the official `.dev` debug application ID suffix, an Android override with API level 29 minimum and one visible `main` window, separate desktop and mobile Rust composition, target-scoped desktop dependencies, a least-privilege Android capability, an app-private mobile default vault, and a typed frontend platform-capability registry. The registry restricts Android navigation to Calendar, Projects, and Notes and rejects desktop detached-view routes. The Svelte shell supplies icon-only top navigation on phones, a rail on larger windows, lazy Calendar, Projects, Notes, Settings, and Quick notes surfaces, mobile Calendar, Projects, Notes, Pomodoro, and Settings adaptations, a VisualViewport keyboard fallback, four-edge safe-area variables, and serialized Back handling for integrated nested layers that releases Tauri's listener at the Calendar root. Settings share one category registry and controller across platforms, while Vite selects platform renderers, detail loaders, image pickers, and zoom adapters so Android exposes truthful portable controls and capability statuses without packaging desktop integration graphs. Mobile Calendar, Quick notes, and theme synchronization use a no-op transport instead of granting core event authority to a one-WebView app. Android assets pin Chromium 111 as their build target, select only the Android Tauri capability, remove desktop frame and script origins from production content security policy, and are checked through a bundle contract that verifies destination closures, required mobile adapters, and desktop exclusions. Managed image assets and Notes database CSV files up to 512 KiB use preflighted WebView document input without broad storage permission. Browser and Rust boundaries validate image signatures, MIME, dimensions, and pixel counts before persistence. Whole-vault import uses a native bounded Storage Access Framework tree adapter and app-private staging, while remaining larger and binary transfers stay behind planned native content-URI adapters. Mobile startup atomically resumes one valid unexpired Pomodoro phase or closes unsafe persisted timer state, and page lifecycle hooks perform best-effort configuration, Notes, and Quick notes flushes.

The generated Android project pins Android Gradle Plugin 8.11.0, Gradle 8.14.3, Kotlin Gradle Plugin 2.2.10, SDK Build Tools 35.0.0, NDK 30.0.15729638, API level 36 for compile and target, API level 29 for minimum installation, and Java and Kotlin 17 bytecode. JDK 21 is the supported build runtime, and the repository Tauri wrapper selects it explicitly for Android commands instead of falling back to an incompatible Android Studio runtime. It compiles ARM64, ARMv7, x86, and x86_64 Rust libraries into universal APK and AAB artifacts. Pull requests build ARM64 only for bounded feedback, while the protected release job builds and verifies signed universal artifacts. The APK passes 16 KB ZIP alignment and its 64-bit libraries use 16 KB ELF load alignment. Launcher resources for every Android density are generated from `icons/icon-manifest.json` with `pnpm --dir apps/client run generate:icons`; the generated-project contract verifies the Ganbaru AI launcher fingerprint, production and development labels, and both normal and round manifest resources so Android project regeneration cannot silently restore Tauri's default icon or merge the two installed identities. The generated base project requests only network access, while the Media3 library manifest narrowly declares its media-playback foreground-service permissions. Android backup and device transfer exclude all app storage, and FileProvider exposes only app-scoped captured pictures. A read-only Android `WindowInsetsCompat` bridge supplies bounded system-bar and display-cutout values in CSS pixels so edge-to-edge content remains usable with gesture navigation, three-button navigation, rotation, and cutouts. Native and WebView force-dark processing is disabled because the app's explicit local theme selection owns its colors. A separate presentation-only bridge synchronizes the status-bar and navigation-bar icon contrast with that selected theme. Native whole-vault import and Media3 local Music are implemented. Remaining transfer, richer lifecycle, notification, alarm, deep-link and contextual permission adapters, encrypted user-controlled backup, complete backup UX, predictive Back validation, broader device and emulator validation, durable signing-key provisioning, signed release acceptance, and distribution remain roadmap work.

The canonical Android vault lives in app-private storage and remains a real filesystem tree for SQLx and managed assets. Android's Storage Access Framework is limited to explicit import, export, backup, restore, attachments, and selected external media. Its content URIs are never treated as `PathBuf` values. See the [Android storage guide](https://developer.android.com/training/data-storage), [Storage Access Framework guide](https://developer.android.com/training/data-storage/shared/documents-files), and [Tauri dialog behavior](https://v2.tauri.app/plugin/dialog/).

Android uses one Activity and one WebView. Compact windows use icon-only navigation in the global top bar and page or sheet details. Medium and expanded windows can use a navigation rail and list-detail panes. A read-only native inset bridge combines Android system-bar and display-cutout insets with the VisualViewport keyboard fallback. The bridge exposes no native mutation or application authority. The integrated Back stack unwinds current transient and navigation layers before yielding at the Calendar root. Physical Back behavior is validated on the Android 10 reference phone, while predictive Back still requires a newer physical device or emulator. The complete adaptive UX, lifecycle, security, performance, and test contract is in [features/mobile.md](features/mobile.md).

Implemented offline mobile capabilities are Calendar, Projects, Notes, Quick notes, foreground Pomodoro, Music, Settings, localization, themes, and scoped import and export flows. Diary, sleep alarm, native Pomodoro boundaries, Android Doomscrolling, sync, and communication-only Chat land as separate later milestones.

Native coding-agent processes, terminals, external working-folder execution, browser preview, Git tools, work environment switching, tray, global shortcuts, updater, edge panel, detached windows, fullscreen enforcement, and desktop activity monitoring remain desktop-only.

### Sleep alarm

The planned alarm system requires platform-specific APIs. Android uses an AlarmManager-backed native adapter. Exact alarm access is requested only if the final sleep-alarm behavior qualifies as genuinely time-critical and the user enables it after contextual explanation. It is not a base install-time permission. iOS uses its own notification and alarm capabilities. On alarm dismissal, the app can transition to the morning diary and start an eligible morning playlist. Setting the evening alarm can trigger the evening diary flow. Sleep duration remains a documented estimate rather than a claim of measured sleep.

### Mobile app blocking

Android Doomscrolling starts with opt-in usage awareness through `UsageStatsManager`. It avoids `QUERY_ALL_PACKAGES`, Accessibility Service, device-owner controls, and unrestricted overlays as substitutes for desktop process closing. A separately disclosed local VPN may be evaluated for domain filtering after privacy, battery, emergency-bypass, coexistence, and Play policy review. Every capability remains clear and useful when special permissions are denied. iOS uses a separate Screen Time and Family Controls design and is not assumed to have Android semantics.

The mobile app also serves as the device that reminds the user to return to their desktop. Calendar notifications for upcoming session blocks function as calls to action.

---

## Report generation

The project management framework generates automatic status reports from Kanban state, calendar data, Pomodoro history, requirement change history, and milestone progress. Reports are exported as markdown (native, just structured text output) or PDF. PDF generation uses Typst, a Rust-native typesetting system that takes structured data and templates as input and produces high-quality PDFs. Invoked via Tauri commands, keeping the rendering pipeline off the frontend thread. For PDF reading and text extraction (importing external documents into the project management framework), `pdfium-render` (Google's PDFium via Rust bindings) handles text extraction and page rendering without requiring a Python runtime.

---

## AI integration architecture

All AI features are opt-in. The app is fully functional with no provider configured. Ganbaru has one coordination model and three provider-access paths: local coding-agent harnesses, a future general BYOK path, and external clients. Channels, teammates, tasks, and decisions do not change identity when the provider path changes.

### Coordination layer

Chat channels, direct messages, reply threads, task discussions, persistent AI teammates, work assignments, manager proposals, context packages, agent runs, reviews, budgets, and provenance are structured SQLite data. Projects owns accepted work. Notes owns durable knowledge. Calendar owns time and capacity. The coordination layer chooses an authorized execution path but is not itself a provider protocol.

The base system seeds no AI teammate. Future teammate templates instantiate ordinary identities under the same access model. Durable teammate context comes from canonical records, selected Notes, bounded conversation context, future scoped memory, decisions, and run summaries rather than one infinite provider conversation. A planning proposal applies only to exact source and authorization revisions and becomes canonical work through typed Rust commands after the applicable approval.

### Local coding-agent path

Rust-owned native transports provide interactive coding execution:

| Provider family | Transport boundary |
|---|---|
| Codex | `app-server` JSONL over an owned child process |
| Claude Code | Native bidirectional stream JSON over an owned child process |
| Cursor Agent and Grok | ACP over owned standard input and output |
| OpenCode | Owned loopback HTTP and server-sent events, or an explicitly configured external origin |

The user installs and authenticates each harness. Ganbaru discovers configured executables, preserves provider-native models, approvals, questions, plans, usage, safety behavior, and continuation identities, and normalizes durable events in Rust. Svelte receives validated DTOs and never receives generic process, credential, or filesystem authority.

xterm.js renders thread-scoped terminal sessions created by narrow Rust commands in an authorized execution environment. It is a workspace tool, not the transport used to scrape or control provider output. Files, review, Git, checkpoints, worktrees, source control, and browser preview use the same Rust-owned authorization boundary.

### Organizational conversations and execution sessions

A channel or future DM is a durable organizational conversation. A provider thread is replaceable execution machinery. `chat_conversations` and `chat_channels` own organizational identity, while messages and reply threads link to work assignments and exact agent runs. An agent run optionally links to a hidden `chat_thread` and always binds an immutable teammate policy and authorization revision. A conversation run has no native filesystem target. A run that uses native files or commands binds one folder or explicit private-scratch target. No channel owns a provider, model, folder, or current provider session.

Calendar can select the linked project and suggest a channel or task while preserving drafts, reviews, and live execution. It never retargets a running provider continuation. Working folders remain execution resources selected by a task, teammate policy, or direct-agent action instead of the left-rail hierarchy.

### Context packages and agent runs

Every approved planning action and delegated run receives a versioned context package with exact task and requirement revisions, selected Notes or files, dependencies, Calendar constraints, relevant conversation context, prior summaries, instructions, effective permissions, and budgets. Package construction is permission-aware and auditable.

An agent run records the objective, teammate or task-agent identity, role-policy revision, provider, model, workspace, execution environment, context package, authority, budgets, lifecycle state, usage, deliverables, review state, and provider continuation. Parallel mutable work uses separate worktrees or execution environments. Work-in-progress, review capacity, dependencies, quotas, and cost ceilings constrain scheduling.

### General BYOK path

The future general path supports OpenAI API, explicitly supported OpenAI-compatible providers, Ollama, and other reviewed integrations. General AI teammates participate in the same authorized channels, DMs, tasks, and context-package model. They can use typed Ganbaru data operations but cannot edit arbitrary files or execute shell commands.

Credentials stay in the operating-system credential store behind opaque references. Each request records the provider, model, destination, consent, effective context scope, usage, and result needed for provenance and cost controls.

### CLI as the local data bridge

The planned `ganbaru-ai` CLI exposes typed Projects, Calendar, Notes, and workspace operations to separately authorized local agents and scripts. Application-owned coordination services can call the same Rust services without starting a shell. Neither the CLI nor an internal service is a participant identity. Agents never gain permission merely because they can guess a row ID, and exported Markdown remains derivative.

### MCP boundaries

The general MCP server is for separately authorized external clients. Chat provider sessions can also receive an ephemeral loopback-only internal MCP endpoint for bounded durable resources and controlled browser-preview tools. It is thread-scoped, bearer-authenticated, removed with the session, and never becomes the general Ganbaru data API.

Ganbaru may later consume external MCP servers for integrations such as email or external calendars. Those integrations follow the same participant, conversation, context-package, approval, and provenance rules as native operations.

### Future human collaboration

Permission-aware sync later lets people join project groups, projects, selected channels, selected Notes folders or pages, selected task discussions, or explicit project working folders. Direct and derived reads share one boundary. Search, notifications, reports, exports, and AI context packages cannot reveal inaccessible data. See `features/agent-coordination.md`, `data/sync.md`, and `data/security.md`.

---

## Monetization model

Everything is free. The project is sustained by donations via GitHub Sponsors.

| Feature       | Cost | Details                                                                     |
| ------------- | ---- | --------------------------------------------------------------------------- |
| Full app      | $0   | All features, local only by default                                         |
| Self-hosted sync | $0 | Bring your own Hocuspocus server (guided setup provided)                   |
| LLM BYOK      | $0   | Bring your own API key for any LLM features, no bill surprises              |

---

## Summary table


| Layer                     | Technology                                           | Reason                                                                             |
| ------------------------- | ---------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Desktop + mobile shell    | Tauri v2                                             | Low footprint, OS access via Rust, multi-window, iOS/Android support               |
| UI framework              | Svelte 5                                             | Fine-grained reactivity, less boilerplate, shared desktop/mobile codebase          |
| Build tool                | Vite                                                 | Default with Tauri + Svelte                                                        |
| Package manager           | pnpm                                                 | Strict dependency isolation, disk efficiency, first-class workspace support        |
| Monorepo orchestration    | Turborepo                                            | Task dependency resolution, build caching, parallel execution across workspaces    |
| Components                | shadcn-svelte                                        | Source-owned, Svelte 5 native, customizable                                        |
| Calendar UI               | calendar widget                                           | Only mature Svelte calendar with drag-and-drop                                     |
| Kanban drag-and-drop      | svelte-dnd-action                                    | Drag-and-drop for Kanban columns and task reordering                               |
| Theming                   | Registry-based theme store                           | One frozen object per theme; palette plus optional shell token overrides           |
| Note editor               | Svelte components + TypeScript planner + Rust commands | Notion-like UX over canonical SQLite page and block graph                        |
| Markdown bridge           | Import and export mappers                            | Derivative markdown for portability and agent bridge output                        |
| Sync / collaboration      | Yjs-compatible document graph planning               | CRDT-based future sync without replacing local SQLite canonical storage            |
| Collaboration cursors     | Planned awareness UI                                 | Live presence in shared workspaces                                                 |
| Sync server               | Hocuspocus                                           | Yjs server with persistence, presence, and auth hooks                              |
| Encryption                | libsodium / `@noble/ciphers`                         | E2E encryption, server sees only ciphertext                                        |
| Local DB                  | SQLite through Rust `sqlx` commands                  | Desktop configured vault and Android app-private canonical vault                     |
| Mobile document transfer  | Android Storage Access Framework adapters  | Implemented whole-vault tree import and theme transfer; planned backup, restore, attachments, and selected external media |
| Media engine (video)      | Platform WebView media element                       | Local video through Tauri's platform WebView stack                                 |
| Media engine (audio-only) | Desktop Rodio + Symphonia; planned Android Media3 + ExoPlayer | Low-resource desktop playback and lifecycle-correct Android background media |
| YouTube playback          | IFrame Player API                                    | Official, free, no developer key, full programmatic control, ToS-compliant         |
| Desktop file watching     | `notify`                                             | Desktop Ganbaru AI folder and authorized working-folder events                     |
| Desktop process control   | `sysinfo` + `std::process`                           | Desktop app blocking and environment switching                                     |
| Desktop activity monitoring | Win32 / X11 / `rdev`                              | Active window tracking, idle detection, and app-switch counting                    |
| Desktop mouse tracking    | `rdev` / Win32 / X11                                 | Edge panel trigger                                                                 |
| Browser bridge            | Native Messaging + Chrome/Firefox extension          | Website blocking and tab environment switching                                     |
| PDF generation            | Typst                                                | Rust-native typesetting, structured data → high-quality PDF reports                |
| PDF reading               | `pdfium-render`                                      | Google PDFium Rust bindings for text extraction and page rendering                 |
| Visual novel layer (deferred) | Custom Svelte components                         | JSON-driven dialogue state machine, NPC interactions in project management         |
| Chat coordination         | SQLite + Svelte + typed Rust commands                | Channels, teammates, work assignments, context packages, agent runs, and review    |
| Coding-agent execution    | Native provider protocols + Rust process ownership  | Durable provider sessions beneath channels and task-linked runs                    |
| Execution terminal        | xterm.js + Rust pseudoterminals                      | Thread-scoped terminal tool in an authorized working folder                        |
| BYOK AI teammates         | OpenAI / reviewed compatible APIs / Ollama          | General assistants in the same permission-aware coordination model                 |
| Mobile timer and alarm    | Implemented foreground Pomodoro recovery, with planned native notification channels and Android `AlarmManager`; later iOS adapter | Persisted Pomodoro boundaries and separately authorized sleep alarms |
| Mobile app awareness      | Planned Android `UsageStatsManager`; later iOS Screen Time adapter | Permission-aware awareness without claiming desktop-equivalent enforcement |
| Agent integration (CRUD)  | `ganbaru-ai` CLI (Rust)                               | Typed local operations over canonical data with explicit authorization             |
| Agent integration (external) | MCP (post-MVP)                                    | External AI clients accessing Ganbaru AI data remotely                              |
| Backend language          | Rust                                                 | Required by Tauri, OS-level APIs, media engine                                     |
| Frontend language         | TypeScript                                           | Type safety across interconnected state                                            |
