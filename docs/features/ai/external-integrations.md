# External AI integrations

External integrations are separate from the internal coding-agent runtime. They require their own authorization and must not infer access from a shell, provider login, or Chat membership.

## BYOK assistant

The planned BYOK assistant is a compact general-user surface for questions, summaries, and bounded actions over explicitly selected Ganbaru AI context. Users choose a hosted or local provider and understand what context will be sent before an action runs.

The assistant must not become an ambient reader of the complete vault. Context is assembled from an explicit current surface, selected records, or a named context action. Destructive or externally visible actions require typed commands and appropriate confirmation.

## External MCP service

Ganbaru AI plans a separately authorized MCP service for external clients. It may expose bounded resources and typed tools for Calendar, Projects, Notes, and derivative reports. It is distinct from the ephemeral loopback MCP endpoint used by internal coding-agent runs.

The internal endpoint is application infrastructure. It is not a teammate, participant, permanent server, or general external API.

## CLI

The `ganbaru-ai` CLI is planned and is not currently implemented. Its intended role is to provide explicit external queries and derivative exports for scripts, agents, and collaborators. It must use application-owned authorization and data services rather than treating direct database or filesystem access as permission.

Examples in future CLI documentation must not imply that the executable already ships.

## Context actions

Reusable context actions can select bounded information such as:

- Today's calendar and active focus block.
- One project, its open tasks, and selected related Notes.
- A chosen Notes page and its linked context.
- A project status or review summary.

Context actions respect participant access, selected workspace, provider, budget, and redaction policy. They do not create one unbounded memory containing the complete vault.

## Deferred multi-person behavior

Future human collaboration introduces encryption, revocation, historical visibility, and per-participant authorization questions. External AI access cannot be finalized independently of those rules. See [Chat teammates and coordination](../chat/teammates-and-coordination.md), [sync](../../data/sync.md), and [access control](../../data/access-control.md).
