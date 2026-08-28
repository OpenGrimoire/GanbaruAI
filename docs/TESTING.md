# Testing

Ganbaru AI uses layered validation across its Svelte frontend, Rust backend, SQLite persistence, browser extension code, production bundle, and dependency graph. This document defines the durable testing strategy and local validation workflow. Mandatory agent behavior remains summarized in `AGENTS.md`.

## Goals

The test and validation system should:

- Protect durable user data and security boundaries.
- Catch behavior, type, styling, platform, and production-bundle regressions.
- Give focused feedback during development and a comprehensive gate when required.
- Keep broad validation predictable on resource-constrained development and CI machines.
- Preserve platform portability across Linux, Windows, and macOS.
- Prefer meaningful boundary coverage over a large test count for its own sake.

The number of tests is not a quality target. Coverage should follow product risk, state complexity, interoperability requirements, and the cost of a regression.

## Validation layers

### Frontend tests

Frontend tests use Vitest. Test files live next to their source with a `.test.ts` suffix. The default environment is Node. A test should request jsdom only when it needs browser or DOM behavior.

Pure domain logic is the preferred testing boundary. When behavior is entangled with Tauri IPC or component rendering, extract pure decisions into explicit functions when that improves the design. Component tests are appropriate when rendering, events, accessibility, loading behavior, or Svelte state integration are the behavior under test.

### Rust tests

Rust tests cover domain behavior, command boundaries, provider protocols, platform integration, persistence, migrations, transactions, and recovery. Tauri-free domain tests live with their workspace crates, while Tauri command and platform integration tests live in `ganbaru-tauri-app`. The native messaging host retains its own binary-local tests.

The desktop binary test harness and library doctest phase are disabled while they contain no tests. Re-enable the relevant phase if real binary tests or doctests are introduced.

### Static checks

Static checks are part of validation even though they are not runtime tests:

- Svelte Check validates Svelte components, runes, props, and TypeScript integration.
- TypeScript validates Node-side configuration and tool files.
- `cargo fmt` validates Rust formatting.
- Clippy rejects Rust warnings.
- Tailwind diagnostics enforce supported and canonical utilities.

Routine provider protocol checks validate committed compatibility artifacts without depending on globally installed tools, which keeps local and CI validation reproducible. The Codex manifest records the CLI version that produced the snapshot as provenance, not as an installation requirement. Provider maintenance can run `pnpm --dir apps/client run check:codex-protocol-installed` to regenerate the locally installed Codex app-server schemas in a temporary directory and compare them with the committed snapshot. `generate:codex-protocol` deliberately updates that snapshot after a protocol change is reviewed.

### Production bundle contracts

The production bundle contract performs a real Vite build and inspects emitted module metadata. It protects route and first-use loading boundaries, source-module ceilings, and forbidden eager imports. Unit tests do not replace this gate because only a production transform exposes the final chunk graph.

Desktop bundle contracts reject Android-only Back state and mobile-only Projects rendering modules. The Android contract uses a Node build wrapper that sets `TAURI_ENV_PLATFORM=android` before Vite and its configuration are imported. This keeps the build portable across shells and writes the isolated artifact to the ignored `.bundle-contracts/android/` directory, which Turbo caches as task output without replacing desktop `dist`.

The Android workflow first validates the committed generated project. It pins the Gradle, Android Gradle Plugin, Kotlin, SDK, NDK, Java, and application SDK contract; checks the production and debug application policies; keeps the generated base manifest limited to network access while feature-owned library manifests declare their narrow permissions; rejects Android TV launch metadata and broad FileProvider paths; and verifies that cloud backup and device transfer exclude every app storage domain. The bundle checker then reads emitted chunk metadata and follows each chunk's static `imports` transitively. It enforces committed source-module ceilings for the mobile shell, Calendar, Projects together with its mobile list, Notes, Quick notes, and Music. Every declared destination root and required mobile composition module must exist. The global contract also rejects desktop App and Back adapters, the desktop Pomodoro window coordinator and effects, reviewed desktop Projects surfaces, real Notes working Markdown, export, and project-history surfaces, benchmark and Chat code, desktop Music adapters and soundscapes, title-bar and Doomscrolling code, and desktop-heavy editor dependencies including CodeMirror, Pierre, Shiki, and React. Platform adapters must resolve at build time so runtime capability checks do not pull inactive implementations into either output.

Pull requests also run an independent `Android ARM64 build` job on Ubuntu 22.04. It installs the pinned JDK 21, API level 36 platform, Build Tools 35.0.0, NDK 30.0.15729638, and ARM64 Rust target, validates the committed native-project contract, then builds one ARM64 debug APK. Building only the physical reference-device ABI keeps pull-request memory and duration bounded. The production bundle contract still covers the frontend graph independently, while the protected release workflow is responsible for compiling every supported ABI into signed universal APK and AAB artifacts.

Run `pnpm --dir apps/client run check:android-bundle` for the direct Android-only package workflow. It rebuilds the isolated Android artifact before checking its metadata. The root `pnpm -w run bundle-contracts` workflow first uses the normal desktop build and then runs the same Android build and checker through Turbo.

Chat's route shell is resident so first navigation has no component-loading gate. Its Review renderer, syntax grammars, and module worker must remain outside both the application entry closure and the unopened panel closure. The CodeMirror core may preload after Chat metadata settles, but its static closure must not include language chunks. The bundle contract rejects eager Pierre, Shiki, React, ReactDOM, Review worker, and editor-language modules in those paths. Focused Review model fixtures verify heavy-workload layout ceilings, paging behavior, search coordinates, and hunk selection. Platform WebView rendering and the plain-text fallback remain part of the manual Tauri acceptance gate.

### Dependency audits

The full security gate runs both pnpm advisory checks and RustSec checks. Reviewed Rust audit exceptions live in `.cargo/audit.toml` and must be documented in `docs/data/security.md`.

## Root commands

Run root scripts with the workspace flag:

| Command | Purpose |
| --- | --- |
| `pnpm -w run check` | Rust formatting and Clippy, followed by frontend Svelte and TypeScript checks. |
| `pnpm -w run test` | Normal Rust tests, followed by normal frontend tests in sequential shards. Benchmark workloads and benchmark-harness contract tests are excluded. |
| `pnpm -w run test:benchmark-contracts` | Benchmark fixture and harness contract tests. This is separate from normal validation. |
| `pnpm -w run editor-check` | Tailwind editor-style diagnostics. |
| `pnpm -w run bundle-contracts` | Cached production build and bundle-contract validation. |
| `pnpm -w run audit` | pnpm and Rust dependency audits. |
| `pnpm -w run validate` | The complete normal code gate. |
| `pnpm -w run validate:full` | Dependency audits followed by the complete normal code gate. |

`validate` is the normal comprehensive gate. `validate:full` is required for dependency, lockfile, security, PR, and release-sensitive work.

## Validation execution model

The broad root scripts intentionally optimize for bounded peak resource use rather than minimum wall time. The complete normal gate runs in this order:

1. Rust formatting and Clippy with one Cargo build job.
2. Rust workspace tests with one Cargo build job and one runtime test thread.
3. Svelte Check with a 1,792 MiB Node old-space limit, followed by TypeScript checking.
4. Four sequential Vitest shards, each with one worker and benchmark-harness tests excluded.
5. Tailwind diagnostics through Turbo.
6. A production build and bundle-contract checks through Turbo.

Rust work runs first because compiler and linker peaks are the least predictable. Rust and frontend tools must not overlap. Do not start another Cargo, Vitest, Svelte Check, Turbo, or broad validation command while a root `check`, `test`, `validate`, or `validate:full` command is active.

The Svelte Check limit was rebaselined from 1,536 MiB on 2026-08-27 after the complete source graph consistently exhausted that heap before producing diagnostics. A direct 1,792 MiB run completed in 58.23 seconds with 1,919,420 KiB peak resident memory and no swap, while a 2,048 MiB comparison completed in 54.35 seconds with 2,014,436 KiB peak resident memory. The lower passing limit remains the repository default to preserve bounded validation on resource-constrained machines.

One Cargo build job prevents multiple large compiler or linker processes from competing for memory. One Rust test thread also serializes data-sensitive integration behavior. Sequential Vitest shards release transformed module graphs between groups while preserving the normal frontend suite.

Normal validation must prove production correctness without running performance, stress, timing, or dense benchmark-fixture workloads. Rust benchmark fixture contracts are ignored by the default Cargo test run, and frontend benchmark-harness tests are excluded from the normal Vitest shards. Run `pnpm -w run test:benchmark-contracts` explicitly when changing an implemented benchmark harness. Benchmark measurements remain manual release-build work and are never part of `validate`.

Do not increase broad concurrency, combine Rust and frontend stages, or remove Vitest sharding solely to make a warm run faster. Any topology change requires measurements and proof that coverage is unchanged.

## Development build resource policy

The Cargo development profile emits line tables by default. This preserves file and line information for backtraces while avoiding the substantially larger compiler and linker memory cost of full variable and type debug information. Incremental compilation, assertions, overflow checks, code generation units, and the standard platform linker retain their Cargo defaults.

The repository-owned Tauri CLI wrapper sets `CARGO_BUILD_JOBS=1` for `tauri dev` only when the caller has not already set the variable. On a Wayland desktop session, it also removes an inherited exact `GDK_BACKEND=x11` override before starting the development child so GTK can select native Wayland, matching installed builds and avoiding XWayland webview repaint failures after occlusion. Real X11 sessions, release builds, CI, direct Cargo commands, and explicit Cargo build-job overrides are unaffected. A contributor who deliberately needs XWayland on a Wayland session can preserve the override with `GANBARU_AI_DEV_PRESERVE_GDK_BACKEND=1`. Android commands independently require and select JDK 21 rather than allowing Tauri to fall back to a newer Android Studio runtime that the pinned Gradle stack cannot parse. `GANBARU_AI_ANDROID_JAVA_HOME` is the explicit override when automatic JDK 21 discovery is insufficient. Vite prepares only `/src/main.ts` before the Tauri readiness endpoint reveals the window. Other components are transformed on demand during navigation, and the Tailwind guard for Svelte style virtual modules remains active.

Full native debug information is exceptional because changing the debug mode invalidates the relevant Cargo artifacts. Use it only for a debugging session that needs local variables or full type information:

```sh
CARGO_PROFILE_DEV_DEBUG=full CARGO_BUILD_JOBS=1 pnpm --dir apps/client tauri dev
```

If a split backend still exceeds available memory with persistent swap configured, `CARGO_PROFILE_DEV_DEBUG=none` is an emergency per-command override. It is not the repository default because switching debug modes fragments the development artifact cache.

Persistent swap is a system-level safety margin, not a replacement for bounded build jobs. It does not reserve RAM for Cargo, and it can make severe memory pressure slower through disk paging. Keep normal development on the line-table profile and one-job Tauri default even when additional swap is available.

## Cache behavior and expected timing

Validation duration depends strongly on what changed and which caches are warm.

Cargo reuses compilation artifacts when Rust inputs and build configuration are unchanged. Rust tests still execute even when compilation is reused.

Turbo hashes task inputs, configuration, environment inputs, and command arguments. The four Vitest shards have distinct command arguments and therefore distinct cache entries. Tailwind diagnostics use declared client source inputs. Bundle contracts depend on the production build, whose `dist` output is cached.

A warm repeated validation may restore frontend checks, tests, diagnostics, and bundle results instead of executing them again. A changed source file, dependency, configuration file, environment input, or command invalidates the affected task. A cold or substantially changed run can take much longer and use more memory than a warm run.

Do not interpret a fast cached run as proof that a cold build has the same resource profile. When changing validation topology, measure at least one affected cache-miss run and one warm repeat.

Do not clear Cargo or Turbo caches as a routine memory fix. Cache removal increases work on the next run and changes the conditions being measured. Clear a cache only when investigating concrete corruption or invalidation behavior.

## Choosing the appropriate gate

Start with the narrowest command that can catch a plausible regression.

### Focused development

Use a focused Vitest file for isolated frontend behavior:

```sh
pnpm --dir apps/client exec vitest run src/path/to/file.test.ts --maxWorkers=1
```

Confirm that Vitest reports only the requested files. Stop and correct the command if the full suite starts unexpectedly.

Use a filtered Rust library test for the package that owns the behavior:

```sh
cargo test -p ganbaru-chat --lib -j 1 test_name -- --test-threads=1
cargo test -p ganbaru-notes --lib -j 1 test_name -- --test-threads=1
cargo test -p ganbaru-db --lib -j 1 test_name -- --test-threads=1
```

Use the Tauri composition library for command-adapter and platform integration behavior:

```sh
cargo test -p ganbaru-tauri-app --lib -j 1 test_name -- --test-threads=1
```

Use the relevant binary package only for binary-local behavior:

```sh
cargo test -p ganbaru-native-messaging --bin ganbaru-ai-native-messaging -j 1 test_name -- --test-threads=1
```

Check the composed desktop target after changing a core crate or Tauri adapter:

```sh
cargo check -p ganbaru-ai --bin ganbaru-ai -j 1
```

Useful focused static commands include:

```sh
pnpm --dir apps/client run check
cargo fmt --check
cargo clippy --workspace -j 1 -- -D warnings
pnpm -w run editor-check
```

### Small UI and documentation changes

Do not run the complete gate merely because a small UI or documentation edit is finished. Use `pnpm -w run check` or `pnpm -w run editor-check` when a change can affect Svelte compilation, TypeScript, Tailwind classes, or shared UI structure. Add focused tests when behavior changes.

Mechanically obvious documentation or copy-only changes do not require validation unless a relevant workflow requires it.

### Data-sensitive changes

Backend, persistence, SQLite, import, export, migration, project-membership, note-saving, and other data-loss-sensitive changes require focused tests during implementation and a broader gate before completion or commit when the risk warrants it.

Tests for persisted data must consider existing installs, stale rows, rollback, partial failure, unknown values, older exports, and cleanup of obsolete data.

### Pull requests, releases, and dependencies

Run `pnpm -w run validate` before opening a normal pull request and before risk-sensitive release work. Run `pnpm -w run validate:full` for dependency or lockfile changes, before pull requests and releases where dependency auditing is required, and when explicitly requested.

If a batch already passed the required gate, do not repeat it unless later changes materially affect behavior covered by that gate. Use focused checks for later isolated edits.

## Writing valuable tests

Test names should describe observable behavior, not implementation details. A useful test protects a decision, invariant, failure mode, or user-visible contract.

Cover relevant cases such as:

- Boundary values and realistic maximum sizes.
- Invalid, missing, stale, truncated, and unknown input.
- Transaction rollback and partial failure.
- Restart, crash recovery, retry, and idempotency.
- Ordering, pagination, deduplication, and stable identity.
- Permission, path, URL, protocol, and redaction boundaries.
- Import and export round trips and interoperability fixtures.
- State-machine transitions and forbidden transitions.
- Cross-platform path and process behavior.

Avoid shallow existence assertions and tests that merely repeat the type system. Before adding a test, inspect nearby tests and match their depth, fixture style, and naming conventions.

### Chat authorization matrix

Organizational Chat tests exercise authorization as an intersection, not as isolated positive flags. Focused coverage includes inert teammate creation, explicit channel-scoped access save, templates without special authority, profile ceilings and membership narrowing, independent read-history and participation capabilities, targetless conversation runs, history boundaries, folder tiers, optional default targets, explicit scratch selection, runtime precedence, provider-enforcement rejection for effective resource grants, optimistic access conflicts, and immediate reduction effects.

Cross-channel cases cover requester access, teammate source access, destination participation, destination-audience subset, retained references after audience changes, scheduling, assignment preflight, scoped history queries, result publication, and generic denial behavior. Runtime cases cover host-tool authentication and bounds, secondary-folder traversal and symlink rejection, existing worktree selection, continuation-scope digests, scratch quarantine, live-run interruption, and suppressed publication. Frontend cases cover English and Spanish catalog parity, grouped channel selection, reviewed channel-scoped creation, impact review, keyboard focus restoration, and coarse-pointer targets. The normative matrix is in [Chat access control](data/access-control.md).

Do not shrink realistic security, data-volume, or interoperability bounds only to shorten the suite. If a stress test is valuable but expensive, prefer bounded serialization, shared fixture design, or focused execution over deleting its coverage.

## Frontend test guidance

Keep the default Node environment when possible. jsdom adds startup and memory cost, so request it only for tests that exercise DOM behavior.

Mock the narrowest external boundary. Avoid loading a broad real component or API graph through a partial mock when the imported behavior is not part of the test. At the same time, retain representative integration tests for important component loading, first-use behavior, accessibility, and user-visible flows.

Tests that import component registries should distinguish between two guarantees:

- Generic loader behavior, such as caching, retries, and single-flight loading, belongs in focused loader tests.
- Production wiring and chunk placement belong in type checking and bundle contracts, with representative runtime smoke coverage where useful.

Do not globally disable Vitest isolation without a dedicated state-leak audit. Shared Svelte stores, module caches, fake timers, DOM globals, and mocks can otherwise make results order-dependent.

## Rust and SQLite test guidance

Use library tests for backend behavior unless the behavior belongs specifically to a binary target. Add `--lib` to focused Cargo commands so unrelated binary targets are not built.

Persistence tests should use isolated temporary databases. Tests that depend on the current schema should apply the real migration chain or a proven equivalent fixture. Migration-specific tests must always exercise the actual migrations and SQLx checksums. The `ganbaru-db` build script declares the complete migration directory as a Cargo input so downstream app and test binaries rebuild whenever a migration is added or edited. Preserve that invalidation boundary if the migration directory moves.

Database fixture optimizations must preserve:

- Isolation between tests.
- Foreign-key behavior.
- Migration checksums and ordering.
- Transaction and rollback semantics.
- Cleanup on every supported platform.

The workspace test profile intentionally limits debug information to keep test binaries and relinks smaller while retaining useful line-based stack traces. Restore fuller debug information only for a concrete debugging session that requires local-variable inspection.

Standard Cargo commands must remain portable. Do not require an external linker repository-wide. An optional linker optimization must retain the standard toolchain as a fallback and must be benchmarked on supported platforms.

## UI verification

Automated tests cannot replace manual inspection of the real Tauri application. Component tests can verify logic, events, accessibility, and deterministic layout decisions, but they do not prove that the complete native window looks or behaves correctly on every platform and window size.

During iterative UI work, use focused checks and tests. Do not launch a development server, Tauri app, or HTTP smoke check as a substitute for requested user inspection. Run the risk-appropriate completion gate after the batch is ready.

For responsive behavior, prefer pure helper tests when layout decisions depend on measured width, available space, anchors, or collision rules. Manual verification should include the app's recoverability floor and realistic desktop sizes.

For a development-build or watcher change, the user should perform the real Tauri acceptance run. On Linux, measure one cache-cold launch and one unchanged warm launch with:

```sh
/usr/bin/time -v pnpm --dir apps/client tauri dev --no-watch
```

After the app becomes usable, close it normally and record elapsed time, maximum RSS, final swap use, target-directory size, and any new kernel OOM entry. Normal watched development should then confirm that the main window stays hidden until frontend readiness, Calendar styling is complete on reveal, Projects, Notes, Chat, and Music load on first navigation, Svelte HMR works, a small Rust edit performs a bounded rebuild, top-level Rust workspace crate changes trigger Tauri rebuilds, and no Tailwind or pre-transform errors appear.

## Coverage reports

Generate frontend coverage with:

```sh
pnpm --dir apps/client run test:coverage
```

Coverage is a diagnostic, not a completion gate. Use it to find untested decisions and branches, not to justify low-value assertions or pursue a percentage without regard to risk. The repository does not currently define a root Rust coverage command.

## Changing the validation system

Changes to concurrency, sharding, task ordering, heap limits, cache inputs, test profiles, or target selection can silently change both coverage and resource use. Before adopting such a change:

1. Record the existing command topology and relevant task counts.
2. Run the proposed command with representative changed inputs.
3. Confirm that no test files or Cargo targets are omitted.
4. Compare wall time, peak memory, and swap behavior.
5. Repeat with warm caches to verify cache keys and invalidation.
6. Preserve standard toolchain behavior on every supported platform.
7. Update this document, `AGENTS.md`, and affected command descriptions together.

Prefer orchestration changes before removing coverage. Serialization, bounded workers, process-level sharding, target selection, and correct caching generally reduce peak pressure with less risk than deleting tests.

## Troubleshooting

When a broad gate fails, identify the first failing stage and reproduce it with the narrowest relevant command. Do not start additional broad commands while the original gate is active.

If Svelte Check reaches its configured heap limit, treat that as a controlled failure. Investigate frontend graph growth, generated inputs, and checker topology before raising the limit. A higher limit must be measured as part of the complete validation process.

If a focused Vitest command unexpectedly collects the full suite, stop it and correct the path or argument placement. If Turbo restores a result that appears stale, first inspect the task inputs and command hash. Do not immediately disable caching or delete every cache.

If a Cargo build becomes unexpectedly large, confirm that the command selects only the intended library or binary target and still uses one build job. Avoid running frontend tools concurrently while diagnosing it.
