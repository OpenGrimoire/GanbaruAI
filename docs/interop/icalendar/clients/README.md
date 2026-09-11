# Client compatibility

Client notes record practical import and export behavior. They are dated observations, not standards rules. The [standards scope](../standards-scope.md) and [conformance audit](../conformance/README.md) define Ganbaru AI's compatibility contract.

## Manual test status

| Client | Status | Last run | Notes |
| --- | --- | --- | --- |
| [Google Calendar](./google-calendar.md) | partial | 2026-05 | Ordinary events and capped recurrence were exercised; stale unbounded recurrence behavior was observed. |
| [Apple Calendar](./apple-calendar.md) | not run | none | Fixture priorities and official source notes only. |
| [Outlook](./outlook.md) | not run | none | Test web and desktop variants separately. |
| [Thunderbird](./thunderbird.md) | not run | none | Include mixed `VEVENT` and `VTODO` coverage. |
| [Nextcloud](./nextcloud.md) | not run | none | Record server and Calendar app versions. |
| [Proton Calendar](./proton-calendar.md) | not run | none | Record product surface and plan when relevant. |
| [Fastmail](./fastmail.md) | not run | none | Verify documented alarm and duplicate-UID behavior. |

`Not run` means no dated Ganbaru AI round trip is recorded. It does not mean the client is incompatible.

## Shared procedure

Use disposable calendars and test accounts. Never begin with a user's primary calendar.

1. Record the client, platform, visible version, test date, timezone, and locale.
2. Create a disposable calendar.
3. Import the relevant Ganbaru AI fixtures from `apps/client/test-fixtures/ics/rfc5545/` and any client-specific raw fixtures.
4. Inspect all-day spans, timed values, recurrence, overrides, alarms, attendees, and unsupported components relevant to that client.
5. Export the same calendar back to `.ics` when the client supports it.
6. Keep the raw client export unchanged as a dated fixture when licensing and privacy allow.
7. Re-import the result into Ganbaru AI.
8. Compare semantic results, parser warnings, preserved structured fields, and bounded recurrence occurrence sets.
9. Record the result in the client document and update the status matrix above.

## Result record

Each manual run should state:

- client and platform
- app, web, or server version when visible
- account or plan type only when it affects behavior
- test date
- timezone and locale
- input fixture names
- import result in the external client
- export and re-import result in Ganbaru AI
- semantic differences, dropped fields, warnings, and UI surprises
- raw fixture path and screenshots when retained

Do not report byte-level differences as compatibility failures by themselves. Legal serializers can normalize ordering, case, escaping, and line folding.

## Shared priority set

Every client should eventually receive at least:

- single-day and multi-day all-day events
- zoned, UTC, and floating timed events
- recurring events with exclusions and moved instances
- attendees, organizer, and inert scheduling metadata
- basic alarms
- non-ASCII and escaped text
- unknown extensions
- custom timezone definitions
- mixed components where the client claims support

Client-specific priorities and official behavior notes remain in the individual files.
