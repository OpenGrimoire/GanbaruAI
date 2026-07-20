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
