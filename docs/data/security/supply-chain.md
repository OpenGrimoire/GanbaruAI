# Supply-chain security

Ganbaru AI's build and release systems can access source code, user-facing binaries, package repositories, and signing material. Dependency and CI choices therefore receive the same scrutiny as runtime code.

## Dependency installation

The repository uses pnpm for web work. Dependency lifecycle scripts are disabled by default. A package that requires a build script must be reviewed and explicitly allowed rather than weakening the global protection.

The workspace release-age gate rejects very recent npm versions so suspicious or compromised releases have time to be detected before entering the lockfile. A dependency update must not bypass that gate for convenience.

Python tooling accepts binary distributions rather than running arbitrary source setup during install. Rust dependencies are checked with cargo-audit through the root audit commands. Reviewed advisory exceptions are narrow and documented in [Dependency audits](dependency-audits.md).

Blocked scripts, binary-only failures, audit findings, yanked releases, and unusual maintainer changes are security signals. Do not disable protections without explicit maintainer approval after explaining the risk.

## Selecting dependencies

Prefer standard-library or platform APIs when they are sufficient. Add a dependency only when its benefit exceeds the maintenance and compromise surface.

Review:

- project ownership and governance;
- maintainer and publisher history;
- release cadence and recent account changes;
- downloads and evidence of real use;
- unresolved security advisories;
- transitive dependency breadth;
- install scripts and native code;
- runtime permissions and network behavior;
- license compatibility;
- whether a small local implementation is safer.

An official package is preferred when it has clear governance and equivalent capability. Popularity alone is not evidence of safety.

Remove unused packages and features. Do not keep broad optional dependencies for hypothetical future work.

## CI and release boundaries

Normal changes enter through pull requests. Protected branches, release tags, environments, and published releases are administrator-controlled because they affect signed installers and update feeds. Exact workflow is documented in [Contributing](../../../CONTRIBUTING.md), [Release](../../operations/release/README.md), and [Repository policy](../../operations/repository-policy.md).

Release jobs follow least privilege:

- checkout credentials are not persisted;
- third-party actions use reviewed immutable commit pins;
- pull request code does not run in a privileged pull_request_target context;
- build, updater signing, release publication, package-repository signing, and AUR publication use separate jobs and credentials;
- signing keys are present only in the protected job that needs them;
- artifacts are handed between jobs explicitly;
- untrusted builds cannot populate a trusted release cache;
- mutable tags and unverified host keys are rejected.

Do not combine privileged publication with dependency installation or arbitrary build execution merely to shorten a workflow.

## Cache poisoning

CI caches are executable supply-chain inputs when they contain package output, build scripts, binaries, or generated code. A less-trusted workflow must not write a cache later restored by a privileged workflow.

The 2026 TanStack npm compromise demonstrated a relevant chain involving privileged workflow behavior, cache poisoning, runner token access, and malicious publication. The repository treats workflow files, cache keys, release tags, environment approvals, artifact provenance, and signing material as one connected boundary. See the [TanStack postmortem](https://tanstack.com/blog/npm-supply-chain-compromise-postmortem).

## Code copied from external sources

Code from a web page, issue, answer, repository, generated response, or pasted terminal transcript is untrusted until reviewed. Before running or committing it:

1. Read it line by line.
2. Identify filesystem, network, environment, credential, and subprocess effects.
3. Check hidden Unicode, encoded payloads, command substitutions, install hooks, and destructive paths.
4. Explain any material risk.
5. Obtain explicit permission before executing externally sourced code when repository instructions require it.

Attribution and license obligations still apply. Reimplement a small idea when copying would introduce unclear licensing or a large unaudited block.

## Generated and downloaded artifacts

Generated schemas, platform projects, extension copies, package metadata, and release assets need a declared source and reproduction path. Do not hand-edit generated output unless the repository explicitly owns that file as an input.

Downloaded tools or binaries require checksum or signature verification from an authoritative source. Never commit credentials, signing material, or a local provider home into a generated artifact.

## Review triggers

Run the full dependency and code gate for dependency, lockfile, audit, release, or security-sensitive changes. Re-review this boundary when:

- a package adds a script or native component;
- ownership or publisher identity changes;
- an advisory affects a reachable path;
- an ignored advisory's assumptions change;
- CI permission, cache, environment, or artifact flow changes;
- a new package publication target or signing key is introduced.
