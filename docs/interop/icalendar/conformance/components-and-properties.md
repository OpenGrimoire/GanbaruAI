# Components and properties

This document records component and property coverage for the [2026-08-30 conformance baseline](./README.md#current-audit-baseline). Preservation refers to semantic structured storage within parser limits.

## Components

### `VCALENDAR`

- Projected: `not-applicable`.
- Preserved: `yes` for accepted component structure and object metadata.
- Exported: `partial`.
- Editable: `no` as an object.
- Tested: `partial`.
- Evidence: import stores a root `vcalendar` component, its properties, object metadata, diagnostics, and ordered children.
- Export behavior: Ganbaru AI generates `PRODID`, `VERSION`, `CALSCALE`, and `X-WR-CALNAME`. It reuses one valid preserved `METHOD` only when the exported calendar has one distinct method.
- Gap: other preserved object-level properties and original component ordering are not merged into generated export.

### `VEVENT`

- Projected: `partial`.
- Preserved: `yes` for accepted structured input.
- Exported: `partial`.
- Editable: `partial`.
- Tested: `yes` for the projected subset and covered merge cases.
- Evidence: parser, serializer, round-trip, fixture, and Rust import tests cover event projection, relational preservation, projection links, and unsupported data surviving supported edits at export.
- Gap: not every legal property, parameter combination, or value shape has fixture evidence or an editable app field.

### `VTODO`

- Projected: `no`.
- Preserved: `yes`.
- Exported: `preserve-only`.
- Editable: `no`.
- Tested: `yes` for top-level passthrough.
- Gap: task semantics have no calendar projection or UI.

### `VJOURNAL`

- Projected: `no`.
- Preserved: `yes`.
- Exported: `preserve-only`.
- Editable: `no`.
- Tested: `partial` through the shared non-event passthrough path.
- Gap: journal semantics have no calendar projection or UI.

### `VFREEBUSY`

- Projected: `no`.
- Preserved: `yes`.
- Exported: `preserve-only`.
- Editable: `no`.
- Tested: `yes` for top-level passthrough.
- Gap: availability semantics have no calendar projection or UI.

### `VTIMEZONE`

- Projected: `partial` through `TZID` mapping.
- Preserved: `yes` for accepted definitions.
- Exported: `partial`.
- Editable: `no`.
- Tested: `partial`.
- Evidence: import preserves `VTIMEZONE`, `STANDARD`, and `DAYLIGHT`. Export emits preserved definitions before generated timezone stubs.
- Gap: recurrence expansion uses projected IANA zones rather than evaluating arbitrary imported transition rules. Original Windows names and custom definitions remain preservation data.

### `VALARM`

- Projected: `partial`.
- Preserved: `yes`.
- Exported: `partial`.
- Editable: `partial`.
- Tested: `partial`.
- Evidence: `ACTION`, `TRIGGER`, and `DESCRIPTION` map to alarm rows. Linked export overlays supported values while retaining covered unsupported alarm fields.
- Gap: repeat, duration, email, audio, attendee, attachment, and extension semantics are not first-class app behavior.

### Nested, custom, and future components

- Projected: `no`.
- Preserved: `yes` when accepted by the parser.
- Exported: `partial`.
- Editable: `no`.
- Tested: `partial`.
- Evidence: recursive component rows keep parent IDs and order. Top-level non-event components pass through, and linked event merging retains non-`VALARM` nested components.
- Gap: preserved `VEVENT` components without a projection, including orphan overrides, do not pass through calendar export.

## Object-level properties

Current import and export status:

- `PRODID`: stored in object metadata and relational properties. Export generates Ganbaru AI's `PRODID`.
- `VERSION`: stored. Export emits `VERSION:2.0`.
- `CALSCALE`: stored. Export emits `CALSCALE:GREGORIAN`.
- `METHOD`: stored. Export uses one valid distinct preserved method or normalizes to `PUBLISH` for local, missing, invalid, or mixed method inputs.
- `X-WR-CALNAME`: preserved relationally. Export generates the selected calendar name rather than the original property.
- `X-WR-TIMEZONE`: preserved but not merged into export.
- `NAME`, `DESCRIPTION`, `COLOR`, `IMAGE`, `REFRESH-INTERVAL`, and `SOURCE`: preserved but not merged into export.
- Unknown object-level `X-*` and registered extensions: preserved but not merged into export.

The future target is intentional field-by-field merging. Blind passthrough of conflicting object metadata is not safe.

## Event properties

### Identity and timestamps

- `UID`: projected as `sourceUid`, exported, and tested.
- `DTSTAMP`: not projected into normalized event rows, but preserved relationally for linked source components. Export generates a current value.
- `CREATED`: not projected, but preserved and merged for linked events.
- `LAST-MODIFIED`: not projected, but preserved and merged for linked events.
- `SEQUENCE`: projected, exported, used for duplicate selection and older-revision rejection, and tested.

Normalized `created_at` and `updated_at` fields are app timestamps, not imported iCalendar timestamp provenance.

### Time fields

- `DTSTART`: projected and exported for UTC, `TZID`, floating, and all-day forms to the documented subset.
- `DTEND`: projected and exported. All-day exclusive-end behavior is tested.
- `DURATION`: projected into an end time. Linked export keeps the imported property shape and regenerates its value from current start and end.

Generated events without preserved source use `DTEND` because no original representation exists.

### Text and location fields

- `SUMMARY`: projected as title, exported, and tested.
- `DESCRIPTION`: projected, sanitized before persistence, exported as TEXT, and tested.
- `LOCATION`: projected, exported, and fixture-covered.
- `URL`: projected, exported, and tested.
- `COMMENT`: preserve-only for linked events.
- `RESOURCES`: preserve-only for linked events.
- `CONTACT`: preserve-only for linked events. Parser tests record that it is preserved but not projected.

Non-modeled text properties are not editable as first-class app fields.

### Recurrence

- `RRULE`: projected into `RecurrenceConfig` and exported for the supported subset.
- `RDATE`: projected as instants and exported. Date-time forms have stronger coverage than date-only and period forms.
- `EXDATE`: projected as local date keys and exported at the event start time.
- `RECURRENCE-ID`: projected as overrides and exported for covered UTC, zoned, and all-day cases.
- `RANGE=THISANDFUTURE`: retained on linked overrides and used to suppress imported cancelled future instances. The app cannot create this range directly.
- Duplicate master revisions: one file keeps the newest master by `SEQUENCE`, then `LAST-MODIFIED`, `DTSTAMP`, or `CREATED` when sequence ties.

Projected `RRULE` parts:

- `FREQ`
- `INTERVAL`
- `COUNT`
- `UNTIL`
- `BYDAY`, including ordinal values
- `BYMONTHDAY`
- `BYMONTH`
- `BYSETPOS`
- `BYYEARDAY`
- `BYWEEKNO`
- `WKST`

Known recurrence gaps:

- `BYSECOND`, `BYMINUTE`, and `BYHOUR` are preserved but not represented in `RecurrenceConfig`.
- Unsupported rule parts are not editable and may be narrowed if the UI regenerates recurrence.
- `RDATE` and `EXDATE` value types are narrowed in projection.
- Recurrence property parameters are preserve-only.
- Orphan override components remain in preservation storage but have no projection and are not emitted by top-level event passthrough.

### Status, visibility, and categorization

- `STATUS`: projected for `CONFIRMED`, `TENTATIVE`, and `CANCELLED`; exported and tested.
- `TRANSP`: projected for `OPAQUE` and `TRANSPARENT`; exported and tested.
- `CLASS`: projected and exported for `PUBLIC` and `PRIVATE`; `CONFIDENTIAL` projects as `PRIVATE`.
- `PRIORITY`: projected, validated, exported, and tested.
- `CATEGORIES`: projected as strings, exported, and tested.
- `GEO`: projected, exported, and tested.
- `RELATED-TO`: preserve-only for linked events.
- `REQUEST-STATUS`: preserve-only for linked events.

Relationship and request-state metadata are not editable as first-class app fields.

### People

- `ORGANIZER`: `CN` and address are projected, exported, and tested.
- `ATTENDEE`: `CN`, `ROLE`, `PARTSTAT`, `RSVP`, and address are projected, exported, and tested.
- `CONTACT`: preserve-only for linked events.
- `SENT-BY`, `DIR`, `DELEGATED-FROM`, `DELEGATED-TO`, `MEMBER`, and `CUTYPE`: preserved and merged where their organizer or attendee property remains.
- Multiple values and unknown parameters: preserved structurally and merged for linked properties where covered.

Ganbaru AI has no current identity-backed RSVP transport. Imported participation state is metadata, not proof that a reply was sent.

### Attachments and extensions

- `URL`: projected and exported.
- URI `ATTACH`: preserved as inert structured data and merged for linked events.
- Binary `ATTACH`: preserved within the inline size limit, but not decoded or opened by the app.
- Event `X-*`: projected into value-only `extendedProperties` and exported.
- Google guest-permission extensions: projected into dedicated booleans and exported.
- Unknown registered properties: preserve-only for linked event export so parameters, value types, and multiplicity are not narrowed into editable strings.

Object-level extension export merging remains future work.

## Non-event property groups

### `VTODO`

Task projection is planned. Current accepted task fields, including identity, dates, completion, status, recurrence, people, attachments, categories, resources, URLs, and extensions, are preservation-only and pass through as part of the top-level component.

### `VJOURNAL`

Journal projection is planned. Current accepted identity, date, text, status, people, relationship, attachment, URL, and extension fields are preservation-only.

### `VFREEBUSY`

Free/busy projection is planned. Current accepted identity, date, period, people, comment, URL, request-status, and extension fields are preservation-only.

## Timezone properties

For accepted `VTIMEZONE`, `STANDARD`, and `DAYLIGHT` components:

- `TZID` is mapped to an IANA zone for projection when possible and remains preserved in the source definition.
- `LAST-MODIFIED`, `TZURL`, `DTSTART`, `TZOFFSETFROM`, `TZOFFSETTO`, `TZNAME`, `RRULE`, `RDATE`, `COMMENT`, and extensions are preserved and emitted with the preserved definition.

The app does not yet use arbitrary preserved transition rules for recurrence math.

## Alarm properties

- `ACTION`, `TRIGGER`, and `DESCRIPTION`: projected, exported, and tested for the supported subset.
- `SUMMARY`, `ATTENDEE`, `DURATION`, `REPEAT`, `ATTACH`, `ACKNOWLEDGED`, `PROXIMITY`, and extensions: preserved and merged for linked alarms where supported by `ical.js`.

Only basic alarm behavior is app-editable. Other legal alarm fields remain preservation-only.
