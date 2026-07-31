# Chat dependency decisions

This document freezes the minimum dependency direction for project-owned coding-agent Chat. It was reviewed on 2026-07-30. A reviewed dependency is not added until the phase that imports it. This keeps unused process, credential, terminal, and rendering code out of the application while preserving an explicit implementation choice.

Package metadata is not evidence that a package is advisory-free. The repository audits are the authoritative advisory gate. Run `pnpm -w run audit` when a reviewed package is first added, and run `pnpm -w run validate:full` for that dependency-sensitive phase. Keep package-security protections enabled.

## Decision summary

| Capability | Decision | First use | Current manifest status |
| --- | --- | --- | --- |
| Async child processes | Tokio process, IO, runtime, sync, and time features | Runtime supervision | Added in Phase 4 |
| Claude interactive transport | Native Rust JSON lines over the existing process owner | Claude provider | Added in Phase 10, no new dependency |
| Cursor interactive transport | Native Rust ACP version 1 over the existing process owner | Cursor provider | Added in Phase 11, no new dependency |
| Process-tree cleanup | Standard process-group support, Unix libc, and existing Windows Job Object APIs | Runtime supervision | Added in Phase 4 |
| Operating-system credentials | `keyring` 4.1.5 with native platform stores | Credential storage | Added in Phase 2 |
| Pseudoterminals | `portable-pty` 0.9.0 | Terminal | Added in Phase 9 |
| HTTP and event streams | Existing Reqwest and Rustls with a bounded Ganbaru SSE decoder | OpenCode | Added in Phase 12, no new dependency |
| Markdown parsing and sanitization | `marked` 18.0.6 plus existing DOMPurify | Timeline | Added in Phase 7 |
| Diff parsing and rendering | `@pierre/diffs` 1.2.12 through its vanilla `CodeView` API | Review workspace | Added for the high-performance review workspace |
| Workspace file observation | `notify` 8.2.0 behind a bounded Rust observer | Live Files, Review, and source-control updates | Added for the high-performance review workspace |
| Terminal emulation | `@xterm/xterm` 6.0.0 and `@xterm/addon-fit` 0.11.0 | Terminal | Added in Phase 9 |
| File editor | CodeMirror 6 through `codemirror` 6.0.2 | Revision-safe file editor | Added for complete local workspace parity |
| ACP protocol types | Official `agent-client-protocol` 2.0.0 crate with ACP v1 negotiation | Cursor and Grok shared driver | Added for negotiated ACP parity |
| Internal MCP bridge | Official `rmcp` 3.0.0 crate | Durable resources and browser tools | Added for complete local workspace parity |

The complete local workspace parity phase added only dependencies explicitly approved for this milestone. CodeMirror replaces the preview-only text surface with a bounded editor. The official ACP crate replaces handwritten core wire types while provider extensions remain validated at the boundary. The official MCP SDK backs an ephemeral loopback endpoint and does not expose a permanent server or change provider-global configuration. The full dependency and advisory gate is `pnpm -w run validate:full`.

Browser capture uses direct target-specific bindings already present in Tauri's locked dependency graph: WebKitGTK 2.0.2 and cairo-rs 0.18.5 on Linux, webview2-com 0.38.2 on Windows, and objc2 WebKit 0.3.2 with block2 0.6.2 on macOS. These bindings avoid a general desktop-capture dependency and capture only the isolated preview webview. Recordings reuse the existing ZIP implementation as bounded PNG frames plus a versioned manifest, so Ganbaru does not add a video codec, media subprocess, or screen-wide capture permission merely to record a preview.

Phase 1 did not import these packages. Phase 4 added only Tokio's narrow process-supervision features, a direct Unix `libc` dependency, and Windows binding features. Phase 9 added the reviewed pseudoterminal, diff, and terminal-emulation dependencies at their first use. Phase 12 implemented OpenCode with existing Reqwest and Rustls dependencies and a repository-owned bounded event-stream decoder.

The Phase 9 dependency audit found no known npm vulnerabilities and only the repository's 18 documented allowed Rust warnings. `portable-pty`, `diff`, `@xterm/xterm`, and `@xterm/addon-fit` introduced no advisory exception.

Phase 10 added no package dependency. Claude Code 2.1.170 and the protocol types in the official Claude Agent SDK 0.3.170 expose every required interactive operation through bidirectional stream JSON. Ganbaru therefore uses the existing Tokio and process-tree boundary directly. The redacted compatibility matrix is stored beside the Claude driver. Claude Code 1.0.92 was observed locally but lacks required partial-message output, so it remains below the supported version floor. An Agent SDK sidecar was rejected because it would add a second runtime, another signed artifact, and a broader supply-chain boundary without adding protocol coverage.

Phase 11 added no package dependency. Cursor exposes ACP version 1 as newline-delimited JSON-RPC through `cursor-agent acp`, so Ganbaru implements the small bounded client directly with the existing Tokio IO, synchronization, time, process owner, Reqwest URL parser, and SHA-256 identity helper. The compatibility matrix and protocol fixtures are stored beside the Cursor driver. The version floor is Cursor Agent 2026.04.08, matching the parameterized model-picker floor in the pinned T3 Code reference. No Cursor executable was installed in the local validation environment, so live compatibility is not claimed. A generic ACP crate or JavaScript sidecar was rejected because the required surface is small, provider extensions still require validation, and either option would add supply-chain or runtime scope without improving the tested boundary.

Phase 12 added no package dependency. Reqwest 0.13 already exposes bounded incremental response chunks, so Ganbaru implements the small Server-Sent Events grammar directly and layers reconnection, deduplication, history reconciliation, cancellation, and payload caps around it. This avoids adding `futures-util` and `eventsource-stream` solely for one transport. The compatibility matrix and redacted protocol fixtures are stored beside the OpenCode driver. OpenCode 1.14.19 is the minimum supported version. No compatible OpenCode executable was installed in the local validation environment, so local compatibility is proven through deterministic process and HTTP fixtures rather than claimed as a live provider probe.

## Async child processes

Phase 4 added a direct Tokio dependency with only the `process`, `io-util`, `rt`, `sync`, and `time` features. The lockfile resolves Tokio 1.52.3. Tauri and SQLx already use Tokio, but Chat does not rely on accidental transitive feature unification. The `rt` feature is required to spawn the bounded stderr reader and per-thread session worker without enabling Tokio's multithreaded runtime or macros.

The Phase 4 dependency audit found no npm vulnerabilities and only the repository's 18 documented allowed Rust warnings. Tokio, `libc`, and the added Windows binding features introduced no advisory exception.

- Maintenance: Tokio is an established Rust asynchronous runtime with active releases and broad ecosystem use.
- Advisories: run the Rust audit when the direct dependency and features are added. The Phase 1 audit covers only the current lockfile.
- Permissions: process creation, pipe IO, timers, and in-process synchronization remain Rust-only. This does not grant the webview shell access.
- Platforms: Tokio process support covers the supported Linux and Windows desktop targets. Platform-specific tree ownership remains separate.
- Size: the runtime is already present transitively. The expected incremental cost is limited to enabled process and IO code, but the packaged delta must be measured when added.
- Standard-library gap: blocking `std::process` alone does not provide the asynchronous bounded IO, cancellation, timeout, and shutdown coordination required by the session supervisor.

## Process-tree cleanup

Do not add a general process-wrapper crate initially. On Unix, configure a process group through the standard process command API and use an explicit Unix `libc` dependency for group signals. On Windows, extend the existing `windows` dependency with the Job Object and required security features, retain the job handle, and enable termination when the job closes.

- Maintenance: `libc` and the official Windows bindings are already established dependencies in the Rust ecosystem and this repository.
- Advisories: audit the lockfile after changing target dependencies or features.
- Permissions: the backend gains only the platform APIs needed to own and terminate children that it launched.
- Platforms: Unix process groups and Windows Job Objects provide native tree semantics. Windows providers start suspended, enter a kill-on-close Job Object, and resume only after assignment, which closes the descendant spawn race. Unix fixtures execute on Linux. Windows-only self-executable fixtures type-check on the installed Windows GNU target and exercise normal exit, malformed output, immediate descendant creation, forced cleanup, and idempotence when run on Windows.
- Size: no cross-platform supervisor framework is added. The incremental code is target-specific bindings and Ganbaru lifecycle logic.
- Rejected alternatives: `command-group` is small and established, but it does not remove the need to prove Ganbaru's Windows ownership race, shutdown deadlines, and idempotent cleanup. Direct platform ownership keeps those guarantees visible.

## Operating-system credentials

Use [`keyring` 4.1.5](https://crates.io/crates/keyring/4.1.5) with its native version 1 store selection. Linux uses Secret Service through zbus, Windows uses Credential Manager, and a later macOS target can use Keychain. Calls that may block run outside the asynchronous session worker.

Phase 2 added this dependency with its default native store selection. `pnpm -w run validate:full` passed after the addition. npm reported no vulnerabilities, and Rust reported only the repository's 18 documented allowed warnings. Chat imports the crate only behind a Rust-owned credential trait and does not register credential commands for the webview.

- Maintenance: the crate is maintained by the Open Source Cooperative, uses MIT or Apache-2.0 licensing, and declares Rust 1.88 as its minimum version. The workspace toolchain is newer.
- Advisories: run `cargo audit` when the crate enters the lockfile. Treat unavailable or locked stores as typed operational states, not reasons to fall back to plaintext.
- Permissions: only the Rust credential service can read or replace a secret. Svelte receives opaque references and redacted status.
- Platforms: the selected native stores cover Linux and Windows. Keychain remains the reviewed future Apple backend.
- Size: the Linux Secret Service and cryptographic stack can add meaningful transitive code. Measure the packaged delta in the phase that adds it.
- Existing-code gap: app state and `config.json` are ordinary files and are prohibited secret stores. A generic Tauri keyring plugin would expose a broader frontend boundary than the Chat-specific service needs.

## Pseudoterminals

Use [`portable-pty` 0.9.0](https://crates.io/crates/portable-pty/0.9.0) behind a Ganbaru terminal service. Put its synchronous reader and writer on dedicated blocking tasks connected to bounded channels.

Phase 9 added the exact reviewed version behind Rust-owned thread and workspace authorization. Terminal sessions are capped per thread and application. Input, output chunks, replay, names, and dimensions are bounded, while generation IDs reject output from restarted sessions. The application exit path terminates every owned pseudoterminal.

- Maintenance: the crate is maintained in the active WezTerm repository and uses the MIT license.
- Advisories: run the Rust audit after addition and test its Unix and Windows transitive backends.
- Permissions: PTY creation, shell selection, input, resize, and termination remain narrow Rust commands tied to a validated workspace and thread.
- Platforms: the crate supports Unix PTYs and Windows ConPTY-style operation. Target builds and process-tree tests remain required.
- Size: it adds platform PTY bindings and supporting crates. Measure the packaged delta when Terminal is implemented.
- Standard-library gap: Rust has no cross-platform pseudoterminal API. Plain pipes cannot provide terminal sizing, interactive applications, or correct terminal semantics.
- Safety note: do not use its transitive shell-word helpers to turn user text into an executable command string.

## HTTP and event streams

Keep the existing Reqwest 0.13 and Rustls 0.23 stack. OpenCode uses Reqwest's incremental response chunks with a repository-owned bounded Server-Sent Events decoder. No additional HTTP, stream, or parser dependency is needed.

- Maintenance: Reqwest and Rustls are active foundational crates already used by the application. The local decoder is small, protocol-specific, and covered by split-chunk, multiline, malformed, replay, and size-bound fixtures.
- Advisories: no manifest or lockfile changed in Phase 12. Continue to run the normal repository audit before pull requests and releases. Do not substitute another event-source client that duplicates the HTTP stack.
- Permissions: Rust owns loopback and explicitly configured external connections. The webview CSP gains no provider transport access.
- Platforms: the selected stack is portable across the supported desktop targets and uses the repository's existing ring-backed Rustls provider.
- Size: Reqwest and Rustls already exist. The decoder adds only repository code and no transitive package graph. The final Chat binary delta must still be recorded.
- Existing-code gap: Reqwest provides bounded response chunks but not Chat's event framing, reconnect, event-ID deduplication, readiness, cancellation, or payload bounds. Ganbaru owns those policies around the decoder.

## Markdown parsing and sanitization

Use [`marked` 18.0.6](https://www.npmjs.com/package/marked) as a lazy timeline chunk. Sanitize every generated fragment with the existing DOMPurify dependency, reject raw HTML, bound source size and nesting, and route links through Ganbaru's scheme and workspace-path validation.

Phase 7 added the exact reviewed Marked version. Chat uses a dedicated parser instance with renderer-object overrides, caps total source, line size, line count, indentation, and quote nesting, and sanitizes the generated fragment with a narrow DOMPurify allowlist. Remote Markdown images are reduced to inert alt text. Explicit HTTP and HTTPS links are revalidated by Rust before the operating system opens them.

The Phase 7 dependency gate passed. npm reported no known vulnerabilities, Rust reported only the repository's documented allowed warnings, and the full static, test, editor, production build, and bundle-contract checks passed.

- Maintenance: Marked is actively released, uses the MIT license, has zero production dependencies, and requires Node 20 or newer. The workspace Node range satisfies that requirement.
- Advisories: run `pnpm audit` when it is added. DOMPurify remains the sanitizer even if parser defaults appear safe.
- Permissions: Markdown parsing is pure frontend work. Opening links and local files remains an explicit validated native action.
- Platforms: the browser package works in the Tauri webview on supported desktop targets.
- Size: the published Marked package is about 451 KB unpacked. Lazy loading prevents it from entering initial route work.
- Existing-code gap: DOMPurify sanitizes HTML but does not parse Markdown or preserve incomplete streaming blocks.

## Diff parsing and rendering

Use [`@pierre/diffs` 1.2.12](https://www.npmjs.com/package/%40pierre/diffs) through its framework-neutral `CodeView` API. Rust remains authoritative for repository authorization, snapshot identity, file and hunk identity, binary and rename metadata, and every Git mutation. The browser renderer receives only validated, whole-hunk patch pages.

The high-performance Review workspace replaces the earlier `diff` parser and hand-built line DOM. Pierre is exact-pinned behind a Ganbaru adapter, lazy-loaded only when Review opens, and isolated with Shiki and its module worker. Grammar and theme payloads remain separate on-demand production chunks instead of being collapsed into the Review core. Plain virtualized text renders before asynchronous syntax highlighting. Worker, parser, or highlighter failure keeps a bounded raw-patch fallback usable.

- Maintenance: Pierre is actively maintained, Apache-2.0 licensed, and established enough for use behind an exact-version adapter. Its young public API requires deliberate upgrade review.
- Advisories: run the npm audit and full dependency gate after addition. Its React peer declarations are marked optional only for this exact package because the vanilla entry has no React runtime import. Resolved React and ReactDOM packages are prohibited from the production dependency graph and emitted bundles.
- Permissions: the renderer and worker receive inert patch text and typed callbacks. They have no filesystem, process, network, or Tauri command capability.
- Platforms: the vanilla browser API targets the WebView engines used by Tauri. The module worker is same-origin and the content security policy does not permit blob workers.
- Size: Pierre and Shiki add several megabytes of packaged grammar and theme assets. A lazy Review core, on-demand grammar and theme chunks, one to three workers, bounded caches, and bundle contracts protect startup and normal Chat use.
- Rejected alternatives: Monaco duplicates the CodeMirror editor and carries a much larger editor runtime. A new custom renderer would recreate virtualization, split alignment, syntax, intraline changes, annotations, and accessibility behavior without an established engine.

## Workspace file observation

Use [`notify` 8.2.0](https://crates.io/crates/notify/8.2.0) behind a Rust-owned Chat workspace observer. One observer follows the active authorized execution environment while Chat is active, coalesces bounded relative-path events, and invalidates Files, Review, and source-control projections.

- Maintenance: `notify` is the established cross-platform Rust filesystem notification crate, uses CC0-1.0, and supports the repository toolchain.
- Advisories: run the Rust audit and full dependency gate after addition. Do not add a JavaScript file-watching sidecar or expose a generic watcher command to the webview.
- Permissions: Rust resolves the authorized root and emits only normalized relative paths and semantic invalidation flags. Symlink escapes, safety-excluded paths, and absolute paths never cross IPC.
- Platforms: the recommended native backend covers Linux, Windows, and macOS. Overflow or unavailable native observation falls back to bounded active-Chat polling with a visible degraded state.
- Resource policy: maintain one observer, coalesce bursts, cap each event batch, and stop observation when Chat closes or changes execution environment.

## Terminal emulation

Use [`@xterm/xterm` 6.0.0](https://www.npmjs.com/package/%40xterm/xterm) with the official [`@xterm/addon-fit` 0.11.0](https://www.npmjs.com/package/%40xterm/addon-fit). Load both only when the Terminal surface opens.

Phase 9 added both exact reviewed versions. The terminal component dynamically imports the emulator, the fit addon, and xterm CSS only when a terminal view mounts. It accepts sequenced generation-scoped bytes from typed Tauri events and does not load network, link, clipboard, image, or WebGL addons.

The Phase 9 dependency gate passed. npm reported no known vulnerabilities, Rust reported only the repository's 18 documented allowed warnings, and the full static, test, editor, production build, and bundle-contract checks passed.

- Maintenance: xterm.js is actively maintained, used by established developer tools, and both selected packages use the MIT license with zero production dependencies.
- Advisories: run `pnpm audit` when they are added. Keep experimental APIs out of the initial implementation.
- Permissions: the emulator receives sequenced terminal bytes from typed Tauri events. It does not receive a WebSocket, generic attach transport, or direct clipboard and link addons.
- Platforms: the browser renderer supports current Chromium-family webviews and includes screen-reader and contrast support.
- Size: the xterm package is about 5.9 MB unpacked and the fit addon about 21 KB unpacked. Dynamic import and route-level bundle checks are mandatory.
- Existing-code gap: a text area cannot correctly emulate ANSI, cursor movement, full-screen terminal applications, Unicode width, selection, or accessibility semantics.
- Initial addon boundary: do not add attach, web-links, clipboard, WebGL, or image addons. Add a capability only after a concrete requirement and review.
