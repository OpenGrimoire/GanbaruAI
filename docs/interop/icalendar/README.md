# iCalendar compatibility

This folder defines Ganbaru AI's iCalendar (`.ics`) interoperability model. It describes how the app accepts, preserves semantically, edits where possible, and exports calendar files without depending on Google, Outlook, CalDAV, email, or any hosted service.

The target is broad offline round-trip compatibility first. Ganbaru AI imports RFC 5545 iCalendar objects, retains unsupported structured data, projects supported `VEVENT` data into the calendar UI, and merges current projected fields over preserved source components during export.

Preservation is semantic, not byte-for-byte. Legal changes to property order, case, escaping, and line folding are acceptable when component meaning and unsupported structured data remain intact within the documented parser limits.

## Non-goals

Offline file compatibility is not the same as scheduling automation. The app can preserve RFC 5546 invitation metadata offline, but sending replies, cancellations, invitations, email alarms, or remote calendar updates requires a user-configured transport such as email, CalDAV, Google, or another provider. Those transports are optional future integrations.

File compatibility is also not the same as showing every component in the UI. `VTODO`, `VJOURNAL`, and `VFREEBUSY` can be preserved before the app has task, journal, or free/busy UI surfaces for them.

## Design summary

Use two layers:

1. **Structured preservation layer.** Store imported iCalendar objects and components in durable relational storage, including properties, parameters, value types, nested components, custom `X-*` fields, and timezone definitions accepted within safety limits.
2. **App projection layer.** Keep the current normalized calendar rows as the lean model for rendering, editing, pomodoro, search, and visible-window queries.

The normal calendar boot path must use only projected rows. It must not parse raw `.ics` content, load every preserved component, or expand unbounded recurrence sets at startup.

## Documents

- [Standards scope](./standards-scope.md): target standards, related RFCs, and boundaries.
- [Architecture](./architecture.md): structured preservation plus normalized projection.
- [Data model](./data-model.md): SQLite responsibilities and table shape.
- [Conformance](./conformance/README.md): current audit, component and property status, serialization, and fixture coverage.
- [Fixtures and clients](./fixtures-and-clients.md): automated fixtures and manual client testing.
- [Edit merge policy](./edit-merge-policy.md): current export overlays and planned edit-safety safeguards.
- [Recurrence and timezones](./recurrence-and-timezones.md): recurrence, value type, `VTIMEZONE`, and DST strategy.
- [Scheduling boundary](./scheduling-boundary.md): offline preservation versus transport-backed actions.
- [Performance budget](./performance-budget.md): startup, memory, import, export, and recurrence limits.
- [Decisions](./decisions.md): architecture decision log.

Client behavior notes and the shared manual procedure live in [clients](./clients/README.md). They record practical interoperability observations. They do not define standards behavior.

## Completion principle

A feature is considered compatible only when it has:

- a standards interpretation
- a storage or preservation rule
- an import rule
- an export rule
- an edit policy
- automated fixtures where practical
- manual client test coverage for major clients when behavior is client-sensitive
