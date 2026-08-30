# Synchronization

Remote synchronization and remote collaboration are not implemented. This document defines the intended local-first contract and the constraints that any implementation must satisfy. Yjs, Hocuspocus, and encrypted operation envelopes are the current proposed architecture, not deployed services.

## Principles

- Local writes remain available without a server or network connection.
- The active local vault remains usable and authoritative for the device's current state.
- The server stores and routes encrypted operations. It does not become the plaintext source of truth.
- Authorization is resource-scoped and evaluated before decrypting, applying, or deriving content.
- Device-local paths, secrets, process state, and native credential material never synchronize.
- Backup and synchronization remain separate features.

## Proposed architecture

Clients will maintain typed local operations and CRDT state for synchronizable resources. A self-hostable Hocuspocus service is the current candidate for durable encrypted update routing, presence, and account or device authentication.

Yjs is suitable for collaborative documents and ordered shared state, but it does not by itself define safe relational synchronization. SQLite domains need explicit resource boundaries, typed operations, referential validation, migration compatibility, and deterministic projection into local tables. Raw database pages or arbitrary SQL statements never synchronize.

End-to-end encryption uses per-resource keys or envelopes so the server cannot read ordinary content. Transport encryption remains required in addition to payload encryption. Exact key hierarchy, recovery, rotation, and multi-device enrollment remain deferred design work.

## Identity and membership

Synchronized resources use stable vault, project, channel, document, participant, and device identities. Membership grants are explicit and revisioned. An AI teammate identity has no authority until it receives the relevant membership and resource grants.

The local owner may operate without a remote account. Enabling sync introduces device enrollment and recovery choices without changing ownership of the local vault.

Device-local external-folder bindings do not synchronize. Another device may bind the same portable working-folder identity to a separately selected and validated local directory.

## What may synchronize

Subject to product and privacy settings, synchronizable data may include:

- calendar and project structured data;
- Notes pages, blocks, databases, comments, and collaboration operations;
- organizational Chat channels, memberships, messages, and canonical coordination history;
- portable preferences and themes;
- managed assets through separately bounded encrypted blobs;
- file-authoritative documents as file operations with explicit conflict handling.

Pomodoro history is private by default. Sharing aggregate availability or a user-authored status must not expose detailed focus, pause, idle, or avoidance history unless the user explicitly changes that scope.

Provider credentials, provider homes, executable paths, external absolute folder paths, active process state, and native diagnostic caches do not synchronize.

The storage distinctions in [Data architecture](architecture.md) remain intact. Sync does not make exported Notes Markdown or the server database canonical.

## Permission-safe derivation

Incoming operations are applied only within their authorized resource envelope. Search indexes, summaries, unread state, context packages, and other derivatives are rebuilt under the recipient's effective access. A client must not download a broader plaintext dataset and rely on UI filtering.

Cross-channel references remain audience-safe. A synchronized reference does not carry the source content or its authority into the destination.

Key access, membership, and history cutoffs are revisioned. Cached derivatives include the authorization revision that produced them and are invalidated by a stricter revision.

## Conflict handling

Collaborative text and ordered structures may use CRDT semantics. Domain operations still validate invariants after merge. A merge that would create a Notes cycle, two active Pomodoro segments, an invalid project relationship, or a widened audience cannot be accepted merely because the underlying CRDT converged.

File-authoritative documents require visible file conflict behavior. Silent last-writer-wins replacement is unacceptable for user-authored Markdown or binary assets. The implementation must preserve both versions or request resolution when automatic merging is unsafe.

Schema versions and operation versions are explicit. A client that cannot understand an operation fails safely and retains the encrypted operation for later upgrade rather than partially applying it.

## Revocation and offline devices

Revocation prevents new key distribution and new authorized operations. It cannot erase plaintext already materialized on an offline or external device. The product must state this limitation clearly.

On reconnect, a revoked device cannot submit operations under an obsolete membership revision. Key rotation limits future access. High-risk revocation may require discarding continuations, scratch state, and cached derivatives on still-controlled devices.

## Privacy-safe capacity

Future coordination features may expose availability or workload capacity. Share the smallest useful claim, such as available, busy, or user-entered capacity, instead of raw Pomodoro sessions, idle intervals, private calendar titles, or health-related inferences.

Aggregates must have explicit audiences and provenance. They remain subject to the same revocation and derivative-data rules as their source.

## Self-hosting

The synchronization server must be self-hostable with documented storage, backup, upgrade, and key-management requirements. A hosted option may be added later, but local-only use remains a first-class mode.

Server operators can observe connection metadata, timing, ciphertext size, and account-level routing unless additional padding or privacy work is introduced. End-to-end encryption does not hide those facts.

## Backups

Sync propagates changes, including mistakes and deletions. A backup captures recoverable state at a point in time and is written to a user-selected destination outside the active vault. Enabling sync does not enable backup, and restoring a backup requires an explicit reconciliation policy before reconnecting.

## Deferred decisions

Implementation must resolve, document, and test:

- device enrollment and owner recovery;
- resource-key hierarchy, rotation, and removal;
- exact typed operation formats for relational domains;
- asset chunking, deduplication, and size limits;
- file conflict UX;
- server authentication and abuse limits;
- protocol and schema compatibility windows;
- encrypted backup interaction;
- metadata minimization and optional padding.

These choices may change the proposed mechanism, but they must not weaken the principles above.
