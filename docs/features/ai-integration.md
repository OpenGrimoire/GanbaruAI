# AI integration

Ganbaru AI's AI features are entirely opt-in. The app is fully functional with no AI configured. When users opt in, three separate paths cover local coding work, a future general assistant, and external clients. These paths may reuse presentation primitives, but they have different permissions and runtime contracts.

## Three paths

### 1. Local coding-agent Chat

The Chat tab is a Ganbaru-owned local workspace over coding-agent harnesses that the user installs and authenticates separately. The first provider families are Codex, Claude, Cursor, and OpenCode. Rust owns each provider transport, process tree, credential references, filesystem access, Git checkpoints, terminal sessions, and durable event ingestion. Svelte renders validated canonical data and never receives a generic shell or arbitrary filesystem capability.

Chat provides:

- **Native harness fidelity** through Codex app-server, the selected Claude transport, Cursor ACP, and OpenCode HTTP plus event streams. Provider sessions, approvals, questions, plans, models, usage, and errors remain provider-native facts.
- **Project workspaces** that link existing Ganbaru Projects or explicit standalone contexts to device-local folders. Project records do not gain machine paths.
- **Durable conversations** in the active vault SQLite database, with incremental projections, native resume identity, archive and recovery, managed attachments, and explicit incompatible-session forks.
- **Safety controls** for Supervised, Auto-accept edits, and Full access, mapped truthfully to provider capabilities. Full access requires confirmation for each provider-instance and workspace trust boundary.
- **Workspace tools** for bounded file inspection, changed-file diffs, thread-scoped terminals, explicit terminal context, and hidden Git checkpoints that do not alter the current branch or real index.

Chat does not use `codex exec`, terminal scraping, a resident Node server, or a Ganbaru-hosted relay for interactive turns. Provider-reported subagents and tasks appear as normalized activity, but background autonomous scheduling is outside the initial Chat scope. The reviewed dependency choices and addition phases live in [Chat dependency decisions](chat-dependency-decisions.md).

Logical Chat workspaces are separate from Ganbaru Project records. A workspace can reference one existing project or declare an explicit standalone context, while its absolute folder binding remains scoped to the active vault and current device. Rust canonicalizes each selected folder, probes credential-stripped repository identity, and repeats both checks before any workspace-authorized operation. A different repository cannot silently replace an existing logical workspace.

Chat persistence uses an append-only canonical event log plus transactionally maintained SQLite projections. Lightweight workspace and thread shells, indexed active and archived title search, and bounded sequence-anchor timeline pages do not load complete conversation history. Drafts persist text, versioned mentions and selections, and managed attachment references. Projection rebuild replays canonical events and preserves attachment links, while archive remains reversible and permanent deletion records deferred cleanup work before removing owned conversation rows.

One Rust session worker owns each live thread across the main and detached windows. Provider work enters a bounded channel behind a shared per-thread operation lock. Sessions start only on demand, ready sessions stop after an idle interval, and active turns, pending provider requests, and terminal-linked work prevent idle cleanup. Each start has a generation token, so output from an old stopped or replaced provider cannot mutate the new session. Settled-turn and mismatched-thread output becomes a bounded warning instead of a normal projection event. Application exit stops accepting commands, flushes event batches while interrupt and stop are pending, gives each driver a bounded stop interval, then aborts any remaining worker so owned process-tree drop cleanup runs.

Canonical provider events are size checked, diagnostic fields are checked for prohibited secret and machine-path material, adjacent compatible content deltas are batched, and each append updates its projections in the same SQLite transaction. Only after commit does Rust emit the lightweight thread sequence and revision notification. Each window accepts contiguous notifications, ignores duplicates, replays sequence gaps, refreshes stale revisions, and reconciles detached state against durable SQLite data.

Provider secrets use opaque credential references. Secret values are nonserializable Rust values stored through the operating-system credential service. Folder-local `config.json` contains only validated portable provider preferences, while executable paths, provider home paths, probe caches, and workspace paths remain in device-local application state.

### Codex runtime boundary

Codex runs through its native `app-server` JSONL protocol over a Rust-owned child process. Ganbaru performs the initialize handshake, correlates bounded requests, handles provider notifications and server requests, and stores the native thread identity required for resume. A confirmed missing native thread starts a fresh Codex thread with an explicit warning while preserving Ganbaru history. Transport, authentication, protocol, permission, and missing-thread failures remain distinct.

Codex safety settings are exact. Supervised uses `untrusted` with `read-only`, Auto-accept edits uses `on-request` with `workspace-write`, and Full access uses `never` with `danger-full-access`. Ganbaru verifies the effective policy reported by Codex before declaring the session ready. Command and file approvals return only provider-offered native decisions. Runtime permission requests return only the requested permission profile, with turn or session scope. Secret structured answers are sent directly to the live provider request but omitted from canonical history.

Direct and shadow Codex homes derive continuation compatibility from the canonical shared session home. A shadow home may keep `auth.json` and `models_cache.json` private while verified links share session and state directories. Ganbaru never replaces an existing conflicting entry. Changing the shared home requires a thread fork. Launch arguments are restricted to Codex configuration and feature flags, and Windows command shims are accepted only when they resolve to the official `@openai/codex` entry point without invoking a shell.

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
