# Proton Calendar

Proton Calendar is a practical compatibility target. Its encryption and product model may affect what it imports, exports, or rewrites, so behavior must be tested from real fixtures.

## Source notes

- Proton documents importing calendars from other services.
- Proton documents exporting a calendar as an iCalendar `.ics` file.
- Proton's import/export flow is account-based, but Ganbaru AI's base `.ics` compatibility does not depend on Proton account access.

Sources:

- <https://proton.me/support/protoncalendar-calendars>
- <https://proton.me/support/easy-switch-calendars>

## Known fixture priorities

- Proton-exported calendar with simple events
- all-day single and multi-day events
- recurring event with exceptions
- moved recurring instance
- reminders
- attendees if exported
- imported calendar exported again from Proton
- non-ASCII text

## Import into Ganbaru AI

Expected handling:

- Preserve Proton-specific `X-*` fields.
- Project supported `VEVENT` rows.
- Preserve recurrence and timezone data.
- Preserve alarms even if Proton has limited alarm import/export behavior.

## Manual test

Use the [shared client procedure](./README.md#shared-procedure). Record the Proton surface and plan only when they affect import or export behavior.

## Behavior to verify

- Whether Proton imports multi-event `.ics` files reliably on web and desktop.
- Whether Proton preserves unknown `X-*` fields.
- Whether Proton preserves or rewrites timezones.
- Whether Proton exports attendees.
- Whether Proton exports alarms.
- Whether Proton accepts custom `VTIMEZONE`.

## Observed behavior log

No manual Ganbaru AI compatibility run recorded yet.
