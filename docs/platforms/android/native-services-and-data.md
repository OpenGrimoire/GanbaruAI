# Android native services and data

## Application-private vault

Android creates the active Ganbaru AI vault under application-private storage. The user does not choose another live-vault path because Android document providers do not provide a safe desktop-style writable folder with equivalent semantics.

First use states clearly that uninstall can remove this data and offers `Start from zero`, import of a complete Ganbaru AI folder, and restore of a portable Android backup.

The displayed data location is informational. Reveal-in-file-manager and arbitrary live-folder switching are absent.

## Folder import

Import uses the system directory-tree picker and streams a bounded selected folder into private staging. It validates the vault marker, paths, depth, entry count, expanded size, and database before atomic activation.

The selected external folder never becomes the live vault and is not edited in place.

## Portable backup and restore

Backup creates a consistent SQLite snapshot and bounded archive in user-accessible Downloads without passing full file bytes through JavaScript. Live SQLite WAL and SHM files are excluded.

Restore uses the system document picker, extracts into private staging with path and size limits, validates the vault and SQLite integrity, blocks new database connections, closes active pools, and atomically activates the restored folder with rollback on failure.

The portable format is the recovery path after reinstall. Android system backup and device transfer exclude the unencrypted live vault until a deliberate encrypted design exists.

## Lifecycle and process death

Visibility and page lifecycle hooks perform best-effort configuration, Notes, and Quick notes flushes. Canonical writes remain responsible for correctness because Android can remove a process without a final callback.

Startup validates the active vault and reconciles domain state. Malformed, duplicated, expired, or unsupported persisted state closes with a typed reason rather than becoming active silently.

Activity removal is not equivalent to ending Pomodoro, stopping Media3 playback, or disabling selected-app enforcement. Their native services own those lifecycles explicitly.

## Calendar notifications

Calendar maintains a bounded rolling native projection derived from authoritative SQLite events, reminders, recurrence, exclusions, and overrides. Startup, event mutation, Activity resume, reboot, package replacement, time change, and timezone change reconcile it.

Each delivery uses a stable domain identifier and native Calendar notification channel. Tapping is validated before opening Calendar. The projection is rebuildable device state, not a second event database.

The event panel can post a real test reminder. If the result is silent, the app opens its native notification settings rather than bypassing silent mode, Do Not Disturb, or user channel controls with media playback.

## Pomodoro service and alarms

An active run owns a special-use foreground service and silent ongoing progress notification. Persisted native phase projections and alarms maintain deadlines while the Activity and WebView are absent.

Opening the app reconciles native state with canonical SQLite run state. Matching identifiers, event, rhythm, phase order, timestamps, and event deadline are validated before replaying elapsed boundaries.

Calendar can project future Pomodoro-enabled occurrence activation so a run can begin at an event boundary according to user settings. The ordinary scheduler becomes canonical when the app opens.

Manufacturer task cleaners can still override standard behavior. The app offers truthful autostart and battery-setting guidance without claiming it can grant those controls.

## Music service

Android local audio uses a Media3 session service and ExoPlayer. The service owns background playback, audio focus, media notification, lock-screen and headset controls, and trusted system controller state.

Local sources use retained document-tree permission and stable tree-relative identities. Scanning and metadata work stay outside the Android main thread and within explicit bounds.

The shared library, playlist, review, assignment, and playback state remain SQLite-canonical. Desktop audio and media-control backends are absent.

## Doomscrolling guardian

Android selected-app rules use a private guardian process containing phase alarm handling, ongoing notification coordination, Usage Access reads, the opt-in Accessibility Service, validated rule projections, and a bounded native journal.

Non-exported provider boundaries publish signed projections and import journal rows. The guardian is the only process opening its private runtime preferences and journal database.

Journal import accepts the current vault identity and writes normalized usage and block history to canonical SQLite. Runtime rows are compacted and bounded. User configuration remains active-vault `config.json`.

## Managed files

Profile images, project icons, custom emoji, Notes icons, covers, attachments, and supported CSV input use system document selection and bounded managed-asset writes. Declared and sniffed MIME, size, dimensions where relevant, digest, and destination are validated before canonical references are stored.

Broad shared-storage permission is unnecessary. Remote Notes images are not rendered directly under the current content policy; a future ingestion flow can download and validate them into managed storage.

## Deferrable work

WorkManager is appropriate for deferrable maintenance and future sync. It does not own Pomodoro countdowns, Calendar reminder deadlines, active Music playback, or real-time selected-app enforcement.
