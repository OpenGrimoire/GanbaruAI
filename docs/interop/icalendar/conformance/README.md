# Conformance

This audit tracks implementation coverage against the iCalendar standards scope. It is broader than the current app projection and separates structured preservation, export, editing, and test evidence.

Compatibility means semantic preservation within documented parser limits. It does not mean byte-for-byte reproduction of source case, ordering, quoting, escaping, or line folding.

## Status model

Each detailed entry considers:

- **Projected:** mapped into normalized Ganbaru AI rows.
- **Preserved:** retained in relational iCalendar storage.
- **Exported:** emitted in valid output with equivalent supported semantics.
- **Editable:** editable through app surfaces without silently discarding preserved data.
- **Tested:** supported by an automated fixture or a recorded manual client run.

Status values:

- `yes`: implemented with direct test evidence.
- `partial`: implemented for a documented subset.
- `no`: not currently supported.
- `preserve-only`: retained and exported, but not projected or app-editable.
- `planned`: intended behavior that is not implemented.
- `not-applicable`: not meaningful for an app projection.

## Current audit baseline

Audit date: 2026-08-30.

Evidence reviewed:

- `apps/client/src/lib/calendar/ics/parser.ts`
- `apps/client/src/lib/calendar/ics/serializer.ts`
- `apps/client/src/lib/calendar/ics/types.ts`
- `apps/client/src/lib/calendar/ics/parser.test.ts`
- `apps/client/src/lib/calendar/ics/serializer.test.ts`
- `apps/client/src/lib/calendar/ics/fixture-suite.test.ts`
- `apps/client/src/lib/calendar/ics/round-trip.test.ts`
- `apps/client/src/lib/components/calendar/rrule.ts`
- `apps/client/src/lib/components/calendar/rrule.test.ts`
- `apps/client/src/lib/components/calendar/recurrence.ts`
- `apps/client/src/lib/components/calendar/recurrence.test.ts`
- `apps/client/src/lib/stores/calendar-bulk-import.ts`
- `apps/client/src/lib/stores/calendar-import-export.ts`
- `apps/client/src-tauri/app/src/calendar_import.rs`
- `apps/client/src-tauri/app/src/calendar_import/preservation.rs`
- `apps/client/src-tauri/app/src/calendar_reads/icalendar.rs`
- `apps/client/src-tauri/migrations/20260830173211_baseline_schema.sql`
- current files under `apps/client/test-fixtures/ics/`

This is a source and test audit, not a claim that every legal RFC 5545 form has a fixture.

## Implementation summary

- The frontend parser uses `ical.js` and produces projected `CalendarEvent` rows, preservation payloads, and warnings.
- Imports store accepted `VCALENDAR` objects as relational components, properties, parameters, value nodes, diagnostics, and projection warnings.
- Projected events, attendees, alarms, and recurrence overrides link back to preserved components.
- Only `VEVENT` is projected into visible calendar rows. Other accepted components remain preservation-only.
- Supported edits update projection rows. Imported relational components remain source provenance.
- Export reconstructs linked source components and overlays regenerated supported fields in memory. It does not rewrite the preservation rows at edit time.
- Unsupported linked event properties and parameters, nested alarms, inert attachments, imported `DURATION` shape, floating date-time shape, and `RANGE=THISANDFUTURE` survive the covered merge paths.
- Preserved `VTIMEZONE` definitions are emitted before generated stubs. Preserved top-level non-event components pass through export.
- Object metadata beyond one safe `METHOD` is preserved but not merged into generated export.
- Unknown properties and parameters are preserved in relational storage. Export coverage is partial because object-level metadata is not merged and unknown registered event properties are preserve-only.

## Component summary

| Component | Projected | Preserved | Exported | Editable | Automated evidence |
| --- | --- | --- | --- | --- | --- |
| `VCALENDAR` | not-applicable | yes within limits | partial | no | partial |
| `VEVENT` | partial | yes within limits | partial | partial | yes for projected subset |
| `VTODO` | no | yes | preserve-only | no | yes for passthrough |
| `VJOURNAL` | no | yes | preserve-only | no | partial |
| `VFREEBUSY` | no | yes | preserve-only | no | yes for passthrough |
| `VTIMEZONE` | partial | yes | partial | no | partial |
| `VALARM` | partial | yes | partial | partial | partial |
| Custom or future components | no | yes when accepted | partial | no | partial |

See [components and properties](./components-and-properties.md) for the detailed component and property audit. See [serialization and fixtures](./serialization-and-fixtures.md) for parameters, value types, serializer behavior, and current fixture evidence.

## Current behavior that remains intentionally limited

- The schema supports `needs-review` and `regenerated`, but supported edits do not automatically transition preservation status or create user-visible export warnings.
- Deleting or archiving a projected event leaves its preserved source rows in the database. Export does not resurrect it because top-level preserved `VEVENT` components are excluded from passthrough.
- Bulk re-import upserts projected events by target calendar and `source_uid`, with `SEQUENCE` rejection for older rows. It does not independently upsert by component type and recurrence identity.
- Multiple or invalid object-level scheduling methods normalize to `METHOD:PUBLISH` during current calendar export. This serializer fallback does not preserve the distinct meaning of mixed scheduling methods.

## Critical compatibility gaps

- Object-level metadata beyond a single safe `METHOD` is not merged into export.
- App recurrence math does not evaluate foreign `VTIMEZONE` transition rules.
- `BYSECOND`, `BYMINUTE`, and `BYHOUR` are preserved but not represented, edited, or expanded by the app.
- `RDATE` and `EXDATE` value shapes are narrowed by projection.
- Attendee and organizer parameters beyond the projected subset are preserve-only.
- `CONTACT` is preserved and exported for linked events, but not projected or editable.
- Floating timed events project through the device zone even though linked export keeps floating shape.
- Generated events use `DTEND` because no imported `DURATION` representation exists.
- `RANGE=THISANDFUTURE` is preserved and applied for imported cancelled overrides, but cannot be created through the recurrence UI.
- Automatic edit-risk status transitions, export warnings, and orphan preservation cleanup are not implemented.
- Manual compatibility runs are absent for every tracked client except a partial Google Calendar run from May 2026.
