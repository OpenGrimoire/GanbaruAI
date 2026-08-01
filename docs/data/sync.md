# Sync and collaboration

Sync turns Ganbaru AI from a local app into a multi-device and optionally collaborative workspace. The user provisions and hosts the sync server; Ganbaru AI does not run shared infrastructure. End-to-end encryption keeps cleartext away from the server, and typed conflict handling preserves the local data invariants.

Human collaboration is a later capability. Its permission model already constrains the design of Chat channels, Projects assignments, Notes folders, reports, search, and AI context packages. Collaboration cannot be added safely as a simple shared-workspace boolean.

## Principles

- Local canonical storage remains usable offline and does not depend on the sync server.
- The server routes and persists encrypted operations, not trusted application data.
- Sync replicates canonical operations and records, not derivative Markdown exports or a raw SQLite database file.
- A participant receives only the resources and keys allowed by their effective membership.
- Direct reads and derived output use the same permission boundary.
- Removing access stops future reads and context assembly without rewriting legitimate history owned by remaining participants.
- Private productivity measurements remain private even when their coarse capacity effect helps team planning.

## Architecture

**Yjs-compatible CRDT operations.** Collaborative state uses CRDT documents or typed operations appropriate to each data family. Notes uses its existing page and block graph rather than introducing an editor-owned second source of truth. Calendar, Projects, Chat, and other relational data require typed operations that preserve foreign keys, lifecycle rules, protected history, and application invariants when concurrent changes converge.

**Hocuspocus server.** A self-hostable synchronization server persists encrypted updates, routes presence, and applies authentication and authorization at the encrypted resource envelope. The user can run it on a VPS, home server, or another host they control.

**End-to-end encryption.** Clients encrypt resource updates before sending them. The server stores ciphertext only. Key distribution follows explicit membership and resource grants rather than one permanent key that gives every collaborator the complete Ganbaru AI folder.

**Live presence.** Authorized participants can appear as cursors, typing indicators, or activity state in relevant views. Presence is ephemeral and scoped to the current resource. It does not create employee monitoring or a permanent online-time record.

## Identity and membership scopes

Future roles can include owner, administrator, member, and restricted guest. Role names do not replace resource grants. A person may join:

- A project group and its permitted projects.
- One project without access to the complete group.
- Selected Chat channels in a project.
- A direct message or task discussion.
- Selected Notes folder subtrees or pages.
- Selected tasks, reviews, or project views.
- Explicit project working folders when filesystem collaboration is intended.

Project groups, projects, channels, Notes folders, Notes pages, task discussions, and project working folders are different resource types. A grant to one does not silently imply the others. Inheritance reduces configuration work, but the effective access result is inspectable before invitation, movement, export, or AI use.

Invitations state whether prior history becomes visible. A participant joining a channel does not automatically receive messages from before the selected visibility boundary. Moving a Note, task discussion, or channel across an access boundary previews who gains and loses access.

## What syncs

- **File-backed documents:** canonical diary entries, project documents, and reports through a document-appropriate synchronization model.
- **Structured data and document graphs:** Notes folders, pages, blocks, comments, Calendar events, Projects tasks, Chat conversations and messages, work environments, and project state through typed operations that preserve graph invariants.
- **Chat execution summaries:** durable run state, approvals, usage, deliverable links, and provider-neutral events needed for authorized history. Device-bound executable paths, provider homes, live processes, terminals, and local trust remain device-local.
- **Pomodoro tracking data:** per-user data available to that user's devices. Other workspace participants never receive raw focus, idle, break, blocker, or diary measurements.
- **Membership and permission changes:** signed, ordered access operations, history-visibility decisions, key-envelope changes, revocation state, and audit metadata.

The two-category storage model in `data/architecture.md` remains intact. File-backed documents stay files on each authorized client. Structured data and Notes graphs stay in local SQLite. Sync does not make exported Markdown or the server database authoritative.

## Permission-safe derivation

Authorization happens before direct reads and before aggregation. The same effective scope applies to:

- Search results and counts.
- Mentions, backlinks, reminders, and notification previews.
- Channel summaries and attention views.
- Project dashboards, saved views, and reports.
- Notes imports, exports, and agent bridge output.
- Calendar availability and scheduling suggestions.
- Manager proposals and AI context packages.
- Agent-run tools, artifacts, and explanations.

An inaccessible resource does not leak through its title, count, participant list, relationship, or a detailed denial reason. AI uses the intersection of the requesting participant, destination conversation, selected role, context package, and run grants.

## Conflict resolution

CRDT semantics handle compatible concurrent edits, but Ganbaru adds domain rules where generic merging is insufficient:

- **Protected history:** past Pomodoro and protected Calendar records cannot disappear through conflict resolution.
- **Active sessions:** one person cannot have two authoritative active Pomodoro sessions. Heartbeats and explicit takeover resolve device conflict.
- **Recurring events:** conflicting scope edits preserve protected occurrences and surface the losing intent for review.
- **Projects requirements:** concurrent scope, assignment, review, budget, or deadline changes produce explicit revisions and downstream-impact recalculation instead of silently combining incompatible commitments.
- **Manager proposals:** acceptance applies to an exact proposal and source revision. A stale proposal is replanned or reviewed against current state.
- **Chat ordering:** messages use stable identities and causal ordering. Edits and replies remain attached to the intended message after offline merge.
- **Permission changes:** access reduction wins over stale content updates for future delivery. A client cannot publish a new operation under a revoked grant after reconnecting.

## Revocation and offline devices

Revocation is not equivalent to deleting shared history. It prevents new authorized reads, updates, exports, notifications, and AI context assembly from the removed participant or device. Remaining authorized participants retain legitimate shared history.

The implementation must define key rotation, cached ciphertext retention, local cleartext cleanup, offline operation rejection, device removal, recovery keys, and re-invitation before collaboration ships. The UI must explain that a person who previously received cleartext could have copied it outside Ganbaru AI; cryptography can stop future access but cannot erase an external copy.

## Privacy-safe capacity

Team scheduling can use coarse signals such as unavailable, available after a date, or a suggested duration range. It never exposes individual focus hours, idle time, break behavior, blocker attempts, diary mood, or comparisons between participants. Team-facing AI receives only the coarse signal needed for the requested decision and cannot query the raw source rows.

## Self-hosting

Ganbaru AI provides a server image, documented Compose configuration, guided setup for common hosts, health checks, backup guidance, and key-recovery warnings. It does not offer a hosted sync service. Donation funding cannot safely support an implicit promise of centralized uptime, storage, or account recovery.

## Backups

Sync is not a backup. Backups go to a user-specified path outside the Ganbaru AI folder on a schedule the user controls. Sync keeps authorized devices converged; backups recover data after loss, corruption, accidental deletion, or a bad synchronized operation.

## Deferred implementation details

Exact CRDT schemas, key hierarchy, invitation protocol, history-visibility storage, encrypted search strategy, offline revocation, conflict UI, server deployment, and recovery flows remain to be designed and threat-modeled before phase 9 implementation. The access scopes and non-leakage rules in this document are product constraints, not claims that collaboration is currently available.
