# Outlook

Outlook is a practical compatibility target, not the source of truth for iCalendar behavior. Test both Outlook on the web and desktop Outlook when possible because their import/export behavior can differ.

## Source notes

- Microsoft documents importing `.ics` files into Outlook calendars and subscribing to iCalendar feeds.
- Microsoft notes that an imported `.ics` file does not refresh if the source calendar later changes.
- Outlook desktop can export an iCalendar file through calendar sharing APIs, with detail controlled by sharing settings.
- Outlook commonly emits Windows timezone names, which Ganbaru AI currently maps to IANA names for projection.

Sources:

- <https://support.microsoft.com/en-us/office/import-or-subscribe-to-a-calendar-in-outlook-com-or-outlook-on-the-web-cff1429c-5af6-41ec-a5b4-74f2c278e98c>
- <https://learn.microsoft.com/en-us/office/vba/api/outlook.calendarsharing.saveasical>

## Known fixture priorities

- Windows `TZID` values such as `Pacific Standard Time`
- recurring timed event across DST
- recurring override with Windows `TZID`
- event with attachments
- event with private details hidden by sharing settings
- meeting invitation with attendees
- cancellation and update messages
- all-day single and multi-day events

## Import into Ganbaru AI

Expected handling:

- Preserve original Windows `TZID` values in structured source data.
- Map recognized Windows timezones to IANA zones for projection.
- Preserve full `VTIMEZONE` blocks when present.
- Preserve organizer, attendees, and scheduling metadata.
- Preserve attachments as inert data until the user explicitly opens them.

## Manual test

Use the [shared client procedure](./README.md#shared-procedure). Record Outlook on the web, Windows, and macOS results separately because their behavior can differ.

## Behavior to verify

- Whether Outlook accepts IANA `TZID` without a full `VTIMEZONE`.
- Whether Outlook rewrites IANA timezones into Windows timezones.
- Whether Outlook strips unknown `X-*` properties.
- Whether Outlook preserves `VTODO` items in `.ics` files.
- Whether Outlook imports `VALARM` repeat and duration.
- Whether Outlook handles `RANGE=THISANDFUTURE`.

## Observed behavior log

No manual Ganbaru AI compatibility run recorded yet.
