# Cross-cutting hazards

These scenarios cross feature, persistence, and platform boundaries. They are not substitutes for tests or invariants. They explain failure sequences that should be considered whenever adjacent behavior changes.

## 1. Event boundary timing

**Scenario.** One calendar block ends exactly when another begins. The first run must close at the boundary before the second block starts or inherits progress. A delayed scheduler may process both after the fact, but it must preserve boundary order and must not count the delay twice.

Current inheritance applies across adjacent or overlapping eligible blocks. A real gap starts a fresh session. There is no configurable five-minute gap tolerance.

An event shorter than its focus phase ends the active segment as interrupted at the block boundary. No future break row is created merely because the derived plan contained one.

**Governed by:** [Pomodoro state machine](../algorithms/pomodoro/state-machine.md), [Pomodoro schema](schema/pomodoro.md), and invariant 6.

## 2. Concurrent event edits during a run

**Scenario.** The user shortens, moves, reconfigures, archives, or deletes the calendar event that owns the active run.

The timer must re-evaluate eligibility and the new block deadline. Elapsed segments remain immutable. A shortened event can interrupt the active phase immediately. Moving an event away from now cannot leave an orphaned running timer. A config edit uses elapsed progress against the new phase duration and persists the reconfiguration before a new phase begins.

Protected events use archive or domain-specific detach behavior instead of hard deletion.

**Governed by:** [Pomodoro state machine](../algorithms/pomodoro/state-machine.md), [Calendar schema](schema/calendar.md), and invariant 6.

## 3. Crash, suspend, and process kill

These cases require different evidence:

- During a live suspend, the frontend and native lifecycle create or normalize suspend state and block ordinary ticking until the return decision is complete.
- On desktop cold startup, stale open state is bounded by the last valid heartbeat and persisted pause evidence. Orphaned work is interrupted rather than extended to the current time.
- On Android cold startup, recovery may use the current time only within a previously committed phase and its accepted deadline. A native reminder or projection supplies no execution evidence. Expired state closes conservatively.
- An open manual or idle pause remains paused after valid recovery. Time away does not become focus time.

Recovery writes must be transactional and idempotent. Repeating startup recovery cannot create another segment, pause, or terminal run event.

**Governed by:** [Pomodoro state machine](../algorithms/pomodoro/state-machine.md), [Plan and history](../algorithms/pomodoro/plan-and-history.md), and invariant 5.

## 4. Overlap, containment, and ownership

**Scenario.** Several Pomodoro-enabled events overlap, including a short event fully nested inside a longer one.

The already-active eligible event must remain the owner. Switching merely because another candidate starts would split one real session into artificial fragments. When there is no active owner, selection must be deterministic and the calendar rail must not show competing bands for the same time.

The scheduler and rail now share the same selector: keep the eligible current owner, then use earliest end, creation identity and occurrence identity. Containment no longer removes an accepted owner. Future proposals preserve their selected owner to its end, while recorded history remains visible even when older runs overlap. Full transactional migration of the live transition controller to Rust remains pending.

**Governed by:** [Time conflict detection](../algorithms/calendar/time-conflict-detection.md) and invariant 4.

## 5. DST and timezone boundaries

Calendar identity uses instants plus the event's home-zone and local recurrence representation where needed. A local clock label is not an elapsed duration.

For example, in New York a spring interval from 01:30 to 03:30 crosses the missing hour and spans one elapsed hour. During the fall transition, 01:00 occurs twice, so 01:00 to 03:00 spans either two or three elapsed hours depending on which 01:00 instant is selected. Parsing must use an explicit disambiguation policy.

Recurring events walk civil dates in their home zone. The same local start should remain the same local start across a DST change even though its UTC offset changes. History rows preserve their original instants; changing the device zone affects display, not stored history.

**Governed by:** [Calendar schema](schema/calendar.md), [Recurrence expansion](../algorithms/calendar/recurrence-expansion.md), and calendar formatting services.

## 6. Rapid and repeated actions

**Scenario.** The user double-clicks start, skips twice, stops while a transition is committing, or two windows issue the same command.

Commands need stable receipts or state preconditions. Database uniqueness prevents duplicate active state, but callers must also handle the losing operation without showing success for a write that did not occur. Notification, overlay, and media side effects run only after the canonical transition commits and must tolerate repeated delivery.

**Governed by:** transactional command services, command receipts where defined, and invariants 1 through 3.

## 7. Multiple runs for one event

Stopping and restarting the same event creates another run. Every segment, pause, adaptive decision, and run event belongs to its exact run. Timeline projection may aggregate runs for display but must not merge their identities or infer one continuous session.

A recurrence instance also preserves template and occurrence identity. Two dates from the same template are not the same event occurrence merely because they share a title and configuration.

**Governed by:** [Pomodoro schema](schema/pomodoro.md), [Calendar schema](schema/calendar.md), and invariant 5.

## 8. Pause boundaries

Manual and idle pauses freeze the visible remaining duration. Resume moves the phase deadline by the effective pause duration. An indefinitely paused phase does not complete merely because its former wall-clock deadline passed.

A pause starting exactly at a phase boundary is assigned according to the committed transition order. The system must not close one phase, open the next, and attach the pause to both. Repeated pause or resume commands are idempotent.

Idle backdating cannot precede the segment start or overlap an already closed pause. Stop while idle closes the run at the appropriate persisted boundary without converting idle time into focus. Suspend handling takes precedence over ordinary idle detection when a large scheduler gap indicates that the operating system was asleep.

**Governed by:** [Idle detection](../algorithms/pomodoro/idle-detection.md), [Plan and history](../algorithms/pomodoro/plan-and-history.md), and invariant 5.

## 9. Reconfiguration chains

Reconfiguration compares elapsed active work with the new phase duration. If elapsed work already satisfies the shorter duration, the next boundary is immediately due. Increasing the duration preserves elapsed work and extends only the remaining amount.

Persisted past segments never change. Future projections derive from the new rhythm and recorded boundary decisions. Repeated reconfiguration must not create a cycle, lose inherited focus, or assign one segment to two rhythm positions.

**Governed by:** [Pomodoro state machine](../algorithms/pomodoro/state-machine.md), [Plan and history](../algorithms/pomodoro/plan-and-history.md), and invariant 3.

## 10. Notes placement and graph divergence

**Scenario.** A page move updates navigation but not project membership, a folder migration updates descendants in only one table, or history restore reintroduces a stale parent.

Placement, project association, folder ancestry, links, and search projections must change through one domain transaction. Cycle checks run against the resulting graph, not only the submitted parent. Restore preserves the current placement unless the restore operation explicitly includes a validated move.

Filesystem exports do not repair or override the canonical Notes graph. Disposable search and derivative Markdown are rebuilt after canonical restore.

**Governed by:** [Notes and projects schema](schema/notes-and-projects.md) and invariant 8.

## 11. Working-folder identity drift

An external path can later point to a different directory because of replacement, mount changes, symlinks, or path reuse. Git's common directory can also change independently of the working tree.

Every privileged operation revalidates the device-local folder identity. Ordinary file authority may remain valid when only Git identity changed, but Git, checkpoint, diff, and restore operations fail closed. Missing or replaced folders never fall back to a similarly named path.

**Governed by:** [Chat access control](access-control.md), [Chat security](security/chat.md), and invariants 9 and 10.

## 12. Conversation and provider-session conflation

A provider continuation may fail, be replaced, fork, or disappear while the project channel remains valid. Provider cleanup must not cascade into organizational message, approval, assignment, or decision deletion.

Conversely, archiving a channel prevents new organization activity but does not pretend that a native provider process stopped. Process shutdown and cleanup remain explicit bounded operations.

**Governed by:** [Chat schema](schema/chat.md), [Chat access control](access-control.md), and invariant 11.

## 13. Permission leaks through derived data

Search, summaries, unread state, suggestions, exports, cached prompts, and synchronization envelopes can reveal restricted content even when direct reads are correct.

Every derivative declares its authorization inputs and invalidates when membership, history cutoff, profile revision, folder grant, or audience changes. Cache hits are denied if any authorization dimension is missing from the key.

**Governed by:** [Chat access control](access-control.md), [Chat security](security/chat.md), and invariant 12.

## 14. AI identity used as an authority bridge

Mentioning an AI teammate, assigning it a role label, or selecting a capable provider can create the false impression that the teammate now has broad project access. None of these actions grants membership, history, folder, terminal, Git, or application-tool authority.

Assignment review must show unresolved targets and denied capabilities without inferring them from natural-language instructions.

**Governed by:** [Chat access control](access-control.md) and invariant 13.

## 15. Restricted context retained by a continuation

After a membership, profile, folder, or scratch reduction, a provider continuation may still contain earlier restricted context. Filtering new reads does not remove that material.

The application interrupts affected work, rejects tools and publication, stops the provider, discards or quarantines the continuation, and requires a new authorization revision. Failed cleanup remains visible as retryable state without leaking native paths.

**Governed by:** [Chat access control](access-control.md), [Chat security](security/chat.md), and invariant 15.

## 16. Destination audience expands around a reference

A message in a restricted channel may be referenced from a broader channel whose participants overlap but are not a subset. Rendering the source title, excerpt, attachment thumbnail, or generated summary would leak information.

The destination receives an opaque unavailable reference unless strict audience and history checks pass. Explicit declassification creates a new destination-owned statement and retains provenance for audit; it does not silently relax the source.

**Governed by:** [Chat access control](access-control.md) and invariant 14.

## Planned work mistaken for execution

A scheduled phone event or cached phase projection must not generate running state, focus history, or phase-dependent enforcement while a desktop is unavailable. Native reminders have a separate contract with no run or phase fields. Recovery accepts only committed execution and caps history at the accepted phase deadline. Linked-device silence will mean unavailable status, never inferred focus or idle. See [Focus authority](../algorithms/pomodoro/focus-authority.md).
