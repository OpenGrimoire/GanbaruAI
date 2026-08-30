# Desktop tray

The tray is Ganbaru AI's operating-system glance and control surface when the main window is hidden, minimized, or behind other windows.

## Purpose

The tray answers:

- Is a focus phase active or paused?
- How much usable focus opportunity remains before the next transition?
- What media is active?

It exposes compact Pomodoro and Music actions. It is not a statistics, cycle-history, or productivity-score surface.

## Pomodoro ring

The tray ring uses the same remaining-focus metric as the app chrome ring. The event deadline clips the opportunity when it arrives before the configured focus duration.

| State | Presentation |
| --- | --- |
| No focus session | Subdued empty ring |
| Focus start | Full remaining-time ring |
| Focus running | Remaining arc shrinks toward empty |
| Manual focus pause | Remaining arc uses a restrained slow pulse |
| Focus complete or stopped | Subdued empty ring |

Active start and idle are intentionally different: a full active ring means the opportunity has just begun, while an empty subdued ring means no active focus.

The paused reminder stops immediately on resume, stop, phase change, idle pause, suspend pause, or deadline. Raster updates remain bounded and do not turn the tray into a high-frame-rate animation surface.

## Menu

The Pomodoro section shows status and the applicable actions:

- Pause or resume focus.
- Extend focus once when the event has room.
- Move to break during focus.
- Start focus during a break.

The Music section shows the active title or a clear unloaded state and provides:

- Play or pause.
- Previous and next.
- Open Music in the main window.
- Inspect the current Calendar or project soundtrack assignment when available.

Controls remain visible but disabled when unavailable so menu positions stay predictable.

Clicking the tray icon opens or focuses the main window. Platform focus-stealing rules can limit how forcefully an existing window is raised.

## Linux AppIndicator

Ubuntu AppIndicator resolves tray images from paths. Replacing a path before the new image exists can briefly show a missing-icon placeholder. Ganbaru AI therefore renders the next PNG into application cache first, reuses an existing derived image when possible, and only then points AppIndicator to it.

Generated tray images are derived application cache, not vault data. Cache filenames include a renderer version so an incompatible rendering change can invalidate old output. The authoritative renderer constant lives in `apps/client/src-tauri/app/src/tray.rs`.

The implementation uses lower-level Tauri tray access on Linux. Tauri upgrades that affect tray internals require explicit Linux verification. Dependency manifests, not this document, own exact version pins.

## Platform notes

**Linux:** tooltip support is not reliable, so the menu is the accessible status surface. The prewritten AppIndicator image path is Linux-specific.

**Windows:** the standard Tauri tray icon and tooltip paths apply.

**macOS:** a future supported desktop build uses the menu-bar presentation and standard icon path unless platform validation requires a specialized adapter.

## Accessibility

The menu text describes ring and media state without relying on icon color. Actions retain stable labels and enabled state. No critical operation requires interpreting the animated pause treatment.

See [Pomodoro progress displays](../../features/pomodoro/progress-displays.md), [Music](../../features/music/README.md), and [native backend architecture](../../architecture/native-backend.md).
