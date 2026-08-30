# Security

Ganbaru AI stores calendar, work, Notes, browsing-control, focus, and AI conversation data. Security is a constraint on every storage and execution decision, not a feature that can be added after those decisions are made.

## Threat model

The application must limit:

- compromised or malicious dependencies during development, build, install, and runtime;
- untrusted provider processes, provider protocol events, repository content, Markdown, terminal output, URLs, media metadata, imports, and native callbacks;
- a webview or frontend defect attempting to exceed its narrow native commands;
- path traversal, symlink escape, replaced external folders, stale Git identity, and unsafe archive or import expansion;
- accidental disclosure through logs, diagnostics, search, summaries, exports, attachments, caches, and cross-channel references;
- network observation or server-side plaintext access when future synchronization is enabled;
- release, updater, package-repository, and CI credential compromise.

The application can constrain native processes it starts and the data it deliberately sends to external services. It cannot stop arbitrary same-user malware from reading an unencrypted vault through ordinary operating-system access. Local data is not application-encrypted. The security boundary assumes the user's operating-system account and disk encryption protect files at rest.

The application also does not defend against a user intentionally modifying their own database, a determined attacker with physical access to an unlocked or unencrypted device, or an AI provider retaining content according to its own policy after the user chose to send it.

## Trust boundaries

Rust application services are the policy boundary for filesystem, process, credential, database, import, export, update, and operating-system operations. The Svelte webview receives validated DTOs and scoped commands, not ambient shell or filesystem access.

Provider-native trust never widens organizational authority. Portable IDs do not authorize a device path. User-authored text, mentions, URLs, repository instructions, and provider events are data, not commands or grants.

The active vault is user-owned durable storage. The platform app config directory is trusted only for device-local bootstrap and binding state. Secrets are resolved through native credential storage and are not copied into ordinary config, SQLite, diagnostics, or provider DTOs.

## Security documents

- [Supply chain](supply-chain.md) covers dependencies, CI, releases, copied code, and contributor rules.
- [Application boundaries](application-boundaries.md) covers Tauri capabilities, native I/O, imports, assets, Notes, overlays, and loopback services.
- [Chat security](chat.md) covers provider processes, working folders, terminals, attachments, previews, checkpoints, and internal host tools.
- [Network and privacy](network-and-privacy.md) covers telemetry, current and future network flows, local encryption, sync encryption, and metadata exposure.
- [Dependency audits](dependency-audits.md) records reviewed advisory exceptions and the current warning snapshot.
- [Chat access control](../access-control.md) is the normative authorization specification.

## Status discipline

Current local features, proposed architecture, and future security requirements must be labeled separately. Remote synchronization, hosted BYOK chat, an external MCP service, and the ganbaru-ai CLI are not implemented. Security requirements for them remain normative design constraints, not claims about current traffic or capability.

## Required properties

Across domains:

- deny unknown, stale, oversized, malformed, or unsupported input;
- validate before mutation and again at the privileged boundary;
- use bounded reads, writes, decompression, parsing, process output, queues, and worker counts;
- remove secrets and unnecessary absolute paths from persisted errors and diagnostics;
- use explicit allowlists for native commands, URL schemes, origins, file types, and tool methods;
- preserve user-authored data when a safe repair or export path exists;
- make cleanup retryable when immediate deletion is unsafe;
- keep security-sensitive decisions in testable Rust services rather than presentation code.

Security documentation records rationale and policy. Exact capabilities, CSP directives, URL scopes, and dependency versions remain authoritative in configuration and lockfiles and must be reviewed when they change.
