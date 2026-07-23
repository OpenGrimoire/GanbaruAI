# Chat

Chat is Ganbaru AI's local workspace for coding agents installed and authenticated by the user. It gives Codex, Claude Code, Cursor Agent, and OpenCode one consistent interface while preserving each provider's native sessions, models, approvals, questions, plans, and safety behavior. Ganbaru does not proxy prompts through a hosted service.

The provider transport and broader AI architecture live in [AI integration](ai-integration.md). Security invariants live in [security](../data/security.md). This document defines the user-facing Chat product.

## Scope

Chat supports:

- Automatically discovered local instances for implemented provider families plus explicitly configured provider instances.
- Existing Ganbaru Projects and standalone coding workspaces.
- Durable local threads, drafts, attachments, search, archive, and restart recovery.
- Streamed assistant messages, reasoning summaries, commands, file changes, tools, tasks, warnings, usage, and cost when the provider reports them.
- Build and Plan interaction modes when supported.
- Native approvals, structured questions, steering, queued follow-ups, interruption, and force stop.
- Bounded file browsing, changed-file inspection, thread terminals, terminal context, and Git checkpoints.
- Wide, compact, detached, increased-font, reduced-motion, keyboard, screen-reader, and minimum-recovery layouts.

Remote clients, Slack, a general BYOK assistant, hosted execution, embedded web preview, branch publication, and pull-request automation are separate future surfaces.

## First use

On the first Chat load for a vault in each app session, Ganbaru runs one bounded discovery pass for every implemented provider family that is not configured or explicitly opted out. It checks each family's known command names on the application path and conventional user CLI directories, including standard pnpm, npm, Bun, Cargo, and Volta locations. When it finds an installed CLI, Ganbaru creates a default instance with the resolved device-local executable path, reuses the CLI's normal home and existing authentication, probes its status, and caches its model catalog when available. A person who already installed and authenticated Codex, Claude Code, Cursor Agent, or OpenCode can therefore bind a workspace and start chatting without entering an executable, provider home, credential, or instance ID.

Refresh all forces a new discovery pass before it re-probes every configured instance, so a CLI installed after Ganbaru started or after the first Chat visit appears without resetting application data. An installed CLI with a broken package, unsupported version, missing authentication, or another probe problem remains visible with a recovery status instead of being treated as absent. A command that is genuinely missing is not added.

If no supported CLI is available, Chat directs the user to provider setup and official installation guidance. The advanced provider setup remains available for additional accounts, custom homes, arguments, environment variables, and credential references. Removing the last instance for a family records an explicit opt-out so later automatic scans do not recreate it. Adding an instance manually enables the family again. The Advanced provider selector lists unconfigured implemented families as setup actions even when only one provider is currently configured.

After provider discovery:

1. Link an existing Ganbaru Project or create a standalone Chat workspace.
2. Bind that logical workspace to a local folder on this device.
3. Review the visible provider and model selection, safety mode, and Build or Plan before the first send.

Failed probes preserve entered fields. Missing folders, changed repository identity, unavailable providers, unsupported capabilities, and another device without a binding have distinct recovery actions. Automatic discovery never supplies credentials, changes the CLI home, installs software, chooses Full access, or hides the active provider and model from the composer.

Chat settings separate Providers, Models, Workspaces, and Behavior into compact tabs. They use the same keyboard-accessible custom selectors, toggle controls, section spacing, and semantic surfaces as the rest of Settings. Routine behavior remains immediately visible, while diagnostics and device-wide maintenance stay in a collapsed advanced area. Provider rows favor status and direct icon actions over large administration cards.

A workspace card states the Ganbaru Project relationship and the device-local folder as separate labeled values. Removing a folder link is an action distinct from the unbound status. New bindings open the native picker at the user's Documents folder when available. Rebinding opens at the current folder, or its nearest available parent if the folder moved or was removed.

## Workspace shell

The internal thread rail groups workspaces by current Ganbaru project groups and keeps standalone contexts separate. A project can have multiple logical Chat workspaces without adding machine paths to its Project record. The rail keeps New chat prominent and opens title search on demand from the header or keyboard shortcut. Drafts do not appear as thread rows. A row appears only after the first message creates a titled thread. The rail also provides indexed search, archive browsing, rename, read state, detach, open folder, restore, and confirmed permanent deletion.

The conversation header appears after a thread begins. It keeps the thread title dominant and places only workspace opening, the current branch, inspector state, and contextual actions on the right. A new draft avoids a duplicate title bar and instead places workspace, local execution, and branch context directly above the composer. The composer keeps only attachment and access controls on the left, then context usage, the selected model and effort, and the send action on the right. The model and effort trigger is transparent until hover, keyboard focus, or opening. Its chevron remains pointed downward instead of changing orientation when the panel opens. Focusing the message field does not repaint the whole composer border. The context indicator fills clockwise from the top and exposes exact usage on hover without a click action. The compact model control opens a low-profile model and effort ladder derived from the provider catalog. While open, the trigger smoothly grows to at least the measured panel width and centers its contents, then returns to the current label's intrinsic width when closed. For the OpenAI GPT-5.6 family, the ladder prepends the adjacent lower-cost tier at Light, then presents the selected tier's supported efforts. Internal `low` and `xhigh` values display as Light and Extra High without changing the values sent to the provider. The first and last stop positions account for the handle radius, so the handle edges align with the track caps and every handle remains centered over its dot. The ladder supports pointer capture and drag selection using the rendered stop centers. Every idle dot uses identical fixed geometry and grows subtly on hover. The handle remains enlarged after release while the pointer is still over it, then returns to its resting size only after the pointer leaves. Once pointer movement crosses a short drag threshold, dragging cross-fades the action row into localized Faster and Smarter guidance until release or cancellation. A stationary press selects normally without showing drag guidance. Dot controls explicitly opt out of the global tooltip host as well as native title tooltips. Ultra hides the earlier normal dots, fills the track with a dense blue-to-purple galaxy treatment, and applies the same semantic purple emphasis to the effort label. The separate lightning button controls Standard or Fast. Speed remains available across models when any applicable model in the provider catalog advertises the shared service tier, rather than disappearing because one model entry omits the duplicate definition. Fast replaces earlier filled option dots at every effort with a seamless, rapidly left-moving particle field while leaving later unselected dots visible; Ultra combines that motion with its purple galaxy palette. Handle movement, Ultra color, dot visibility, particles, and Fast indicators use short eased transitions with overlapping particle layers instead of abrupt state changes. Reduced-motion preferences keep the particle field static. Advanced smoothly replaces the compact view: the compact controls recede upward while Advanced rises from below and the panel interpolates between their measured heights. Its model, effort, speed, provider, interaction mode, and provider-specific rows reveal adjacent option panels on hover or keyboard focus, while click remains available for touch input. Direction-aware safe triangles keep a flyout open only while the pointer crosses the gap toward it, then close it after a bounded grace period when the pointer travels elsewhere. The same boundary logic supports flyouts beside or above the menu and preserves keyboard focus travel. Standard omits the optional Fast service tier, while Fast requests the provider tier explicitly and adds a speed icon to the model label. Voice input is not shown until Chat has an implemented local voice workflow. Keyboard shortcuts cover new chat, search, composer focus, next and previous thread, rail, inspector, Stop, and the command menu.

The desktop visual hierarchy follows the interaction patterns reviewed in T3 Code commit `62cf4617594c2a0d8c9c5dc4bbc2012d0ead802d`, adapted to Ganbaru rather than copied as a separate product shell. The rail uses an on-demand search field, readable project hierarchy, consistently spaced one-line thread rows without competing timestamps, and a pinned settings action. The new-draft surface contains only a compact workspace context strip and the composer, anchored at the bottom of the conversation area. Active conversations keep one compact header and a wide centered reading column. User messages size to their content within a bounded right-aligned bubble, assistant work stays open on the main surface, and secondary message metadata appears on hover or keyboard focus while remaining visible on touch input. Chat chrome uses the same thin icon weight, compact hit targets, semantic surfaces, and custom floating menus across the rail, header, composer, and inspector.

Ganbaru keeps its own main tabs, project groups, semantic theme tokens, local workspace identity, inspector, and responsive recovery model. It does not reproduce T3 Code branding, Electron window chrome, remote environment controls, or project management that already belongs elsewhere in Ganbaru.

Thread and workspace shells load without message history. Selecting a thread reads only its latest bounded timeline page. Older pages load around sequence cursors, and the client retains no more than eight loaded pages while preserving selected content and the scroll anchor.

## Timeline

The timeline renders provider-neutral projections from durable canonical events. User messages retain exact text, managed attachments, immutable file mentions, bounded terminal context, time, copy, and checkpoint-aware restore entry points. Assistant messages use sanitized Markdown, safe external links, bounded images and tables, incremental content, code copy and wrap controls, duration, effective model, token usage, cost, and changed-file summaries when known.

Commands, output, reasoning, file changes, tools, web activity, images, tasks, hooks, warnings, and failures use stable activity rows. Consecutive settled work can fold without hiding the final assistant response. Plan cards support continue planning, implementation in a later turn, copy, and dismissal. Interrupted and failed turns remain readable and accurately labeled.

Virtualization uses measured rows and stable IDs. Prepending older pages preserves the visible anchor. Scrolling away during streaming disables follow mode, shows unread work and Jump to latest, and never forces the reader to the end. Long threads may show a minimap in a safe side gutter.

Provider change notifications are coalesced to one serialized refresh per paint frame. Rust also batches adjacent compatible content deltas. This preserves durable sequence ordering without token-level SQLite reads or layout storms.

## Composer and provider interaction

The new-draft hero and docked composer share one persistent controller, so responsive layout changes do not replace the draft or undo state. Draft persistence includes text, mentions, attachment references, and explicit provider, model, trait, safety, and interaction selections. The docked composer floats over a protected bottom timeline inset. Its prompt is the dominant surface, while attachments, provider and model, traits, safety, interaction mode, context, and Send or Stop form one compact footer. The circular Send or Stop action remains inside the composer boundary at every supported width. This keeps routine conversation immediate without hiding the selections that define provider behavior.

Images can be chosen, pasted, or dropped. File and folder mentions are searched inside the authorized workspace and revalidated at send time. Provider skills and commands appear only when discovered, with stale entries rejected or retained as plain text. The composer exposes provider, model, typed traits, safety, Build or Plan, context, attachments, and Send or Stop directly. Provider, model, trait, safety, and interaction choices use keyboard-accessible custom popovers with icons and descriptions rather than platform-native selects. The model picker lists healthy configured instances, configured instances that need attention, and supported provider families that are not configured. Unavailable entries stay visible with a direct configuration path instead of disappearing from the picker.

Sending writes the user message, turn, attachment references, and idempotent command receipt before dispatch. A launch failure therefore leaves recoverable durable history with Retry, Edit into a new draft, and Change provider actions.

During active work, the provider capability decides whether a prompt steers the turn, becomes a durable queued follow-up, or remains an unsent draft. Stop is idempotent and has a bounded stopping state. Force stop is explicit. Approvals expose only choices offered by the provider and never choose a permissive default. Structured questions preserve partial local answers and validate required fields before submission.

## Safety modes

Supervised asks through the provider's native permission mechanism. Auto-accept edits accepts only the provider's verified edit scope and continues to ask for commands or wider access. Full access uses the provider's native unrestricted mode only after confirmation for the exact provider instance and logical workspace.

Full-access trust is device-local and cannot arrive through stale UI state. The active turn keeps its mode snapshot even if a later draft changes modes. Provider capability differences remain visible and disabled rather than simulated.

## Inspector, terminal, and checkpoints

The wider right inspector remains closed until the person opens it, even when a full column fits. Smaller layouts keep the inspector available as a sheet without covering the conversation by default. Opening or switching chats does not reveal the inspector or bottom dock automatically. The right inspector and bottom dock render the same workspace-panel component with different initial tabs. The compact tab uses the selected filename when a file is open, and its add button opens Terminal, Changes, Plan, or Files on demand instead of reserving permanent navigation for every tool. The add menu renders above clipping containers, measures the available viewport space, and opens upward when it cannot fit below its trigger. A selected workspace draft owns inspector state before its first message, so the file tree can browse the bound folder immediately instead of requiring a persisted thread. Changes distinguishes provider-reported and Git-observed paths, preserves rename and binary metadata, and shows bounded unified or fitting split diffs with file-type icons, stable line gutters, and compact scope controls.

Files use a compact, lazily loaded tree on the left and a code editor on the right. The tree can be hidden, keeps folders independently expandable, and virtualizes large directories instead of mounting every entry. The editor virtualizes source lines, applies safe local syntax tokenization, and uses coding-font and line-number conventions. A repository-owned SVG icon set generated from a pinned `vscode-icons` revision provides recognizable, color-preserving icons for common languages, frameworks, tools, infrastructure, data, media, exact filenames, compound suffixes, and lockfiles. Unknown formats retain a neutral fallback. Each generated icon passes through Vite's asset pipeline and is bundled as a same-origin local image, so desktop WebViews do not depend on cross-document SVG fragment support or a runtime network request. Search, ignored-file visibility, external opening, path copying, and file or selection attachment remain available as icon actions. Binary, oversized, invalid UTF-8, traversal, and symlink-escape paths remain unavailable.

The bottom dock remains closed until the person opens it. When opened for a workspace draft, it starts with a running Terminal. The draft allocates its future thread ID before the first message, and the first send persists that same ID, so its terminal remains attached without being recreated or transferred. One tab row holds terminal sessions and any Changes, Plan, or Files panels the person adds through the adjacent add menu. This avoids stacked tab bars and permanent tool tabs while keeping every panel available without replacing the conversation or right inspector. Terminals are thread-scoped pseudoterminals owned by Rust. Each tab is named automatically from the local user, device, and workspace path. Long names fade at the tab edge, and a terminal can be closed from the tab on hover. Terminal panels do not expose rename, restart, clear, or context utility toolbars. Tabs retain bounded sequenced output. The terminal resolves theme colors and uses a cross-platform coding font stack with regular output weight instead of inheriting the application text font or relying on a CSS variable that xterm cannot resolve. Multiline paste and closing a running session retain explicit confirmation. The webview never receives a generic shell command capability.

Git workspaces capture hidden refs under `refs/ganbaru-ai/chat/` using an isolated index. Capture does not move HEAD, change branch, touch the real index, alter staging, or modify working files. Restore first produces a revision-bound preview with affected files and warnings. Native provider rollback must succeed before filesystem restore when available. Confirmed restore preserves the target branch and staging state, invalidates later conversation projections durably, and queues only exact later refs for cleanup.

## Data ownership and retention

| Data | Location |
|---|---|
| Workspaces, threads, canonical events, projections, drafts, attachment metadata, receipts, checkpoints, cleanup | Active vault SQLite database |
| Managed image and text attachment bytes | Active vault `assets/chat/attachments/` |
| Absolute workspace paths, executable paths, provider homes, probe caches, diagnostics, full-access trust | Device-local application state |
| Provider tokens and passwords | Operating-system credential store behind opaque references |
| Provider processes and terminal sessions | Memory owned by the current app process |

Archive is reversible. Permanent deletion requires confirmation, removes owned conversation rows, and records deferred attachment or checkpoint cleanup before destructive external work. Attachments remain while referenced and use a grace period before file deletion. Checkpoint cleanup always names an exact hidden ref.

Optional protocol diagnostics are off by default, redacted, bounded, device-local, and retained for 1 to 30 days. Users can inspect captured field categories, export the same redacted representation, delete diagnostics immediately, stop all Chat processes, and rebuild projections only when runtime and durable-turn safety checks pass.

## Responsive and accessible behavior

Chat keeps three columns while they fit, removes the inspector column next, then presents the rail or inspector as a focus-contained sheet. At the 280 by 180 recovery floor, one primary surface fills the workspace and every essential action remains reachable. Layout changes preserve the selected thread, draft, undo state, timeline anchor, search, inspector state, terminal state, approvals, and questions.

Separators support pointer and keyboard resizing with accessible values and reset actions. Pointer resizing is coalesced to animation frames, and panel transitions are disabled during the gesture so the tree and editor track the pointer without delayed interpolation. Modal surfaces enter useful focus, contain Tab navigation, and restore their trigger. Statuses use text or shape in addition to color. Live regions announce provider state, settled responses, approvals, questions, errors, and Stop without reading every streamed delta. English and Spanish catalogs use locale-aware formatting. Reduced motion removes smooth scrolling, transitions, and progress animation.

## Performance contract

Chat is lazy-loaded and does not probe or start providers at app boot. The first Chat load may run the bounded provider discovery described above. Each active vault runs that automatic pass at most once per app session, and ordinary Settings refreshes reuse cached provider state. Refresh all is the explicit forced rescan. The deterministic benchmark fixture uses cached provider state. The `dense-chat-v1` benchmark covers 20 projects, 100 threads, 2,000 turns, 4,000 messages, 4,000 activities, 1,000 plans, 500 attachments, 200 checkpoints, and 10,000 canonical events in the isolated benchmark database.

The benchmark measures route activation, recent-thread switching, latest-page SQLite read and projection, loaded and indexed rail search, streamed paint cadence, app process-tree idle CPU, memory, and owned process stop. The target latest-page read is normally below 100 ms, indexed search below 150 ms, normal stop below two seconds, and streaming is coalesced to paint frames. Performance results are recorded only from the installed release benchmark harness using the method in [Performance benchmark harness](performance-benchmark.md).
