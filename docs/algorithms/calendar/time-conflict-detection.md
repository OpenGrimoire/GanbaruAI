# Time conflict detection

**Status: Implemented locally.** The Calendar scheduler and timeline rail share one owner selector. Selecting a scheduled commitment does not authorize execution. Linked-device control and conflict suspension during replication remain planned.

## Interval model

Eligible timed events have a Pomodoro configuration, a finite valid range, and are neither all-day nor cancelled. The interval includes its start and excludes its end. At exactly 10:00, an event ending at 10:00 is no longer eligible and an event starting at 10:00 becomes eligible.

## Owner selection

At the current instant:

1. Preserve the currently executing owner if it remains eligible.
2. Otherwise choose the eligible event with the earliest end.
3. Break equal-end ties by creation identity, then occurrence ID, using stable string ordering independent of device locale.

Missing creation identities use the empty string for deterministic legacy ordering. Rhythm settings, containing another event, and recently interrupted status confer no priority. Permuting candidate input order does not change the result.

The scheduler wakes at event boundaries and on Calendar or lifecycle invalidation. Desktop automatic admission additionally requests fresh local activity; while waiting for that evidence, it retries at a bounded interval. Android schedules reminders and requires an explicit start. See [Focus authority and evidence](../pomodoro/focus-authority.md).

## Timeline projection

The rail retains recorded history for every event, including interrupted or older overlapping runs. Historical evidence is never removed merely because another event now owns that window.

For the future proposal, the rail uses the same selector, preserves the selected owner until it ends, and then selects another eligible event. There is at most one proposed rhythm for a given instant. A gap clears inherited rhythm. A containing event can take over the remaining window after a nested owner ends. Planned bands are proposals and do not create recorded focus or breaks.

An accepted active phase uses its recorded start, remaining duration, pauses, and current configuration for projection. An untracked commitment begins its proposed rhythm at the current instant when its scheduled start has already passed. It does not fill the missed interval as completed work.

## Examples

Event A spans 09:00 to 12:00. Event B spans 10:00 to 11:00.

- At 10:15 with no active run, both scheduler and rail select B because it ends first.
- If A already owns an accepted run when B begins, both retain A while it is eligible.
- If B owns the run, its recorded evidence and projection remain visible despite containment by A.
- At B's end, the remaining A window becomes eligible for selection. A new executing interval still requires admission.
- In a proposal made before 09:00, A is selected first and retains its proposed window when B begins. This is a forecast conditional on that earlier start, not evidence that A executed.

For equal windows, creation identity and occurrence ID decide; shorter focus duration does not affect ownership.

## Event changes and limits

Moving, resizing, archiving, deleting, or changing eligibility invalidates selection. The running timer still handles reconfiguration and run closure through its transition controller. Full migration of those transitions into Rust remains active work. A real gap starts a fresh run; inheritance across a configurable small gap is not implemented.

Fixtures cover nested owners, late arrivals, equal windows, input permutations, cancellation, exact end boundaries, retained recorded history, and sequential inheritance. Cross-device schedule conflicts will suspend automatic activation once replication is implemented.
