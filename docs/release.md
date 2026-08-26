# Release process

Ganbaru AI releases are published through GitHub Releases. The release workflow builds Linux x64 packages and Windows x64 installers, signs updater assets, builds signed Android universal APK and AAB artifacts in a protected job, writes the desktop `latest.json` updater feed, and uploads everything to a draft release for inspection before publishing. After the draft is published, the release workflow updates the GitHub Pages package repository for `.deb` and `.rpm` users and publishes the `ganbaru-ai-bin` AUR package for Arch-based users.

The workflow is intentionally conservative:

- GitHub Actions permissions default to read-only.
- Only the publish and package repository jobs get `contents: write`.
- The Tauri updater private key is used only in the signing job.
- The Android keystore and passwords are used only in the signing job.
- The package repository GPG private key is used only in the package repository job.
- The AUR SSH private key is used only in the AUR publish job.
- The signing, publishing, package repository, and AUR jobs use the protected `release` GitHub Environment.
- Dependency caches are disabled in release jobs.
- Third-party release upload actions are not used.
- GitHub Actions are pinned to full commit SHAs.
- Runner labels are fixed to `ubuntu-22.04` and `windows-2022`, not `latest` aliases.

## Release targets

- Linux x64: `.deb`, `.rpm`, and `.AppImage` bundles from the Tauri Linux build.
- Windows x64: Tauri Windows installers for Windows 10 and Windows 11 users.
- Android 10 or newer: a signed universal APK for direct installation and a signed universal AAB for later Google Play submission. Both contain ARM64, ARMv7, x86, and x86_64 libraries.
- macOS is intentionally not part of the first release workflow.

The workflow runs on Ubuntu 22.04 for Linux artifacts to keep glibc compatibility broader than newer Ubuntu runners. Windows artifacts are built on GitHub's hosted Windows runner, but the installers target normal Windows desktop installs, not the runner OS specifically.

## GitHub setup

Create a GitHub Environment named `release` before running the workflow. Configure required reviewers for that environment. Store signing secrets in this environment, not as broad repository secrets:

- Environment secret `TAURI_SIGNING_PRIVATE_KEY`: the private key file content.
- Environment secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the key password, if one was set.
- Environment secret `ANDROID_KEYSTORE_BASE64`: the complete Android release keystore encoded as canonical base64 without surrounding text.
- Environment secret `ANDROID_KEYSTORE_PASSWORD`: the Android keystore password.
- Environment secret `ANDROID_KEY_ALIAS`: the release key alias inside the keystore.
- Environment secret `ANDROID_KEY_PASSWORD`: the password for that key. This is normally the same as the keystore password for a PKCS12 keystore.
- Environment secret `GANBARU_AI_PACKAGE_REPO_GPG_PRIVATE_KEY`: the ASCII-armored private key for signing package repository metadata.
- Environment secret `GANBARU_AI_PACKAGE_REPO_GPG_PASSPHRASE`: the package repository signing key passphrase.
- Environment secret `AUR_SSH_PRIVATE_KEY`: the dedicated private SSH key that can push to `ssh://aur@aur.archlinux.org/ganbaru-ai-bin.git`.

Store these repository variables:

- Repository variable `TAURI_UPDATER_PUBLIC_KEY`: the public key from the signer output.
- Repository variable `GANBARU_AI_PACKAGE_REPO_PUBLIC_KEY`: the ASCII-armored public key for the package repository.

Enable GitHub Pages from the `gh-pages` branch root before publishing package-manager updates. The release workflow creates or updates that branch when a GitHub Release is published, but Pages must be enabled in repository settings.

Also configure repository protections:

- Protect `main` so only organization admins can merge release pull requests from `dev`. Require pull requests, merge queue, required checks, conversation resolution, signed commits if enabled for the organization, and no force-push or deletion. Do not require release PR branches to be up to date with `main`; merge queue validates the merge result without changing `dev`.
- Protect `dev` so normal changes reach it only through pull requests from topic branches. Require pull requests, signed commits, required checks, conversation resolution, and no direct pushes, force-pushes, or deletion.
- Protect `app-v*` tags so only organization admins can create, update, or delete release tags.
- Require review for changes to `.github/workflows/release.yml` and release scripts.
- In GitHub Actions settings, allow only selected actions and prefer SHA-pinned actions where the repository settings support that policy.
- Keep the `check` workflow on pull requests and merge queue checks, not branch push checks. Direct branch updates are blocked by rulesets, so branch push checks only add a redundant post-merge run.

The exact intended rulesets, including disabled fields and rationale, live in `docs/rulesets.md`.

## Signing setup

Tauri updater artifacts must be signed. The signature check cannot be disabled, so a release build needs an updater key pair before the first public release.

Generate the key pair locally:

```sh
pnpm -C apps/client tauri signer generate -w ~/.tauri/ganbaru-ai.key
```

Keep a backup of the private key in a password manager or another durable secret store. Losing it means existing users cannot receive future updates through the updater and must install a new release manually. Changing the public key is a key rotation and has the same user impact.

This signing is for Tauri updater verification only. It is not Windows Authenticode signing, so Windows may still warn that the installer is from an unknown publisher until a separate code-signing certificate is added.

The package repository also needs a stable OpenPGP signing key before `.deb` and `.rpm` releases are useful through system package managers. Create or approve that key as a maintainer action, export its ASCII-armored public key into `GANBARU_AI_PACKAGE_REPO_PUBLIC_KEY`, and store the ASCII-armored private key plus passphrase in the protected `release` environment secrets listed above. Losing this key means package-manager users must replace their configured repository key before future apt, dnf, or zypper updates can verify metadata.

The AUR package publish job needs a dedicated SSH key, separate from personal SSH keys. Add the public key to the maintainer's AUR account and store the private key in `AUR_SSH_PRIVATE_KEY` in the protected `release` environment. This key should only have AUR access and should be rotated if it is exposed.

### Android signing

Android requires every installable APK and AAB to be signed. The certificate that signs the first direct APK becomes part of the permanent identity of `org.opengrimoire.ganbaruai`. Android only installs later builds as updates when their package identifier and signing certificate match.

Generate the durable release keystore once on a trusted maintainer machine. The command is interactive because its passwords and certificate identity must not enter shell history:

```sh
mkdir -p ~/.config/ganbaru-ai
keytool -genkeypair -v -keystore ~/.config/ganbaru-ai/android-release.jks -storetype PKCS12 -keyalg RSA -keysize 4096 -validity 10000 -alias ganbaru-ai
```

Back up the keystore and passwords in two durable, access-controlled locations before installing or distributing the first production APK. Do not commit the keystore or `gen/android/keystore.properties`. Losing a self-managed app-signing key prevents existing direct APK installations from receiving normal updates. If Google Play distribution is added later, provide this app-signing key to Play App Signing when cross-channel certificate compatibility is required, then use a separate upload key for routine Play submissions.

For a local signed build, create the ignored `apps/client/src-tauri/gen/android/keystore.properties` file:

```properties
storeFile=/absolute/path/to/android-release.jks
storePassword=replace-with-keystore-password
keyAlias=ganbaru-ai
keyPassword=replace-with-key-password
```

Then build the production APK and AAB:

```sh
pnpm --dir apps/client tauri android build --ci
```

Release Gradle tasks fail if this file is absent. Android debug builds do not read release credentials and continue to use `org.opengrimoire.ganbaruai.dev` with the `Ganbaru AI Dev` launcher label.

For GitHub Actions, encode the keystore without line wrapping and place the result in `ANDROID_KEYSTORE_BASE64`:

```sh
base64 -w 0 ~/.config/ganbaru-ai/android-release.jks
```

The protected signing job decodes the keystore into runner-temporary storage, writes private Gradle properties with restrictive file permissions, builds minified universal artifacts, verifies the APK with `apksigner`, verifies the AAB with `jarsigner`, and removes the temporary signing files before uploading the release artifacts. The signing key must never be exposed to pull-request jobs or development APK artifacts.

## Branch flow

Ganbaru AI uses `dev` as the integration branch and `main` as the release source branch.

- Normal work starts from `dev` on a short-lived topic branch.
- Topic branches open pull requests into `dev`.
- Release preparation changes, such as version bumps and release documentation updates, go through normal pull requests into `dev`.
- Release promotion opens one pull request from `dev` into `main`, then adds it to the `main` merge queue after review and green pull request checks.
- A merge to `main` does not publish by itself. The release workflow is intentionally tag-based so signed desktop and Android assets can be inspected before publishing.

This keeps frequent development PRs visible for review and generated release notes while preserving an explicit release gate for installers, updater metadata, checksums, and signing.

Only organization admins may merge release PRs into `main`, create or update `app-v*` tags, approve the protected release environment, or publish GitHub Releases. Today that means the organization owner unless release authority is explicitly delegated.

Do not update `dev` with `main` only to satisfy a release PR. That would add release merge commits to the integration branch and conflict with the linear-history policy on `dev`. The `main` merge queue is the stale-base protection for release PRs.

Pull requests that target `main` and do not come from `dev` should be retargeted to `dev` or closed. Do not add a `pull_request_target` workflow for branch routing unless a separate security review explicitly accepts the added privileged automation surface.

This policy exists because CI and release infrastructure are part of the supply chain. The May 2026 TanStack npm compromise chained a `pull_request_target` trust-boundary issue, GitHub Actions cache poisoning, and token access into malicious package releases. Ganbaru AI keeps release authority, signing jobs, release tags, and updater metadata behind explicit maintainer controls for the same class of risk. See TanStack's postmortem: <https://tanstack.com/blog/npm-supply-chain-compromise-postmortem>.

## Release notes

Draft releases use GitHub's generated release notes to compile merged pull requests since the previous release. The workflow prepends `docs/release-notes-template.md`, then appends generated notes using `.github/release.yml` for categories and exclusions.

Use concise PR titles because they become release-note entries. Labels control categorization. Add `skip-changelog` or `ignore-for-release` when a PR should not appear in release notes.

## Publishing

1. Update the app version in `apps/client/package.json`, `apps/client/src-tauri/Cargo.toml`, and `apps/client/src-tauri/tauri.conf.json` through a normal pull request into `dev`.
2. Run `pnpm -w run validate:full`.
3. Open a release pull request from `dev` into `main`.
4. After review and green pull request checks, add the release PR to the `main` merge queue.
5. Wait for the merge queue checks to pass and for the queue to merge the PR.
6. Create a tag like `app-v0.1.0` on the release commit.
7. Push the tag to GitHub.
8. Approve the `release` environment when GitHub asks.
9. Wait for the `release` workflow to finish.
10. Download and smoke test the draft release assets. Install the signed APK on the Android 10 reference phone, confirm it appears as `Ganbaru AI`, confirm `Ganbaru AI Dev` can coexist, and verify offline restart after the development server is stopped.
11. Inspect generated release notes, `latest.json`, and `SHA256SUMS`.
12. Publish the draft GitHub Release.
13. Wait for the `publish package repo` and `publish AUR package` jobs triggered by the published release event.
14. Verify that GitHub Pages serves the updated apt and RPM metadata.
15. Verify that AUR shows `ganbaru-ai-bin` for the released version.

The workflow also supports manual dispatch from the default branch. On manual dispatch, the workflow creates or updates `app-v<version>` for the current app version at the selected commit. Prefer a pushed tag when publishing a public release because it is easier to audit.

## Update checks

Release builds inject a generated `src-tauri/tauri.release.conf.json` at CI time. The file embeds the public updater key and points the app at:

```text
https://github.com/<owner>/<repo>/releases/latest/download/latest.json
```

The generated file is ignored by git and must not be committed.

The desktop build job creates unsigned installers with the public updater configuration embedded. The signing job signs updater assets (`.AppImage`, `.exe`, and `.msi`) with the Tauri signer and separately builds Android with the protected Android keystore. The publish job writes `latest.json` from desktop updater signatures and points each supported desktop platform to the tag-specific release asset URL. Android does not consume this desktop updater feed. Direct Android installations update through a newly downloaded APK signed with the same certificate, while future store installations follow that store's update channel.

Release builds check the configured GitHub Releases feed at most once per day by default to notify users when a new version is available. Users can turn this off in Settings, Updates. The automatic check never downloads or installs anything.

When an update is available, the main window shows a small prompt without a Later button. AppImage and Windows installs show Update and restart, Release notes, and dismiss actions. Update and restart downloads the signed artifact, verifies it through Tauri's updater, installs it, and restarts the app. Linux package-manager installs (`.deb`, `.rpm`, and AUR packages) do not use the Tauri self-updater because the Linux updater artifact is the AppImage. Those installs show Copy command, Release notes, and dismiss actions. Copy command copies the exact package-manager command, such as `sudo apt update && sudo apt install ganbaru-ai`, `sudo dnf upgrade ganbaru-ai`, `sudo zypper refresh && sudo zypper update ganbaru-ai`, `yay -Syu ganbaru-ai-bin`, or `paru -Syu ganbaru-ai-bin`. The app does not execute privileged package-manager commands. Release notes opens the matching GitHub Release page in the default browser, using a Tauri opener permission scoped to Ganbaru AI release pages. Users can also run the same check manually from Settings, Updates.

## Linux package repositories and AUR

Published releases provide package-manager update paths through the project GitHub Pages site:

- APT repository: `https://opengrimoire.github.io/ganbaru-ai/packages/apt`
- RPM repository: `https://opengrimoire.github.io/ganbaru-ai/packages/rpm`

Release `.deb` packages install a Deb822 source file at `/etc/apt/sources.list.d/ganbaru-ai.sources` and a repository key at `/usr/share/keyrings/ganbaru-ai-package-repo.asc`. Release RPM packages install `/etc/yum.repos.d/ganbaru-ai.repo`, `/etc/zypp/repos.d/ganbaru-ai.repo` when zypper is present, and `/etc/pki/rpm-gpg/RPM-GPG-KEY-ganbaru-ai`. These files are created by package install scripts, not by the running app. Removal scripts delete them only on Debian purge or full RPM removal, not on normal upgrades.

The tag workflow builds and signs the draft release assets first. The package repository is updated only when the GitHub Release is published, through the `publish package repo` job. That job downloads the published `.deb` and `.rpm`, copies them into the Pages repository, regenerates apt and RPM metadata, signs apt `Release` plus RPM `repomd.xml` metadata with the package repository GPG key, and pushes the updated `packages/` directory to `gh-pages`.

AUR remains separate from the Pages package repository. The published `ganbaru-ai-bin` package repackages the released `.deb` artifact and removes the apt repository files that are only useful on Debian-like systems. The release workflow updates `ganbaru-ai-bin` through the `publish AUR package` job after a GitHub Release is published. Ganbaru AI suggests `yay -Syu ganbaru-ai-bin` by default for Arch-like systems and switches to `paru -Syu ganbaru-ai-bin` when `paru` is detected without `yay`.
