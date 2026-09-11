# Calendar schema

The calendar domain preserves editable local events, imported source fidelity, recurrence identity, notification state, project scheduling links, and Pomodoro history without making any one UI projection authoritative.

## Calendars and events

Every event belongs to a calendar. The built-in local calendar uses a stable semantic identity. Imported calendars receive their own identity so an import can be reviewed, removed, or repeated without mixing source ownership into the local calendar.

Current event rows hold the editable canonical projection used by the app. Archive rows preserve events that cannot be hard-deleted because history, project links, imported identity, or another durable relationship still refers to them. Archive is a data-preservation state, not merely a hidden UI filter.

Event start and end are instants with enough local and home-zone information to render and edit calendar intent. All-day events use date semantics. Callers must not infer elapsed duration by subtracting wall-clock labels across a timezone transition.

Calendar services validate positive ranges, visibility, ownership, recurrence consistency, and protected relationships before writing. Multi-row edits commit transactionally.

## Recurrence identity

A recurring series has three relevant identities:

- the template or original event;
- the civil recurrence instant selected from the rule;
- a concrete override or generated occurrence.

Pomodoro and project history retain both concrete event identity and original-series identity where needed. Editing an occurrence must not make earlier history point to a newly generated ID.

Normalized recurrence data stores the supported rule components, exception dates, additional dates, and overrides needed by the app. Raw iCalendar preservation is kept separately for source fidelity. The normalized projection drives current behavior; preservation rows allow export or future conformance improvements without pretending unsupported properties were applied.

Recurring edits use an explicit commit plan. Operations such as this occurrence, this and following, or entire series may create, update, split, archive, or detach several rows. The plan is validated completely before one transaction applies it.

Expansion semantics and current conformance gaps are documented in [Recurrence expansion](../../algorithms/calendar/recurrence-expansion.md).

## Import preservation

iCalendar import is staged. Parsing and validation produce a bounded plan before canonical rows are committed. Source calendar, component, property, parameter, timezone, alarm, organizer, attendee, attachment, and unknown-property preservation remain associated with stable imported identities.

Preservation data is not a second editable event model. When the user edits an imported event, the application updates the canonical projection while retaining enough provenance to explain or export source values according to the interoperability policy.

Import deduplication uses source identities and explicit replacement behavior. Titles, timestamps, or display order are not sufficient identity. A failed import leaves the previous calendar intact.

## Notifications

Calendar notification definitions are portable event data. Scheduled native alarm handles and delivery state are platform-specific projections. Reconciliation compares current canonical notification intent with platform capability and recreates missing native schedules where safe.

Delivery receipts and interaction state must be idempotent. A repeated platform callback cannot create duplicate application actions. Android exact-alarm and permission state remain device-local capability facts.

## Projects and Pomodoro

Project task scheduling links reference exact task and event identities and record link meaning. Moving or archiving an event does not erase the task. Deleting a task does not generically erase a protected event.

Pomodoro configuration is attached to the calendar event, but each run snapshots the rhythm and event context needed for history. Later calendar edits never rewrite elapsed runs or segments. Protected deletion and archive behavior is defined by invariants 6 and 7 in [Data invariants](../invariants.md).

## Deletion and repair

Hard deletion is allowed only when no protected relationship or preservation requirement remains. Domain services choose among hard delete, archive, recurrence detach, and relationship removal. Generic callers do not bypass that decision.

Repair routines may rebuild derived indexes and native schedules. They must not synthesize new event identity from mutable presentation fields or discard unknown imported data merely because the current UI does not expose it.
