# Pomodoro state machine

The pomodoro feature's behavior is governed by a small set of pure decision functions. Each function takes the current state (the run, the active segment, recent ticks, the calendar context) and returns a decision (continue, advance, transition, reconfigure, recover, do nothing). The functions have no side effects: they read state and return a decision; the caller applies the decision by writing to the database and updating the UI.

This doc covers the decision functions, why they are pure, the heartbeat that backs crash recovery, the named constants, and the recovery procedure. The user-facing surfaces are in `features/pomodoro.md`, the data model is in `algorithms/pomodoro-segments-and-plan.md` and `data/schema.md`, and the future adaptive optimization model is in `algorithms/pomodoro-adaptive-rhythm.md`.

## Pure decision functions

There are six decision functions. Each is a `(state) -> decision` mapping with no I/O.

### `decideTick`

Called every second while a session is active.

**Inputs:** the current run, the active segment, the time since the last tick, the current wall-clock time, the calendar event window.

**Returns:** one of:

- `tick`: derive the visible countdown from the phase deadline without persisting the visual tick. The derived value is clamped so a backward wall-clock adjustment cannot add focus or break time.
- `notify`: 60 seconds remain in focus; emit a notification.
- `advance`: phase ended, advance to the next phase.
- `expire`: event time expired, end the run (or transition if a consecutive event exists).
- `suspend_pause`: on desktop, tick gap > `SUSPEND_THRESHOLD_MS`, create a suspend pause. Mobile WebView tick gaps are normal background behavior and never create a pause.
- `noop`: nothing to do.

The order of checks inside `decideTick` is: desktop suspend first (because a long desktop tick gap means the rest of this tick's reasoning is based on stale state), then expire, then phase end, then notify, then normal tick. Android stops WebView-owned visual ticks while hidden and reconciles the native projection when visible again.

### `decideAdvancePhase`

Called when `decideTick` returns `advance`.

**Inputs:** the current run, the active segment, the run's rhythm snapshot, the current rhythm position.

**Returns:** the next phase (`focus`, `short_break`, `long_break`), the selected planned duration, and the rhythm position for the next focus or the break owed.

The rule is straightforward: focus asks the active rhythm which break is owed at the current position, and break advances to focus at the next rhythm position. The simple four-focus rhythm is one count plan: short, short, short, long, repeated. A sequence rhythm looks up the current step directly. The `skipNextBreak` flag, if set, causes the function to skip directly from focus to focus at the next position, recorded as a `skip_break` run event and the absence of a break segment between two focus segments.

Adaptive plans can silently select a different duration for a future phase, following `algorithms/pomodoro-adaptive-rhythm.md`, but this decision still belongs in `decideAdvancePhase`, `decideReconfigure`, or the boundary policy path immediately before a new started segment is written. Once selected, the duration is persisted through the segment's planned timestamps and adaptive decision rows so the plan remains auditable.

### `decideTransition`

Called when `decideTick` returns `expire` and the calendar has a consecutive or overlapping pomodoro event.

**Inputs:** the ending run, the ending segment, the new event, the time of the transition.

**Returns:** the inherited state (`inherited_focus_minutes`, `inherited_rhythm_position`) for the new run and the first phase of the new run (derived using the new event's rhythm snapshot and the inherited state, see `algorithms/pomodoro-segments-and-plan.md` "Plan vs segments").

If the calendar has no consecutive event, `decideTick` returns `expire` directly (without calling `decideTransition`) and the run ends with `end_reason = completed`.

### `decideReconfigure`

Called when the user changes the pomodoro config mid-session.

**Inputs:** the current run, the active segment, the time of reconfiguration, the new config.

**Returns:** the inherited state for the new run and the bridge segment specification. Running focus is not silently shortened by a config change. The elapsed time is preserved against the new phase duration, which may leave zero remaining and then advance at the next boundary. Break reconfiguration preserves elapsed break time in the same way.

The reason for preserving the current phase: a reconfiguration mid-focus should let the user finish their current train of thought, not snap them into a break or a fresh focus immediately. The new config controls future boundaries and projections, but the active focus is not cut short without the user explicitly ending it.

### `decideIdleCheck`

Called by the threshold-aware idle scheduler during focus.

**Inputs:** the current run, the active segment, the user's idle time (from per-OS detection sources, see `algorithms/idle-detection.md`), the run's config (`idle_timeout_minutes`).

**Returns:** one of:

- `idle_pause`: idle time exceeds threshold, create an idle pause.
- `noop`: idle time is below threshold, or threshold is null, or active phase is a break.

The function does not check whether a pause is already active; the caller does that to keep the function pure.

### `decideStartFromBlock`

Called by the auto-start poll (every ~1 second, on app open, on calendar change).

**Inputs:** the calendar's pomodoro events overlapping `now`, the currently running session (if any).

**Returns:** one of:

- `noop`: same block, same config, same end. Nothing changed.
- `update_end_only`: same block, end time changed (the user resized the event). Update the stored end time.
- `reconfigure`: same block, config changed. End the current run with `reconfigured`, start a new one.
- `transition`: different block. End the current run with `block_transition`, start a new one with inheritance.
- `new_session`: no existing session. Create a new run starting fresh.

The selection of which event to pick when multiple overlap follows the auto-start tiebreakers in `algorithms/time-conflict-detection.md`.

## Why pure functions

Keeping the decision logic in pure functions has several benefits.

**Testability.** A test can construct any combination of inputs and assert the decision, without needing a database, a calendar, or wall-clock time. This is how the corner cases (tick exactly on event end, reconfigure during a paused break, transition with no inheritable state) get covered.

**Predictability.** Two reads of the same state produce the same decision. Bugs in the side-effecting layer (the writer) cannot corrupt the decision logic; bugs in the decision logic surface as test failures, not as data loss.

**Auditability.** Given a run's history (segments, pauses, end_reason), the decisions that were made are reconstructable by replaying inputs through the same functions. This is useful when investigating "why did the timer do X" reports from users.

**Safe to call multiple times.** A pure function returning the same decision twice is a no-op for the caller (the writer can guard with idempotency checks). This makes recovery easier: replaying ticks during a recovery scan does not corrupt state.

The opposite design (decisions interleaved with writes) would make all of the above harder. Tests would need a database. Bugs could create dirty state that hides real bugs. Replaying would risk double-writes.

## Heartbeat

`pomodoro_runs.last_heartbeat` is updated approximately every 30 seconds while the session is active. Desktop crash recovery uses it as the last confirmed live instant when an unexpected exit leaves a run open. Android also validates the heartbeat, but routine operating-system process eviction is not treated as proof that the user stopped the timer. Mobile cold recovery reconstructs current phase progress from persisted segment and pause timestamps when the state is still safe to resume.

The one-second display scheduler is intentionally separate from persistence and native side effects. Segment rows and plans change only at phase transitions, pause or resume actions, reconfiguration, extensions, and session closure. The heartbeat is the only bounded write during an uninterrupted segment. Tray updates are coalesced by structural state and rendered percentage, while break overlays receive an absolute end timestamp and do not require per-second IPC.

Heartbeat properties:

- **Frequency:** every ~30 seconds. Chosen to bound the recovery error to ~30 seconds while keeping write traffic minimal.
- **Atomic single-row update.** Each heartbeat is one `UPDATE pomodoro_runs SET last_heartbeat = ? WHERE id = ? AND ended_at IS NULL`. SQLite handles this in a microsecond or two, no contention.
- **Independent of segment writes.** Heartbeats fire on their own schedule, not tied to phase boundaries. This way a crash mid-segment still has a recent heartbeat.

Without the heartbeat, desktop recovery would have to use the active segment's planned end (overestimating focus time on a crash mid-phase) or now (catastrophic if the app crashed and reopened hours later). The heartbeat gives desktop closure and invalid mobile-state fallback a tight, persisted bound. A valid Android recovery instead uses the current time only after proving that the event window and active phase have not expired.

## Constants

| Constant | Value | Used by | Reason |
|----------|-------|---------|--------|
| `SUSPEND_THRESHOLD_MS` | 15000 (15 seconds) | `decideTick` | A normal tick gap is ~1 second. 15 seconds is unusual enough to flag as suspend without misclassifying brief CPU starvation. |
| `NOTIFICATION_THRESHOLD` | 60 (seconds) | `decideTick` | Gives the user a one-minute heads-up before focus ends so they can reach a stopping point. |
| `MAX_BREAK_OVERTIME_SECONDS` | 1800 (30 minutes) | break-end logic | Caps overtime at 30 minutes. After that, the system auto-advances to focus to prevent indefinite breaks from corrupting analytics. |
| `HEARTBEAT_INTERVAL_MS` | 30000 (30 seconds) | heartbeat scheduler | Bounds crash recovery error to ~30 seconds. |
| `AUTO_START_POLL_MS` | 1000 (1 second) | auto-start scheduler | Catches calendar boundary crossings promptly enough that event-start notifications and pomodoro auto-start feel aligned with the system clock. |
| `IDLE_CHECK_MIN_INTERVAL_MS` | 1000 (1 second) | idle scheduler | Bounds threshold detection delay without polling every second during the full focus period. |
| `IDLE_CHECK_MAX_INTERVAL_MS` | 15000 (15 seconds) | idle scheduler | Keeps normal active-focus polling coarse until OS-reported idle time gets close to the configured threshold. |

These values are constants, not user settings, because changing them changes the meaning of the timestamps in the database. Users who want different thresholds (e.g. longer suspend tolerance) would need a code change. The trade is intentional: a stable schema is more valuable than a knob no one will turn.

## State diagram

A run lives in one of four high-level states. Transitions are triggered by the decision functions.

```
                  decideStartFromBlock returns new_session
                 ─────────────────────────────────────────
                                                          │
                                                          v
   ┌─────────────┐    decideTick: noop/tick/notify    ┌────────┐
   │   IDLE      │ <───────────────────────────────── │ ACTIVE │
   │ (no run)    │                                    └────────┘
   └─────────────┘                                       │  ^
       ^   ^                                             │  │
       │   │  end_reason set                             │  │
       │   │  (completed, stopped,                       │  │
       │   │   interrupted, reconfigured,                │  │
       │   │   block_transition)                         │  │
       │   │                                             │  │
       │   └─────────────────────────────────────────────┘  │
       │                                                    │
       │       decideAdvancePhase, decideTransition,        │
       │       decideReconfigure                            │
       │                                                    │
       │              ┌──────────┐                          │
       │              │ ACTIVE'  │ (new run from inherited) │
       │              └──────────┘ ─────────────────────────┘
       │                   ^
       │                   │
       │   ┌────────────┐  │  decideIdleCheck: idle_pause
       └── │ PAUSED     │ <┘
           │ (active +  │
           │  open      │
           │  pause)    │
           └────────────┘
                ^
                │
                v
           User resumes (closes pause)
```

In ASCII this is approximate. The states are:

- **IDLE**: no active run. The system polls auto-start.
- **ACTIVE**: a run is open with an active segment. `decideTick` fires every second.
- **PAUSED**: an active run with an open pause. `decideTick` is suppressed until the pause closes.
- **ACTIVE' (transition target)**: a new run created from an old run via `decideTransition` or `decideReconfigure`. Functionally an ACTIVE state, distinguished here only because the transition is the most subtle state change.

A session can move from ACTIVE to PAUSED and back many times within a single segment. A session moves from ACTIVE to a new ACTIVE' (closing the old run) only on transition, reconfiguration, or end-and-restart.

## Recovery procedures

Recovery runs before calendar-driven session work can observe persisted Pomodoro state. Desktop and mobile deliberately apply different policies because an Android process can be removed while the user reasonably expects a deadline-driven timer to continue.

### Desktop orphan cleanup

Desktop startup closes every run left with `ended_at = NULL`:

1. Use `last_heartbeat` as the last confirmed live instant.
2. Set the run to `end_reason = interrupted` at that instant.
3. Interrupt its active segment and close any open pauses at the same normalized instant.
4. Record `crash_recovery` in `pomodoro_run_events`.

After cleanup, `decideStartFromBlock` can create a fresh run for a current calendar event. Desktop never reopens a row it already closed. An unexpected desktop exit is treated as a broken concentration boundary, and the heartbeat bounds lost work to approximately 30 seconds.

### Android cold recovery

Android cold startup performs one typed reconciliation transaction before Calendar loads:

1. No open run returns `none` without changing history.
2. Multiple open runs violate the single-open-run invariant. Recovery closes all of them as interrupted with reason `multiple_open_runs`.
3. One open run is resumable only when its run window, live event reference, rhythm snapshot, settings, timestamps, single active segment, and pause chronology are all valid. Invalid state closes as interrupted with reason `invalid_state`.
4. A matching Android projection is accepted only when its bounded run, event, event date, event deadline, generated timestamp, unique segment identifiers, rhythm positions, phases, durations, and contiguous boundaries validate against the open SQLite run. The first projected phase must match the single active segment and its proven deadline. Invalid projection data is ignored.
5. If a valid native projection proves that one or more boundaries elapsed, recovery completes and inserts those segments in the same transaction, records phase completion and start events, then rechecks the resulting single active segment. Replaying the same startup cannot duplicate those writes because the original active segment no longer matches the projection's first phase.
6. If the calendar event window has expired, recovery closes the run at its persisted event deadline as completed with reason `run_window_expired`. A valid native projection is replayed first so completed background phases remain in history.
7. If a valid active phase still has work and event time remaining, recovery returns `resumed`. Running elapsed time is derived from the active segment's actual start minus closed pauses. An open manual or idle pause remains paused and time away does not count. Android compatibility recovery normalizes legacy suspend pauses created by background WebView throttling to zero duration, so that time continues to count. Visible remaining time is capped by both the phase and event deadlines.
8. If the active phase expired without a valid matching native projection, recovery closes the run as interrupted at the proven phase deadline with reason `phase_expired`. It does not invent unobserved phase transitions.

The transaction either returns a validated in-memory snapshot or closes unsafe persisted state. Closure updates the run, active segment, pauses, and audit event together. Repeating recovery sees no open rows after a closure, while a resumed row remains open and can be reconstructed again after another process eviction.

This policy resumes an existing open row in place. It never closes and then reopens history. It preserves a valid Android timer across Activity and process removal without claiming that focus continued across an expired phase, malformed state, or mismatched native projection.

### External tools and recovery

External tools (CLI exports, analytics scripts, and backup utilities) might read the database while the app is not running, including before platform startup recovery has run. An open row is pending runtime reconciliation, not proof of either completed work or a crash. Read-only tools should exclude it from finalized analytics or label it as open. They must not silently apply desktop heartbeat closure to a mobile vault because that would destroy a resumable phase.

The future `ganbaru-ai` CLI should expose this state explicitly. A separately authorized repair command may invoke the platform-neutral validation and closure path, but ordinary exports and third-party readers must not mutate the database as a side effect of reading it.

## Why the state machine is the source of truth

Every database write that the pomodoro feature performs traces back to a decision function returning a non-`noop` decision. The application code that runs the writes does not invent new transitions; it executes what the functions return. This makes the state machine the single source of truth for "what can happen to a session." Adding a new transition (e.g. a new end_reason) means adding a new decision function output, then handling it in the writer.

This separation also makes the cross-cutting hazards (see `data/hazards.md`) tractable: each hazard maps to one or more decision functions, and the test surface for the hazard is the same as the test surface for those functions.
