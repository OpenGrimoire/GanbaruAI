# Data invariants

These hold across the whole app. Any operation that can break one is wrong, regardless of which feature it belongs to. The list is numbered for cross-referencing from feature and algorithm docs; the numbers are stable.

A violation is a bug in whatever code produced it: the data layer, the derivation logic, or the renderer's visual math. The renderer must not paper over bad data with cosmetic clamps, and the data layer must not lean on the renderer to hide impossible state. Each side has to be correct on its own.

## 1. Green never appears after the current time

A focus fill band on the rail can only exist for a segment with `actual_start` in the past and recorded work. Any time the rail shows green beyond the current moment, something is wrong: the segment data, the projection logic, or the visual math that maps time to pixels under the current scroll position, calendar zoom, or app scaling.

**Why:** the rail is a record of what happened, not a forecast. Green ahead of "now" would suggest progress the user has not actually made, undermining the trust the rail builds.

**What would break:** the user could be misled into thinking they have already focused, leading them to skip a session they meant to do. Analytics derived from green time would inflate.

**Enforced by:** segment fetch (no future timestamps from the database), the active-segment renderer (clamps to `now`), the projected-band renderer (emits break marks only, never green), and a single time-to-pixel transform shared by the `now` indicator and any green band. That transform must stay correct under every viewport state: scroll position, calendar zoom (25 to 200 px/hour), and app or OS scaling. A green pixel past the `now` line under any zoom or scroll combination is a real violation, not a rounding artifact to ignore.

## 2. Exactly one segment is `active` at a time

Across all runs in the database, at most one segment row carries `status = active`. Two would mean the timer is running two things simultaneously.

**Why:** the timer state machine assumes a single active phase. Pauses, transitions, reconfigurations, and crash recovery all key off the active segment.

**What would break:** double pauses on the wrong segment, transitions that end the wrong run, recovery picking the wrong segment to mark interrupted.

**Enforced by:** session start/transition/reconfigure code paths (always close the previous active segment before creating a new one), and crash recovery (closes any stale active segments on startup).

## 3. Break positions are stable within a session

For a running session, the planned positions of upcoming breaks are deterministic from the run's `started_at`, rhythm snapshot, and inherited state (`inherited_focus_minutes`, `inherited_rhythm_position`). Once the session is running, these positions do not shift.

For events without an active session, projected break marks are computed from "now" and naturally shift each tick. This is a different surface, see `features/pomodoro-progress-displays.md`.

**Why:** if break positions slid around while the user was working, the rail would feel unreliable; the user would not know whether the break mark in front of them was a real commitment or a moving target.

**What would break:** users would lose trust in the schedule. The state machine's assumption that "next break is at minute X" would be invalidated mid-session.

**Enforced by:** plan derivation reads only the run's snapshot fields, never "now," for active sessions.

## 4. No duplicate bands in the same time range

The rail shows one coherent schedule at any point in time. Two overlapping events must never both contribute bands (green or break) to the same minute range.

**Why:** the user must not see, for example, a 25/5 break cadence and a 40/5 break cadence interleaved over the same hour. That makes the rail unreadable.

**What would break:** visual ambiguity, conflicting break notifications, double-counting in analytics.

**Enforced by:** timeline band computation (containment filter and active-event suppression, see `features/pomodoro-progress-displays.md`).

## 5. Persisted data is the source of truth for the past

The rail renders past time from segment records, never from re-computation of what "should have happened." If a session was interrupted, the green stops where it stopped. If a break was skipped, no break band appears for that slot.

Future time is rendered from config-based projections derived from the run's `started_at`, config snapshot, and inherited state. The planned schedule is never stored as separate rows because it is fully deterministic from these fields. See `algorithms/pomodoro-segments-and-plan.md` for the derivation.

**Why:** treating the past as truth is the foundation for honest analytics. Rebuilding it from "what should have happened" would mask gaps and inflate focus time.

**What would break:** AI suggestions and stats would optimize for the imagined ideal instead of the user's real patterns.

**Enforced by:** rail rendering reads segments for past time and pauses for green-fill splitting; never recomputes.

## 6. Past progress is never erased

Once a segment has `actual_start` set and its status is `completed` or `interrupted`, no user action may delete, overwrite, or hide it. Skipping a break, stopping the session, dismissing the idle overlay, reconfiguring pomodoro settings, the app closing unexpectedly, deleting a calendar event, or archiving the calendar event: none of these remove previously recorded work. There is no mechanism to delete individual segments.

**Why:** the system is honest with the user about their patterns. Letting users (or operations) wipe out evidence breaks the contract that the app is a record, not a manipulable narrative.

**What would break:** users would learn to "clean up" sessions they regret, defeating the anti-procrastination feedback loop.

**Enforced by:** absence of any delete-segment API. Even structural calendar operations (detach, split, template-wide edit) preserve segments by transferring run references rather than dropping them.

## 7. Protected events are never deleted

A calendar event that has started, is in progress, is in the past, or has pomodoro tracking can only be archived, never hard deleted. This applies regardless of whether the event has completed focus segments. An event where the user planned to focus but never opened the app is still valuable: the absence of work on a planned block is itself a procrastination pattern. Only future events with no run or segment history can be truly deleted.

**Why:** this is invariant 6 generalized to events. The shape of the user's schedule is part of the historical record; deleting past blocks rewrites the past.

**What would break:** analytics would lose context (what was planned versus what happened). AI estimates would mistake the absence of an event for the absence of an attempt.

**Enforced by every programmatic boundary:**

- **UI:** protected events show archive behavior instead of delete. Active pomodoro events show End event first; delete and archive become available only after the run is closed and the event is past.
- **CLI (`ganbaru-ai`):** delete commands on past events are rejected with a descriptive error pointing to archive. The rejection is logged with timestamp, command, and event ID.
- **MCP handlers:** event deletion handlers refuse past events at the handler level and return a structured error including the archive alternative.
- **Internal Tauri commands and database layer:** `calendar_delete_event` accepts the concrete rendered identity and rejects protected rows with an archive-required error. Hard delete is allowed only when the exact event or occurrence is future-only and untracked. `calendar_archive_event`, `calendar_clear_events`, and `calendar_remove_calendar` snapshot protected rows into archive tables and null live pomodoro FKs.
- **AI agent integration:** system prompts and tool descriptions communicate the policy. Repeated rejection attempts by an agent are logged for diagnostic purposes.

The user owns the SQLite file and can modify it directly with a third-party tool. The app does not attempt to prevent that. But every code path inside the app must refuse.

Recurring events have additional protection: structural changes that would cause protected occurrences to silently stop expanding (an EXDATE on a protected date, an UNTIL moved earlier, a pattern change that excludes protected dates) must preserve those occurrences first. This is not a visible-window-only rule. For supported recurrence rules, structural edit code must reason over all affected occurrences from the template start through the captured edit time, using each occurrence's start time rather than only its date. Same-day occurrences that already started are protected; same-day occurrences that have not started and have no tracking remain mutable. A capped historical template is preferred when it can preserve the protected range without changing its meaning. Detached standalone events or archive snapshots are required when an occurrence needs its own event ID or cannot be represented safely by the capped template. Delete/archive requests and recurrence edit saves use one semantic frontend plan and one atomic backend batch so protected archive snapshots, detachments, template caps, splits, and active Pomodoro reference transfers cannot partially apply. The frontend may build occurrence materialization payloads because it owns live preview and wall-clock edit semantics; the backend remains authoritative for persisted writes and invariant enforcement. Occurrences with runs, segments, overrides, exceptions, active sessions, or persisted references are always protected. See `features/calendar-recurrence.md`.

## 8. Notes folder placement has one valid owner path

A Notes folder belongs to exactly one project, folder parents stay inside that project, and the folder graph is acyclic. A page may have a folder id only while its canonical parent is `workspace`, and that folder must belong to the page's project. Page-parented, block-parented, and data-source-parented pages never carry folder placement.

**Why:** folders organize the page navigation graph without replacing the Notion-compatible page graph. Allowing both paths at once would make one note appear to have two locations and would desynchronize paired child-page blocks.

**What would break:** navigation could duplicate or lose pages, folder deletion could affect another project, imports and exports could infer the wrong page hierarchy, and project history could restore an invalid graph.

**Enforced by:** SQLite foreign keys and placement triggers, folder create and update validation, the atomic page move command, defensive mixed-tree planning, migration invariant tests, and focused folder and page-movement tests.

## 9. Every Chat run has one native execution target

**Statement:** every provider run resolves exactly one current-folder, existing-worktree, or private-scratch native execution target. A target stays locked during an active continuation. A run may read or edit other explicitly granted project folders only through application-brokered tools, and a communication surface never inherits an execution target.

**Why:** commands and native provider filesystem access need one stable root, while communication and bounded context may span several separately authorized resources. Keeping those identities separate prevents provider convenience from becoming an organizational permission boundary.

**What would break:** a provider continuation could resume in another repository, folder-specific trust could leak across contexts, concurrent runs could collide, a channel could become unreadable when one folder disappears, or changing a membership default could retarget active work.

**Enforced by:** typed execution-target records, assignment authorization revisions, one-target dispatch resolution, continuation scope digests, folder grants, private scratch scopes, execution-environment reservations, and focused target-inference tests. No conversation row owns or inherits an execution folder.

## 10. Working-folder filesystem access stays bounded

**Statement:** the frontend passes an authorized folder, scratch, or execution-environment identity and normalized relative path, never an arbitrary root path. Rust recanonicalizes the device-local binding and rechecks the directory's filesystem identity before every filesystem-sensitive operation. Git-sensitive operations additionally recheck the Git common storage identity. Secondary folders remain behind bounded broker tools, and shell commands run only in the selected target.

**Why:** folder selection grants a narrow project capability, not general filesystem access.

**What would break:** traversal, symbolic-link escapes, stale bindings, folder replacement, Git storage replacement, or vault overlap could expose or modify data outside the selected context.

**Enforced by:** the shared Rust authorization boundary, device-local vault and device scoping, vault-overlap validation, symlink rejection, bounded Markdown scanning, expected revision saves, and authorization tests.

## 11. Organizational conversations outlive provider sessions

**Statement:** a channel, DM, task discussion, or reply thread has stable identity and durable history independently of any provider instance, model, provider continuation, working folder, or agent run. Replacing, forking, archiving, losing, or deleting an execution session cannot silently replace, merge, fork, or delete its organizational conversation.

**Why:** people organize around purposes and participants, while providers and execution contexts are replaceable. Treating both as one record recreates isolated chat navigation and makes long-term project memory depend on a vendor session.

**What would break:** a provider change could create a fake new relationship, context compaction could fragment a channel, unrelated legacy chats could merge into `#general`, or deleting execution artifacts could erase decisions and provenance.

**Enforced by:** separate conversation, assignment, agent-run, and provider-thread identities; immutable materialized communication revisions; exact run provenance; protected `#general` creation; foreign keys that do not make a run the room owner; and lifecycle tests covering provider replacement and archive. Pre-user development vaults are reset instead of receiving a speculative legacy-thread migration.

## 12. Effective access applies to derived context

**Statement:** if a participant cannot read a resource directly, no search result, mention, backlink, notification, count, dashboard, report, export, summary, manager proposal, or AI context package may reveal its content or existence beyond a permission-safe generic result.

**Why:** AI and aggregated views can leak restricted information without opening the original record. Future channel and Notes restrictions are meaningless if a model can summarize inaccessible content into an authorized room.

**What would break:** a restricted collaborator could infer private Notes, tasks, channels, working folders, personal productivity measurements, or participant activity through generated or aggregated output.

**Enforced by:** future resource grants and membership tables, authorization before query and derivation, permission-scoped indexes or post-query filters with non-leaking counts, context-package manifests, destination-scope checks, revocation tests, and audit records. The local single-user implementation uses the same APIs with one effective owner rather than bypassing the boundary.

## 13. An AI mention never expands authority

**Statement:** mentioning or adding an AI teammate can invoke only the intersection of the requester's invocation authority, destination visibility, teammate principal grants, explicit resource grants, run grants, budgets, and provider safety policy. Channel membership and a shared teammate display identity cannot widen filesystem, Notes, Calendar, provider, external-service, or cross-conversation access.

**Why:** a mention is a communication action, not a credential delegation or permission grant. Persistent teammates must remain useful across channels without becoming a bridge between otherwise separate resources.

**What would break:** a teammate addressed in a legal channel could edit an engineering codebase, a restricted collaborator could cause private Notes to enter a shared thread, one channel could spend another channel's budget, or an organizational teammate could act with the tagger's personal credentials.

**Enforced by:** separate participant membership and AI access records, teammate principals, access-profile ceilings, typed work-assignment preflight, frozen references, exact folder grants, layered budget checks, verified provider enforcement, permission-safe denial results, and audit tests covering cross-channel and cross-resource invocation.

## 14. Cross-channel disclosure never widens the audience

**Statement:** a channel reference is valid only when the requester and teammate can read its source, the teammate can participate in the destination, and the destination read-history audience is a subset of the source read-history audience. There is no override.

**Why:** channel access would be meaningless if an authorized reader could ask a teammate to summarize a restricted source into a broader destination.

**What would break:** private leadership, legal, security, customer, or personal information could enter channels whose readers were never granted the source.

**Enforced by:** permission-filtered candidates, send and schedule validation, assignment preflight, scoped history handles, result-publication validation, destination membership impact checks, frozen audience revisions, and retained-reference tests.

## 15. Materialized context cannot outlive its authority

**Statement:** a provider continuation, host-tool handle, or scratch generation can be reused only while every materialized source remains authorized for the same destination. Access contraction revokes the scope and never becomes reversible through a later expansion.

**Why:** provider continuations and scratch files retain data after the direct database read. Checking only future tool calls would let stale context launder revoked information.

**What would break:** a removed channel member, lost source grant, destination move, or reduced folder capability could leave restricted content available to a live provider or later result.

**Enforced by:** authorization-scope digests, source provenance, revocation records, active-run interruption, generic host-tool denial, suppressed publication, continuation discard, scratch quarantine, clean generations, and retryable cleanup jobs.

## Adding new invariants

When an operation reveals a constraint the system depends on but had not stated explicitly, add it here as the next number. Number reuse is forbidden; numbers may be marked deprecated but never recycled. Each new invariant gets the same five fields: statement, why, what would break, enforced by, plus any cross-doc links.

Operations are not invariants. "We always validate input" is a practice; "no segment may exist without a parent run" is an invariant. The test is whether the property must hold across every state of the database, regardless of which code path produced that state.
