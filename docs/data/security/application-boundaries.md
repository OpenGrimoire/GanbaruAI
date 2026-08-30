# Application security boundaries

Tauri webviews are presentation and orchestration clients. Rust owns database access, filesystem paths, native dialogs, imports, exports, process control, credentials, notifications, media hosts, application blocking, and other operating-system effects.

## Tauri capabilities

Capabilities grant only the plugin operations needed by a window or platform. The desktop app does not expose generic filesystem or shell plugins to the webview. It does expose narrowly scoped window, event, shortcut, zoom, and URL-opening operations required by current UI.

Desktop URL opening is allowlisted to specific product destinations rather than arbitrary web addresses. Android currently permits HTTPS opening more broadly through its platform capability. Any frontend-controlled URL is still parsed and checked by the owning feature before delegation.

Capability files and Tauri configuration are authoritative for exact grants. Documentation should not copy an exhaustive permission list that can silently become stale.

## Content security policy

Production CSP permits bundled application assets, Tauri IPC, and the explicit origins required by current features. Music and preview surfaces require selected YouTube and ephemeral loopback media or frame sources. These are scoped exceptions, not a generic permission for remote scripts or frames.

Development policy additionally supports Vite and hot reload. A development exception must not be copied into production configuration.

Inline style support exists for dynamic Svelte layout, theme, and calendar geometry. Inline script and arbitrary object embedding remain prohibited unless a later feature receives a separate review.

## Native dialogs and document transfer

File import and export flows are Rust-owned. Desktop commands open native dialogs, apply extension and size constraints, validate the selected path, and return bounded parsed data or a narrow result. The webview does not receive a reusable ambient filesystem grant.

Android uses system document providers and MediaStore through repository-owned plugins. The native layer validates display names, MIME ambiguity, size, UTF-8 where required, and operation mode. It exposes selected document content or a bounded export result, not a generic content URI or storage API.

User cancellation is an ordinary typed outcome. It is not reported as a path or permission error.

## Vault and configuration

Vault selection validates the marker, schema version, directory relationship, permissions, and database availability before changing the active pointer. The application never treats an unrelated non-empty folder as a new vault and never silently deletes a damaged configured vault.

Portable config writes occur in Rust through validated patching and atomic replacement. Independent windows must not rewrite the entire config from stale frontend copies. Unknown or obsolete persistent values follow explicit migration and drop rules.

The active vault path is configuration, not a hardcoded Documents location. Managed child paths are derived below the validated active root.

## Managed assets

Profile images, project icons, Chat attachments, Notes assets, and browser artifacts use feature-owned managed locations. Import validates source type, size, content signature where applicable, path identity, and symlink status before copying.

Managed relative paths are stored only after normalization. Cleanup rechecks live references before deletion and records retry state if immediate removal is unsafe. An arbitrary external path cannot be persisted as if it were a managed asset.

Text attachments require valid bounded text. Supported image types are detected from signatures, not only extensions. Errors and diagnostics do not echo source absolute paths unnecessarily.

## Notes content and history

Notes rich content, formulas, imports, transfer manifests, comments, and asset references are untrusted structured input. Rust validates types, sizes, nesting, IDs, graph relationships, and path boundaries before mutation.

Compressed Notes history is validated before parsing. Reads check encoding, declared uncompressed size, decompression bound, content digest, and JSON shape. Chunked history has per-chunk and reconstructed-size limits. Restore applies canonical writes transactionally and rebuilds disposable indexes after success.

Markdown and HTML renderers sanitize or avoid active content. Export is not a route for executing embedded scripts or resolving arbitrary local paths.

## Calendar and import parsers

iCalendar import has bounds on file size, component and property counts, nesting, text expansion, recurrence expansion, and preserved source data. Unknown properties may be retained as bounded data but are not executed or reinterpreted as application commands.

Recurrence expansion has a hard occurrence guard and a requested date window. A malformed or unsupported rule returns a controlled result rather than consuming unbounded CPU or memory.

Notification payloads and platform callbacks are untrusted. They identify a bounded application action and repeat authorization or current-state validation before mutation.

## Pomodoro overlays and enforcement

Desktop Pomodoro blockers and overlays are native windows controlled by Rust and narrowly scoped frontend events. They do not expose a general transparent click-capture or arbitrary window-creation API.

Native enforcement state follows canonical Pomodoro transitions. A stale window event cannot resume, stop, or mutate a newer run. Multi-monitor blocker creation and cleanup are bounded and idempotent.

Browser and desktop Doomscrolling enforcement receive only the rule and focus state needed for their surface. The Chrome native-messaging host validates message shape, size, origin assumptions, and allowed command set. It is independently testable and is not a generic shell bridge.

Mobile enforcement plugins expose only launchable-app discovery, selected settings intents, foreground checks, and durable usage operations required by the feature. Accessibility or usage-access grants do not authorize unrelated data collection.

## Music loopback service

Music uses an ephemeral loopback service for selected local media and YouTube player integration. It binds to 127.0.0.1 on an ephemeral port and protects routes with a per-process random credential. The credential is not logged, persisted, or returned in errors.

The server accepts a small allowlisted HTTP subset with bounded headers, deadlines, worker count, registrations, and resident artwork. Responses disable caching and MIME sniffing and apply route-specific CSP. Plain media and text routes deny active content. YouTube routes permit only the origins required by the embedded player.

Local media registrations use generation and retention rules so a stale frontend load cannot remove the source used by current playback. Switching, stopping, or destroying the host releases registrations that no longer have a live consumer.

The loopback origin is not trusted merely because it is local. Route credentials, host and method validation, size bounds, and SSRF-resistant source handling remain required.

## URL and network adapters

Update pages, licenses, icon sources, Notion import, Freesound, YouTube, provider traffic, and future network features each use a feature-owned adapter and allowlist. Do not add a shared open-any-URL or fetch-any-host command.

Redirects, DNS resolution, IP literals, loopback, link-local, private ranges, credential-bearing URLs, and response-size limits are considered where the adapter can reach user-controlled destinations.

Current and planned network flows are summarized in [Network and privacy](network-and-privacy.md).

## Blocking native work

Filesystem, process, media, Git, and operating-system work that may block runs outside the async database executor. It never holds a SQL transaction or shared async mutex while waiting on blocking work. Cancellation, output, time, and concurrency are bounded.

The repository-wide runtime rules are in [Native work](../../architecture/native-work.md).
