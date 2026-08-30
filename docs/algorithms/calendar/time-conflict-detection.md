# Time conflict detection

Several Pomodoro-enabled calendar events may overlap the current instant. The application still has one global timer and must choose one owner for automation and one non-duplicated visual projection.

Current auto-start and timeline-band code do not use one policy. This document records both current behaviors and the invariants they must satisfy while the implementation gap remains open.

## Interval model

Timed event eligibility uses a half-open instant interval:

- start is inclusive;
- end is exclusive.

At exactly 10:00, an event from 09:00 to 10:00 is no longer eligible and an event from 10:00 to 11:00 is eligible.

An event participates only when it has a valid Pomodoro configuration and a finite valid range. All-day and imported event behavior follows the calendar feature policy before reaching this selector.

## Required invariants

Regardless of final tie-breakers:

1. An already-active event remains the owner while it is still eligible.
2. One time range does not display competing active or planned Pomodoro bands.
3. Equal inputs produce the same owner independent of input array order.
4. A visual projection does not imply that a different event owns the running timer.
5. Event edits and boundary crossings trigger prompt re-evaluation without a permanent poll loop.

## Current auto-start selection

Auto-start builds candidates that contain now, then applies:

1. Keep the active block when it remains a candidate.
2. Choose the candidate with the shortest remaining time until event end.
3. On equal remaining time, choose the earliest creation timestamp.
4. On another tie, choose the lexicographically earlier stable event ID.

Shortest remaining time prioritizes the event that will expire soonest. Creation time and ID exist only to make exact ties deterministic.

The helper also accepts a set of recently interrupted block IDs and can prefer those candidates before ordinary tie-breakers. The production calendar scheduler does not currently provide that set, so recently interrupted priority is not current runtime behavior.

The scheduler calculates the next relevant event start or end boundary and wakes at that deadline, with lifecycle and calendar invalidation as additional triggers. It does not use a one-second auto-start poll.

## Current timeline-band selection

The timeline projection first removes fully contained events:

- a longer containing range is retained over a nested range;
- for an equal range, the event with the shorter first focus duration is retained;
- remaining events are processed in start order;
- partial overlaps therefore give earlier ranges visual priority for already occupied time.

This behavior reduces duplicate bands but differs from auto-start. It can display the outer event while auto-start chooses a shorter nested event.

There is also an ordering defect: containment filtering occurs before lookup of the active event. An active nested event may be removed, so the later active-priority logic cannot recover it.

## Containment examples

### No active session

Event A spans 09:00 to 12:00. Event B spans 10:00 to 10:30.

- At 10:15, current auto-start chooses B because it has less remaining time.
- Current timeline filtering retains A because A contains B.

The UI and timer can therefore disagree. This is an implementation gap, not an intentional two-owner model.

### Active outer event

If A already owns the timer when B begins, A remains the auto-start owner while A is eligible. The visual projection must also preserve A as active.

### Active nested event

If B already owns the timer, B must remain the owner while eligible. Current timeline filtering may remove B before checking active identity. This violates the required invariant.

### Equal windows

Two events both span 10:00 to 11:00.

- Auto-start uses creation timestamp, then ID.
- Timeline filtering may use the first focus duration.

Input order must not decide the result, but the two consumers still need a shared semantic tie-breaker.

## Event changes

Moving, resizing, archiving, deleting, or changing Pomodoro eligibility invalidates selection. If the active event remains eligible, it stays stable. If it becomes ineligible, the current run closes or transitions according to the [Pomodoro state machine](../pomodoro/state-machine.md), then the selector evaluates remaining candidates.

A real gap between events starts a fresh run. Inheritance across a configurable small-gap threshold is not implemented.

## Resolution requirement

The durable goal is one shared ownership decision consumed by both auto-start and timeline projection. Resolving the gap requires an explicit product choice for:

- nested ranges when no event is active;
- equal ranges with different rhythm configuration;
- whether recently interrupted events receive priority;
- how partial overlaps are clipped for display after ownership is selected.

Whichever policy is selected must keep active ownership first, use stable deterministic tie-breakers, and be tested through both automation and projection fixtures. Until then, do not describe the current consumers as equivalent.
