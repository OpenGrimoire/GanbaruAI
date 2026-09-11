# AI integrations

Ganbaru AI is fully usable without AI. Every AI path is opt-in, has an explicit authority boundary, and remains subordinate to the user's local data and organizational model.

## Integration paths

| Path | Status | Purpose |
| --- | --- | --- |
| Local coding-agent Chat | Implemented | Run user-installed coding harnesses inside an authorized project working folder or private scratch scope. |
| BYOK general assistant | Planned | Provide a small general-purpose assistant backed by a user-selected hosted or local model. |
| External MCP and CLI access | Planned | Expose separately authorized, bounded data and derivative exports to external clients and scripts. |

The local coding-agent path is not a general vault assistant. It is the execution layer beneath [Chat](../chat/README.md). Chat owns channels, participants, messages, assignments, and review. Providers own bounded reasoning or execution sessions. A provider or model is never the canonical identity of a teammate, channel, task, or decision.

## Authority model

Rust resolves the effective project, conversation, participant, working folder, context package, provider, model, interaction mode, budget, and authorization before starting work. Provider-native trust or approval cannot widen Ganbaru AI authority.

An organizational run has exactly one execution target:

- An authorized project working folder.
- A private scratch generation owned by its initiating reply thread.
- No filesystem target for conversation-only work.

Svelte renders validated canonical events and read models. It does not receive a generic shell or arbitrary filesystem capability. Credentials remain behind operating-system credential references, and provider processes receive only the environment needed for their configured runtime.

## Data and privacy

Execution events, provider-thread identities, messages, projections, drafts, attachments, checkpoints, command receipts, and authorization records live in the active vault according to the [data architecture](../../data/architecture.md). External paths, executable paths, provider homes, process state, and native credentials remain device-local.

Every path must explain what leaves the device:

- Local coding harnesses follow the provider the user installed and authenticated.
- A future BYOK assistant sends only the selected context to the configured provider. Local providers can keep model traffic on the device.
- A future external MCP or CLI surface requires its own authorization and never inherits internal Chat authority.

See [Provider runtimes](provider-runtimes.md), [External integrations](external-integrations.md), [Chat execution and workspace](../chat/execution-and-workspace.md), and [Access control](../../data/access-control.md).
