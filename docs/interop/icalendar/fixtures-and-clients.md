# Fixtures and clients

This document defines automated fixture coverage and manual client testing. Standards fixtures provide repeatable evidence for specific behaviors. Client fixtures provide practical interoperability evidence.

## Fixture principles

- Store fixtures in a dedicated test fixture folder, grouped by standards area and client source.
- Prefer small, readable fixtures for unit tests.
- Add at least one large synthetic fixture for performance testing.
- Keep original client exports unchanged in a raw fixture folder.
- Add normalized expected-output fixtures only when deterministic serialization is required.
- Record parser warnings expected from each fixture.
- Use semantic equivalence checks where property order can legitimately differ.

## Automated fixture classes

The current standards fixture pack lives under `apps/client/test-fixtures/ics/rfc5545/` and is exercised by `apps/client/src/lib/calendar/ics/fixture-suite.test.ts`.

Current files:

- `core-events.ics`: minimal UTC event, all-day single and multi-day events, `DURATION`, floating time, escaped text, categories, and geo.
- `recurrence-timezones.ics`: custom `VTIMEZONE`, DST-adjacent recurrence, `EXDATE`, `RDATE`, yearly all-day recurrence, and `RECURRENCE-ID;RANGE=THISANDFUTURE`.
- `scheduling.ics`: `METHOD:REQUEST`, organizer parameters, attendee participation parameters, delegation, members, and `REQUEST-STATUS`.
- `components.ics`: mixed `VEVENT`, `VTODO`, `VJOURNAL`, `VFREEBUSY`, `VTIMEZONE`, and nested `VALARM`.
- `attachments-extensions.ics`: URI attachment, binary attachment, RFC 7986 properties, object-level `X-*`, and component-level `X-*`.

The suite links preserved jCal back to projected events before export so it exercises the same overlay path used by imported calendars with preservation rows.

Additional current client-oriented fixtures live at:

- `apps/client/test-fixtures/ics/google-calendar-sample.ics`
- `apps/client/test-fixtures/ics/outlook-sample.ics`
- `apps/client/test-fixtures/ics/edge-cases.ics`

Parser, serializer, round-trip, and fixture-suite tests provide additional inline cases. See [conformance fixture coverage](./conformance/serialization-and-fixtures.md#current-automated-fixture-coverage) for the audited status.

## Planned fixture backlog

The following cases are coverage goals. They are not all present as standalone fixture files or fully supported app semantics.

Recurrence and timezone priorities:

- explicit `BYSECOND`, `BYMINUTE`, and `BYHOUR` preservation cases, which are not projected today
- broader combinations of the supported `BY*` parts
- date-only and period-valued `RDATE`
- date-only and parameter-rich `EXDATE`
- all-day `RECURRENCE-ID;VALUE=DATE`
- recurrence across both DST transitions using custom `VTIMEZONE` rules

Component priorities:

- comprehensive `VTODO` fields, recurrence, and alarms
- broader `VJOURNAL` properties
- multiple `VFREEBUSY` periods and `FBTYPE` variants
- nested custom and future components

Scheduling and people priorities:

- `METHOD:CANCEL`
- `METHOD:REPLY`
- multiple parameter values, quoted values, and broader RFC 6868 cases
- edited offline scheduling objects with export diagnostics

Alarm priorities:

- absolute trigger alarm
- audio alarm with attachment
- email alarm with attendees and summary
- complete `REPEAT` and `DURATION` semantics

Attachment and extension priorities:

- object-level RFC 7986 and custom-property export merging
- additional binary value encodings and limits
- property groups, multiplicity, and registered extension properties

Security fixture priorities:

- malformed line folding
- oversized property
- oversized recurrence count
- nested component depth stress
- external URI fields
- hostile HTML in descriptions
- zip entry path traversal
- decompression bomb shape

## Test assertions

Each fixture should assert:

- parse success or expected warning/failure
- preserved component shape
- projected row shape when applicable
- export serialization validity
- parse to export to parse semantic equivalence
- covered unsupported data remains present after a supported edit and export overlay

## Manual client testing

The shared procedure, required test metadata, and current status matrix live in the [client compatibility index](./clients/README.md). Manual tests use disposable calendars. Never test with the user's primary calendar first.

## Client docs

Client notes live under [clients](./clients/README.md):

- [Google Calendar](./clients/google-calendar.md)
- [Outlook](./clients/outlook.md)
- [Apple Calendar](./clients/apple-calendar.md)
- [Thunderbird](./clients/thunderbird.md)
- [Nextcloud](./clients/nextcloud.md)
- [Proton Calendar](./clients/proton-calendar.md)
- [Fastmail](./clients/fastmail.md)

Client docs must not redefine standards behavior. They record what real clients export, accept, reject, or rewrite.

## Fixture naming

Use stable, descriptive names:

- `rfc5545-event-all-day-single.ics`
- `rfc5545-recur-exdate-zoned.ics`
- `rfc5545-vtimezone-custom.ics`
- `google-2026-05-14-yearly-all-day.ics`
- `outlook-2026-05-14-windows-tzid.ics`

Raw client exports should include a date in the file name when behavior may change over time.

## Comparison strategy

Use semantic comparison, not byte-for-byte comparison. Legal iCalendar serializers can reorder properties, fold lines differently, normalize case, and choose equivalent escaping without changing meaning. Exact source lexical representation is not a preservation guarantee.
