# Chat security

The local coding-agent Chat starts native provider processes and operates on user-selected repositories. Provider CLIs, protocol events, model output, repository instructions, terminal output, URLs, Git metadata, and errors are untrusted. Rust is the policy boundary.

Organizational authorization is specified in [Chat access control](../access-control.md). This document covers how native boundaries enforce it.

## Provider lifecycle

Rust owns provider discovery, startup, transport, cancellation, event ingestion, and shutdown. The frontend selects a configured provider and sends typed application commands. It cannot spawn an arbitrary executable, choose an arbitrary provider home, or connect directly to a provider's local server.

One mutable driver generation owns an active thread. Events from a stopped, settled, or mismatched generation cannot enter the ordinary projection stream. Shutdown rejects new commands, flushes pending bounded event batches, and enforces deadlines even when a provider hangs.

Provider-native approval, trust, sandbox, or workspace state is defense in depth. It never widens folder, channel, runtime, or internal-tool authority granted by Ganbaru AI.

## Provider configuration and secrets

Portable provider configuration stores nonsecret values and opaque credential references. External executable paths, provider homes, discovery caches, and current process state are device-local.

Secrets are resolved from the operating-system credential store only immediately before the operation that needs them. Temporary provider environments drop credential references before reaching drivers and are not persisted in diagnostics or DTOs.

Replacing or removing a credential invalidates affected probe and model caches. Cleanup errors are reported generically without exposing secret values or native credential-store details.

The provider-file settings portal is a narrow exception for user configuration and instruction files. Rust resolves a fixed allowlist from provider family and instance. Reads and writes reject symlinks, invalid UTF-8, null bytes, parent traversal, oversized data, stale revisions, and unsupported files. Saves use restrictive permissions and sibling temporary replacement. File contents may contain secrets and are not copied into the vault, SQLite, diagnostics, or logs.

## Working-folder authorization

Portable working-folder identity contains no external absolute path. Managed folders resolve below the active vault. External folders resolve through a device-local binding scoped by vault, device, and logical folder ID.

Rust validates the opened directory's opaque filesystem identity. Unix uses device and inode identity. Windows uses the equivalent opened-directory volume and file identity while rejecting reparse-point escapes. Git-sensitive operations separately validate the Git common-storage identity.

Missing directories, stale paths, replacement, traversal, absolute injection, symlink escape, vault nesting, and unbound devices fail closed. Remote URLs, branch names, and mutable Git configuration do not participate in authority.

The same authorization service is used for provider startup, terminals, attachments, Markdown operations, file tools, mentions, previews, watchers, Git, checkpoints, and restores. No adapter reimplements a weaker path check.

## File reads and writes

Workspace operations accept validated relative paths below the authorized root. Listing is on demand and applies Git ignore rules plus bounded exclusions for common generated or secret-bearing directories.

Text preview rejects binary content, invalid UTF-8, symlinks, traversal, and oversized files. The webview receives bounded text and relative metadata only. Opening a file externally repeats native authorization immediately before delegation.

Writes use explicit expected revision or conflict checks where user edits could race provider edits. Temporary replacement remains inside the authorized parent. A tool cannot create a symlink or use a renamed ancestor to escape the root.

Provider-reported changed files are hints. Ingestion canonicalizes only verified relative paths and removes duplicates and paths outside the root before persistence.

## Workspace observation

The active workspace observer owns one native watcher for the current authorized target. Switching targets stops the previous generation before a new one can emit events.

Watcher output is coalesced into bounded semantic invalidations. Absolute paths are removed. Excluded directories are filtered. Overflow emits one typed invalidation requiring a refresh instead of forwarding an unbounded event stream.

A watcher event never proves current authorization or file existence. Every later read repeats validation.

## Terminals and processes

A terminal is bound to one authorized thread, execution target, runtime approval, and process generation. Working directory and environment are constructed in Rust. The frontend cannot supply an arbitrary host path or inherit the full application environment.

Input, output, scrollback, event queues, and process lifetime are bounded. Terminal output is untrusted display data. Escape sequences and link detection must not create an unreviewed native action.

Stopping, revoking, changing target, or closing the owning scope terminates or quarantines the terminal according to policy. A stale terminal handle cannot attach to a newer run.

## Internal MCP host tools

Each native Chat run may receive an ephemeral loopback MCP endpoint with a small application-owned method allowlist. It is not exposed as a general localhost service and is not an organizational participant.

Every request validates a run-scoped credential, method, bounded arguments, current run and thread, channel membership, access-profile revision, execution target, and revocation state. Return values exclude credentials, external absolute paths, and unrelated vault content.

The endpoint closes with the run. Replayed credentials and calls after interruption or revocation fail. A future external MCP service requires separate authorization and cannot reuse this endpoint.

## Event ingestion and diagnostics

Provider event size, nesting, sequence, identifier, and payload shape are bounded before canonical persistence. Unknown event kinds are handled as typed unsupported data or safe diagnostics, not dynamically executed.

Diagnostics reject secret-like fields, authorization material, unnecessary home paths, provider environments, and raw credentials. Persisted errors provide enough context to debug the application boundary without becoming a second transcript of sensitive provider output.

Projection changes commit before the frontend receives a notification. A notification contains identity and invalidation information, not an unbounded event payload.

## Attachments and context

Attachment import rejects symlink sources, unsupported signatures, invalid text, traversal, and files above the feature limit. Imported bytes are hashed and written with restrictive permissions below the managed Chat asset root. Portable metadata stores managed relative identity rather than the source path.

Message and draft references own retention. Deferred cleanup rechecks references immediately before removal so a newly reused attachment is not deleted.

Mentions, workspace files, Notes, tasks, channel messages, browser artifacts, and other context sources are resolved under current authority. Natural-language instructions or provider requests cannot introduce a hidden source. Audience restrictions continue to apply after content is summarized or transformed.

## Browser previews

Preview URLs and navigation are parsed and restricted by the preview service. Loopback, private, link-local, credential-bearing, and unsupported schemes are denied unless a narrowly reviewed local-preview route requires them.

Webview identity, port, generation, navigation events, screenshots, recordings, and cleanup are tied to the owning thread and target. A stale preview cannot emit current events or retain a broad opener capability.

Browser artifacts are bounded managed assets with explicit references and cleanup. Cookies, local storage, and authenticated browsing are not silently imported into provider context.

## Checkpoints and restore

Checkpoint capture and restore verify folder identity and Git common-storage identity immediately before mutation. Dirty state, changed files, untracked files, repository boundaries, and expected checkpoint revision are validated.

Restore is a scoped Git or filesystem operation, not a database rollback. It cannot change organizational messages, memberships, approvals, or another project's files. Previewing a restore and applying it are separate authority checks.

Cleanup and restoration use bounded blocking work outside database transactions. Canonical status and retryable cleanup records prevent a failed native step from appearing successful.

## Revocation

When membership, history, profile, target, runtime, scratch, or audience authority is reduced, new native actions fail immediately. A continuation that already materialized restricted context is interrupted and discarded or quarantined before reuse.

Provider stop, tool shutdown, terminal cleanup, preview cleanup, and scratch cleanup have bounded deadlines. Incomplete cleanup persists as a retryable record without weakening the revocation or exposing native paths.
