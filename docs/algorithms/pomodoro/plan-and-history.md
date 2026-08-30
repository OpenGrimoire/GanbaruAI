# Pomodoro plan and history

A Pomodoro run stores facts about the session that occurred. It does not pre-create every future focus and break phase. Future plan is derived from the run snapshot, inherited state, current event boundary, and recorded adaptive decisions. Segments are written only when a phase starts.

This separation keeps stop, reconfiguration, calendar edits, and recovery from deleting speculative rows or rewriting history.

## Durable roles

### Run

The run is the session header. It snapshots event and recurrence identity, planned event window, actual boundaries, rhythm, idle setting, title context, start and end reason, heartbeat, and inherited state.

One event may have several runs. A cross-block transition creates a new run for the new event rather than moving older segments to it.

### Segment

A segment is a focus, short break, or long break that actually started. It records phase, rhythm position, planned and actual boundaries, selected duration, and status.

An active segment is canonical current phase. A completed segment reached its accepted boundary. An interrupted segment ended for stop, event expiry, focus failure, reconfiguration transition, crash recovery, or another recorded reason.

Planned and skipped frontend bands are projections. They do not become segment rows merely because the rhythm predicts them.

### Pause

A pause is an interval inside one segment. It records reason and start, with a null end only while open. Multiple non-overlapping pauses may occur in one segment.

Pause duration is excluded from active progress. Resume closes the pause and moves the phase deadline. A pause does not close the segment by itself.

### Run event

Run events explain lifecycle decisions such as skipped break, extension, reconfiguration, transition, stop, completion, focus failure, and recovery. They are append-only audit evidence, not a replacement state machine.

Adaptive choices use dedicated decision and outcome records in addition to any useful lifecycle event.

## Future plan derivation

For a run with no further adaptive change, projection starts from:

- run start;
- rhythm snapshot;
- normalized inherited rhythm position;
- inherited focus already accumulated;
- event end;
- persisted segments and pauses.

Count rhythm projection alternates focus and break, selecting a long break after the configured focus cadence. Sequence rhythm projection follows its bounded repeating positions and each position's focus and break definition.

Projection consumes persisted history first. It never generates a second band over an existing segment. It then derives only the unpersisted future and clips it at the calendar event boundary and current display window.

An adaptive boundary decision can change the selected value for a later phase. Once chosen, the decision and the new segment's planned timestamps are persisted together. Projection after that boundary uses the recorded value rather than recomputing a possibly different policy result.

## Inherited progress

Adjacent or overlapping eligible calendar blocks may continue one work rhythm while still creating a new run for organizational identity.

The outgoing run supplies:

- accumulated non-paused focus toward the current position;
- normalized rhythm position;
- whether a compatible break is active;
- source run identity for audit.

The new run stores the inherited values directly. It does not derive them later by traversing older runs.

If inherited focus meets the incoming focus duration, the incoming run begins with the appropriate break. Otherwise it begins or continues focus with incoming duration minus inherited focus. A real gap resets inheritance and starts fresh.

## Phase start transaction

Starting a phase commits together:

1. closure of the outgoing active segment if one exists;
2. its terminal reason and actual boundary;
3. any run event explaining the transition;
4. any adaptive boundary decision;
5. the new active segment and selected planned deadline;
6. heartbeat and run state needed for recovery.

No caller should observe two active segments or a persisted adaptive decision without the phase it selected.

## Paused time

Focus progress is elapsed instant time inside the segment minus closed and currently effective pauses. Wall-clock labels are not used for duration arithmetic.

Idle detection may backdate a pause to the inferred input-inactivity start. The start is clamped to the segment and cannot overlap prior pause evidence. Suspend lifecycle state is handled before idle so one away interval is not subtracted twice.

Stopping while paused closes the pause and segment at the domain-selected boundary. It does not convert paused time into focus. An indefinitely paused phase does not finish from wall time alone.

## Reconfiguration

Changing config during a phase preserves elapsed non-paused progress:

new remaining = max(0, new phase duration minus elapsed progress)

The old remaining duration is not preserved. Therefore:

- increasing duration extends only the amount still required under the new duration;
- decreasing duration shortens remaining time;
- decreasing below elapsed progress makes the boundary immediately due.

Completed and interrupted segments remain unchanged. The reconfiguration decision and any resulting transition are auditable.

## Event boundary

The calendar block end is a hard cap. If a phase would continue past it, the projection clips at the block boundary. At expiry, the active segment closes as interrupted unless it had already completed at the same instant under transition ordering.

No future break segment is written merely because a focus was planned to finish after event end. A following adjacent block may inherit eligible focus under its own new run.

## Plan versus actual

Analytics compares:

- the plan implied by the run snapshot and recorded boundary decisions;
- segments that actually started;
- pause-adjusted active duration;
- skips, extensions, focus failures, stops, and block expiry;
- later outcomes linked to adaptive assignments.

Do not reconstruct old plans from the event's current configuration. Use the run snapshot and persisted decisions.

## Worked examples

### Ordinary count rhythm

Configuration is 25 minute focus, 5 minute short break, 15 minute long break every four focuses. A fresh run starts at 09:00.

| Time | Durable result |
| --- | --- |
| 09:00 | Run and active focus segment at position 1 are created. |
| 09:25 | Focus segment completes. Short-break segment at position 1 starts. |
| 09:30 | Short break completes. Focus segment at position 2 starts. |

The position 3 and 4 plan remains derived until those phases start.

### Idle pause and resume

A 25 minute focus starts at 10:00 with a five-minute idle threshold.

| Time | Durable result |
| --- | --- |
| 10:15 | User input stops. Nothing is written yet. |
| 10:20 | Detection reaches threshold. Idle pause is created with start backdated to 10:15. Fifteen focus minutes have elapsed and ten remain. |
| 10:30 | User resumes. Pause closes after 15 minutes. Phase deadline moves to 10:40. |
| 10:40 | Focus completes with 25 active minutes and 15 paused minutes. |

The segment's wall-clock span is 40 minutes, but its focus duration is 25 minutes.

### Shorter reconfiguration

A 40 minute focus starts at 09:00. At 09:28 the user changes the current rhythm to 25 minute focus.

Elapsed progress already exceeds the new duration. Remaining becomes zero and the focus boundary is immediately due. The focus segment closes at the reconfiguration boundary and the new rhythm selects the next break or phase. It does not continue for the old 12 remaining minutes.

### Longer reconfiguration

A 25 minute focus starts at 11:00. At 11:10 it changes to 40 minutes. Ten active minutes are preserved, so 30 remain and the new deadline is 11:40, subject to event end.

### Event truncation

An event ends at 15:00. A 40 minute focus starts at 14:30. At 15:00 the event boundary interrupts the segment after 30 active minutes. No break segment is created. An adjacent eligible event may inherit those 30 minutes in its own run.

### Focus failure

Idle is detected and backdated to 13:15. The overlay remains unresolved for 60 seconds. The focus segment is marked interrupted at 13:15 with focus-failed reason. If the user chooses to restart and the event is still active, a fresh focus segment starts from the configured duration rather than reopening the failed segment.

## Recovery

Desktop cold recovery uses heartbeat and persisted rows as a conservative bound. Android may use valid native lifecycle evidence when JavaScript was suspended. Both paths preserve open manual or idle pauses and write terminal state idempotently.

Recovery never materializes an entire missed future plan. It closes or resumes the one active phase, then later projection derives what remains. See [Pomodoro state machine](state-machine.md).

## Required tests

Protect:

- count and sequence rhythm projection;
- long-break cadence and skipped breaks;
- persisted segment suppression of duplicate projections;
- multiple pauses and exact boundary pauses;
- adjacent inheritance and real-gap reset;
- shorter and longer reconfiguration;
- event truncation and simultaneous phase completion;
- adaptive decision and segment atomicity;
- repeated stop, transition, and recovery;
- plan reconstruction after current event config changes.
