# Mobile architecture and experience

Ganbaru AI mobile is the same local-first product as the desktop app, presented through a smaller, platform-appropriate capability set. Android is the first delivery target. iOS follows the same shared contracts later, but Android work must not wait for an artificial lowest-common-denominator design.

The Android baseline is Android 10, API level 29. Release builds target and compile against API level 36. Tauri itself supports Android 7, API level 24, but supporting versions below API 29 would increase storage, permission, background-work, and testing branches without serving the current product objective. A higher target SDK does not raise the minimum install version. See the [Tauri Android distribution guide](https://v2.tauri.app/distribute/google-play/), the [Tauri 2.11.2 Android Gradle template](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/crates/tauri-cli/templates/mobile/android/app/build.gradle.kts), and the [Google Play target API requirements](https://support.google.com/googleplay/android-developer/answer/11926878?hl=en-GB_ALL).

## Current implementation status

The Android product is not yet delivered. The repository now has an implemented source foundation:

- Tauri v2, Svelte 5, and Vite already provide the shared native shell and frontend toolchain.
- Tauri-free Rust crates hold substantial database, Notes, working-folder, Chat contract, and domain behavior that can be reused where its authority is valid on mobile.
- Production uses `org.opengrimoire.ganbaruai`. Android debug builds append the official `.dev` application ID suffix, and the Android platform override also defines API level 29 as the minimum with one visible `main` window.
- The reviewed generated Android project builds a universal debug APK and AAB containing ARM64, ARMv7, x86, and x86_64 Rust libraries with API level 36 as the compile and target SDK. Its committed contract pins Android Gradle Plugin 8.11.0, Gradle 8.14.3, SDK Build Tools 35.0.0, NDK 30.0.15729638, Java 17 source and target compatibility, and Kotlin JVM target 17.
- `apps/client/src-tauri/src/lib.rs` provides the Tauri mobile entry. Separate desktop and mobile Rust composition roots register different commands, plugins, setup, and lifecycle behavior at compile time. Desktop-only crates and Tauri features are target-scoped out of Android.
- An Android capability grants the `main` WebView only Back-listener registration and removal plus scoped HTTPS URL opening. Cross-window Calendar, Quick notes, and theme synchronization resolves to a build-time no-op transport because Android owns one WebView. Android therefore receives no core event authority. It also receives no unencrypted HTTP, email, telephone, file revealing, or path opening authority. The desktop capability retains its separate desktop window, event, and updater authority.
- The mobile command surface currently exposes app-private SQLite behavior for Calendar, Projects, Notes, Pomodoro, Quick notes, Themes, Vault, and managed assets. It excludes local Chat execution, desktop notification and overlay windows, Doomscrolling enforcement, desktop music playback, soundscapes, benchmarks, working-folder Markdown, and native path pickers.
- The mobile default vault is created under Tauri's app data directory as `Ganbaru AI` or `Ganbaru AI Dev`, while desktop keeps its configured Documents default.
- A typed build-platform profile and immutable frontend capability registry distinguish desktop from Android and iOS before Svelte mounts. Mobile route parsing and navigation currently admit Calendar, Projects, and Notes while rejecting Chat and desktop detached-view behavior.
- Vite selects separate desktop and mobile entry graphs at build time. Android production assets use mobile adapters for Pomodoro effects, music assignments, working-folder settings, and desktop-only Notes panels. Desktop-owned Chat actions and music mention state enter shared feature views through optional typed integrations, so the mobile shell can omit them without importing desktop stores. The Android artifact therefore does not package the desktop App shell, Chat store, coding workspace, desktop music player, native media controls, Doomscrolling runtime, or benchmark surfaces.
- An initial one-WebView Svelte shell provides phone bottom navigation, a larger-window rail, lazy Calendar, Projects, Notes, Settings, and Quick notes surfaces, an in-app Pomodoro sheet, 48 dp primary targets, and localized English and Spanish copy. Calendar starts in day mode without precision pointer editing, Projects starts in list mode, and Notes omits desktop working-folder controls. Lazy Settings loading keeps the initial shell closure within its existing source-module budget even though the complete category registry is discoverable.
- Android selection for profile images, project icons, custom emoji, Notes page icons, and Notes covers uses the WebView document picker and copies bounded data into managed app assets. Byte size is bounded before full allocation. Browser and Rust validation then compare declared and sniffed PNG, JPEG, or WebP MIME, reject excessive image dimensions and pixel counts from bounded header metadata, and normalize blank browser MIME to the sniffed canonical value. This is metadata validation, not a complete image decode. Notes database CSV import uses the same picker only for files at or below the Rust 512 KiB boundary. These bounded inputs require no broad storage permission. Direct remote Notes image references are absent on every platform because the production content policy intentionally blocks remote image loading. Managed remote-image ingestion can restore that UX later without widening the policy.
- A serialized Tauri Back-listener controller intercepts only consumable frontend state and unregisters at the Calendar root, allowing Tauri to resume native Android Back behavior on the next press. The shared stack covers global Pomodoro, Settings, and Quick notes sheets, confirmation dialogs, Quick notes editor layers, Calendar event and picker layers, Project task details, and Notes parent and editor surfaces. Root yielding and sheet dismissal pass on the physical API level 29 reference phone. Predictive Back and any remaining transient-layer gaps still require newer-device validation.
- VisualViewport tracking provides an input-method fallback and stable CSS geometry for keyboard-aware sheets and full-screen editors. A native bridge publishes the union of Android system-bar and display-cutout insets as four bounded CSS pixel values, while CSS safe-area values remain the fallback. Portrait and landscape layouts pass on the physical API level 29 reference phone with three-button navigation. Gesture navigation, cutout, input-method, and split-screen cases remain release-matrix work. Focused pure tests cover platform selection, navigation, viewport calculations, Back-listener state, modal focus, and lifecycle flushing.
- Mobile startup reconciles Pomodoro state before Calendar loads. It resumes one valid unexpired running or paused phase from persisted timestamps, or atomically closes multiple, malformed, event-expired, or phase-expired state with a typed reason. The visible timer remains foreground-only until the native notification boundary adapter exists.
- Visibility and page lifecycle hooks perform best-effort flushes for configuration, Notes writes, and mounted Quick notes editors. Canonical writes remain responsible for correctness because Android can still remove a process without a final callback.
- The host Tauri check and focused mobile command-surface contract tests pass.
- The Android Rust target compiles with the NDK toolchain. Desktop music playback and the remaining desktop-only vault, picker, benchmark, networking, and recovery paths are excluded from the Android compile graph instead of being linked as dormant code.
- The base Android manifest requests only network access. Android backup and device transfer exclude the unencrypted app-private vault and all other application storage until an intentional encrypted backup design exists. The capture FileProvider exposes only the app-scoped `Pictures/` directory.
- The universal debug APK passes 16 KB ZIP alignment verification. Its ARM64 and x86_64 native libraries use 16 KB ELF load alignment, which is the Android 15 or newer 64-bit compatibility requirement. Physical 16 KB runtime testing remains a release gate.

The following remain roadmap work and must not be presented as available:

- Release signing, Android CI artifacts, a minified production release AAB, and the remaining emulator and release-device acceptance matrix.
- Native Storage Access Framework streaming transfers, notification and alarm boundaries, deep links, contextual runtime permissions, encrypted user-controlled backup, and Media3 playback.
- Android backup UX, restoration for remaining noncanonical drafts and navigation state, physical process-death recovery, root-yielding and predictive Back validation, and remaining touch or narrow-layout adaptations inside shared feature components.
- Android device and emulator validation, Play policy work, and a production Android release.
- Mobile sync, remote Chat communication, sleep alarm, and Android Doomscrolling enforcement.

## Product boundary

Mobile should make planning, capturing, focusing, reviewing, and responding convenient away from a desk. It should not imitate desktop controls that Android cannot support reliably.

The first useful offline Android release includes the app-private vault, Calendar, Projects, Notes, Quick notes, Pomodoro, Settings, localization, themes, import and export, and notification-based timer boundaries. Music follows through a native Android media adapter. Sync, communication-only Chat, diary, sleep alarm, and policy-sensitive Doomscrolling capabilities can land independently after their own foundations are ready.

Desktop-only features remain absent from Android:

- Local coding-agent processes, provider executables, terminals, PTYs, Git worktrees, source-control commands, browser preview servers, and external working-folder execution.
- Tray, global shortcuts, single-instance handling, self-updater, native browser messaging, edge panel, detached windows, and always-on-top or fullscreen enforcement windows.
- Work-environment process control, desktop app closing, desktop foreground monitoring, and desktop browser-extension orchestration.
- Desktop recursive folder assumptions and reveal-in-file-manager actions.

Unavailable features are removed from primary navigation. Settings are different because they are also the product's capability map: every durable category remains discoverable except keyboard shortcuts, which have no mobile meaning. A category with no truthful Android control presents a concise implementation status instead of disabled desktop controls. Contextual explanations are also shown where a shared record refers to a desktop action.

### Settings architecture and section matrix

Settings use one shared category registry, one controller, and shared section components where the underlying behavior is genuinely portable. Vite resolves the section renderer, detail loader, profile picker, and WebView zoom adapter at build time. Android therefore does not package desktop Chat, Doomscrolling, music, updater, native path-picker, or window-event graphs. The mobile presentation uses a category list followed by a full-screen section. Android Back returns from a section to the category list before closing Settings. Desktop keeps its modal navigation, lazy detail panels, keyboard handling, and draft protection.

| Section | Android behavior |
| --- | --- |
| Appearance | Shared themes, font, text scale, language, time format, calendar zoom, and dimming controls. WebView zoom, keyboard hints, desktop quick-theme shortcuts, native theme file actions, and the floating desktop theme editor are omitted. Bounded manual theme JSON import remains available. |
| Profile | Shared display and full names. Profile image selection uses the bounded document-input and managed-asset boundary, with browser preflight and Rust revalidation before an atomic write. |
| Calendars | Shared calendar records, event counts, and deletion. ICS import and export present an Android document-picker status until the streaming content-URI adapter exists. |
| Projects | The global category directs users to the project-local settings panel, where project workflow and field configuration belong on both platforms. |
| Notes | Shared default-open and version-history retention settings. File transfers and native notification delivery show separate adapter statuses until their Android boundaries exist. |
| Chat | Visible capability status only. Android does not launch local coding-agent processes. Future synchronized communication belongs behind its own remote authorization boundary. |
| Focus | Foreground Pomodoro and persisted recovery remain available elsewhere in the mobile shell. Settings show the native idle and background-boundary status instead of exposing desktop-effect controls. |
| Music | Visible Media3 and Android media-control adapter status only. |
| Doomscrolling | Visible Android-specific enforcement status only. Desktop website and process controls are not reused because their semantics and permissions do not apply. |
| Data | Explains the app-private canonical vault, uninstall risk, and planned document-picker backup boundary. Desktop folder switching and reveal actions are omitted. |
| Updates | Explains Android store or signed-package distribution. The desktop self-updater is not packaged. |
| About | Shared application identity, version, licensing, and project information. |
| Shortcuts | Omitted because hardware keyboard shortcuts are not part of the mobile product contract. |

This matrix is a capability decision, not a second settings product. New portable settings should normally enter the shared section. A platform-specific adapter belongs behind a typed build-time boundary. A control must not be shown merely because its preference can be persisted if the platform cannot apply the behavior.

## Platform and configuration baseline

Android uses a platform override such as `tauri.android.conf.json`. Tauri merges this over the base configuration using JSON Merge Patch semantics, and arrays replace rather than append. Platform arrays must therefore be complete and reviewed explicitly. See [Tauri platform configuration](https://v2.tauri.app/develop/configuration-files/).

The implemented Android override defines:

- `minSdkVersion` 29.
- One immediately visible main window backed by one Activity.
- The production identifier `org.opengrimoire.ganbaruai` from the base configuration.
- `debugApplicationIdSuffix` set to `.dev`, which gives Android debug installs the distinct application ID `org.opengrimoire.ganbaruai.dev`.

Tauri synchronizes the Android debug suffix into the generated Gradle debug build. Android development retains the base identifier for the generated Kotlin package and does not merge the desktop `tauri.dev.conf.json` identifier override. See the Tauri 2.11.2 [Android development flow](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/crates/tauri-cli/src/mobile/android/dev.rs) and [`sync_debug_application_id_suffix`](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/crates/tauri-cli/src/mobile/android/mod.rs).

Mobile-appropriate bundle icons and metadata remain part of the generated-project and release work. The stable production identifier must not change after signing and distribution. Tauri replaces hyphens with underscores in mobile identifiers used by deep links, so the canonical identifier avoids hyphens rather than depending on that transformation. See [Tauri deep linking](https://v2.tauri.app/plugin/deep-linking/).

The generated Android source pins the build versions needed for reproducibility. The current matrix is Android Gradle Plugin 8.11.0, Gradle 8.14.3, SDK Build Tools 35.0.0, NDK 30.0.15729638, API level 36 for compile and target, and API level 29 for minimum installation. Version changes are deliberate compatibility updates, not automatic Android Studio upgrades.

Android renders the frontend in the device's updatable Chromium WebView. Tauri does not bundle a browser engine on Android. Android production builds explicitly target Chromium 111 instead of inheriting a moving Vite-major default. The support contract is Android 10 or newer with System WebView 111 or newer. Android 10 can receive newer WebView providers independently through system component updates. The physical API level 29 reference phone currently uses Google System WebView 150.0.7871.183. Release acceptance still verifies the selected provider instead of assuming the development-device version. See [Tauri WebView versions](https://v2.tauri.app/reference/webview-versions/) and [Vite build targets](https://v8.vite.dev/config/build-options).

## Composition and authority

Desktop and mobile use separate compile-time composition roots. Shared domain code stays in Tauri-free crates. Each root registers only the commands, plugins, managed state, setup hooks, and dependencies supported by that platform.

Runtime user-agent checks are not a security or architecture boundary. Platform-specific dependencies, modules, commands, and setup calls are gated at compile time. This prevents Android from linking desktop process stacks and avoids paying their binary-size, startup, memory, and audit costs.

Platform adapters expose small domain-oriented interfaces. Examples include notification scheduling, document import and export, media playback, deep-link delivery, app lifecycle, and permission status. Shared Calendar or Pomodoro logic depends on those interfaces rather than an Android or desktop implementation.

The frontend receives one typed platform-capabilities snapshot during bootstrap. It uses explicit capabilities to select navigation and affordances. It does not infer authority from viewport width, operating-system strings, or whether an IPC call happened to fail.

### Capability matrix

| Capability | Shared domain and UI | Android adapter | Desktop adapter |
| --- | --- | --- | --- |
| Calendar, Projects, Notes, Quick notes | Shared contracts, validation, and adaptive screens | App-private SQLite and managed assets | Active desktop vault SQLite and files |
| Pomodoro | Shared state machine and persisted deadlines | Implemented cold reconciliation, with alarm or notification boundaries still pending | Desktop windows, notifications, and enforcement |
| Import and export | Shared typed transfer plans | Storage Access Framework content URIs | Native paths and folder dialogs |
| Music | Shared queue, playlist, and transport contracts | Media3, ExoPlayer, MediaSessionService | Rodio, Symphonia, WebView media, and desktop controls |
| Deep links | Shared validated navigation intents | Cold-start and running-intent delivery | Desktop protocol and single-instance delivery where supported |
| Updates | Shared release information UI only when available | Play or signed package distribution | Tauri updater or package-manager instructions |
| Chat communication | Shared authorized records and read DTOs | Future synchronized communication and review client | Communication plus native coding workspace |
| Coding execution | None on mobile | Unsupported | Native provider processes, PTYs, working folders, and Git |
| Doomscrolling | Shared user rules and explanations where semantics match | Future permission-aware Android subsystem | Browser extension and desktop enforcement |
| Window integration | Adaptive in-app surfaces | One Activity and one WebView | Multi-window, tray, detached windows, and overlays |

Tauri capabilities are split by platform and selected explicitly in each platform configuration. The implemented desktop capability lists Linux, macOS, and Windows with the existing desktop window, updater, event, and scoped opener permissions. The implemented Android capability applies only to `main` and grants Back-listener registration and removal plus explicit HTTPS URL opening for user-authored links. It has no core event permissions because mobile cross-window synchronization is a build-time no-op transport. Neither capability grants unencrypted HTTP, email, telephone, file revealing, or arbitrary path opening through the opener plugin. Mobile does not receive create-window, always-on-top, updater, process, shell execution, global-shortcut, tray, dialog, or broad filesystem permissions. Android production content security policy permits only app-owned scripts and disables frames, so desktop YouTube origins do not enter the mobile WebView authority. A later iOS capability is defined separately instead of widening the Android file. See [Tauri capability configuration](https://v2.tauri.app/security/capabilities/) and [Tauri opener permissions](https://v2.tauri.app/plugin/opener/).

## Canonical vault and Android storage

The Android canonical vault lives under app-private internal storage. The implemented mobile default resolves to `<app_data_dir>/Ganbaru AI` in production or `<app_data_dir>/Ganbaru AI Dev` in development and is selected automatically at mobile bootstrap. Its logical structure remains a Ganbaru AI vault, including `vault.json`, `config.json`, `ganbaru-ai.sqlite`, managed documents, and assets, but its physical root is assigned by Android and is not a public `Documents` path.

App-private storage is the only suitable home for the canonical SQLite database because it provides a real private filesystem path, requires no storage permission, and supports the existing SQLx and bounded filesystem model. Android's scoped-storage model applies to modern targets, while broad legacy write permission no longer provides unrestricted shared-storage access. See the [Android data storage guide](https://developer.android.com/training/data-storage).

The Storage Access Framework is an interoperability boundary, not the live vault:

- Import, export, backup, restore, and attachment selection use system document UI.
- Android document results are opaque content URIs, not paths.
- Imported canonical documents are validated and copied into managed storage before direct filesystem use.
- Exports stream to the selected URI and never construct a path from it.
- A persisted URI grant is retained only when an external object must remain referenced.
- Revoked, moved, deleted, or unavailable URI content becomes an explicit recoverable state.
- The app does not request all-files access or depend on `MANAGE_EXTERNAL_STORAGE`.

The implemented WebView file input is appropriate only for bounded image assets that are copied immediately into the vault and bounded text imports such as the 512 KiB Notes database CSV path. General Notes attachments, larger imports, exports, backups, and large media must use a native streaming content-URI adapter. They must not base64-encode a large file through the JavaScript bridge, which would multiply memory use on the Android 10 reference device.

Tauri's dialog plugin returns content URIs on Android and does not support Android folder selection. Rust code using `PathBuf` or `std::fs` cannot consume those URIs directly. See the [Tauri dialog documentation](https://v2.tauri.app/plugin/dialog/) and the [Android Storage Access Framework guide](https://developer.android.com/training/data-storage/shared/documents-files).

Large local music files are a deliberate exception to copying. A native Android picker may retain access to user-selected audio content URIs, and Media3 can read them through Android's content resolver. Playlist records store a platform source reference and cached display metadata, never a fabricated filesystem path. The desktop recursive folder scanner is not exposed on Android. A future Android library scan must use a reviewed MediaStore or native document-tree adapter with the narrowest possible permission.

Android Auto Backup includes much app-private data by default. The generated manifest therefore disables backup and supplies both legacy backup rules and Android 12 or newer data-extraction rules. Every app storage domain is excluded from cloud backup and device-to-device transfer. This prevents the unencrypted vault, credentials, caches, and device-local state from entering an implicit platform backup. A future encrypted, user-controlled backup is an explicit product flow and can narrow these rules only after its threat model is reviewed. See [Android Auto Backup](https://developer.android.com/identity/data/autobackup) and [Android backup security guidance](https://developer.android.com/privacy-and-security/risks/backup-best-practices).

Uninstalling the app can remove app-private data. First-use and Settings copy must explain this plainly and make export, backup, and later encrypted sync easy to discover.

## Mobile shell and adaptive UX

Android uses one Activity and one WebView. Additional Tauri windows map to additional Android Activities, not desktop-style child windows. The baseline Android 10 phone therefore uses in-app navigation, pages, dialogs, and sheets. Activity Embedding for side-by-side Activities requires Android 12L, API level 32, and is not the base architecture. See [Tauri mobile multiwindow](https://v2.tauri.app/learn/mobile-multiwindow/).

Layout responds to the current app window, not a device-name or portrait-phone assumption. Android window-size guidance defines compact width below 600 dp, medium width from 600 dp, and expanded width from 840 dp. The same window can change class through rotation, split screen, a fold, or desktop windowing. See [Android window size classes](https://developer.android.com/develop/ui/views/layout/use-window-size-classes) and [adaptive layout guidance](https://developer.android.com/design/ui/mobile/guides/layout-and-content/adapt-layout).

- Compact width uses bottom navigation for the small set of primary destinations. Secondary tools live in a More destination or contextual action sheet.
- Medium width uses a navigation rail and may show list and detail together when both remain usable.
- Expanded width uses a rail or drawer and stable list-detail or supporting panes. It does not stretch compact cards across the viewport.
- Exact primary labels can evolve with the product information architecture, but Calendar, Projects, and Notes remain directly reachable. Active Pomodoro context stays available without consuming a permanent desktop-style title bar.
- Global session, timer, and media state uses a compact activity shelf above navigation. It shows one clear primary state and expands into its full control surface. Competing stacked mini-players and timer bars are avoided.
- Desktop popovers become anchored menus only when they fit. Complex popovers and modal panels become bottom sheets, side sheets, or full pages.
- Hover is never required. Drag interactions have visible handles and an accessible button or menu alternative.
- Interactive targets are at least 48 dp and expose semantic names, states, and roles. See [Android accessibility guidance](https://developer.android.com/design/ui/mobile/guides/foundations/accessibility).
- Motion respects reduced-motion preferences. Long lists and dense timelines are virtualized, and controls remain reachable at increased font and display scaling.

Feature screens adapt their information density rather than merely shrinking desktop CSS:

- Calendar defaults to a useful phone agenda or focused day presentation. Week and month views remain available, but event editing uses a page or sheet instead of a desktop floating dialog. Touch creation must not depend on precise drag placement.
- Projects use list-detail navigation. Boards support horizontal lanes, explicit move actions, and touch handles. A list view remains available for dense task work.
- Notes show tree, editor, database controls, and history as separate compact destinations. Medium and expanded widths can restore multiple panes.
- Quick notes use a single-column compact list unless measured width safely supports more. Editing occupies a stable sheet or page above the keyboard.
- Settings group platform permissions with plain status, purpose, and a route to Android Settings when the system owns the decision.

## Insets, keyboard, and system back

Targeting API level 35 or newer enables enforced edge-to-edge presentation on Android 15 and newer. Every screen must consume status-bar, display-cutout, navigation-bar, gesture, and IME insets. Important content and touch targets never sit behind system UI. Android 10 has different sibling inset-dispatch behavior, so API 29 receives explicit regression testing. See [Android edge-to-edge guidance](https://developer.android.com/develop/ui/views/layout/edge-to-edge).

The completed Android shell will expose current native insets to the WebView through stable CSS custom properties. CSS `env()` values may be used when reliable, but the product does not assume they are sufficient across the supported Android WebView range. Insets update on rotation, resizing, fold changes, keyboard animation, and system-bar changes.

The generated Activity will use `adjustResize`. Editors, search fields, Chat composer surfaces, bottom sheets, and primary actions reflow above the input method instead of being obscured. Focus is restored deliberately when sheets close. See [Android input-method visibility](https://developer.android.com/develop/ui/views/touch-and-input/keyboard-input/visibility).

System Back follows one deterministic priority:

1. Let the input method dismiss when it owns Back.
2. Close the top transient menu, dialog, sheet, or full-screen editor layer.
3. Return from detail to its parent list.
4. Pop the current in-app destination stack.
5. At the root destination, return control to Android so the Activity can go to the background or finish normally.

Tauri's `onBackButtonPress` callback has an important limitation: the callback returns `void`, and the existence of a listener consumes Tauri's native fallback for that press. The `{ canGoBack }` payload is informational, so returning a Boolean cannot delegate the event. When no listener exists, the repository's locked Tauri 2.11.2 core first calls `WebView.goBack()` if browser history is available. Otherwise it delegates to the Activity's Back dispatcher. See the [`onBackButtonPress` API](https://v2.tauri.app/reference/javascript/api/namespaceapp/#onbackbuttonpress), its [TypeScript source](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/packages/api/src/app.ts), and the [Android app plugin implementation](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/crates/tauri/mobile/android/src/main/java/app/tauri/AppPlugin.kt).

The current mobile shell registers one serialized Tauri Back listener while it has an integrated parent layer, a hash-backed nested route, or a non-root destination to consume. It unregisters the listener when the Calendar root is reached. The press that returns to root is consumed. The next press uses Tauri's normal Android fallback and finishes or backgrounds the Activity according to Android navigation, without force-quitting the Rust process. Registration changes are serialized so asynchronous setup and cleanup cannot leave duplicate listeners. Complete implementation must extend the shared stack to every nested picker, popover, dialog, sheet, editor, and detail surface, then confirm root yielding and browser-history behavior on a physical device.

Routine Back navigation does not lose user input. Editors persist drafts or canonical changes continuously enough to survive process death. A confirmation is reserved for a genuinely destructive or invalid transition. The native callback and frontend navigation stack must also prepare for Android predictive Back rather than permanently disabling it. See [Android predictive Back](https://developer.android.com/guide/navigation/custom-back/predictive-back-gesture).

## Lifecycle and process death

Android owns the process lifetime. It may kill a background process without calling a reliable final cleanup hook. Configuration changes may recreate the Activity. See the [Android Activity lifecycle](https://developer.android.com/guide/components/activities/activity-lifecycle) and [process lifecycle](https://developer.android.com/guide/components/activities/process-lifecycle).

- Correctness never depends on `RunEvent::ExitRequested`, `onDestroy`, a force-quit path, or an in-memory timer.
- Canonical writes are transactional when the user makes them.
- Drafts are persisted with bounded debounce and flushed on ordinary visibility changes, but the visibility callback is an optimization rather than the only save path.
- SQLite, vault validation, and recovery are idempotent on cold start.
- UI restoration stores small navigation identifiers and draft references, not large document bodies or bitmaps.
- Resume rechecks permission state, selected URI availability, active alarms, wall-clock changes, current locale and timezone, database revision, and sync state.
- Native event listeners are registered once per process and disposed without accumulating across Activity or WebView recreation.
- Restart and force-quit controls are absent. Recovery uses normal navigation, an explicit data reset flow, or Android's app settings where appropriate.

## Notifications and Pomodoro

Mobile Pomodoro is deadline-driven. SQLite stores the authoritative run, phase, segment, and boundary deadline. The visible countdown is derived from the current clock. It does not advance canonical state through a background JavaScript interval.

The implemented cold-start path resumes one valid unexpired phase and preserves an open manual pause without counting time away. It closes unsafe or expired state rather than inventing phases that no native boundary delivery observed. The completed Android adapter will schedule one native alarm or notification event at a phase boundary. Once that delivery is persisted, foreground resume or cold start can reconcile it and advance through elapsed boundaries deterministically. Reboot, timezone, wall-clock, and daylight-saving changes are explicit test cases.

The Android notification path must:

- Create stable notification channels before posting.
- Ask for `POST_NOTIFICATIONS` only in context on API level 33 and newer. Android 10 does not require that runtime prompt. See [Android notification permission](https://developer.android.com/develop/ui/compose/notifications/notification-permission).
- Provide useful behavior when notifications are denied. The in-app timer remains correct, while Settings clearly reports that background alerts are unavailable.
- Route notification actions through the same validated navigation-intent parser used by deep links.
- Avoid a foreground service solely to keep a countdown alive.
- Never force the Activity to the foreground when a timer ends. Android 10 restricts background Activity starts, so the alert remains a notification until the user chooses it. See [Android 10 privacy changes](https://developer.android.com/about/versions/10/privacy/changes).

Exact alarms are not a default permission request. Android recommends inexact alarms for most cases and reserves exact alarm access for genuinely time-critical user-facing behavior. Pomodoro first uses the least privileged scheduling that meets measured reliability. A separate sleep-alarm design may justify exact-alarm access with contextual education and graceful fallback. See [Android alarm guidance](https://developer.android.com/develop/background-work/services/alarms).

The official Tauri notification plugin supports Android scheduling and actions, but its capabilities and exact-alarm fallback must be validated against the required product behavior before adoption. See the [Tauri notification plugin](https://v2.tauri.app/plugin/notification/) and its [Android notification manager](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/android/src/main/java/TauriNotificationManager.kt).

Deferrable maintenance and future sync use WorkManager. A Pomodoro countdown does not. See [Android persistent background work](https://developer.android.com/develop/background-work/background-tasks/persistent).

## Music

The mobile music backend is a native Android adapter built with Media3 and ExoPlayer. Playback ownership lives in a `MediaSessionService`, allowing audio to continue when the Activity or WebView stops and providing lock-screen, headset, Bluetooth, and notification controls. See [Android Media3 background playback](https://developer.android.com/media/media3/session/background-playback).

Shared playlist and transport contracts remain provider-neutral. Desktop continues using its existing Rodio, Symphonia, and WebView adapters. Android does not compile or initialize the desktop audio backend, desktop loopback media host, or desktop media-control integration.

The Android service handles audio focus, noisy-output transitions, metadata, queue changes, resume position, and a media notification. It starts from an explicit user playback action because newer Android versions restrict starting foreground services from the background. See [Android foreground-service start restrictions](https://developer.android.com/develop/background-work/services/fgs/restrictions-bg-start).

The media service declares the normal `FOREGROUND_SERVICE` and `FOREGROUND_SERVICE_MEDIA_PLAYBACK` manifest permissions only when the Media3 adapter is included. No generic long-running foreground service is used for timers, sync polling, or keeping the WebView resident.

User-selected local audio can remain outside the vault through persisted content URI grants. Loss of access is visible and repairable. YouTube remains the official IFrame Player path while the WebView is visible. Ganbaru AI does not extract or proxy YouTube streams to create unsupported background audio.

Tauri Android plugin commands run on the main thread by default. Media preparation, content reads, parsing, and database work move to coroutines or Rust async work so the main thread cannot be blocked into an application-not-responding failure. See [Tauri mobile plugin development](https://v2.tauri.app/develop/plugins/develop-mobile/).

## Responsible Android Doomscrolling

Android Doomscrolling is a separate native subsystem, not a port of desktop process closing. Its product copy distinguishes observation, reminders, browser filtering, and enforceable restrictions so the app never promises control it does not have.

Delivery is staged:

1. Usage awareness may use `UsageStatsManager` after the user deliberately grants Usage Access in Android Settings. See [UsageStatsManager](https://developer.android.com/reference/android/app/usage/UsageStatsManager).
2. App selection avoids broad installed-package enumeration. Targeted package visibility and user-selected launchable apps are preferred. `QUERY_ALL_PACKAGES` is not requested. See [Android package visibility](https://developer.android.com/training/package-visibility) and the [Google Play package visibility policy](https://support.google.com/googleplay/android-developer/answer/10158779?hl=en).
3. Domain filtering may use an explicitly disclosed local VPN only after battery, privacy, conflict, emergency bypass, and Play policy design. See the [Android VPN service guide](https://developer.android.com/develop/connectivity/vpn).
4. Accessibility Service, device-owner controls, and unrestricted overlays are not used to simulate desktop app killing. Accessibility Service is reserved for assisting users with disabilities and is subject to strict disclosure and Play policy. See [Android AccessibilityService](https://developer.android.com/reference/android/accessibilityservice/AccessibilityService) and the [Google Play accessibility policy](https://support.google.com/googleplay/android-developer/answer/17190352?hl=en&rd=2).

The feature remains useful when every special permission is denied. Rules and limits can still inform planning, while enforcement status states exactly what Android can and cannot do. All usage and blocking records remain local unless the user separately enables encrypted sync.

## Chat and coordination

Android does not run local coding-agent harnesses. Mobile Chat is a future communication and review client over synchronized canonical channels, DMs, task discussions, proposals, attention items, and deliverables. It can approve, reject, comment, or schedule authorized work, but execution occurs on an explicitly authorized desktop or future remote runner.

The mobile client never:

- Spawns Codex, Claude, Cursor, OpenCode, a shell, a PTY, Git, or a sidecar.
- Claims access to desktop external working folders.
- Exposes terminal, source-control, browser-preview, checkpoint, or local file-explorer UI.
- Broadens a teammate's channel, project, folder, provider, or data permissions.

When a channel contains desktop execution events, mobile renders safe read-only projections and review artifacts. Actions that require a desktop show a concise handoff state instead of a disabled desktop workspace replica. Offline writes queue only after permission-aware sync defines conflict, revocation, and receipt semantics.

## Deep links and external entry

Android deep links support cold starts and already-running delivery. The app reads the current link at bootstrap and subscribes for later links. Every link is parsed into a bounded typed navigation intent, validated, and authorized before state changes.

A custom scheme is sufficient for development. Production HTTPS App Links require a stable package identifier, release-signing certificate fingerprint, owned HTTPS domain, and matching `assetlinks.json`. See [Tauri deep linking](https://v2.tauri.app/plugin/deep-linking/) and [Android App Link verification](https://developer.android.com/training/app-links/verify-applinks).

Notification actions, widgets, shortcuts, and App Links converge on the same navigation-intent contract. No external entry point directly invokes an arbitrary Tauri command.

## Security and privacy

Mobile follows least privilege:

- Permissions are requested at the moment a user enables the capability, with purpose and fallback explained first.
- Denial, permanent denial, restricted device policy, and later revocation are normal states.
- No broad storage, all-files access, unrestricted package visibility, overlay, Accessibility Service, background location, or unnecessary foreground service is part of the base app.
- File and content URI input is treated as untrusted. Imports enforce type, count, byte, nesting, and parser limits before canonical writes.
- Deep links and notification payloads are untrusted and bounded.
- Native credentials remain in the operating-system credential facility. They are never stored in the vault, frontend storage, logs, or Android backup.
- WebView navigation, remote origins, content security policy, and bridge command permissions stay narrow. External links open through the system browser unless a reviewed embedded flow requires otherwise.
- Native plugins and Android libraries are limited to maintained, auditable components with narrow authority. The mobile app includes no advertising, tracking, or opaque analytics SDK.
- Play Data safety declarations and special-permission disclosures must match actual code and bundled SDK behavior.

The Tauri updater, single-instance, tray, and global-shortcut plugins are desktop-only and are absent from the Android composition. Android app updates use Google Play or an explicitly documented signed-APK channel. See the [Tauri updater](https://v2.tauri.app/plugin/updater/), [single-instance plugin](https://v2.tauri.app/plugin/single-instance/), [tray API](https://docs.rs/tauri/latest/tauri/tray/index.html), and [global-shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/).

### Permission plan

| Permission or system access | When it is justified | Base behavior without it |
| --- | --- | --- |
| `POST_NOTIFICATIONS` on API level 33 and newer | Requested after the user enables timer, calendar, or alarm notifications | In-app deadlines remain correct; Settings reports that background alerts are unavailable |
| `SCHEDULE_EXACT_ALARM` special access | Considered only for a separately enabled sleep alarm whose user promise requires exact delivery | Use an explicitly described inexact alarm or mark exact wake-up delivery unavailable |
| `RECEIVE_BOOT_COMPLETED` and `WAKE_LOCK` scheduling support | Added only with native scheduled-boundary restoration | Reconcile when the app next opens; never imply reboot-safe delivery without the receiver |
| `FOREGROUND_SERVICE` and `FOREGROUND_SERVICE_MEDIA_PLAYBACK` | Declared with the Media3 playback service | Playback stops when the foreground media capability is unavailable |
| Usage Access through `PACKAGE_USAGE_STATS` | User enables Android usage awareness from its settings flow | Limits can still be configured, but device-app usage is not observed or enforced |
| VPN service consent | Future local domain filtering after separate privacy and policy review | Browser traffic is not filtered |
| Persisted Storage Access Framework grants | User selects an external document or media source | Canonical app data remains available; that external item is unavailable until repaired |

The base app does not request `MANAGE_EXTERNAL_STORAGE`, legacy broad shared-storage access, `QUERY_ALL_PACKAGES`, `SYSTEM_ALERT_WINDOW`, Accessibility Service binding, device-owner status, contacts, microphone, camera, or location. A future feature that genuinely needs another permission requires an updated spec, localized disclosure, denial UX, tests, and store-policy review before the permission enters the manifest.

## Performance and low-resource behavior

The mobile release is built as a focused application, not the desktop binary with hidden controls.

- Link and initialize only mobile capabilities.
- Select the mobile frontend entry graph and platform adapters at build time so hidden desktop features are absent from the packaged assets.
- Keep one Activity and one WebView.
- Render the shell first, then lazily open noncritical services and hydrate destination data.
- Keep database migrations and vault repair off the native main thread and show bounded progress when they block first use.
- Query Calendar, Notes, Projects, and Chat by visible windows or pages rather than loading whole histories.
- Virtualize large lists and release decoded artwork, editor models, preview buffers, and inactive destination caches under pressure.
- Suspend visual tickers, filesystem polling, and nonessential observers while backgrounded.
- Use native alarms, WorkManager, and MediaSessionService only for work that must outlive the Activity.
- Measure release builds. Android recommends startup tracing and deferring unnecessary initialization to improve time to initial and full display. See [Android launch-time vitals](https://developer.android.com/topic/performance/vitals/launch-time) and [startup optimization](https://developer.android.com/topic/performance/appstartup/analysis-optimization).

The app ships native Rust libraries, so 16 KB page-size compatibility is a release gate. The current AGP 8.11.0 and NDK 30 build passes 16 KB APK ZIP alignment, and the packaged ARM64 and x86_64 Rust libraries use 16 KB ELF load alignment. A release still verifies the AAB alignment request, every packaged 64-bit shared object, and runtime behavior on a 16 KB emulator. See [Android 16 KB page-size support](https://developer.android.com/guide/practices/page-sizes).

Edge-to-edge rendering remains enabled on every supported Android version because Android 15 enforces it for apps targeting API level 35 or newer. The Activity reads the union of `WindowInsetsCompat.Type.systemBars()` and `displayCutout()`, converts device pixels to CSS pixels, and exposes only those four bounded values through a read-only WebView bridge. The frontend applies them to its safe-area variables and refreshes them after native inset changes. This protects interactive chrome from three-button navigation, gesture navigation, cutouts, rotation, and window changes without hardcoded device dimensions. The bridge must never gain filesystem, application-command, permission, or input authority. See [Android edge-to-edge guidance](https://developer.android.com/develop/ui/views/layout/edge-to-edge).

Ganbaru AI themes, rather than Android or WebView force-dark processing, own all frontend colors. Both Android theme variants disable native Force Dark, and the Activity disallows algorithmic WebView darkening when supported. A presentation-only bridge accepts one boolean and updates only the light or dark icon appearance of the status and navigation bars, keeping their contrast aligned with the explicit Ganbaru AI theme. It exposes no data, permission, input, navigation, or application-command authority. See [Android WebView dark-theme guidance](https://developer.android.com/develop/ui/views/layout/webapps/dark-theme).

Performance results belong in `docs/PERFORMANCE.md` only after the Android benchmark methodology and device metadata are stable. The physical Android 10 phone is the low-resource reference, not merely a debugging convenience.

## Build and release

The required local toolchain is Android Studio, Android SDK Platform 36, Platform Tools, SDK Build Tools 35.0.0, command-line tools, side-by-side NDK 30.0.15729638, JDK 21 to run Gradle and Tauri, and the selected Rust Android targets. Java and Kotlin application bytecode remains explicitly targeted to version 17. This separates the supported build-runtime LTS from the Android application language level and avoids machine-dependent bytecode. Configure `JAVA_HOME`, `ANDROID_HOME`, and `NDK_HOME` as described by the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

The supported Rust target set is `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, and `x86_64-linux-android`. Local development installs the connected device or emulator target first. Release configuration states its ABI support explicitly and tests every included ABI. It must not become ARM64-only by accident.

Normal commands from the repository are:

```sh
pnpm --dir apps/client run tauri android init
pnpm --dir apps/client run tauri android dev
pnpm --dir apps/client run tauri android build --apk
pnpm --dir apps/client run tauri android build --aab
```

Run `pnpm --dir apps/client run generate:icons` after changing the source logo or deliberately regenerating either native project. Tauri stores mobile launcher resources directly in the generated Android and Apple projects, while the desktop `bundle.icon` list alone does not replace those generated mobile resources. The Android project contract verifies the expected Ganbaru AI launcher fingerprint plus normal and round manifest resources so the default Tauri launcher cannot return unnoticed. See the [Tauri app icon guide](https://v2.tauri.app/develop/icons/).

The APK is for direct device and emulator testing. Google Play distribution uses an Android App Bundle. Development may build only the connected device ABI for speed. Release ABI policy is explicit and validated against supported devices rather than assumed from API level.

Release signing uses a protected keystore and private Gradle signing properties. Signing secrets never enter source control. The package identifier and release key are permanent product identity once distributed. See [Tauri Android signing](https://v2.tauri.app/distribute/sign/android/).

Once created, the generated Android project and intentional native changes are reviewed as source. Regeneration is deliberate and followed by a diff. Release builds validate minification, manifest merging, capabilities, permissions, content providers, deep links, backup rules, native library alignment, and installation from the built AAB, not only a debug APK.

## Delivery order

Android work is divided into independently useful milestones:

1. **Build foundation:** Android configuration and project, legal identifier, separate mobile Rust composition, mobile capabilities, one visible Activity, app-private vault bootstrap, CI compilation, signing design, and an installable shell.
2. **Offline core:** adaptive navigation, Calendar, Projects, Notes, Quick notes, Settings, localization, themes, and lifecycle-safe drafts over the local SQLite vault.
3. **Focus and interoperability:** persisted Pomodoro deadlines, notification boundaries, deep links, document import and export, backup UX, permission states, and release-device validation.
4. **Native media:** Media3 service, system controls, local content URI sources, and mobile player UX.
5. **Connected experience:** encrypted sync, communication-only Chat, task review, attention, and cross-device handoff after Phase 9 authorization and conflict semantics exist.
6. **Policy-sensitive features:** diary and sleep alarm, usage awareness, and carefully reviewed Doomscrolling capabilities. Each remains useful with special permissions denied.
7. **iOS alignment:** implement equivalent shared contracts through iOS-native adapters without weakening Android architecture or pretending platform capabilities are identical.

The current source foundation completes the local build and physical-shell portions of milestone 1 and parts of milestone 2. The native Android project builds all four supported ABIs into a universal debug APK and AAB. An ARM64 debug APK installs and passes initial Android 10 acceptance for cold start, background process restart, portrait and landscape insets, compact and wide navigation, system-bar theme contrast, root Back, Calendar, Projects, Notes, Quick notes, Pomodoro, and Settings. Signing, CI, production AAB validation, emulators, and the complete release matrix remain milestone 1 work. Milestones 1 through 4 do not depend on sync and can proceed before Phase 9. Milestone 5 depends on Phase 9. Sleep alarm depends on the diary domain and native alarm design. Policy-sensitive features require their own store-policy review before release.

## Test matrix

Android is not release-ready until the following matrix passes on minified release artifacts where applicable.

| Target | Required coverage |
| --- | --- |
| Physical Android 10, API 29, ARM64 phone | Primary low-resource reference, current System WebView, cold and warm start, real storage provider, notifications, audio, battery/background behavior, process death, three-button and gesture navigation where available |
| API 29 compact emulator | Minimum SDK installation, rotation, Activity recreation, insets, IME, Back, denied permissions, offline startup, and deterministic core flows |
| API 33 emulator | Runtime notification permission grant, denial, permanent denial, and Settings recovery |
| API 35 or 36 compact emulator | Enforced edge-to-edge, predictive Back behavior, current target-SDK changes, foreground-service rules, and current WebView |
| API 35 or newer 16 KB ARM64 emulator | Native library load, SQLite, media dependencies, AAB alignment, startup, and representative feature flows |
| Medium and expanded resizable emulator | Navigation rail, list-detail layouts, rotation, split screen, keyboard, pointer, and no stretched phone UI |

Every release candidate also covers:

- Fresh install, upgrade, uninstall warning, reset, export, backup, restore, and corrupt or partial import.
- Local and cloud-backed document providers, URI grant revocation, missing media, zero-byte files, oversized input, and interrupted transfer.
- Process kill from the background, low-memory recreation, reboot, clock change, timezone change, daylight-saving transition, and stale scheduled boundaries.
- Notification channels and actions, notification denial, exact-alarm unavailability, deep-link cold start, repeated intent delivery, malformed links, and unauthorized targets.
- Media interruption, audio focus loss, headphones unplugged, Bluetooth route change, lock-screen controls, service restart, and killed Activity during playback.
- TalkBack, switch or keyboard navigation, 48 dp targets, increased font and display scale, high contrast, reduced motion, and Spanish plus English layouts.
- Play pre-launch reports, permission and Data safety declarations, release signing, AAB installation through bundle tooling, ABI splits, minification, and 16 KB native alignment.

Automated tests should keep shared domain logic outside Tauri where possible, add focused tests for capability selection and adaptive state, and add Android instrumentation only for lifecycle, Activity, permission, content URI, alarm, notification, deep-link, and service behavior that cannot be proven in Rust or TypeScript.
