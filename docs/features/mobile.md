# Mobile architecture and experience

Ganbaru AI mobile is the same local-first product as the desktop app, presented through a smaller, platform-appropriate capability set. Android is the first delivery target. iOS follows the same shared contracts later, but Android work must not wait for an artificial lowest-common-denominator design.

The Android baseline is Android 10, API level 29. Release builds target and compile against API level 36. Tauri itself supports Android 7, API level 24, but supporting versions below API 29 would increase storage, permission, background-work, and testing branches without serving the current product objective. A higher target SDK does not raise the minimum install version. See the [Tauri Android distribution guide](https://v2.tauri.app/distribute/google-play/), the [Tauri 2.11.2 Android Gradle template](https://github.com/tauri-apps/tauri/blob/tauri-v2.11.2/crates/tauri-cli/templates/mobile/android/app/build.gradle.kts), and the [Google Play target API requirements](https://support.google.com/googleplay/android-developer/answer/11926878?hl=en-GB_ALL).

## Current implementation status

The Android product is not yet delivered. The repository now has an implemented source foundation:

- Tauri v2, Svelte 5, and Vite already provide the shared native shell and frontend toolchain.
- Tauri-free Rust crates hold substantial database, Notes, working-folder, Chat contract, and domain behavior that can be reused where its authority is valid on mobile.
- Production uses `org.opengrimoire.ganbaruai`. Android debug builds append the official `.dev` application ID suffix and override the launcher label with `Ganbaru AI Dev`. The Android platform override also defines API level 29 as the minimum with one visible `main` window.
- The reviewed generated Android project builds a universal debug APK and AAB containing ARM64, ARMv7, x86, and x86_64 Rust libraries with API level 36 as the compile and target SDK. Its committed contract pins Android Gradle Plugin 8.11.0, Gradle 8.14.3, SDK Build Tools 35.0.0, NDK 30.0.15729638, Java 17 source and target compatibility, and Kotlin JVM target 17.
- `apps/client/src-tauri/src/lib.rs` provides the Tauri mobile entry. Separate desktop and mobile Rust composition roots register different commands, plugins, setup, and lifecycle behavior at compile time. Desktop-only crates and Tauri features are target-scoped out of Android.
- An Android capability grants the `main` WebView Back-listener registration and removal, scoped HTTPS URL opening, the official notification commands required to check and request runtime permission, and the narrow app-owned commands required to reconcile Calendar deliveries and consume a validated notification tap. Calendar channel creation and scheduling remain behind the app-owned Android notification adapter. Cross-window Calendar, Quick notes, and theme synchronization resolves to a build-time no-op transport because Android owns one WebView. Android therefore receives no general core event authority. It also receives no unencrypted HTTP, email, telephone, file revealing, or path opening authority. The desktop capability retains its separate desktop window, event, and updater authority.
- The mobile command surface exposes app-private SQLite behavior for Calendar, Projects, Notes, provider-free Chat communication, Pomodoro, Quick notes, Themes, Vault, managed assets, and the shared Music library. Android Music uses a narrow native Media3 and document-tree adapter. Chat reuses the shared channel, message, reply-thread, search, draft, and scheduling contracts while excluding provider processes and workspace execution. The mobile composition also excludes desktop notification and overlay windows, Doomscrolling enforcement, desktop audio and tray integrations, soundscapes, benchmarks, working-folder Markdown, and generic native path pickers.
- First use reuses the desktop setup content and visual hierarchy instead of presenting Android implementation details as onboarding. The default action creates the canonical vault under Tauri's app data directory as `Ganbaru AI` or `Ganbaru AI Dev`. Import opens Android's native directory-tree picker, streams a bounded existing Ganbaru AI folder into a private staging directory, validates it with the shared vault boundary, and atomically activates the imported copy.
- After vault setup, Android shows one compact Focus access screen using the same full-height visual hierarchy. It shows only notification and exact-alarm access that the current Android version reports missing, plus Background autostart when a verified manufacturer settings adapter is available and battery-saver review. Each row keeps its name and description on the left with one Review action on the right. Supported access refreshes from Android after returning from Settings. Manufacturer autostart controls expose no supported status API, so the screen records that the user returned from reviewing the control without displaying a false enabled state. Continue unlocks after five seconds and persists completion locally. Focus settings retain the same direct recovery actions.
- A typed build-platform profile and immutable frontend capability registry distinguish desktop from Android and iOS before Svelte mounts. Mobile route parsing and navigation admit Calendar, Projects, Notes, and Chat while rejecting desktop detached-view and local-execution behavior.
- Vite selects separate desktop and mobile entry graphs at build time. Android production assets use mobile adapters for Pomodoro effects, Music paths and external controls, music assignments, working-folder settings, and desktop-only Notes panels. Chat loads the shared responsive workspace and provider-free store only when its destination opens. Its local-execution capability keeps provider recovery, working-folder reconciliation, editor preload, terminals, Git, the local review panel, preview, and workspace observers out of the Android runtime path. The shared feed can still render durable review activity already present in a conversation. The Android artifact does not package the desktop App shell, Rodio player, desktop tray or media-control bridge, Doomscrolling runtime, or benchmark surfaces.
- The one-WebView Svelte shell provides icon-only phone navigation in the global top bar, a larger-window rail, lazy Calendar, Projects, Notes, Chat, Settings, Quick notes, and Music surfaces, an in-app Pomodoro sheet, touch-sized primary targets, and localized English and Spanish accessible names. The compact bar does not repeat the current destination as text. It uses the same exact height, background, and bottom divider as the internal workspace headers, with a restrained bottom indicator for the selected destination instead of a full-cell highlight. Every visible destination and utility receives one equal-width cell. Pomodoro, Quick notes, Music, and Settings use small circular pressed and open states. Their order matches desktop: Pomodoro, Quick notes, Music, then Settings. The Pomodoro menu centers beneath its ring when space permits and shifts only enough to remain inside the visible safe viewport. Quick notes and the Music player use the same floating, safe-area-clamped presentation, while the Music playlist builder expands to full screen. Projects, Notes, and Chat retain one stable, non-scrolling identity row with the same side inset and bounded, text-first breadcrumb behavior. Identity icons are omitted from those phone breadcrumbs so names retain the available width, while separators, disclosure chevrons, and action icons remain. Tapping a breadcrumb segment opens a full-height touch selector at that level instead of reproducing the adjacent desktop navigation panels. One hierarchy level is visible at a time, and both the visible back action and Android Back unwind toward Group before dismissing the selector. Notes keeps that row while a full-page note is open, with the page action bar below it. Chat reuses the desktop responsive channel workspace rather than maintaining a mobile copy, and its identity row retains the group, project, and channel segments. On phones, a separate divider-free, Chat-background-colored control bar immediately below it owns the channel-navigation toggle while the conversation is visible. Opening channel navigation replaces both that control row and the conversation with a full-width channel surface; Search occupies the top row with its close control at the right. The channel surface never overlays or dims the conversation, uses touch-sized controls, and returns to the conversation after navigation. Opening it or navigating to a channel does not summon the input method until the user explicitly selects Search or the composer. Music reuses the shared player, playlists, library review, source management, metadata, artwork, YouTube, and import or export UI through these adaptive player and builder presentations. Calendar starts in day mode, supports direct touch navigation and editing, and exposes day, work-cycle, week, month, zoom, calendar visibility, and calendar settings through compact toolbar controls. Projects starts in list mode and reuses the desktop List component, including its computed minimum grid width, native horizontal overflow, synchronized sticky rows, sections, grouping, columns, inline editing, selection, and bulk actions. The entire List viewport supports native two-axis touch panning, including diagonal gestures and momentum, without requiring the user to drag its scrollbar. Coarse-pointer devices keep selection and task-detail affordances visible instead of relying on hover. Dashboard, Kanban, Calendar, and Gantt remain shared as well. The phone toolbar keeps the project breadcrumb and three equal icon controls: a view selector, a customization menu for filters, sorting, columns, and grouping, and direct project settings. Complex controls open as touch-sized overlays. Notes omits desktop working-folder controls. Lazy feature loading keeps inactive Project views, Chat, Music, and Settings outside the initial shell closure.
- Android selection for profile images, project icons, custom emoji, Notes page icons, and Notes covers uses the WebView document picker and copies bounded data into managed app assets. Byte size is bounded before full allocation. Browser and Rust validation then compare declared and sniffed PNG, JPEG, or WebP MIME, reject excessive image dimensions and pixel counts from bounded header metadata, and normalize blank browser MIME to the sniffed canonical value. This is metadata validation, not a complete image decode. Notes database CSV import uses the same picker only for files at or below the Rust 512 KiB boundary. These bounded inputs require no broad storage permission. Direct remote Notes image references are absent on every platform because the production content policy intentionally blocks remote image loading. Managed remote-image ingestion can restore that UX later without widening the policy.
- A serialized Tauri Back-listener controller intercepts only consumable frontend state and unregisters at the Calendar root, allowing Tauri to resume native Android Back behavior on the next press. The shared stack covers global Pomodoro, Settings, and Quick notes sheets, confirmation dialogs, Quick notes editor layers, Calendar event and picker layers, Project task details, and Notes parent and editor surfaces. Root yielding and sheet dismissal pass on the physical API level 29 reference phone. Predictive Back and any remaining transient-layer gaps still require newer-device validation.
- VisualViewport tracking provides an input-method fallback and stable CSS geometry for keyboard-aware sheets and full-screen editors. A native bridge publishes the union of Android system-bar and display-cutout insets as four bounded CSS pixel values, while CSS safe-area values remain the fallback. Portrait and landscape layouts pass on the physical API level 29 reference phone with three-button navigation. Gesture navigation, cutout, input-method, and split-screen cases remain release-matrix work. Focused pure tests cover platform selection, navigation, viewport calculations, Back-listener state, modal focus, and lifecycle flushing.
- Physical-device development warms the mobile shell and default Calendar surface before Tauri considers Vite ready. If Android WebView still caches a failed lazy-module request during a development-server transition, Retry performs the required document reload. Ordinary backend and persistence failures remain retryable in place. Packaged production modules do not depend on the development server.
- Mobile startup reconciles Pomodoro state before Calendar loads. It resumes one valid unexpired running or paused phase from persisted timestamps. When Android has a matching persisted native phase projection, recovery validates its run, event, rhythm, phase order, identifiers, timestamps, and event deadline before transactionally replaying elapsed phase boundaries. Multiple, malformed, unsupported, or unproven expired states close with a typed reason.
- An active Android Pomodoro session owns a `specialUse` foreground service with a silent ongoing progress notification. The service is independent of the Tauri Activity task, keeps the process eligible for foreground execution when the UI closes, and stops only when the session ends or the user explicitly stops it. Exact native phase alarms and the persisted projection remain the correctness boundary across service or process recreation. Manufacturer task cleaners can override standard Android service behavior. Focus settings identify relevant OEM controls and link to autostart and unrestricted-battery settings where available, without claiming the app can grant those controls itself.
- Calendar event reminders use Android native notification channels and `AlarmManager`. Startup, Calendar mutations, Activity resume, and reboot reconcile a persisted rolling schedule derived from authoritative SQLite events, offsets, overrides, exclusions, and recurrence expansion. The projection covers the next 12 months and keeps at most the next 256 deliveries so it remains below Android's per-app alarm limit with capacity reserved for other domains. Android 13 notification permission is requested only when a user enables reminders. Android 12 exact-alarm access has an in-context status and recovery action, with an explicit inexact fallback. Android 10 schedules exact reminders without either prompt. The event panel can post a real test reminder and then open the app-specific Android notification screen when an OEM or the user suppresses sound.
- Visibility and page lifecycle hooks perform best-effort flushes for configuration, Notes writes, and mounted Quick notes editors. Canonical writes remain responsible for correctness because Android can still remove a process without a final callback.
- The host Tauri check and focused mobile command-surface contract tests pass.
- The Android Rust target compiles with the NDK toolchain. Desktop music playback and the remaining desktop-only vault, picker, benchmark, networking, and recovery paths are excluded from the Android compile graph instead of being linked as dormant code. The Android composition registers the shared Music persistence commands plus the Media3 and selected-document adapters only.
- The base Android manifest requests only network access. Android backup and device transfer exclude the unencrypted app-private vault and all other application storage until an intentional encrypted backup design exists. The capture FileProvider exposes only the app-scoped `Pictures/` directory.
- The universal debug APK passes 16 KB ZIP alignment verification. Its ARM64 and x86_64 native libraries use 16 KB ELF load alignment, which is the Android 15 or newer 64-bit compatibility requirement. Physical 16 KB runtime testing remains a release gate.
- Pull requests build an ARM64 `Ganbaru AI Dev` APK from the committed Android project with the pinned JDK, SDK, NDK, Gradle, and Rust target contract. The artifact is retained for seven days for targeted acceptance testing.
- The protected release workflow is configured to build minified universal APK and AAB artifacts with a durable Android signing key, verify both signatures, and include them in the draft GitHub Release. The workflow cannot run until the protected Android signing secrets are configured.

The following remain roadmap work and must not be presented as available:

- Durable release-key creation and backup, protected GitHub secret configuration, the first minified production build, and the remaining emulator and release-device acceptance matrix.
- Remaining Storage Access Framework backup, restore, and attachment transfers, Notes notification boundaries, deep links, encrypted user-controlled backup, native local-playlist queue handoff, and broader Android media-format acceptance.
- Android backup UX, restoration for remaining noncanonical drafts and navigation state, broader physical process-death recovery, root-yielding and predictive Back validation, and remaining touch or narrow-layout adaptations inside shared feature components.
- Android device and emulator validation, Play policy work, and a production Android release.
- Mobile sync, remote Chat execution, sleep alarm, and Android Doomscrolling enforcement.

## Product boundary

Mobile should make planning, capturing, focusing, reviewing, and responding convenient away from a desk. It should not imitate desktop controls that Android cannot support reliably.

The first useful offline Android release includes the app-private vault, Calendar, Projects, Notes, provider-free Chat communication, Quick notes, Pomodoro, Music, Settings, localization, themes, import and export, and notification-based timer boundaries. Sync, remote agent execution, diary, sleep alarm, and policy-sensitive Doomscrolling capabilities can land independently after their own foundations are ready.

Desktop-only features remain absent from Android:

- Local coding-agent processes, provider executables, terminals, PTYs, Git worktrees, source-control commands, browser preview servers, and external working-folder execution.
- Tray, global shortcuts, single-instance handling, self-updater, native browser messaging, edge panel, detached windows, and always-on-top or fullscreen enforcement windows.
- Work-environment process control, desktop app closing, desktop foreground monitoring, and desktop browser-extension orchestration.
- Desktop recursive folder assumptions and reveal-in-file-manager actions.

Unavailable features are removed from primary navigation. Settings are different because they are also the product's capability map: every durable category remains discoverable except keyboard shortcuts, which have no mobile meaning. A category with no truthful Android control presents a concise implementation status instead of disabled desktop controls. Contextual explanations are also shown where a shared record refers to a desktop action.

### Settings architecture and section matrix

Settings use one shared category registry, one controller, and shared section components where the underlying behavior is genuinely portable. Focus exposes the same preference set on desktop and Android, followed by compact Android background-recovery links that use the same settings pattern. These links remain available after the one-time Focus access screen and do not imply that Android can inspect manufacturer-specific controls. Vite resolves the section renderer, detail loader, profile picker, WebView zoom adapter, and Music platform boundaries at build time. Android packages the provider-free Chat workspace but not local provider, terminal, Git, local review-panel, preview, or working-folder execution graphs. It also excludes Doomscrolling, updater, general native path-picker, tray, and desktop window-event graphs. The scoped theme document adapter is the narrow exception for Settings: an app-owned Tauri mobile plugin selects one import document or creates one public download without exposing a generic filesystem API to Svelte. The mobile presentation uses a category list followed by a full-screen section. Android Back returns from a section to the category list before closing Settings. Desktop keeps its modal navigation, lazy detail panels, keyboard handling, and draft protection.

| Section | Android behavior |
| --- | --- |
| Appearance | Shared themes, interface scale, font, text scale, language, time format, calendar zoom, and dimming controls. Theme duplicate, built-in inspection, custom editing, live preview, reset, Save, Cancel, and dirty-Back confirmation use a lazy full-screen mobile host over the shared editor model. Color picking also becomes a Back-aware full-screen touch surface. Theme JSON import uses Android's installed document-provider chooser; export writes a uniquely named file to Downloads. Export buttons show pending progress, prevent duplicate writes, and finish with a dismissible success or error notification. Success identifies the final Downloads file name. Both flows are bounded by the Rust-owned mobile document adapter. Android interface scale uses persisted CSS content zoom because Tauri's native WebView zoom command is unsupported on mobile. Keyboard hints and desktop quick-theme shortcuts are omitted. |
| Profile | Shared display and full names. Profile image selection uses the bounded document-input and managed-asset boundary, with browser preflight and Rust revalidation before an atomic write. |
| Calendars | Shared calendar records, event counts, and deletion. ICS import and export present an Android document-picker status until the streaming content-URI adapter exists. |
| Projects | The global category directs users to the project-local settings panel, where project workflow and field configuration belong on both platforms. |
| Notes | Shared default-open and version-history retention settings. File transfers and native notification delivery show separate adapter statuses until their Android boundaries exist. |
| Chat | Channels, messages, reply threads, search, drafts, and scheduled messages use the shared local Chat model and responsive workspace. Settings explain that Android does not launch local coding-agent processes or workspace tools. Future remote execution belongs behind its own authorization boundary. |
| Focus | Shared idle, notification, and break preferences plus compact Android autostart, battery, and Recents recovery guidance. Notification permission and exact-alarm access remain contextual to features that require them. |
| Music | Visible Media3 and Android media-control adapter status only. |
| Doomscrolling | Visible Android-specific enforcement status only. Desktop website and process controls are not reused because their semantics and permissions do not apply. |
| Data | Explains the app-private canonical vault, uninstall risk, and planned document-picker backup boundary. Desktop folder switching and reveal actions are omitted. |
| Updates | Explains Android store or signed-package distribution. The desktop self-updater is not packaged. |
| About | Shared application identity, version, licensing, and project information. |
| Shortcuts | Omitted because hardware keyboard shortcuts are not part of the mobile product contract. |

This matrix is a capability decision, not a second settings product. New portable settings should normally enter the shared section. A platform-specific adapter belongs behind a typed build-time boundary. A control must not be shown merely because its preference can be persisted if the platform cannot apply the behavior.

The mobile theme editor is excluded from the initial shell closure and loads when Appearance is opened. It reuses the shared theme editor, validation, contrast tooling, and in-memory session model but owns mobile presentation, safe-area padding, 44 dp minimum editor targets, full-screen color selection, and Android Back ordering. A dirty Back action asks before rolling the session back. A failed SQLite Save leaves the session open for retry. Theme JSON Apply changes only the in-memory draft, so the outer Cancel still restores the complete opening snapshot. Theme JSON Save to file uses the scoped mobile document adapter, shows in-button progress, and reports the exact Downloads result through the shared bottom notification surface. Bottom notifications account for system-navigation and keyboard insets so they remain inside the visible viewport. It does not call the desktop path command.

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
| Pomodoro | Shared state machine and persisted deadlines | Ongoing notification, native phase alarms, persisted phase projection, and transactional cold reconciliation | Desktop windows, notifications, and enforcement |
| Import and export | Shared typed transfer plans | Native tree import implemented; remaining transfers use Storage Access Framework content URIs | Native paths and folder dialogs |
| Music | Shared queue, playlist, and transport contracts | Media3, ExoPlayer, MediaSessionService | Rodio, Symphonia, WebView media, and desktop controls |
| Deep links | Shared validated navigation intents | Cold-start and running-intent delivery | Desktop protocol and single-instance delivery where supported |
| Updates | Shared release information UI only when available | Play or signed package distribution | Tauri updater or package-manager instructions |
| Chat communication | Shared authorized records and read DTOs | Future synchronized communication and review client | Communication plus native coding workspace |
| Coding execution | None on mobile | Unsupported | Native provider processes, PTYs, working folders, and Git |
| Doomscrolling | Shared user rules and explanations where semantics match | Future permission-aware Android subsystem | Browser extension and desktop enforcement |
| Window integration | Adaptive in-app surfaces | One Activity and one WebView | Multi-window, tray, detached windows, and overlays |

Tauri capabilities are split by platform and selected explicitly in each platform configuration. The implemented desktop capability lists Linux, macOS, and Windows with the existing desktop window, updater, event, and scoped opener permissions. The implemented Android capability applies only to `main` and grants Back-listener registration and removal plus explicit HTTPS URL opening for user-authored links. It has no core event permissions because mobile cross-window synchronization is a build-time no-op transport. Neither capability grants unencrypted HTTP, email, telephone, file revealing, or arbitrary path opening through the opener plugin. Mobile does not receive create-window, always-on-top, updater, process, shell execution, global-shortcut, tray, dialog, or broad filesystem permissions. Android production content security policy permits only app-owned scripts and disables frames, so desktop YouTube origins do not enter the mobile WebView authority. A later iOS capability is defined separately instead of widening the Android file. See [Tauri capability configuration](https://v2.tauri.app/security/capabilities/) and [Tauri opener permissions](https://v2.tauri.app/plugin/opener/).

## Canonical vault and Android storage

The Android canonical vault lives under app-private internal storage. On a fresh install, mobile bootstrap mounts the shared vault setup content and does not create a vault until the user chooses the default or imports an existing Ganbaru AI folder. The selected default resolves to `<app_data_dir>/Ganbaru AI` in production or `<app_data_dir>/Ganbaru AI Dev` in development. Its logical structure remains a Ganbaru AI vault, including `vault.json`, `config.json`, `ganbaru-ai.sqlite`, managed documents, and assets, but its physical root is assigned by Android and is not a public `Documents` path.

App-private storage is the only suitable home for the canonical SQLite database because it provides a real private filesystem path, requires no storage permission, and supports the existing SQLx and bounded filesystem model. Android's scoped-storage model applies to modern targets, while broad legacy write permission no longer provides unrestricted shared-storage access. See the [Android data storage guide](https://developer.android.com/training/data-storage).

The Storage Access Framework is an interoperability boundary, not the live vault:

- Import, export, backup, restore, and attachment selection use system document UI.
- Android document results are opaque content URIs, not paths.
- Imported canonical documents are validated and copied into managed storage before direct filesystem use.
- Whole-vault import uses `ACTION_OPEN_DOCUMENT_TREE`, streams recursively into a bounded app-private staging directory without buffering the vault in JavaScript or Rust memory, rejects unsafe names, excessive depth, entry count, byte count, and virtual documents, then atomically promotes only a valid Ganbaru AI vault.
- Exports stream to the selected URI and never construct a path from it.
- A persisted URI grant is retained only when an external object must remain referenced.
- Revoked, moved, deleted, or unavailable URI content becomes an explicit recoverable state.
- The app does not request all-files access or depend on `MANAGE_EXTERNAL_STORAGE`.

The implemented WebView file input is appropriate only for bounded image assets that are copied immediately into the vault and bounded text imports such as the 512 KiB Notes database CSV path. General Notes attachments, larger imports, exports, backups, and large media must use a native streaming content-URI adapter. They must not base64-encode a large file through the JavaScript bridge, which would multiply memory use on the Android 10 reference device.

Tauri's dialog plugin returns content URIs on Android and does not support Android folder selection. Rust code using `PathBuf` or `std::fs` cannot consume those URIs directly. See the [Tauri dialog documentation](https://v2.tauri.app/plugin/dialog/) and the [Android Storage Access Framework guide](https://developer.android.com/training/data-storage/shared/documents-files).

Large local music files are a deliberate exception to copying. The native Android picker starts at the standard shared `Music/` directory when the system picker honors initial-location hints, retains access only after the user confirms the selected tree, and lets Media3 read its audio through Android's content resolver. Playlist records store a platform source reference and cached display metadata, never a fabricated filesystem path. The native document-tree scanner recursively catalogs the granted tree with bounded file and depth limits. The desktop filesystem scanner is not exposed on Android.

Android Auto Backup includes much app-private data by default. The generated manifest therefore disables backup and supplies both legacy backup rules and Android 12 or newer data-extraction rules. Every app storage domain is excluded from cloud backup and device-to-device transfer. This prevents the unencrypted vault, credentials, caches, and device-local state from entering an implicit platform backup. A future encrypted, user-controlled backup is an explicit product flow and can narrow these rules only after its threat model is reviewed. See [Android Auto Backup](https://developer.android.com/identity/data/autobackup) and [Android backup security guidance](https://developer.android.com/privacy-and-security/risks/backup-best-practices).

Uninstalling the app can remove app-private data. Settings and the future backup flow must explain this plainly and make export, backup, and later encrypted sync easy to discover. This implementation boundary does not belong in first-use copy.

## Mobile shell and adaptive UX

Android uses one Activity and one WebView. Additional Tauri windows map to additional Android Activities, not desktop-style child windows. The baseline Android 10 phone therefore uses in-app navigation, pages, dialogs, and sheets. Activity Embedding for side-by-side Activities requires Android 12L, API level 32, and is not the base architecture. See [Tauri mobile multiwindow](https://v2.tauri.app/learn/mobile-multiwindow/).

Layout responds to the current app window, not a device-name or portrait-phone assumption. Android window-size guidance defines compact width below 600 dp, medium width from 600 dp, and expanded width from 840 dp. The same window can change class through rotation, split screen, a fold, or desktop windowing. See [Android window size classes](https://developer.android.com/develop/ui/views/layout/use-window-size-classes) and [adaptive layout guidance](https://developer.android.com/design/ui/mobile/guides/layout-and-content/adapt-layout).

- Compact width places the small set of primary destinations as icon-only controls in the global top bar. Their accessible names preserve clarity without repeating visible labels or reserving a bottom bar. Selection uses the bar's bottom edge as a short indicator, preserving the full header surface rather than painting the complete destination cell.
- Medium width uses a navigation rail and may show list and detail together when both remain usable.
- Expanded width uses a rail or drawer and stable list-detail or supporting panes. It does not stretch compact cards across the viewport.
- Exact primary labels can evolve with the product information architecture, but Calendar, Projects, and Notes remain directly reachable. Active Pomodoro context stays available in the same compact global bar.
- Global session, timer, notes, media, and settings controls follow the desktop utility order after the primary destinations. Pomodoro, Quick notes, and the Music player open as floating panels above the active destination without adding stacked mini-players or another persistent bar. The Music playlist builder expands to a full-screen workspace.
- Desktop popovers become anchored menus only when they fit. Complex popovers and modal panels become bottom sheets, side sheets, or full pages.
- Hover is never required. Drag interactions have visible handles and an accessible button or menu alternative.
- Interactive targets are at least 48 dp and expose semantic names, states, and roles. See [Android accessibility guidance](https://developer.android.com/design/ui/mobile/guides/foundations/accessibility).
- Motion respects reduced-motion preferences. Long lists and dense timelines are virtualized, and controls remain reachable at increased font and display scaling.

Feature screens adapt their information density rather than merely shrinking desktop CSS:

- Calendar defaults to a focused day presentation. Day, work-cycle, week, and month remain available, and horizontal surface swipes navigate by the active view unit. Timed events support hold-and-drag movement and hold-and-drag range creation through the shared desktop calendar model. Finger resizing is omitted, while exact start and end changes remain in the full-screen editor. Ordinary vertical movement outside event blocks retains native scrolling. The event editor uses a page or sheet instead of a desktop floating dialog and remains the exact, accessible editing path, so touch creation never depends on precise drag placement.
- Projects use list-detail navigation. Boards support horizontal lanes, explicit move actions, and touch handles. A list view remains available for dense task work.
- Notes show tree, editor, database controls, and history as separate compact destinations. Medium and expanded widths can restore multiple panes.
- Quick notes use a floating, safe-area-clamped collection panel with a single-column compact list unless measured width safely supports more. Editing occupies a stable sheet or page above the keyboard.
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

Android receives a bounded projection of the active phase and the remaining event plan after the canonical run start is committed. The native adapter persists that projection, posts a dedicated ongoing notification, and schedules the next phase boundary with `AlarmManager`. Android's system chronometer keeps the visible countdown moving without a JavaScript interval. The standard horizontal progress indicator is refreshed at bounded percentage steps while the frontend is available and resets when a native boundary starts the next phase.

Calendar reconciliation also gives Android a bounded rolling projection of future Pomodoro-enabled event occurrences. Each projection contains its exact event start, deadline, event title, and deterministic phase plan. An exact activation alarm can start the foreground service and ongoing notification at the event boundary while the Activity, WebView, and Rust runtime are absent. Reboot, package replacement, wall-clock, and timezone broadcasts restore those alarms. Opening the app during a natively activated occurrence starts or recovers the canonical SQLite run through the ordinary Calendar scheduler, which replaces the temporary native projection. Calendar edits reconcile and cancel stale future activations.

When the Activity or app process is absent, the native receiver advances the notification through the projected phases and posts a separate phase-completion alert. Cold startup validates the stored projection against SQLite before it trusts any boundary. A valid projection can transactionally complete elapsed segments, insert later elapsed segments, and resume the phase containing the current time. An invalid or mismatched projection cannot widen or rewrite the run. A manual pause publishes only its current phase, cancels automatic phase advancement, and remains paused until the user resumes or the event window ends.

The active notification uses Android's ongoing and no-clear flags on a low-importance, silent channel and is owned by the active-session foreground service. The phase-boundary channel is separate and audible according to the user's system settings. Android 14 and newer allow users to dismiss most ongoing notifications with an individual swipe, so the app cannot promise an absolutely permanent notification on those versions. Canonical recovery does not depend on notification visibility. Some manufacturers can kill even a foreground service from their Recents cleaner and suppress sticky restart until the user grants manufacturer-specific background access. This behavior is independent of whether the build is development or production. Xiaomi's own guidance requires Autostart, No restrictions, and locking the app in Recents for persistent background operation. Samsung instead documents Background usage limits and Never sleeping apps, not a separate autostart grant. See [Xiaomi background-operation guidance](https://www.mi.com/uk/support/faq/details/KA-231124/) and [Samsung application-management guidance](https://developer.samsung.com/mobile/app-management.html).

The shared Android notification path must:

- Create stable notification channels before posting.
- Ask for `POST_NOTIFICATIONS` only in context on API level 33 and newer. Android 10 does not require that runtime prompt. See [Android notification permission](https://developer.android.com/develop/ui/compose/notifications/notification-permission).
- Provide useful behavior when notifications are denied. In-app state remains correct, while the event panel reports that background alerts are unavailable and opens Android's app-specific notification settings.
- Treat notification action payloads as untrusted and validate their kind, action, and bounded identifier before navigation.
- Run the foreground service only for an active session deliberately configured by the user, including a Pomodoro-enabled Calendar event whose start alarm fires. The persisted alarm and system chronometer drive the countdown and phase boundaries without polling or continuous CPU work.
- Never force the Activity to the foreground when a timer ends. Android 10 restricts background Activity starts, so the alert remains a notification until the user chooses it. See [Android 10 privacy changes](https://developer.android.com/about/versions/10/privacy/changes).

Ganbaru AI presents one short first-run Focus access screen after the vault is ready. Android still owns every runtime permission dialog and special-access Settings screen. The screen requests notification permission only when API level 33 or newer reports it missing and offers Alarms and reminders only when API level 31 or newer reports that exact access is unavailable. Background autostart appears only when a verified manufacturer adapter resolves its settings destination; battery-saver restrictions use the manufacturer destination when verified and Android's standard battery settings otherwise. The adapter registry covers Xiaomi, Redmi, Poco, Huawei, Honor, Oppo, Realme, OnePlus, Vivo, iQOO, Asus, Samsung, Transsion brands, ZTE, Nubia, Lenovo, Nokia, HMD, Meizu, and LeTV where those devices expose a relevant control. Samsung routes only to its documented Never sleeping apps control under battery restrictions. Stock-like Google and Motorola devices do not receive a misleading autostart row.

Every OEM action or component is checked with Android's package manager on the current phone before it is exposed, with the packages declared narrowly for Android package-visibility filtering. Opening tries all resolved candidates in order because an exported activity can still reject a launch. If every OEM battery destination fails, the app opens Android's standard battery-optimization screen and then the app-details screen as the final fallback. If an autostart component disappears between detection and launch, the app opens app details instead. If no dedicated autostart component resolves during detection, the row is hidden. This follows Android's guidance to resolve settings intents before starting them and avoids pretending that one private OEM component is universal. Supported states refresh on return; autostart records review rather than claiming an unverifiable grant. See [Android common-intent guidance](https://developer.android.com/guide/components/intents-common.html) and the established open-source [AutoStarter OEM intent registry](https://github.com/judemanutd/AutoStarter/blob/master/autostarter/src/main/java/com/judemanutd/autostarter/AutoStartPermissionHelper.kt).

The five-second Continue delay prevents immediate accidental dismissal without making unavailable or declined access a permanent blocker. Android documents Calendar as a valid precise-alarm use case, so Ganbaru AI declares `SCHEDULE_EXACT_ALARM`; denial keeps an explicitly described `setAndAllowWhileIdle` fallback. Pomodoro still uses the least privileged scheduling that meets measured reliability. A separate sleep-alarm design may reuse exact access only with contextual education and graceful fallback. See [Android alarm guidance](https://developer.android.com/develop/background-work/services/alarms).

Calendar uses a narrow Ganbaru-owned Android adapter for persisted scheduling, cancellation, validated tap delivery, and reboot or package-replacement restoration. The official Tauri notification plugin remains the runtime-permission boundary, while the Ganbaru adapter owns the bounded rolling schedule projection, identifier range, stable Calendar channel, app-specific notification settings, exact-alarm settings, and native posting behavior. Native Calendar notifications use Android's calendar-event category, high channel importance, the symbolic default notification sound, and reminder-specific audio attributes. Sound, vibration, and lights come exclusively from the channel rather than deprecated per-notification defaults. Calendar deliveries remain independent notifications instead of incomplete notification-group children. The adapter uses exact `AlarmManager` delivery when allowed and `setAndAllowWhileIdle` fallback when Android 12 or newer has not granted exact-alarm access. Its small device-local schedule projection is rebuilt from authoritative Calendar data, restored after reboot or package replacement, and never becomes a second Calendar source of truth. The event panel exposes a real test notification because standard Android APIs cannot determine whether every OEM-level sound control produced audible output. If the test arrives silently, one explicit action opens the app-specific native notification settings; Ganbaru AI never bypasses silent mode, Do Not Disturb, or user-controlled notification sound with direct media playback. Pomodoro, Notes, and future sleep alarms reuse the permission boundary while retaining distinct channels, identifier ranges, schedule capacity, and domain reconciliation. See the [Tauri notification plugin](https://v2.tauri.app/plugin/notification/), [Tauri mobile plugin development](https://v2.tauri.app/develop/plugins/develop-mobile/), [Android notification categories](https://developer.android.com/reference/android/app/Notification#CATEGORY_EVENT), and [Android notification channel behavior](https://developer.android.com/develop/ui/compose/notifications/channels).

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

Android does not run local coding-agent harnesses. Mobile Chat currently reuses the shared provider-free channel, message, reply-thread, search, draft, and scheduling model against the active local vault. Future synchronization will extend it into a cross-device communication and review client for task discussions, proposals, attention items, and deliverables. Coding-agent execution will occur only on an explicitly authorized desktop or future remote runner.

The mobile client never:

- Spawns Codex, Claude, Cursor, OpenCode, a shell, a PTY, Git, or a sidecar.
- Claims access to desktop external working folders.
- Exposes terminal, source-control, browser-preview, checkpoint, or local file-explorer UI.
- Broadens a teammate's channel, project, folder, provider, or data permissions.

When a channel contains desktop execution events, mobile renders safe read-only projections and review artifacts. Actions that require a desktop show a concise handoff state instead of a disabled desktop workspace replica. Local provider-free communication works without sync. Cross-device writes queue only after permission-aware sync defines conflict, revocation, and receipt semantics.

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
| `POST_NOTIFICATIONS` on API level 33 and newer | Requested contextually when the user enables or uses a notification-dependent feature and Android reports that access is missing | Native deadlines remain scheduled, but Android can hide the user-facing notification |
| `SCHEDULE_EXACT_ALARM` special access | Offered when Android 12 or newer reports it unavailable for precise Calendar Focus activation; future sleep alarms reuse the same truthful boundary | Use `setAndAllowWhileIdle`, explain that starts can be late, and retain an action that opens Alarms and reminders access |
| `RECEIVE_BOOT_COMPLETED` scheduling support | Declared by the app-owned Calendar and Focus notification adapter for persisted schedule restoration | Domains without persisted native delivery reconcile only when the app next opens and never imply reboot-safe delivery |
| `FOREGROUND_SERVICE` and `FOREGROUND_SERVICE_SPECIAL_USE` | Declared for an active user-configured Focus event, with the required ongoing notification and Play declaration | A Focus event cannot reliably remain active after its alarm starts while the UI is absent |
| `FOREGROUND_SERVICE` and `FOREGROUND_SERVICE_MEDIA_PLAYBACK` | Declared with the Media3 playback service | Playback stops when the foreground media capability is unavailable |
| Usage Access through `PACKAGE_USAGE_STATS` | User enables Android usage awareness from its settings flow | Limits can still be configured, but device-app usage is not observed or enforced |
| VPN service consent | Future local domain filtering after separate privacy and policy review | Browser traffic is not filtered |
| Persisted Storage Access Framework grants | User selects an external document or media source | Canonical app data remains available; that external item is unavailable until repaired |

The app does not pre-request unused sensitive access for current or future features. The base app does not request `MANAGE_EXTERNAL_STORAGE`, legacy broad shared-storage access, `QUERY_ALL_PACKAGES`, `SYSTEM_ALERT_WINDOW`, Accessibility Service binding, device-owner status, contacts, microphone, camera, or location. Usage Access, VPN consent, and any future Accessibility Service disclosure enter their own enable flow only when the corresponding Doomscrolling capability exists. A future feature that genuinely needs another permission requires an updated spec, localized disclosure, denial UX, tests, and store-policy review before the permission enters the manifest.

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

The required local toolchain is Android Studio, Android SDK Platform 36, Platform Tools, SDK Build Tools 35.0.0, command-line tools, side-by-side NDK 30.0.15729638, JDK 21 to run Gradle and Tauri, and the selected Rust Android targets. Java and Kotlin application bytecode remains explicitly targeted to version 17. This separates the supported build-runtime LTS from the Android application language level and avoids machine-dependent bytecode. The repository-owned Tauri wrapper selects JDK 21 from `GANBARU_AI_ANDROID_JAVA_HOME`, a compatible `JAVA_HOME`, the GitHub runner JDK variable, the Java executable on `PATH`, or reviewed Linux installation paths. It fails with a direct setup error instead of falling back to an incompatible Android Studio runtime. Configure `ANDROID_HOME` and `NDK_HOME` as described by the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

The supported Rust target set is `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, and `x86_64-linux-android`. Local development installs the connected device or emulator target first. Release configuration states its ABI support explicitly and tests every included ABI. It must not become ARM64-only by accident.

Normal commands from the repository are:

```sh
pnpm --dir apps/client run tauri android init
pnpm --dir apps/client run tauri android dev
pnpm --dir apps/client run tauri android build --apk
pnpm --dir apps/client run tauri android build --aab
```

The Android platform configuration keeps its development URL at the Vite origin root. Android
WebView requests are intercepted at `tauri.localhost` and proxied to that URL by Wry. A path-based
readiness endpoint would become the base of every proxied module request, causing Vite to return
the HTML entry document for JavaScript modules. Desktop development may retain a path-based
readiness endpoint because its WebView loads the Vite origin directly.

Run `pnpm --dir apps/client run generate:icons` after changing the source logo or deliberately regenerating either native project. Tauri stores mobile launcher resources directly in the generated Android and Apple projects, while the desktop `bundle.icon` list alone does not replace those generated mobile resources. The Android project contract verifies the expected Ganbaru AI launcher fingerprint plus normal and round manifest resources so the default Tauri launcher cannot return unnoticed. See the [Tauri app icon guide](https://v2.tauri.app/develop/icons/).

The APK is for direct device and emulator testing. Google Play distribution uses an Android App Bundle. Development may build only the connected device ABI for speed. Release ABI policy is explicit and validated against supported devices rather than assumed from API level.

Release signing uses a protected keystore and private Gradle signing properties. Release Gradle tasks fail when signing properties are absent, while debug tasks remain independent of release credentials. Pull requests build an ARM64 debug APK under the `.dev` identifier. The protected tag workflow builds all four supported Rust ABIs into one signed universal APK and AAB, verifies their signatures, and stages them with the desktop assets. Signing secrets never enter source control. The package identifier and release key are permanent product identity once distributed. See [Tauri Android signing](https://v2.tauri.app/distribute/sign/android/).

Once created, the generated Android project and intentional native changes are reviewed as source. Regeneration is deliberate and followed by a diff. Release builds validate minification, manifest merging, capabilities, permissions, content providers, deep links, backup rules, native library alignment, and installation from the built AAB, not only a debug APK.

## Delivery order

Android work is divided into independently useful milestones:

1. **Build foundation:** Android configuration and project, legal identifier, separate mobile Rust composition, mobile capabilities, one visible Activity, app-private vault bootstrap, CI compilation, signing design, and an installable shell.
2. **Offline core:** adaptive navigation, Calendar, Projects, Notes, Quick notes, Settings, localization, themes, and lifecycle-safe drafts over the local SQLite vault.
3. **Focus and interoperability:** persisted Pomodoro deadlines, notification boundaries, deep links, document import and export, backup UX, permission states, and release-device validation.
4. **Native media:** Media3 service, system controls, local content URI sources, and mobile player UX.
5. **Connected experience:** encrypted sync, remote Chat execution, task review, attention, and cross-device handoff after Phase 9 authorization and conflict semantics exist.
6. **Policy-sensitive features:** diary and sleep alarm, usage awareness, and carefully reviewed Doomscrolling capabilities. Each remains useful with special permissions denied.
7. **iOS alignment:** implement equivalent shared contracts through iOS-native adapters without weakening Android architecture or pretending platform capabilities are identical.

The current source foundation completes the local build, CI wiring, protected signing configuration, and physical-shell portions of milestone 1 and parts of milestone 2. The native Android project builds all four supported ABIs into universal APK and AAB artifacts, while pull requests build only ARM64 for bounded feedback. An ARM64 debug APK installs and passes initial Android 10 acceptance for cold start, background process restart, portrait and landscape insets, compact and wide navigation, system-bar theme contrast, root Back, Calendar, Projects, Notes, Quick notes, Pomodoro, and Settings. Creating and protecting the durable release key, validating the first minified signed artifacts, emulators, and the complete release matrix remain milestone 1 work. Milestones 1 through 4 do not depend on sync and can proceed before Phase 9. Milestone 5 depends on Phase 9. Sleep alarm depends on the diary domain and native alarm design. Policy-sensitive features require their own store-policy review before release.

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
- TalkBack, switch or keyboard navigation, sufficiently large touch hit areas with compact visual affordances where appropriate, increased font and interface scale, high contrast, reduced motion, and Spanish plus English layouts.
- Play pre-launch reports, permission and Data safety declarations, release signing, AAB installation through bundle tooling, ABI splits, minification, and 16 KB native alignment.

Automated tests should keep shared domain logic outside Tauri where possible, add focused tests for capability selection and adaptive state, and add Android instrumentation only for lifecycle, Activity, permission, content URI, alarm, notification, deep-link, and service behavior that cannot be proven in Rust or TypeScript.
