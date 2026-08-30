# Pomodoro progress displays

Pomodoro progress appears in app chrome, the desktop tray, an Android ongoing notification, and the Calendar timeline rail. They share canonical timer and segment state but answer different immediate questions.

## Shared principle

Glance surfaces show time until the next meaningful transition, not total productivity, cycle statistics, or a reward score. The user should be able to decide whether to continue the current thought or prepare for a break without reading a dashboard.

## App chrome ring

The title-bar or mobile-top-bar ring appears during an active focus phase. Its arc represents remaining usable focus opportunity, clipped by the Calendar event end.

At focus start the remaining arc is full and shrinks toward empty. Manual pause uses a restrained pulse while preserving the same opportunity semantics. If the event deadline continues to approach during pause, the displayed opportunity reflects that hard boundary.

The ring is hidden during breaks and when no focus is active. It does not show cycle number, total focus, future breaks, or daily statistics.

The ring menu exposes applicable Pomodoro actions and compact Music status and controls through the canonical player.

## Desktop tray ring

The tray mirrors the app chrome ring's focus-only metric and pause intent. Raster rendering and platform menu behavior are documented in [Desktop tray](../../platforms/desktop/tray.md).

The tray remains visible when the main window is hidden. It does not add another progress definition or show extra statistics simply because more menu space exists.

## Android ongoing notification

Android shows an ongoing notification during active focus, break, and manual pause. It uses system-owned countdown presentation for the deadline and a bounded progress indicator for elapsed ratio.

The notification identifies the event when available, otherwise the current phase. Paused state is explicit and retains the last bounded progress value.

A foreground service and native phase projection own background continuity. Notification visibility itself is not the correctness boundary because newer Android versions can allow individual dismissal. Phase-completion alerts use a separate channel governed by system notification settings.

See [Android native services](../../platforms/android/native-services-and-data.md).

## Calendar rail

Day, work-cycle, and week views can show a narrow time-aligned rail beside Pomodoro-enabled events. Month view omits it because cells cannot communicate this detail legibly and the view is primarily for planning.

The rail combines:

- Persisted focus actually credited.
- Active and completed official break allowance.
- Projected future break positions.
- Empty time for future focus, pauses, idle, suspend, stopped gaps, and uncredited overtime.

Days without a Pomodoro-enabled event have no rail.

## Rail states

The rail has three semantic states:

| State | Meaning |
| --- | --- |
| Focus fill | Persisted completed, interrupted, or active focus time, excluding pauses and never extending beyond now |
| Break marker | Planned future breaks and official active or completed break allowance, including configured extensions and ten seconds of end grace |
| Empty | Unperformed future focus, absence, pause, suspend, stopped gaps, unstarted breaks, and overtime beyond grace |

Theme tokens own the colors. No extra shade distinguishes active, projected, or historical variants. Text and surrounding context explain meaning without relying on color alone.

## Focus fill

Focus fill begins at a segment's actual start and ends at its completed or interrupted end, or at now for an active segment. Every pause interval removes fill from its range.

Future focus is never painted as completed work. A segment can therefore have several visible focus bands while remaining one canonical segment with multiple pauses.

## Break markers

Persisted break segments show only the official interval that ran, capped to planned end plus ten seconds of grace. Configured extensions move planned end and therefore count as official allowance.

Projected breaks come from the current run plan. When an event still has time but no active run, projections can restart from now to show that recovery remains reachable if the user resumes. These projections move with now and are never persisted.

Starting a new run converts its current plan into stable active projections. A break that never starts creates no historical marker.

## Protected history and recurrence

Calendar recurrence edits preserve historical event time so focus and break segments remain aligned with their original blocks. The rail renders from canonical occurrence identity and does not move historical fill to a newly edited future schedule.

## Visibility matrix

| Surface | Shown | Hidden |
| --- | --- | --- |
| App chrome ring | Active focus, including manual pause while opportunity remains | Break, no run, or expired event |
| Desktop tray ring | Same focus state regardless of main-window visibility | Break, no run, or expired event |
| Android ongoing notification | Active focus, break, or manual pause | No active run or completed deadline |
| Calendar rail | Day, work-cycle, or week with a Pomodoro-enabled event | Month or a day without such events |

## Accessibility and motion

Progress surfaces expose text equivalents for remaining time, phase, and pause state. Pulse is a reminder, not the sole paused indicator, and respects reduced motion where the platform permits. The rail's meaning is available through event and session details for users who cannot distinguish its colors.
