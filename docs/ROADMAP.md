# Roadmap

Phased development plan for Ganbaru AI. Each phase produces a working, testable increment. Feature dependencies are sequential where noted. Platform foundations can start earlier when they do not depend on a later feature layer.

---

## Phase 1: core loop

The minimum viable cycle: plan sessions, focus with a timer, track tasks.

**Includes:**

- Monorepo scaffold: Turborepo + pnpm workspaces + Tauri v2 + Svelte 5 + Vite
- SQLite setup through Rust commands with initial schema (calendar events, pomodoro configs, pomodoro runs, tasks)
- Calendar with session blocks: day/week/month views, drag-and-drop event creation and resizing, session blocks carrying task references and Pomodoro rhythm settings
- Basic Pomodoro timer: configurable focus/break durations, simple cycle counting per session block, timer state in Svelte runes, auto-start when session block activates
- Basic personal Kanban: four default columns (backlog, to do, in progress, done), task cards with priority tiers (easy/medium/hard/epic), estimated Pomodoro count, drag-and-drop reordering (svelte-dnd-action), task-to-session-block linking
- UI shell: shadcn-svelte component setup, global CSS theme variables, main window layout with navigation between calendar/kanban views

**Depends on:** nothing

**Out of scope:** fullscreen break overlay, edge panel, multi-window, work environments, browser extension, notes, diary, music, contracts, sync, mobile, NPC layer, gamification (XP, skill tree, streaks)

**Complexity:** large

**Platform:** cross-platform (desktop focus, no mobile-specific work)

---

## Phase 2: notes and Ganbaru AI folder

The local-first knowledge layer. Users can take notes linked to their tasks and projects.

**Includes:**

- Ganbaru AI folder structure on disk (vault.json, config.json, ganbaru-ai.sqlite, notes/, diary/, projects/)
- Tiptap note editor: block-based editing, slash commands, rich formatting, drag-to-reorder blocks
- SQLite-backed page and block graph, with Markdown and HTML kept as explicit import and derivative export formats
- Project-scoped navigation folders that organize workspace notes without replacing compatible nested-note relationships
- App-wide Quick notes panel with lightweight formatted notes, colors, masonry cards, pinning, Archive, and Trash
- Bidirectional backlinks tracked in SQLite (note-to-note, note-to-task, note-to-project)
- Note tags and search indexing in SQLite
- Daily notes (auto-created dated markdown files)

**Depends on:** phase 1 (SQLite, Tauri file system access, Kanban tasks for linking)

**Out of scope:** Yjs/collaboration, AI features, diary system (phase 3)

**Complexity:** medium

**Platform:** cross-platform

---

## Phase 3: diary and consistency

Daily touchpoints and engagement mechanics that make the app habit-forming.

**Includes:**

- Daily diary: morning and evening entry forms stored as dated markdown files in `Ganbaru AI/diary/`
- Diary indexed fields in SQLite: mood (5 options), energy level (5 options), sleep quality (5 options), daily intention, evening reflection
- Consistency tracking: session block completion rate, Pomodoro cycle completion rate

**Depends on:** phase 2 (notes and Ganbaru AI folder for diary storage and backlinks), phase 1 (Pomodoro/calendar for consistency data)

**Out of scope:** sleep alarm (mobile, phase 10), AI mood analysis (phase 11)

**Complexity:** small

**Platform:** cross-platform

---

## Phase 4: desktop experience

Multi-window desktop features that make Ganbaru AI an always-present productivity companion.

**Includes:**

- Tauri multi-window: notification popup (frameless, always-on-top, bottom-right), fullscreen break overlay (frameless, always-on-top, covers taskbar, semi-transparent), edge panel window (narrow, always-on-top, right edge)
- Fullscreen break screen: countdown timer, session completion stats, option to extend break, break playlist placeholder
- Edge panel: quick-access module icons, live Pomodoro timer indicator, active work environment name, quick-add Kanban task, task checkboxes for current session. Initially triggered by keyboard shortcut (global mouse polling added later)
- Work environment management: saved configurations (apps to open/close, browser tabs, blocker rules, playlist assignment), automatic activation when session block starts, app open/close via sysinfo + std::process::Command
- Chrome browser extension (manifest v3): native messaging bridge to Tauri backend, URL blocklist enforcement, redirect to branded block page, tab management for environment switching
- Doomscrolling: blocker trigger events logged as productivity analytics

**Depends on:** phase 1 (Pomodoro timer, calendar session blocks, Kanban tasks), phase 3 (diary for break screen context)

**Out of scope:** content-specific blocking (phase 11), Firefox extension (phase 11), music playback (phase 6), global mouse edge-panel trigger via rdev (can be added incrementally)

**Complexity:** large

**Platform:** desktop only

---

## Phase 5: gamification system (deferred)

The experience measurement engine, skill tree rewards, and contract system. Consolidated from the original phases 5, 6, and 7 into a single deferred phase.

**Includes:**

- Complete Will system with four categories (Focus, Clarity, Intensity, Execution)
- Full compound Activity XP formula with anti-grinding mechanics
- Skill tree: SVG + Svelte center-snap navigation, neighborhood culling, sub-layer navigation, skill decay, cross-branch connections
- Skill point spending, tier upgrades, badges, Skill Capsules
- Contract system: self-imposed conditions with in-app benefits and penalties
- Desktop activity monitoring for Will metrics (active window tracking, idle detection)
- Endowed progress effect (onboarding-based skill pre-population)
- Profile sharing

**Depends on:** phases 1-4

**Out of scope:** AI-assisted condition suggestions (phase 11)

**Complexity:** large

**Platform:** cross-platform (desktop-only for OS-level monitoring)

---

## Phase 6: music player

Local-first media playback integrated into the productivity workflow.

**Includes:**

- Internal Rust media player module for Rodio/Symphonia audio playback, app-command IPC, and frontend integration
- Local file playback: Rodio and Symphonia backend for common local audio, platform WebView media playback for local video, broader audio fallback candidates for unsupported codecs, lower audio-only memory overhead, and VLC-class transport latency
- YouTube IFrame API integration: load videos by ID/URL, start/end timestamps, volume and speed control, seek, desktop WebView client identity through a loopback HTTP player host plus referrer policy, `origin`, and `widget_referrer`, playback position persistence in SQLite, ad-compatible (YouTube Premium removes ads if user is logged in)
- Playlist management: definitions stored in SQLite (track paths, ordering, environment associations), create/edit/delete playlists, assign playlists to work environment templates and session blocks
- Session block integration: automatic playlist start when block activates, focus/break playlist switching on Pomodoro phase change
- User preconfiguration per session block: source selection, timestamps, skip ranges, volume, playback speed, break source override
- Edge panel music controls: play/pause, skip, volume
- Morning playlist placeholder (actual alarm trigger in phase 10)

**Depends on:** phase 1 (calendar session blocks for playlist assignment, SQLite), phase 4 (edge panel for controls, work environments for playlist association)

**Out of scope:** Spotify (explicitly not supported, as documented in project), collaborative playlists, AI music recommendations, and stream extraction from YouTube or YouTube Music

**Complexity:** medium

**Platform:** cross-platform (desktop primary, mobile audio-only path)

---

## Phase 7: CLI, native Chat execution, and channel foundation

The local agent execution and communication foundation. Native harness transports make provider work durable and inspectable. Project channels now sit above those provider sessions, so the left rail no longer treats isolated working-folder conversations as the organizational model.

**Includes:**

- `ganbaru-ai` CLI: Rust binary linking to the same SQLite, human-readable and JSON output, commands for projects, tasks, calendar, workspace, pomodoro, import, and export
- Project-owned working folders: one managed `projects/{project-id}/` folder for every project, optional device-bound external folders, and shared authorization for Chat and filesystem Notes
- Native coding-agent execution: Codex app-server, Claude native streaming, Cursor and Grok ACP, and OpenCode HTTP plus events through Rust-owned transports
- Durable provider threads: normalized canonical events, resume, approvals, questions, plans, usage, recovery, attachments, terminals, review, checkpoints, source control, worktrees, and browser preview
- Channel foundation: one durable `#general` channel per project, room drafts and history, channel-first navigation, archive and search, and links from channels to one or more bounded provider sessions
- Organizational teammate access foundation: inert vault-wide teammate identities, independent channel history and participation capabilities, targetless conversation runs, immutable access-profile revisions, exact folder tiers, one target for native work, explicit private scratch, strict channel disclosure, scoped internal host tools, and revocation-safe continuations
- Teammates studio and roster: wide Overview and Access editing, Group, Project, and Channel navigation, atomic impact-reviewed access replacement, device binding state, current-channel membership management, and typed English and Spanish copy
- Left-rail redesign: Channels and later Direct messages replace working-folder groups and New chat as the primary mental model; working folders remain visible where execution requires them
- Stable vocabulary and pre-release reset: organizational reply threads remain distinct from provider continuations, and development vaults reset instead of silently merging unrelated provider history
- Markdown export and import: CLI exports approved project state as Markdown to repositories for collaborators and agents without the CLI

**Depends on:** phase 1 (SQLite, Kanban, calendar for context), phase 2 (notes for project docs)

**Out of scope:** automatic planning and task spawning, proactive channel observation, shared or long-term teammate memory, multi-agent fan-out and orchestration, group or workspace channels, human collaboration, cloud runners, BYOK general assistant, external MCP, and content-specific blocking

**Complexity:** large

**Platform:** desktop only (terminal requires desktop OS)

---

## Phase 8: project management framework

Structured project lifecycle templates and the first manager-coordination layer. Chat discussions can become reviewable plans and approved Projects commitments without making conversation history the task database.

**Includes:**

- Project lifecycle phases: Genesis (brainstorming), Forging the idea (evaluation), The journey ahead (planning, MVP, execution, post-execution) with all subphases
- Phase templates: all planning subtemplates (deep brainstorming, market analysis, competitor research, specification, resources, review), MVP subtemplates (PoC, MVP, funding), execution subtemplates (alpha, beta, launch, polish), post-execution subtemplates
- Actionable methodology templates: reverse brainstorming, value proposition canvas, business model canvas, SWOT analysis, market research frameworks. Structured forms, not static documents
- Project Kanban boards: per-project boards linked to project phases
- Manager planning: reviewable proposals for objectives, tasks, subtasks, dependencies, acceptance criteria, assignments, reviewers, estimates, deadlines, budgets, risks, and scheduling effects
- Persistent AI teammate workflows and context packages: build planning and task behavior on ordinary user-created identities and the access foundation from phase 7, with versioned thread and task context, selected Notes and files, Calendar constraints, future scoped memory, authority, and budgets
- Mention-led participation: actionable `@teammate` invocation, channel membership, shared reply threads, human steering, semantic work state, and DMs without making temporary worker runs permanent sidebar contacts
- Teammate routing beyond the foundation: add Notes, Calendar, external-service, memory, and budget grants without weakening channel, folder, provider, scratch, and revocation enforcement; provider, model, effort, and fallback policy remain execution settings behind the teammate
- Task-linked agent runs: bounded execution, structured status, deliverables, usage, review-ready state, and links to exact provider execution timelines
- Requirement version control: timestamped revisions with requester, reason, origin discussion or review, approval, and downstream task, date, budget, Notes, deliverable, and run impact
- Review workflow: human or AI reviewer assignment, review target and late reason, corrections versus requirement revisions versus related work
- Sustainable delegation: work-in-progress limits, review-queue limits, dependency-aware parallelism, token and monetary ceilings, and exception digests
- Calendar date cascade: inserting or extending session blocks shifts downstream blocks, dependency graph propagation, conflict highlighting
- Automatic report generation: markdown reports from Kanban state, calendar data, Pomodoro history, requirement changes, milestone progress. PDF generation via Typst
- PDF reading: pdfium-render for importing external documents (text extraction, page rendering)
- AI-enhanced workflows: Chat teammates and task agents research competitors, help fill templates, validate ideas, execute approved work, and preserve decisions and discarded approaches with provenance

**Depends on:** phase 1 (Projects and Calendar), phase 2 (Notes for project knowledge), phase 4 (work environments for project contexts), phase 7 (native agent execution, channels, CLI, and workspace tools)

**Out of scope:** NPC visual layer, human collaboration, unrestricted automatic authority, and general BYOK provider support

**Complexity:** large

**Platform:** cross-platform (templates and forms), desktop only (report PDF generation via Typst, AI terminal)

---

## Phase 9: permission-aware sync and human collaboration

Multi-device sync and real-time collaboration via CRDTs and E2E encryption, with resource-level access boundaries that also constrain Chat, search, reports, exports, and AI context assembly.

**Includes:**

- Yjs-compatible CRDT integration: typed operations for Notes, Calendar, Projects, Chat, diary, and other collaborative state that merge without replacing local canonical storage
- Notes collaboration: real-time editing and cursors over the existing page and block graph rather than introducing a second editor-owned source of truth
- Hocuspocus server: apps/server package, persistent document state, presence and awareness, authentication and authorization hooks for resource access control, self-hostable
- E2E encryption: libsodium / @noble/ciphers, client-side personal root context, encrypted resource or subtree keys distributed according to group, project, channel, Notes, task-discussion, and working-folder grants, revocation epochs, server stores only ciphertext
- Two sync tiers: local-only (scheduled encrypted export to user-specified path), self-hosted (user's own Hocuspocus server with guided setup, automatic cloud backup)
- Multi-device sync: desktop-to-desktop, desktop-to-mobile (mobile in phase 10)
- Participant identity and roles: owner, administrator, member, restricted guest, invitation lifecycle, and device authorization
- Scoped membership: group, project, selected channels, selected Notes folders or pages, selected tasks or task discussions, and explicit project working-folder access
- Permission-safe derivations: search, mentions, backlinks, notifications, digests, dashboards, reports, exports, summaries, and AI context packages never reveal inaccessible data
- History and revocation: explicit prior-history visibility when inviting, future-read removal, offline revocation behavior, encrypted key rotation, and audit records
- Collaborative channels and DMs: human and AI-teammate messages, work threads, replies, mentions, membership, read state, and durable links to authorized Projects, Notes, Calendar, and agent runs
- Collaborative workspaces: shared documents, live presence, and conflict handling without replacing local canonical storage
- Local backup: scheduled encrypted zip export of the Ganbaru AI folder

**Depends on:** phase 2 (canonical Notes page and block graph), phase 1 (SQLite data model and Ganbaru AI folder structure), phase 7 (channels and stable execution identity), phase 8 (participant-ready assignments and reviews)

**Out of scope:** mobile sync client and hosted Ganbaru infrastructure. AI execution remains opt-in, but every AI read must honor the collaboration permissions introduced here.

**Complexity:** large

**Platform:** cross-platform (server is standalone Node.js)

---

## Phase 10: Android-first mobile

Tauri v2 mobile builds deliver a focused version of Ganbaru AI through platform-appropriate native adapters. Android is the first target, with Android 10, API level 29, as the minimum and API level 36 as the initial target and compile SDK. The durable architecture and UX contract lives in [features/mobile.md](features/mobile.md).

**Implemented foundation:**

- Shared Svelte 5, Vite, Tauri v2, typed frontend boundaries, SQLite services, and Tauri-free Rust domain crates
- A stable production identifier, an official `.dev` Android debug application ID suffix, an Android override with API level 29 minimum and one visible `main` window, and a Tauri mobile library entry point
- Separate desktop and mobile Rust composition roots, target-scoped desktop dependencies, a least-privilege Android capability, and an app-private mobile default vault
- A typed build-platform profile and frontend capability registry that restrict mobile routes to Calendar, Projects, and Notes and excludes desktop detached-view behavior
- An initial one-WebView shell with icon-only phone navigation in the global top bar, a larger-window rail, lazy Calendar, Projects, Notes, Settings, and Quick notes surfaces, all shared Calendar and Projects view modes behind compact touch controls, responsive Project toolbar overlays, Notes adaptations, Pomodoro, a shared capability-aware Settings system, VisualViewport tracking, a bounded native four-edge inset bridge, native system-bar theme contrast, nested root-yielding Back handling, and localized mobile copy
- A build-time mobile frontend entry and adapter graph that reuses the shared Music product through Android adapters while excluding the desktop App shell, Chat workspace and store, Rodio player, native desktop media controls, Doomscrolling runtime, benchmark surfaces, and working-folder panels from Android production assets
- Permission-free Android document selection for preflighted managed image assets and Notes database CSV files up to 512 KiB, including browser and Rust MIME, dimension, and pixel-count checks, plus a native bounded directory-tree importer for existing Ganbaru AI vaults; remaining larger or binary transfers stay behind native streaming content-URI adapters
- Atomic mobile Pomodoro cold-start reconciliation that resumes one valid unexpired running or paused phase and closes malformed, ambiguous, or expired state, plus lifecycle-triggered best-effort flushes for configuration, Notes, and Quick notes
- A production Android bundle contract with destination-specific source-module ceilings, required mobile adapters, and explicit exclusions for desktop and heavyweight editor graphs
- A reviewed generated Android project that builds ARM64, ARMv7, x86, and x86_64 Rust libraries into a universal debug APK and AAB with API level 36, pinned Gradle, Android Gradle Plugin, Build Tools and NDK versions, Java and Kotlin 17 bytecode, 16 KB compatible 64-bit native alignment, privacy-preserving backup exclusions, a narrow capture FileProvider, and only the base network permission
- Initial physical acceptance on the Android 10, API level 29, ARM64 reference phone for installation, cold start, background process restart, root Back, light system-bar contrast, portrait and landscape insets, compact top navigation, wide navigation rail, and the primary Calendar, Projects, Notes, Quick notes, Pomodoro, and Settings surfaces
- Explicit mobile first-use storage consent, separate `Ganbaru AI Dev` launcher identity, an ARM64 pull-request APK build, and protected signed universal APK and AAB release workflow configuration
- Shared Music player, playlist, review, source, artwork, YouTube, and interchange surfaces on Android, with bounded selected-document audio scanning and a Media3 `MediaSessionService` for background local playback and Android system controls

Remaining Storage Access Framework backup, restore, and attachment adapters, notification, alarm and deep-link adapters, native local-playlist queue handoff, encrypted user-controlled backup, complete adaptive backup UX, predictive Back and gesture-navigation validation, emulator validation, durable signing-key provisioning, signed release acceptance, and distribution are not implemented yet.

**Includes:**

- Android build foundation: legal stable package identity, generated native project, app-private canonical vault, separate mobile Rust composition, least-privilege mobile capabilities, release signing design, and Android CI compilation
- Storage interoperability: canonical SQLite and managed assets in app-private storage, with Storage Access Framework content URIs limited to import, export, backup, restore, attachments, and selected external media
- One-Activity mobile shell: compact top navigation, larger-window navigation rail and list-detail layouts, system Back, predictive Back preparation, edge-to-edge insets, input-method resizing, lifecycle restoration, and process-death recovery
- Mobile-adaptive Calendar, Projects, Notes, Quick notes, Settings, themes, localization, and touch interactions without hover or precision-drag requirements
- Mobile Pomodoro: persisted boundary deadlines, native notifications or alarms, foreground reconciliation, and notification-based breaks without a fullscreen overlay or a continuously running background timer
- Android music adapter: Media3, ExoPlayer, MediaSessionService, system media controls, and selected content URI sources behind shared queue and transport contracts
- Sleep alarm: a separately reviewed exact-alarm path only when justified by Android policy and user expectations, with an inexact or unavailable state handled explicitly
- Alarm-to-diary flow: morning alarm dismissal can open the morning diary and start an eligible morning playlist; setting the evening alarm can open the evening diary
- Responsible Android Doomscrolling: opt-in usage awareness first, targeted package visibility, no broad installed-app access, and no Accessibility Service or overlay used to imitate desktop process blocking
- Mobile Chat and review after sync: authorized channels, DMs, attention items, task discussions, proposals, and deliverable review without local coding-agent processes or desktop workspace tools
- Mobile sync through the permission-aware Phase 9 protocol
- Calendar notifications as calls to action, including reminders to return to desktop for upcoming session blocks
- iOS adapters after the Android contracts and shared mobile UX are stable, without assuming identical operating-system capabilities

**Delivery order:** Android build foundation, offline core, focus and storage interoperability, native media, connected experience, policy-sensitive features, then iOS alignment. The build foundation and offline Android core do not wait for Phase 9.

**Depends on:** phases 1 and 2 for the offline structured-data and Notes foundation; phase 6 for shared music contracts; phase 3 for diary and sleep-alarm flows; phase 9 only for sync, cross-device Chat, and collaboration

**Out of scope:** local coding-agent processes and terminals, external working-folder execution, work environment management, tray, global shortcuts, self-updater, edge panel, detached windows, fullscreen break overlay, browser extension, always-on-top windows, and desktop activity monitoring. These require desktop authority or a separate platform design.

**Complexity:** large

**Platform:** mobile only (iOS, Android)

---

## Phase 11: BYOK teammates, advanced AI, and MCP

The general-user provider path, advanced AI capabilities, and external access layer. It joins the same channels, DMs, teammates, tasks, context packages, permissions, and provenance model established by the local coding-agent path.

**Includes:**

- BYOK AI teammates: general assistants participate in authorized channels, DMs, and workflows without creating a second isolated per-project chat system
- LLM provider support: OpenAI API, OpenAI-compatible APIs (Groq, Together, Mistral, and any provider using a compatible chat format), Ollama for local models (Llama, Mistral, Gemma, no API key needed), and other explicitly supported provider APIs when users supply their own credentials
- BYOK configuration UI: API key management (stored locally), model selection, provider setup with guided instructions, consent controls
- Permission-aware context: every request shows or records the teammate, destination, context package, provider, model, consent, and effective data scope
- Natural language calendar management: "move my 3pm session to tomorrow", an authorized teammate proposes the change, accepted changes use typed Calendar operations, and external local agents can use the CLI
- Mood-aware motivation: using diary mood/energy baselines, AI adapts communication and suggests schedule adjustments
- Adaptive Pomodoro rhythm decisions: local-only analysis of focus rhythm outcomes, custom cadence experiments, and opt-in automatic tuning bounded by recovery and satisfaction guardrails
- Content-specific browsing relevance detection: LLM analyzes page content (not just URLs) for task relevance, smarter blocking on YouTube and similar platforms
- Local LLM diary analysis: small local models (via Ollama) analyze diary language for goal-setting, reflection quality, mood trends, no data leaves the device
- MCP server: exposes Ganbaru AI data to external AI clients (ChatGPT, teammate agents, and other MCP-compatible clients) that don't run locally
- MCP client: consumes external MCP servers for integrations (email, external calendars)
- Firefox browser extension: port of Chrome extension to Firefox manifest
- Edge panel global mouse trigger: rdev / Win32 / X11 polling for cursor position (replaces keyboard shortcut from phase 4), Wayland detection and graceful fallback

**Depends on:** phase 8 (teammates, context packages, proposals, and review), phase 7 (CLI and native provider foundation), phase 3 (diary for mood baselines), phase 4 (browser extension for content blocking)

**Out of scope:** this is the final planned phase

**Complexity:** large

**Platform:** cross-platform (BYOK teammates and AI features), desktop only (mouse trigger, Firefox extension, content-specific blocking)

---

## Systems coverage verification

Every system from the product spec is accounted for:


| System                                                           | Phase         |
| ---------------------------------------------------------------- | ------------- |
| Calendar (session blocks)                                        | 1             |
| Kanban (personal)                                                | 1             |
| Pomodoro timer                                                   | 1             |
| Note-taking (SQLite block graph, markdown bridges, backlinks)   | 2             |
| Daily diary (morning/evening)                                    | 3             |
| Consistency tracking                                             | 3             |
| Doomscrolling (desktop/browser)                                  | 4             |
| Work environment management                                      | 4             |
| Edge panel                                                       | 4             |
| Multi-window (break overlay, notification)                       | 4             |
| Browser extension (Chrome)                                       | 4             |
| Gamification (Will, skill tree, XP, contracts, badges, capsules) | 5 (deferred)  |
| Music player (local + YouTube)                                   | 6             |
| `ganbaru-ai` CLI                                                  | 7             |
| Native coding-agent execution and terminal                       | 7             |
| Project channels and channel-first Chat navigation               | 7             |
| Provider-session recovery beneath durable channels               | 7             |
| Markdown export/import for project repos                         | 7             |
| Project management lifecycle templates                           | 8             |
| Kanban (project, requirement version control)                    | 8             |
| Persistent AI teammates, mentions, and shared work threads       | 8             |
| Manager plans, context packages, task-linked agent runs           | 8             |
| Assignment, review, budgets, and sustainable parallelism         | 8             |
| Methodology templates                                            | 8             |
| Calendar date cascade                                            | 8             |
| Report generation (markdown + PDF)                               | 8             |
| Sync (Yjs + Hocuspocus)                                          | 9             |
| E2E encryption                                                   | 9             |
| Human identity, scoped membership, channels, and DMs              | 9             |
| Permission-aware collaboration and AI context                    | 9             |
| Android-first mobile (Tauri v2; iOS adapters later)              | 10            |
| Mobile Chat communication and task review                       | 10            |
| Sleep alarm (mobile)                                              | 10            |
| Doomscrolling (mobile awareness and policy-reviewed controls)    | 10            |
| BYOK AI teammates (OpenAI, compatible APIs, Ollama)              | 11            |
| AI: natural language calendar management                         | 11            |
| AI: mood-aware motivation                                        | 11            |
| Adaptive Pomodoro rhythm decisions                               | 11            |
| AI: content-specific blocking                                    | 11            |
| AI: local LLM diary analysis                                     | 11            |
| MCP server/client (external AI access)                           | 11            |
| Browser extension (Firefox)                                      | 11            |
| NPC characters and visual novel (deferred with gamification)     | 5 (deferred)  |
