# Chat dependency decisions

This document freezes the minimum dependency direction for the local coding-agent Chat workspace. It was reviewed on 2026-07-20. A reviewed dependency is not added until the phase that imports it. This keeps unused process, credential, terminal, and rendering code out of the application while preserving an explicit implementation choice.

Package metadata is not evidence that a package is advisory-free. The repository audits are the authoritative advisory gate. Run `pnpm -w run audit` when a reviewed package is first added, and run `pnpm -w run validate:full` for that dependency-sensitive phase. Keep package-security protections enabled.

## Decision summary

| Capability | Decision | First use | Current manifest status |
| --- | --- | --- | --- |
| Async child processes | Tokio process, IO, runtime, sync, and time features | Runtime supervision | Added in Phase 4 |
| Process-tree cleanup | Standard process-group support, Unix libc, and existing Windows Job Object APIs | Runtime supervision | Added in Phase 4 |
| Operating-system credentials | `keyring` 4.1.5 with native platform stores | Credential storage | Added in Phase 2 |
| Pseudoterminals | `portable-pty` 0.9.0 | Terminal | Added in Phase 9 |
| HTTP and event streams | Existing Reqwest and Rustls, plus `futures-util` and `eventsource-stream` 0.2.3 | OpenCode | Deferred |
| Markdown parsing and sanitization | `marked` 18.0.6 plus existing DOMPurify | Timeline | Added in Phase 7 |
| Diff parsing and rendering | `diff` 9.0.0 plus a bounded Ganbaru renderer | Inspector | Added in Phase 9 |
| Terminal emulation | `@xterm/xterm` 6.0.0 and `@xterm/addon-fit` 0.11.0 | Terminal | Added in Phase 9 |

Phase 1 did not import these packages. Phase 4 added only Tokio's narrow process-supervision features, a direct Unix `libc` dependency, and Windows binding features. Phase 9 added the reviewed pseudoterminal, diff, and terminal-emulation dependencies at their first use. The HTTP and event-stream dependencies remain deferred until OpenCode is implemented.

The Phase 9 dependency audit found no known npm vulnerabilities and only the repository's 18 documented allowed Rust warnings. `portable-pty`, `diff`, `@xterm/xterm`, and `@xterm/addon-fit` introduced no advisory exception.

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

Keep the existing Reqwest 0.13 and Rustls 0.23 stack. When OpenCode is implemented, declare the Reqwest `json` and `stream` features directly, add direct `futures-util`, and use [`eventsource-stream` 0.2.3](https://crates.io/crates/eventsource-stream/0.2.3) as the bounded Server-Sent Events parser.

- Maintenance: Reqwest, Rustls, and futures are active foundational crates. `eventsource-stream` is small and stable, but its release cadence is slower, so its narrow parser boundary needs fixture coverage.
- Advisories: run the Rust audit when features and direct dependencies change. Do not substitute `reqwest-eventsource` while it would duplicate the repository's Reqwest major version.
- Permissions: Rust owns loopback and explicitly configured external connections. The webview CSP gains no provider transport access.
- Platforms: the selected stack is portable across the supported desktop targets and uses the repository's existing ring-backed Rustls provider.
- Size: Reqwest and Rustls already exist. The incremental parser graph is small compared with another HTTP client, but the final binary delta must still be recorded.
- Existing-code gap: Reqwest does not provide Chat's reconnect, event-ID deduplication, readiness, cancellation, or payload bounds. Ganbaru owns those policies around the parser.

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

Use [`diff` 9.0.0](https://www.npmjs.com/package/diff) to parse bounded unified patches, then render unified and fitting split views in focused Svelte components. Rust remains authoritative for file identity, binary state, rename metadata, and checkpoint comparison.

Phase 9 added the exact reviewed version. The Changes panel loads it dynamically only after a bounded Rust patch is selected, converts the parsed result into inert text rows, and caps the rendered line collection. The parser never determines file identity or restore scope.

- Maintenance: jsdiff is actively released, uses BSD-3-Clause licensing, and has zero production dependencies.
- Advisories: run `pnpm audit` when it is added and keep malformed-patch fixtures at the wrapper boundary.
- Permissions: the parser receives already bounded text. It has no filesystem or process access.
- Platforms: it is portable browser JavaScript.
- Size: the published package is about 616 KB unpacked and should be lazy-loaded with the inspector.
- Rejected alternative: a Shiki-based diff renderer brings a much broader syntax, theme, worker, and HTML transformation graph than the required safe text diff. Ganbaru does not need that supply-chain or bundle cost.

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
