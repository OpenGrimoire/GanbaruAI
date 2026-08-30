# Native backend architecture

Rust owns operations that require durable storage, native authority, bounded filesystem access, provider processes, media playback, or operating-system integration.

## Tauri composition

`apps/client/src-tauri/` is the Tauri package. Its desktop `main.rs` and mobile `lib.rs` delegate to the ordinary Rust library in `apps/client/src-tauri/app/`.

The application library has separate desktop and mobile composition roots. Target-scoped dependencies ensure that unsupported desktop services are not linked into mobile builds. Tauri commands are adapters: they validate transport values, resolve managed state, enforce vault or platform authority, and call focused services.

## Domain crates

The Cargo workspace extracts domains that benefit from Tauri-free contracts and tests:

- `ganbaru-db` owns pool configuration, migrations, and database test support.
- `ganbaru-notes` owns the Notes graph, persistence, transfers, history, assets, and bounded file operations.
- `ganbaru-chat-contracts` defines provider-neutral identifiers, commands, events, read models, and errors.
- `ganbaru-chat-providers` owns provider processes, transports, normalization, cancellation, and registry behavior.
- `ganbaru-chat` owns Chat persistence, runtime services, Git workspaces, checkpoints, review, and source control.
- `ganbaru-working-folders` owns portable folder identities, bindings, repository kinds, and device-local state shapes.
- `ganbaru-native-messaging` is the independent Chromium native messaging host.
- `ganbaru-mobile-*` crates expose narrow Android notification, document, media, and Doomscrolling plugins.

New code should remain in the application crate when it is only composition or Tauri adaptation. Move it to a core crate when the domain boundary, portability, or independent tests justify the extraction.

## SQLite and migrations

The active vault SQLite database is the source of truth for structured data. SQLx embeds migrations from `apps/client/src-tauri/migrations/`. Migration filenames use UTC timestamps and are applied in order.

The database layer does not expose a generic query bridge to the frontend. Domain services own statements, transactions, validation, and result shapes. Cross-table changes that represent one user action commit atomically where partial success would violate the product contract.

Exact columns and indexes belong to migrations. The [schema guide](../data/schema/README.md) documents domain ownership, evolution rules, and important relationships without mirroring every SQL declaration.

## Filesystem authority

The active vault and explicitly authorized working folders are separate roots. Every command resolves identifiers through the owning service, canonicalizes paths where needed, rejects traversal and unsafe symbolic-link escapes, and applies byte, depth, count, or time bounds appropriate to user-controlled input.

Managed assets use validated relative paths beneath the vault. External working files remain externally owned. Device-local absolute paths and provider configuration stay outside synchronized vault records.

## Asynchronous and blocking work

SQLx operations stay asynchronous. Potentially blocking platform APIs, process observation, archive work, image or media probing, and bounded filesystem walks use replaceable or dedicated blocking workers. Cancellation must not publish a partial authoritative result or hold database transactions across blocking waits.

See [Native work](native-work.md) for the shared rule and current examples.

## Native services

Desktop composition can include the tray, updater, detached windows, native notifications, process control, browser native messaging, Rodio and Symphonia audio, MPRIS or Windows media controls, provider processes, terminals, Git workspaces, and browser previews.

Android composition uses plugin adapters and system-owned surfaces instead of importing desktop implementations. Notification schedules, selected document grants, Media3 playback, and Doomscrolling access projections remain subordinate to canonical vault data.

## Error and security boundary

Native commands return bounded, user-meaningful errors without embedding secrets, full remote bodies, unnecessary home paths, or unchecked provider data. Secrets are stored only through operating-system credential references where the feature requires persistence.

Provider output, external files, archives, browser messages, and mobile plugin responses are untrusted input. Validation occurs before persistence or frontend projection. See [Security](../data/security/README.md).
