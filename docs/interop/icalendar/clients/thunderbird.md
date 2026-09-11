# Thunderbird

Thunderbird is both a practical compatibility target and a useful reference because its calendar model is built around iCalendar concepts.

## Source notes

- Thunderbird source documentation says its calendar item model leans heavily on iCalendar.
- Thunderbird implements `VEVENT` events and `VTODO` tasks, but not `VJOURNAL`.
- Thunderbird uses `ical.js` beneath the surface, which is relevant because Ganbaru AI currently also uses `ical.js` for parsing.
- Thunderbird Help documents exporting calendars in iCalendar `.ics` format.

Sources:

- <https://source-docs.thunderbird.net/en/latest/calendar/item_model.html>
- <https://support.mozilla.org/gu-IN/kb/exporting-and-sharing-a-calendar>

## Known fixture priorities

- `VEVENT` event export
- `VTODO` task export
- recurring task
- task alarms
- mixed event and task calendar
- attendee and organizer fields
- custom properties from Thunderbird
- calendar export with non-ASCII values

## Import into Ganbaru AI

Expected handling:

- Project `VEVENT` rows.
- Preserve `VTODO` rows even before task projection exists.
- Preserve alarms and custom fields.
- Preserve unsupported task fields for later kanban or task integration.

## Manual test

Use the [shared client procedure](./README.md#shared-procedure). Include a mixed event and task calendar, and record the Thunderbird version, operating system, and calendar storage type.

## Behavior to verify

- Exact `VTODO` shape Thunderbird exports.
- Whether Thunderbird preserves unknown `X-*` fields.
- Whether Thunderbird accepts Ganbaru AI-generated `VTODO` after future task support.
- How Thunderbird handles custom `VTIMEZONE`.
- Whether Thunderbird preserves `VALARM` repeat and duration.

## Observed behavior log

No manual Ganbaru AI compatibility run recorded yet.
