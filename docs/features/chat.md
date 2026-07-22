# Chat

Chat is Ganbaru AI's local workspace for coding agents installed and authenticated by the user. It gives Codex, Claude Code, Cursor Agent, and OpenCode one consistent interface while preserving each provider's native sessions, models, approvals, questions, plans, and safety behavior. Ganbaru does not proxy prompts through a hosted service.

The provider transport and broader AI architecture live in [AI integration](ai-integration.md). Security invariants live in [security](../data/security.md). This document defines the user-facing Chat product.

## Scope

Chat supports:

- An automatically discovered default Codex instance plus explicitly configured provider instances.
- Existing Ganbaru Projects and standalone coding workspaces.
- Durable local threads, drafts, attachments, search, archive, and restart recovery.
- Streamed assistant messages, reasoning summaries, commands, file changes, tools, tasks, warnings, usage, and cost when the provider reports them.
- Build and Plan interaction modes when supported.
- Native approvals, structured questions, steering, queued follow-ups, interruption, and force stop.
- Bounded file browsing, changed-file inspection, thread terminals, terminal context, and Git checkpoints.
- Wide, compact, detached, increased-font, reduced-motion, keyboard, screen-reader, and minimum-recovery layouts.

Remote clients, Slack, a general BYOK assistant, hosted execution, embedded web preview, branch publication, and pull-request automation are separate future surfaces.

## First use

On the first Chat load for a vault without a Codex instance, Ganbaru checks the installed `codex` command on the application path and conventional user CLI directories, including the standard pnpm location. When it is available, Ganbaru creates the default Codex instance, reuses the CLI's normal home and existing authentication, probes account status, and caches the provider model catalog. A person who already installed and authenticated Codex can therefore bind a workspace and start chatting without entering an executable, provider home, credential, or instance ID.

If `codex` is unavailable, Chat directs the user to provider setup and official installation guidance. The advanced provider setup remains available for additional Codex accounts, custom homes, other provider families, arguments, environment variables, and credential references. Removing the last Codex instance records an explicit opt-out so automatic setup does not recreate it. Adding Codex manually enables the family again.

After provider discovery:

1. Link an existing Ganbaru Project or create a standalone Chat workspace.
2. Bind that logical workspace to a local folder on this device.
3. Review the visible provider and model selection, safety mode, and Build or Plan before the first send.

Failed probes preserve entered fields. Missing folders, changed repository identity, unavailable providers, unsupported capabilities, and another device without a binding have distinct recovery actions. Automatic discovery never supplies credentials, changes the CLI home, installs software, chooses Full access, or hides the active provider and model from the composer.

## Workspace shell

The internal thread rail groups workspaces by current Ganbaru project groups and keeps standalone contexts separate. A project can have multiple logical Chat workspaces without adding machine paths to its Project record. The rail provides new draft, local title filtering, indexed search, archive browsing, rename, read state, detach, open folder, restore, and confirmed permanent deletion.

The conversation header shows the thread title, project and workspace breadcrumb, provider identity, branch when available, status, inspector control, and contextual actions. Keyboard shortcuts cover new chat, search, composer focus, next and previous thread, rail, inspector, Stop, and the command menu.

Thread and workspace shells load without message history. Selecting a thread reads only its latest bounded timeline page. Older pages load around sequence cursors, and the client retains no more than eight loaded pages while preserving selected content and the scroll anchor.

## Timeline

The timeline renders provider-neutral projections from durable canonical events. User messages retain exact text, managed attachments, immutable file mentions, bounded terminal context, time, copy, and checkpoint-aware restore entry points. Assistant messages use sanitized Markdown, safe external links, bounded images and tables, incremental content, code copy and wrap controls, duration, effective model, token usage, cost, and changed-file summaries when known.

Commands, output, reasoning, file changes, tools, web activity, images, tasks, hooks, warnings, and failures use stable activity rows. Consecutive settled work can fold without hiding the final assistant response. Plan cards support continue planning, implementation in a later turn, copy, and dismissal. Interrupted and failed turns remain readable and accurately labeled.

Virtualization uses measured rows and stable IDs. Prepending older pages preserves the visible anchor. Scrolling away during streaming disables follow mode, shows unread work and Jump to latest, and never forces the reader to the end. Long threads may show a minimap in a safe side gutter.

Provider change notifications are coalesced to one serialized refresh per paint frame. Rust also batches adjacent compatible content deltas. This preserves durable sequence ordering without token-level SQLite reads or layout storms.

## Composer and provider interaction

The new-draft hero and docked composer share one persistent controller, so responsive layout changes do not replace the draft or undo state. Draft persistence includes text, mentions, attachment references, and explicit provider, model, trait, safety, and interaction selections.

Images can be chosen, pasted, or dropped. File and folder mentions are searched inside the authorized workspace and revalidated at send time. Provider skills and commands appear only when discovered, with stale entries rejected or retained as plain text. The composer exposes provider, model, typed traits, safety, Build or Plan, context, attachments, and Send or Stop directly.

Sending writes the user message, turn, attachment references, and idempotent command receipt before dispatch. A launch failure therefore leaves recoverable durable history with Retry, Edit into a new draft, and Change provider actions.

During active work, the provider capability decides whether a prompt steers the turn, becomes a durable queued follow-up, or remains an unsent draft. Stop is idempotent and has a bounded stopping state. Force stop is explicit. Approvals expose only choices offered by the provider and never choose a permissive default. Structured questions preserve partial local answers and validate required fields before submission.

## Safety modes

Supervised asks through the provider's native permission mechanism. Auto-accept edits accepts only the provider's verified edit scope and continues to ask for commands or wider access. Full access uses the provider's native unrestricted mode only after confirmation for the exact provider instance and logical workspace.

Full-access trust is device-local and cannot arrive through stale UI state. The active turn keeps its mode snapshot even if a later draft changes modes. Provider capability differences remain visible and disabled rather than simulated.

## Inspector, terminal, and checkpoints

The inspector provides Changes, Plan, Files, and Terminal tabs. Changes distinguishes provider-reported and Git-observed paths, preserves rename and binary metadata, and shows bounded unified or fitting split diffs. Files are listed and previewed on demand within the current canonical workspace boundary. Binary, oversized, invalid UTF-8, traversal, and symlink-escape paths remain unavailable.

Terminals are thread-scoped pseudoterminals owned by Rust. Tabs retain bounded sequenced output. Multiline paste, closing running sessions, and attaching terminal context require explicit actions. The webview never receives a generic shell command capability.

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

Separators support pointer and keyboard resizing with accessible values and reset actions. Modal surfaces enter useful focus, contain Tab navigation, and restore their trigger. Statuses use text or shape in addition to color. Live regions announce provider state, settled responses, approvals, questions, errors, and Stop without reading every streamed delta. English and Spanish catalogs use locale-aware formatting. Reduced motion removes smooth scrolling, transitions, and progress animation.

## Performance contract

Chat is lazy-loaded and does not probe or start providers at app boot. The first Chat load may run the bounded default Codex discovery described above. Configured vaults and the deterministic benchmark fixture use cached provider state and do not repeat that discovery during ordinary route activation. The `dense-chat-v1` benchmark covers 20 projects, 100 threads, 2,000 turns, 4,000 messages, 4,000 activities, 1,000 plans, 500 attachments, 200 checkpoints, and 10,000 canonical events in the isolated benchmark database.

The benchmark measures route activation, recent-thread switching, latest-page SQLite read and projection, loaded and indexed rail search, streamed paint cadence, app process-tree idle CPU, memory, and owned process stop. The target latest-page read is normally below 100 ms, indexed search below 150 ms, normal stop below two seconds, and streaming is coalesced to paint frames. Performance results are recorded only from the installed release benchmark harness using the method in [Performance benchmark harness](performance-benchmark.md).
