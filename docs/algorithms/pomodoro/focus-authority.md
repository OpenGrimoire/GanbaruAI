# Focus authority and evidence

**Status: Partial.** Local desktop activity admission, Android commitment reminders, explicit scheduled starts, bounded accepted-phase notifications, and conservative Rust recovery are implemented. Linked-device ownership, remote commands, live companion status, and the full Rust transition runtime remain planned.

## Independent facts

A commitment is a Calendar plan. An executing run is an accepted interval with persisted state. Activity is an observation from an authorized local source or an explicit user confirmation. Freshness describes when a remote response was received. None implies the others.

A timer measures an accepted interval. It does not prove productive work. Calendar projections, phone backgrounding, screen-off time, and silence from another device are not activity evidence.

## Current local admission and recovery

Desktop automatic starts require an eligible Calendar event and a newly requested native activity observation. Input must occur at or after the relevant boundary and be no older than the typed 15-second activity limit. Missing, malformed, future or stale observations cannot authorize execution. The scheduler retries activity checks at bounded intervals only while an eligible commitment is waiting. Manual acceptance remains available when the observation source is unavailable.

Android starts scheduled sessions through explicit user action. Calendar alarms contain only reminder data. They cannot carry a run ID, phase plan, or running flag. Reminder receipts suppress repeated delivery through lifecycle reconciliation. Notification permission denial does not start a session.

A new session publishes running state and starts countdown effects after its initial database write succeeds. Failed initial persistence returns an error and leaves no executing timer. A late start uses its actual acceptance time.

Native Android notifications describe only the current accepted phase. At its deadline they remind the user to return to the app; they cannot advance into another focus interval or publish a later phase to Doomscrolling. Current accepted-phase storage uses a separate key from obsolete automatic activation projections.

A finished break waiting for return is not a paused accepted phase. It cannot extend native phase validity to the event end. New segment activation publishes after native persistence; insertion errors propagate and prevent countdown advancement. Complete outgoing and incoming transitions still require migration into one Rust transaction.

Rust recovery consumes canonical SQLite state only. It may resume a valid previously committed phase and its pauses, or close it conservatively. It never accepts a native notification projection as evidence of a run or subsequent phases. If the entire event elapsed, recorded time is still bounded by the accepted phase deadline. Repeated recovery does not create history.

Mobile recovery waits for queued local writes. Closed or absent runs clear cached execution without rewriting history. A resumed phase retains its accepted deadline across response latency; a response arriving after that deadline cannot reactivate it. Backgrounding stops JavaScript countdown, overtime, pause-window and heartbeat work until reconciliation.

Break completion enters the return or overtime flow. Waiting for any duration cannot start focus. The former 30-minute automatic return is removed. Starting focus again requires acceptance. The UI labels the return prompt “Ready to return.”

## Target execution ownership

A linked vault has one explicitly selected focus controller, defaulting to the pairing desktop. This is separate from coordination among windows on one device. Disconnection never elects a replacement; the current controller may continue offline. An unlinked phone supports explicit local sessions.

A live durable handoff must:

1. Commit final state and fence further execution under the old ownership epoch.
2. Issue a transfer identifying the recipient and checkpoint.
3. Persist the transfer on the recipient before it executes.
4. Retry the same transfer after lost acknowledgements without reactivating the old controller.

Lost-device recovery creates a new generation. Recovered records from the old generation remain historical evidence and receive overlap review. Remote commands require command ID, ownership epoch, expected run revision, and expiry. Only an acknowledged committed result is success; expired commands cannot run after reconnection.

The target `ganbaru-focus` service owns admission, transitions, pauses, recovery, history, ownership, and adaptive boundary decisions. Currently it owns persistence, validation, history, recovery and the desktop admission predicate. Svelte still owns live phase transition orchestration. Kotlin must eventually invoke Rust through a narrow bridge without requiring a WebView.

## Target companion freshness

Live status uses a separate connection from durable replication. During an active companion connection, send heartbeats every 15 seconds and expire status after 45 seconds without a fresh response. Use local monotonic time and connection generations, not peer clocks or a cached history row.

When status expires, show last confirmed state and “Status unavailable.” Do not infer focus, idle, failure or completed phases. Cached deadlines may produce clearly labeled scheduled reminders only. Idle warnings require a fresh valid observation from the controller. Phone activity does not reset the desktop idle clock or override webcam suppression and source failures.

Phase-dependent Music and Doomscrolling consume confirmed state with bounded validity. Independent local rules keep their own schedules. Android background restrictions must be visible as degraded connectivity.

## Verification

Protect PC-off event starts, late arrivals, missed break returns, source failure, future observations, exact deadlines, failed writes and repeated recovery. Controller work additionally requires stale generations, handoff interruption, lost acknowledgements, expired remote commands, overlapping recovered history, and proof that replication cannot cause native execution.
