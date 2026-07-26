# AI integration

Ganbaru AI's AI features are entirely opt-in. The app is fully functional with no AI configured. When users opt in, three separate paths cover local coding work, a future general assistant, and external clients. These paths may reuse presentation primitives, but they have different permissions and runtime contracts.

## Three paths

### 1. Local coding-agent Chat

The Chat tab is a project-owned interface over coding-agent harnesses that the user installs and authenticates separately. The first provider families are Codex, Claude, Cursor, and OpenCode. Rust owns each provider transport, process tree, credential references, filesystem access, Git checkpoints, terminal sessions, and durable event ingestion. Svelte renders validated canonical data and never receives a generic shell or arbitrary filesystem capability.

Chat provides:

- **Native harness fidelity** through Codex app-server, the selected Claude transport, Cursor ACP, and OpenCode HTTP plus event streams. Provider sessions, approvals, questions, plans, models, usage, and errors remain provider-native facts.
- **Project working folders** stored under each Ganbaru Project, with one vault-relative managed folder and optional device-bound external folders.
- **Durable conversations** in the active vault SQLite database, with incremental projections, native resume identity, archive and recovery, managed attachments, and explicit incompatible-session forks.
- **Safety controls** for Supervised, Auto-accept edits, and Full access, mapped truthfully to provider capabilities. Full access requires confirmation for each provider instance and working-folder trust boundary.
- **Workspace tools** for bounded file inspection, changed-file diffs, thread-scoped terminals, explicit terminal context, and hidden Git checkpoints that do not alter the current branch or real index.

Chat does not use `codex exec`, terminal scraping, a resident Node server, or a Ganbaru-hosted relay for interactive turns. Provider-reported subagents and tasks appear as normalized activity, but background autonomous scheduling is outside the initial Chat scope. The reviewed dependency choices and addition phases live in [Chat dependency decisions](chat-dependency-decisions.md).

The user-facing working-folder behavior, lifecycle, recovery states, and data ownership rules live in [Chat](chat.md). This document keeps the provider and AI-surface architecture.

Project working folders are the only coding context. Every project has one managed folder at `projects/{project-id}/` and can own several external folder associations. Managed paths resolve from the active vault. External absolute bindings remain scoped to the active vault, current device, and working-folder id. Rust canonicalizes each selected folder, probes credential-stripped repository identity, and repeats both checks before every authorized operation. A different repository cannot silently replace an existing binding.

Chat persistence uses an append-only canonical event log plus transactionally maintained SQLite projections. Every thread has non-null project and working-folder ownership, proven by a composite foreign key. Lightweight folder and thread shells, indexed active and archived title search, and bounded sequence-anchor timeline pages do not load complete conversation history. Drafts persist per working folder with text, versioned mentions and selections, and managed attachment references. Projection rebuild replays canonical events and preserves attachment links, while archive remains reversible and permanent deletion records deferred cleanup work before removing owned conversation rows.

One Rust session worker owns each live thread across the main and detached windows. Provider work enters a bounded channel behind a shared per-thread operation lock. Sessions start only on demand, ready sessions stop after an idle interval, and active turns, pending provider requests, and terminal-linked work prevent idle cleanup. Each start has a generation token, so output from an old stopped or replaced provider cannot mutate the new session. Settled-turn and mismatched-thread output becomes a bounded warning instead of a normal projection event. Application exit stops accepting commands, flushes event batches while interrupt and stop are pending, gives each driver a bounded stop interval, then aborts any remaining worker so owned process-tree drop cleanup runs.

Canonical provider events are size checked, diagnostic fields are checked for prohibited secret and machine-path material, adjacent compatible content deltas are batched, and each append updates its projections in the same SQLite transaction. Only after commit does Rust emit the lightweight thread sequence and revision notification. Each window accepts contiguous notifications, ignores duplicates, replays sequence gaps, refreshes stale revisions, and reconciles detached state against durable SQLite data.

Frontend change notifications are coalesced to the next paint frame and serialized while a refresh is active. This keeps durable ordering while avoiding a SQLite read, shell refresh, interaction refresh, and component update for every provider token. The timeline retains at most eight loaded pages, evicts pages away from the selected sequence, and never deserializes all thread histories for route activation or switching.

Provider secrets use opaque credential references. Secret values are nonserializable Rust values stored through the operating-system credential service. Folder-local `config.json` contains only validated portable provider preferences keyed by working-folder id. Executable paths, provider home paths, probe caches, and external folder paths remain in device-local application state.

Diagnostics are off by default. When explicitly enabled, Rust captures only bounded redacted protocol labels and field visibility for 1 to 30 days in device-local state. The diagnostics dashboard reports projection health, credential presence as a boolean, provider probes, live owned processes, terminals, attachments, cleanup, and checkpoint state. Export applies the same redaction and bounds. Users can delete diagnostics immediately, stop all Chat processes with an exact confirmation phrase, and rebuild projections only when no runtime or unsettled durable turn can race the operation.

The implemented Chat shell lazy-loads inside the existing last main tab and reuses the Calendar internal background token. Projects, Notes, and Chat share the selected project. The rail scopes ordinary threads to that project and groups them by working folder, while global search and archive recovery span all projects. First-use routing begins with project context, uses its managed folder automatically, and blocks only for provider recovery or an unavailable selected external folder.

Chat layout is selected by a pure container-fit model that accounts for available width, height, font scale, configured panel widths, and hysteresis. It restores configured panel geometry before presenting the shell, keeps three columns while they fit, moves the inspector or rail into focus-contained sheets as space contracts, and exposes one recoverable primary surface at the 280 by 180 floor. The same mounted controllers preserve thread, draft, timeline, inspector, request, and terminal state across those presentations. Outer panel separators expose bounded pointer and keyboard resizing, context-sensitive fitting through double-click or Enter, and an eased magnetic drag-to-close edge without storing an unusable zero size. Transient surfaces move focus to useful content, contain Tab navigation when modal, and restore the trigger on close.

Meaningful provider, settled-response, Stop, approval, question, and error states use bounded live announcements without replaying streamed tokens. Statuses pair semantic text or shape with color, diff markers retain signs and source labels, and Chat uses existing surface, signal, and action tokens plus user-selected provider accent data. English and Spanish catalogs cover provider-neutral UI, while dates, durations, counts, costs, tokens, file sizes, and relative activity times use the active locale helpers. Reduced-motion preference removes smooth timeline scrolling, panel transitions, and progress animation.

Chat settings live in Ganbaru's existing Settings modal. A vault without a Codex instance checks the installed `codex` command on its first Chat load, including conventional user CLI directories that desktop launchers may omit, and creates a default instance that uses the CLI's existing home and authentication. The advanced setup uses Provider, Identity, and Connection steps for additional accounts, custom homes, other provider families, native executable and folder pickers, field-level validation, operating-system credential references, and an explicit probe before use. Provider cards expose health, version, hidden account identity, last successful probe, refresh, edit, enable, disable, and removal. Model controls remain scoped to one provider instance and preserve hidden, favorite, stale, unavailable, deprecated, and supported custom-model states. Working-folder bindings and provider preferences are edited from Project settings.

The implemented conversation timeline projects canonical events and bounded SQLite pages into provider-neutral messages, work activity, plans, pending state, and settled-turn summaries. Stable row IDs support measured virtualization, page prepend anchoring, loaded-page eviction, fold and expansion state, follow or anchored reading intent, unread counts, and an optional long-thread minimap. User rows expose durable attachments, workspace mentions, bounded terminal context, timestamps, copy, and checkpoint-aware restore entry points. Assistant rows use bounded Marked parsing followed by DOMPurify sanitization, preserve incomplete streaming Markdown, enhance code blocks with copy and wrap controls, and show final duration, effective model, usage, and changed-file metadata.

The implemented composer uses one persistent controller for its new-draft hero and docked layouts. It saves exact prompt text, explicit provider and model selections, typed model traits, safety and interaction modes, workspace mentions, managed image references, and a recoverable sent snapshot. Sending commits the user message, turn, attachment references, and idempotent command receipt before provider dispatch. Provider launch failures therefore keep a durable user row and support Retry, Edit into a new draft, or Change provider through a new native thread.

Composer actions follow live provider capabilities. An active turn can accept native steering, a durable queued follow-up, or retain the unsent draft when neither is available. Stop and Force stop are separate, idempotent operations. Approvals expose only provider-offered decisions, while structured questions persist partial answers locally and validate required, single-select, multi-select, and free-form responses before submission. Context usage, automatic compaction, account state, rate limits, and cost appear only when reported by the provider. Full access trust remains device-local and is scoped to the exact provider instance and working folder.

The implemented inspector keeps Changes, Plan, Files, and Terminal state per thread for the current window session. It remains a resizable third column while space permits, becomes a right sheet at narrower widths, and can maximize without replacing the selected tab or file. The Files tree and Changes file list expose bounded pointer and keyboard resizing, with independent sizes for the inspector and bottom panel. Changes can compare the current turn or the full thread, distinguish provider-reported paths from Git-observed paths, preserve rename and binary metadata, and render bounded unified or fitting split patches. Git disagreement is visible instead of being silently merged.

Files are listed on demand through the authorized workspace boundary. Search can include ignored files only when the user asks, and common generated directories remain excluded. Text previews are UTF-8 validated, size bounded, and unavailable for binary files, oversized files, traversal attempts, or symlink escapes. Copy path, Open externally, and Attach selection operate on validated workspace-relative paths. They do not grant the webview a reusable absolute path or generic filesystem API.

Each thread can own multiple named terminal tabs in its selected workspace. The inspector and bottom panel manage independent terminal collections and selections, so a terminal is never mirrored or resized by both panels. Rust creates each pseudoterminal with the platform default shell, owns input and resize commands, and retains only bounded sequenced replay. The frontend dynamically loads xterm.js when the Terminal surface first opens. Closing a panel keeps its renderer mounted at a stable content size while an outer clipping shell animates, which preserves the visible screen, input line, selection, and scroll position without shrinking the terminal grid. A stopped terminal remains available for scrollback with an immediate path to create a replacement. Multiline paste, closing a running terminal, and attaching terminal context are explicit actions. Context records the source terminal, capture kind, exact bounded text, line count, byte count, and timestamp. Thread deletion, workspace removal, and app shutdown terminate the matching owned terminals.

Git workspaces receive hidden checkpoint refs under `refs/ganbaru-ai/chat/`. Capture uses an isolated index and records the current HEAD context, real index tree, worktree tree, and index fingerprint without moving HEAD, changing branches, touching the real index, or altering staged and unstaged files. Initial, pre-turn, and post-turn checkpoints support turn and full-thread diffs. Ref reads recheck the logical repository identity and exact object ID.

Restore is a two-step preview and confirmation flow. Preview binds the current thread revision, repository identity, HEAD context, worktree tree, real index tree, index fingerprint, affected files, target ref, and provider rollback capability. Execution rejects a stale preview before modification. A provider with native rollback must succeed before the workspace changes. A provider without native rollback is detached and the next turn starts a fresh compatible provider history. After Git restores the worktree and real staging state, SQLite appends a durable reverted event, invalidates later turns and their canonical events, cancels pending work, clears incompatible continuation data, and queues only exact later checkpoint refs for cleanup. Invalidated audit rows remain durable and projection rebuild does not revive them. A partial failure retains exact refs and records a recovery-required operation.

### Codex runtime boundary

Codex runs through its native `app-server` JSONL protocol over a Rust-owned child process. Ganbaru performs the initialize handshake, correlates bounded requests, handles provider notifications and server requests, and stores the native thread identity required for resume. A confirmed missing native thread starts a fresh Codex thread with an explicit warning while preserving Ganbaru history. Transport, authentication, protocol, permission, and missing-thread failures remain distinct.

Codex safety settings are exact. Supervised uses `untrusted` with `read-only`, Auto-accept edits uses `on-request` with `workspace-write`, and Full access uses `never` with `danger-full-access`. Ganbaru verifies the effective policy reported by Codex before declaring the session ready. Command and file approvals return only provider-offered native decisions. Runtime permission requests return only the requested permission profile, with turn or session scope. Secret structured answers are sent directly to the live provider request but omitted from canonical history.

Direct and shadow Codex homes derive continuation compatibility from the canonical shared session home. A shadow home may keep `auth.json` and `models_cache.json` private while verified links share session and state directories. Ganbaru never replaces an existing conflicting entry. Changing the shared home requires a thread fork. Launch arguments are restricted to Codex configuration and feature flags, and Windows command shims are accepted only when they resolve to the official `@openai/codex` entry point without invoking a shell.

### Claude runtime boundary

Claude runs through Claude Code's native bidirectional stream JSON protocol over a Rust-owned child process. Phase 10 selected native Rust standard IO after a redacted compatibility matrix covered partial messages, session and assistant UUIDs, native resume, permission callbacks, `AskUserQuestion`, interrupt, model changes, and native Plan mode for Claude Code 2.1.170. The matrix is derived from the public protocol types shipped with the official Claude Agent SDK 0.3.170. The locally observed Claude Code 1.0.92 lacks partial-message output and is rejected as unsupported instead of receiving a reduced integration. Ganbaru does not install an Agent SDK sidecar or a resident Node or Bun service.

Rust launches one shell-free Claude process per live session with bounded JSON lines, correlated control requests, cancellation deadlines, bounded diagnostics, and the shared process-tree owner. `CLAUDE_CONFIG_DIR` forms the account and continuation boundary. A fresh session receives a Ganbaru-generated UUID, while continuation persists both the session UUID and the last assistant UUID. A confirmed missing native session remains a distinct recoverable error and requires an explicit thread fork. Windows command shims are accepted only when they resolve to the official Claude package executable or JavaScript entry point without invoking a shell.

Claude safety modes remain native. Supervised uses permission callbacks, Auto-accept edits uses `acceptEdits`, Full access uses `bypassPermissions` plus Claude's dangerous-skip confirmation flag after Ganbaru's workspace trust check, and Plan uses Claude's `plan` permission mode. Ganbaru responds only to the exact pending permission request. `AskUserQuestion` uses a separate structured-input path. `ExitPlanMode` records a proposed-plan card and is denied for that turn so implementation waits for a later user request. The next turn reapplies its requested native mode, so Plan does not silently persist or become an emulated system prompt.

### Cursor runtime boundary

Cursor runs through ACP version 1 over a Rust-owned `cursor-agent acp` process. Ganbaru uses newline-delimited JSON-RPC 2.0 with bounded requests, response correlation, notifications, cancellation deadlines, malformed-frame termination, bounded stderr diagnostics, and the shared process-tree owner. The optional API endpoint is a tokenized `-e` argument and must use HTTPS or loopback HTTP without embedded credentials. Provider arguments cannot replace the endpoint, credentials, or ACP subcommand.

The compatibility boundary follows T3 Code commit `5d34f9ff235115d43a6cb4b4561d10badf218b87` and Cursor's public ACP extension schemas. Cursor Agent `2026.04.08` is the minimum tested version for parameterized model selection. No Cursor executable was available in the local validation environment, so compatibility is proven through redacted ACP fixtures rather than claimed as a live local probe. Ganbaru added no ACP package or sidecar.

Startup initializes ACP capabilities, authenticates through the advertised `cursor_login` method, creates or loads the native ACP session, applies validated model traits and mode before a prompt, then persists the ACP session ID as the resume cursor. Continuation compatibility includes the provider home, endpoint, and probed account identity. Only a provider error that specifically identifies a missing session becomes a recoverable resume-not-found result.

Cursor model metadata exposes reasoning, context window, fast mode, and thinking only when corresponding configuration options are advertised. Model and trait values must exactly match provider-offered options. Ganbaru validates the complete requested configuration before sending any update, applies updates in stable order, and attempts to restore earlier values if a later provider update is rejected. Build and Plan use advertised native modes. The Plan control is disabled when Cursor does not advertise a Plan or Architect mode.

ACP permission decisions use only exact option IDs offered by Cursor. Supervised always asks. Auto-accept edits selects an offered `allow_once` option only for an edit request whose existing path chain resolves inside the canonical workspace, including protection against symlink escapes. It never selects a session-wide option. Commands and external paths still ask. Full access may select only an offered `allow_always` or `allow_once` option. `cursor/ask_question` remains a separate structured-input request, while `cursor/create_plan` and `cursor/update_todos` become canonical plan events.

Assistant text, displayable thoughts, tool lifecycle, bounded tool output, in-workspace diffs, plans, mode updates, permissions, questions, and Cursor extension data normalize into provider-neutral events with native identifiers. Unsupported session updates, tool content blocks, requests, and notifications become bounded unknown events or explicit method-not-supported responses. Negotiated unsupported capabilities remain disabled instead of being emulated.

### OpenCode runtime boundary

OpenCode runs through its HTTP API and Server-Sent Events protocol. In local mode, Ganbaru launches `opencode serve` on an ephemeral loopback port, verifies the exact readiness origin, adds an optional credential-store password only to the child environment and Rust-owned authorization header, and terminates the complete owned process tree when the session or application stops. Startup has bounded output, early-exit handling, and a fixed deadline. Ganbaru never installs OpenCode or runs its installation script.

External mode accepts only an HTTP or HTTPS origin without embedded credentials, paths, queries, or fragments. HTTPS is required for non-loopback servers unless the user explicitly acknowledges the unencrypted connection. A separate confirmation is required before Ganbaru sends a local workspace path to any external server. External servers are never treated as owned processes and are not stopped by Ganbaru.

The Rust client creates or resumes native sessions, reasserts the selected permission rules, forks the OpenCode session when its recorded directory differs from the canonical workspace, and starts fresh only after a confirmed not-found response. Prompts preserve model, agent, variant, scoped instructions, text, and validated local attachments. Native operations cover steering, abort, paged history, permission replies, structured questions, and message or part revert cursors.

The event stream is cancelable and incrementally decoded with strict size limits. After a disconnect, Ganbaru reconciles bounded native message history before reconnecting and deduplicates replayed message and part updates. Session, assistant, reasoning, tools, commands, files, permissions, questions, todos, diffs, usage, cost, model, agent, MCP, warning, error, and unknown provider events normalize into the canonical event model with native identifiers.

OpenCode 1.14.19 is the minimum supported version. The compatibility boundary follows OpenCode v1.14.19 and the pinned T3 Code reference commit `5d34f9ff235115d43a6cb4b4561d10badf218b87`. No compatible OpenCode executable was available in the local validation environment, so local process behavior and the declared capability suite are verified through deterministic executable, HTTP, event-stream, and lifecycle fixtures rather than claimed as a live provider run. The implementation adds no package dependency.

### 2. BYOK general assistant (future general-user path)

A separate assistant interface can connect to the user's chosen model API. Three provider categories cover most users:

- **OpenAI API**.
- **OpenAI-compatible API** (Groq, Together, Mistral, and any provider using a compatible chat format).
- **Ollama** for local models (Llama, Mistral, Gemma) running on the user's machine, no API key needed.
- Other provider APIs when users supply their own credentials and the integration is implemented explicitly.

The general assistant can read and write authorized Ganbaru AI data through the planned CLI bridge. It cannot edit arbitrary workspace files or execute commands. Those capabilities belong only to the coding-agent Chat and its explicit workspace and safety boundaries.

### 3. MCP (external clients only)

Ganbaru AI exposes calendar, project, and notes data via an MCP server for use by external AI clients (ChatGPT, teammate agents, and other MCP-compatible clients on a different machine). MCP is also consumed for integrations with external systems (email, external calendars).

MCP is **not** the path for internal provider interaction. Coding-agent Chat speaks each harness's native local protocol, while the future general assistant uses its own provider API contract.

## The CLI as the data bridge

The `ganbaru-ai` CLI is a Rust binary that reads the same SQLite database the app uses. Agents call it via Bash:

```
ganbaru-ai task list --project foo --status in_progress
ganbaru-ai event create --start 2026-04-20T14:00 --duration 90m --project foo
ganbaru-ai export projects --project foo
```

This is the bridge between AI agents and Ganbaru AI's data. It works with any agent that can run a shell command (Codex, Cursor, custom scripts), with no plugin or MCP server required for the local case.

The current Notes implementation also exposes a deterministic agent bridge markdown export from the app UI. It is a local derivative view over selected Notes pages, optional subpages, database views, backlinks, and project task context read from SQLite. It is useful for agents that need a portable context file before the full CLI bridge command set exists, but it follows the same source-of-truth rule: regenerate it from Ganbaru AI instead of editing it as canonical project state.

## Workflow phase prompts

Each project lifecycle phase (see `features/project-management.md`) has a structured system prompt:

- **Brainstorming:** guides structured ideation.
- **Evaluation:** helps assess ideas against Want/Can/Need criteria.
- **Planning:** assists with specifications and resource estimation.
- **Execution:** helps with implementation and blockers.

These prompts can become explicit context actions for both coding-agent Chat and the future general assistant. They never bypass the selected workspace, provider, model, interaction mode, or safety confirmation.

## Prompt buttons

The UI can show contextual actions such as "Plan this sprint," "Research competitors," and "Create calendar events for these tasks." Each action inserts a reviewable draft into the compatible AI surface. It does not dispatch automatically.

## Privacy and data flow

- The coding-agent Chat path: data flows through the selected installed harness under the user's provider account and configuration. Ganbaru stores normalized history locally, but the provider may send prompts, files, and tool output to its own service.
- The future BYOK path: data flows to the provider the user configured. Local providers such as Ollama can keep model traffic on-device.
- The MCP path: external clients receive only the data the user authorizes them to see.

No AI features are required to use Ganbaru AI. All data is processed locally by default. AI is an enhancement, not an infrastructure dependency.

## Team features and the AI as data fiduciary (deferred)

When team features arrive, the AI takes a specific privacy role: it sees individual tracking data (focus hours, idle patterns, break habits) but exposes only privacy-safe aggregate signals to managers and team leads. The AI is conceptually a fiduciary, similar to how a doctor sees a patient's full medical records but only tells an employer "fit for work."

What the team-facing AI outputs:

- "Assign task X to person A, estimated 3 days."
- "This reassignment would delay project B by approximately 2 days."
- "The suggested timeline accounts for existing workload."

What it never outputs:

- Individual focus hours, break patterns, idle time, or pause frequency.
- Comparative statements between people.
- Rankings or leaderboards.
- Explanations that reveal individual habits.

Mitigations against extraction attempts: hard architectural boundaries (the team-facing AI receives pre-computed signals only, not raw segments), output filtering for individual identifiers, refusal of comparison and per-person queries, response granularity capped at days and weeks, and a transparency log so each user can see every AI response that involved their data.

This design exists because accurate task estimates need individual-level data, but exposing that data to managers is surveillance, which contradicts the app's philosophy. The AI mediates: the data stays personal, the team gets useful estimates.
