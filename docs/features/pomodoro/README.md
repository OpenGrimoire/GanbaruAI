# Pomodoro

Pomodoro turns a calendar commitment into an adaptive sequence of focus and recovery phases. It protects attention without rewarding exhaustion or treating one fixed interval as universally correct.

**Status: Partial.** The timer lifecycle and platform surfaces are implemented. Calendar ownership alignment and explicit idle-source failure handling remain active correctness work.

## Current scope

| Capability | Status |
| --- | --- |
| Manual and Calendar-linked runs | Implemented with Calendar ownership hardening pending |
| Focus, short-break, long-break, pause, resume, stop, and recovery | Implemented |
| Presets and custom count rhythms | Implemented |
| Adaptive focus rhythm | Implemented with ongoing tuning |
| Idle and suspend detection | Implemented with a source-failure visibility gap |
| Break screen, title-bar ring, tray ring, Calendar rail, and Android notification | Implemented by platform |
| Native notifications and deadline recovery | Implemented |

## Philosophy

- A focus interval is a commitment opportunity, not a test of willpower.
- Breaks are required recovery, not a failure to continue working.
- The current calendar event is the hard scheduling boundary.
- Pauses, idle time, suspend, and interruption remain visible in history.
- Defaults adapt to observed local behavior without hiding the current plan.
- Metrics help the user tune work and recovery; they do not measure personal worth.

## Rhythm model

A simple rhythm defines focus count, focus duration, short-break duration, long-break duration, and which focus boundary receives the long break. Presets provide common patterns and custom configuration uses the same explicit fields.

Adaptive mode proposes a focus opportunity from local completed history and current context. It remains bounded by the active event deadline and a documented safe range. The visible plan states the current opportunity and next transition.

Detailed pure logic belongs in:

- [Adaptive rhythm](../../algorithms/pomodoro/adaptive-policy.md)
- [Segments and plan](../../algorithms/pomodoro/plan-and-history.md)
- [State machine](../../algorithms/pomodoro/state-machine.md)
- [Idle detection](../../algorithms/pomodoro/idle-detection.md)

## Run lifecycle

A run can be started manually or from a Pomodoro-enabled Calendar event. It records one coherent configuration snapshot so later preference edits do not rewrite an active or historical run.

The state machine advances by persisted deadlines. UI countdowns render those deadlines but are not the correctness boundary. Resume, process restart, device wake, and Android native recovery reconcile elapsed transitions from canonical state.

Only one active run owns the global focus surfaces. Starting another requires an explicit stop or handoff.

## Calendar boundary

An event-linked run cannot plan work beyond the event end. When less time remains than a normal phase, the opportunity clips to the deadline. Event deletion, archive, recurrence edits, and time changes follow Calendar active-session protection.

Historical runs retain exact original occurrence identity even when the live event is later archived or materialized.

## Pause and interruption

Manual pause freezes the active phase according to the state machine and records why. Idle pause and suspend recovery remain distinct because they represent different evidence.

Stopping records an interrupted or completed terminal state. Undoing a Calendar deletion never restarts a stopped run or removes history.

## Breaks

Break transitions can open a full break screen, use a smaller surface, or remain notification-only according to preference and platform. Overtime is explicit and never silently converts a break into focus.

See [Break screen](break-screen.md) and [Idle and suspend](idle-and-suspend.md).

## Progress surfaces

The title-bar ring, tray icon, Android ongoing notification, and Calendar rail share one semantic plan but present different detail. They never use contradictory progress definitions.

See [Progress displays](progress-displays.md).

## Notifications

Native notifications identify the current transition and provide only actions that the platform and state can perform truthfully. Desktop delivery depends on the running app lifecycle; Android native scheduling maintains phase boundaries while the Activity is absent.

Notification permission denial does not prevent timer use. Exact-alarm denial on Android uses the documented less-precise fallback and explains the consequence.

## Cross-feature behavior

- Calendar owns event timing and active-event identity.
- Music can apply phase soundtrack assignments and optional manual-pause ownership.
- Doomscrolling can apply fresh phase rule snapshots.
- Projects provide event and rhythm defaults.
- Work environments may later add context defaults without replacing run state.
