# Device linking and synchronization

**Status: Planned.** Device enrollment, replication, encryption, conflicts, and the optional relay are not implemented. The first prerequisite work separates Android reminders from recorded execution, extracts focus persistence and recovery into `ganbaru-focus`, and configures authoritative SQLite connections for durable commits. Those changes do not enable device linking.

This contract links one person's devices. Multi-person sharing is later work. The phone must provide full offline access to portable Notes, Projects, Calendar, and other synchronized content. Focus execution has its own controller and evidence rules in [Focus authority](../algorithms/pomodoro/focus-authority.md).

## Architecture

SQLite remains the durable local store. Replicas exchange validated domain operations and immutable assets, never raw database pages, arbitrary SQL, or unclassified application configuration. Foreground saves do not wait for a network.

Collaborative text uses Rust Yrs with a compatible Yjs editor adapter. Binary CRDT state is canonical in SQLite; rich-text payloads and plain-text columns are deterministic query and rendering projections. Text document identities are independent of block placement. Active documents use a bounded lazy cache.

Local network linking works without an account or server. An optional user-hosted Rust relay stores opaque encrypted records for cross-network and asynchronous delivery. Hocuspocus is no longer the proposed relay: its normal persistence loads and stores server-side Yjs documents, which does not match this encrypted record boundary. See [Hocuspocus persistence](https://tiptap.dev/docs/hocuspocus/guides/persistence).

The intended crates are `ganbaru-sync-contracts`, `ganbaru-sync`, and an optional `ganbaru-sync-relay` binary. They have not been created. Domain services retain validation and projection ownership; the sync engine owns delivery and calls those adapters. Durable replication, live presence, and executable commands are distinct protocols.

## Storage ownership

Every persisted field requires an explicit replication classification before it can leave a device. Unknown fields fail closed. This table is the target ownership contract, not a claim that existing mixed configuration has already migrated.

| Domain | Portable data | Device-local data |
| --- | --- | --- |
| Notes, Projects, Calendar, Quick notes | Content, relationships, templates, archive, Trash, retained history, favorites | Recents, current selection, navigation, viewport layout |
| Preferences | Profile, themes, language, time format, rhythm defaults | Font scale, layout, shortcuts, notification delivery, explicit presentation overrides |
| Doomscrolling | Rule definitions and device-attributed history | Permissions, application bindings, enforcement state |
| Music | Library identities, playlists, assignments, portable preferences | Source bindings, media bytes, current playback, volume, routing |
| Chat | Organizational content, portable review history, origin-attributed drafts | Execution processes, credentials, provider homes, terminals, native trust, paths, caches |
| Managed assets | Immutable content and metadata | Transfer staging, local availability, caches |
| Focus | Committed history, explicit controller ownership history | Live presence, local activity sources, native alarms and effects |

Unsent Chat drafts retain their origin and can be explicitly continued on another device. Sending clears only the revision sent. Arbitrary project source trees and external music files keep their existing device boundaries.

Shared preferences move into SQLite so preference changes and outbound records can commit atomically. Device preferences stay in application configuration storage. Remove canonical `config.json` and the unused `.yjs` vault skeleton only after their consumers migrate. Both still exist today.

## Transactional operation boundary

Every synchronizable mutation, including imports, restores, scheduled jobs, and native background writes, must commit these together:

- Canonical changes and required relational projections.
- Stable operation identity, cryptographic device identity, writer generation, causal dependencies, resource scope, authorization revision, and protocol version.
- The operation receipt and durable outbound record.
- Required history and asset references.

UI invalidations and native effects follow commit. Incoming operations use the same validators and transaction boundary. Missing dependencies remain pending. Malformed or unauthorized records receive bounded diagnostics. A relay receipt means delivery to the relay; it does not mean another device committed the change.

Authoritative connections use WAL and `synchronous=FULL`, including replacement connections in a pool. SQLite documents that WAL with `NORMAL` can lose committed transactions on power failure; `FULL` synchronizes the WAL at each commit. Hardware and filesystem behavior still require failure testing. See [SQLite synchronous](https://www.sqlite.org/pragma.html#pragma_synchronous).

The UI distinguishes saved on this device, received by relay, and confirmed on another device.

## Conflict semantics

Valid concurrent operations converge regardless of delivery order. Rejecting the second operation to arrive is insufficient.

- Independent fields merge. Concurrent values for the same scalar retain alternatives, a deterministic displayed value, and a visible resolution action.
- Calendar start, end, timezone, and recurrence form one coupled value. Conflicts suspend automatic occurrence activation until resolved.
- Notes and project placement use stable identities and a cycle-safe replicated tree move algorithm. Stable ordering identifiers replace floating positions. Validate against a simple reference model of the [replicated move algorithm](https://martin.kleppmann.com/papers/move-op.pdf).
- Deletion creates tombstones. Concurrent edits remain recoverable in Trash or conflict recovery. Old operations cannot silently restore deleted content.
- Database property type changes retain incompatible values for resolution.
- History restore writes a safety version and new operations against current state. It never rewinds causal history.
- Cross-feature actions such as task scheduling form one operation group.
- Scheduled messages and other executable jobs have an execution device and stable execution receipts. Receiving their records cannot execute them.

## Notes editing

Whole-block replacement is not a collaborative text protocol. The editor must submit incremental operations, preserve relative selections and comment anchors, and keep locally authored undo separate from concurrent remote changes. TypeScript and Rust use UTF-16 positions. Composition, autocorrect, paste, marks, mentions, and Unicode need adapter-level tests.

Prepare incoming changes in isolated working state. Validate and atomically commit binary updates and SQL projections before publishing them to the active cache. Discard the working state on failed persistence. Preserve unsaved input and show saving until native acknowledgement. Writer IDs must be unique across installations, restored copies, and simultaneous windows. See [Yrs](https://docs.rs/yrs/latest/yrs/).

Current Notes writes still replace complete block payloads. Their collaboration log covers comments and suggestions, not general replica synchronization.

## Enrollment and key lifecycle

The first desktop is the administration device. A short-lived, single-use QR invitation contains its identity fingerprint and a high-entropy enrollment secret. Manual entry accepts the complete invitation. Both devices show a verification code; the existing device explicitly confirms enrollment before releasing vault keys.

Direct connections use TLS 1.3 with pinned device identity. Operations are signed. Records and asset chunks use XChaCha20-Poly1305. Resource-key distribution uses HPKE with X25519 and HKDF-SHA256. Secrets use native desktop credential storage or Android Keystore wrapping. Review maintained implementations, minimal features, pinned versions, advisories, and the protocol composition before enabling transport. [HPKE](https://www.rfc-editor.org/rfc/rfc9180.html) does not provide application authorization, replay protection, or downgrade protection by itself.

Enrollment and revocation follow signed administration history. A separate owner recovery identity and recovery kit receive resource-key envelopes alongside authorized devices. Revocation rotates affected keys and rejects new operations from the removed device once revocation is known. Preserve rejected pending content for explicit recovery. Removal cannot erase copies already held by that device.

## Bootstrap, assets, backup, and compaction

Bootstrap transfers a consistent typed snapshot and causal checkpoint followed by incremental operations. Stage, validate references and integrity, then activate atomically. If a phone has a different vault, retain it as a recoverable local vault. Combining vaults requires an explicit import preview.

Managed assets use immutable identities, encrypted manifests, authenticated hashes, bounded resumable chunks, and atomic publication. Native code transfers bytes. Filenames remain metadata and cannot choose destination paths. Structured data synchronizes automatically; managed attachments default to unmetered transfer with explicit download and offline controls.

Backups capture a consistent database and pinned asset set using authenticated encryption. Restore defaults to an isolated recovery copy. Rejoining the original vault requires current membership reconciliation and a fresh writer generation. Do not restore credentials, rewind acknowledgements, or resurrect tombstoned resources.

Offline enrolled devices retain the causal state and tombstones they need. Compaction requires acknowledged checkpoints; retirement is explicit. Asset collection considers live references, retained history, pending transfers, and backup pins.

Vault replacement must fence the generation across processes, pause native work, close pools, swap staging, and restart against the new generation. The existing process-local replacement guard is not sufficient for this target.

## Settings and Android delivery

Onboarding and Settings will provide device linking, linked-device identity, connection method, last successful synchronization, pending changes, unavailable assets, conflicts, recovery status, focus controller, pause, retry, removal, recovery export, and optional relay configuration.

Android uses WorkManager for deferred synchronization and a visible, user-enabled connected-device service for live companion status. Alarms deliver scheduled reminders. Permission denial, process death, reboot, network changes, and background restrictions must expose degraded connectivity truthfully. Background service availability never establishes focus or idle activity.

## Delivery and acceptance

| Milestone | Status | Remaining work |
| --- | --- | --- |
| Focus correctness and contracts | Partial | Move all transition decisions and command receipts into Rust, add durable device controller ownership and native runtime bridge |
| Durable mutation and storage boundaries | Partial | WAL durability is configured; scoped preferences, cryptographic writers, journal and domain-wide atomic mutation coverage remain |
| Local replica convergence | Planned | Notes text and tree merging, all portable domain adapters, two- and three-replica failure tests |
| Secure local linking | Planned | Enrollment, key storage, staged bootstrap, assets, recovery, onboarding and Settings |
| Relay and Android sync | Planned | Encrypted relay, native background runtime, encrypted backup, measurements and physical acceptance |

Required tests include reordered, duplicated, delayed and interrupted delivery; text and tree convergence; deletion, undo and history restore; crashes at persistence and acknowledgement boundaries; full disks, corrupt staging and missing assets; invalid identity, signature, invitation replay, revocation, key epochs, payload bounds and protocol versions; controller handoff failure and expired commands; duplicate jobs; long-offline replicas, compaction, restored backups and cloned writers.

Measure bootstrap, input and save latency, bandwidth, memory, battery, and database contention. Queues and caches remain bounded, with lazy loading, incremental indexing, and background backoff. Run the serialized `validate:full` gate for the complete dependency and security changes. Pairing, key lifecycle, native bridges and remote capabilities require a separate security review. Physical Android and desktop acceptance is mandatory for sleep, force-stop, reboot, permissions, manufacturer restrictions, clock changes, and disconnected use.
