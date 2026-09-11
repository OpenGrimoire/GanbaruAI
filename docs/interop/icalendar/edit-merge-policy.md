# Edit merge policy

This policy separates the implemented export-overlay behavior from safeguards that remain planned.

## Core rule

Supported user edits currently update normalized projection rows. Imported relational component rows remain source provenance. During export, Ganbaru AI reconstructs the preserved component and overlays generated values for fields owned by the projection.

Unsupported structured fields remain in preservation storage and are merged where the current serializer supports them. The app must not silently discard accepted legal data just because the UI does not show it.

## Supported field merge

For linked `VEVENT` components, export currently replaces or regenerates these supported fields from projection data:

- Title maps to `SUMMARY`.
- Description maps to `DESCRIPTION` after persistence sanitization and safe serialization.
- Start and end map to `DTSTART`, `DTEND`, or an updated imported `DURATION` shape.
- All-day state controls date value types and exclusive `DTEND` semantics.
- Recurrence maps to the supported `RRULE`, `RDATE`, `EXDATE`, and override subset.
- Status maps to `STATUS`.
- Transparency maps to `TRANSP`.
- Visibility maps to `CLASS` as `PUBLIC` or `PRIVATE`; imported `CONFIDENTIAL` values project as `PRIVATE`.
- Supported attendee values replace generated-owned parts of `ATTENDEE`, while preserved parameters are merged where the property remains.
- Supported alarm values replace generated-owned parts of `VALARM`, while unsupported alarm fields are retained where the linked alarm merge applies.

## Unsupported field preservation

Fields the UI does not model remain in the preserved component. Examples:

- `COMMENT`
- `RESOURCES`
- `RELATED-TO`
- unsupported `ATTENDEE` parameters
- unsupported `ORGANIZER` parameters
- attachment parameters
- extension properties
- scheduling request metadata
- custom `VTIMEZONE` definitions

When exporting a linked projected event, these fields remain where covered by the structured merge unless the corresponding property is intentionally replaced or removed by projection semantics.

## Structural edit risks

Some edits can make preserved data questionable:

- converting timed recurring event to all-day
- changing recurrence frequency while preserving complex overrides
- changing timezone while original custom `VTIMEZONE` still exists
- deleting an attendee from a scheduling request
- changing organizer
- editing an event with `METHOD:REQUEST` or `METHOD:CANCEL`
- modifying a component with unknown recurrence properties
- editing a `VTODO` or `VJOURNAL` before those have projection models

These cases should eventually set preservation status to `needs-review` and retain diagnostics when the merge cannot be proven safe. Automatic edit-time status changes are not implemented yet.

## Status transitions

Planned transitions:

- `lossless` to `partial`: projection or editing narrows semantics while retaining source data.
- `partial` to `needs-review`: a structural edit makes the export merge uncertain.
- `needs-review` to `regenerated`: user accepts app-generated output that may drop unsupported data.
- any status to `invalid`: parser or export validation found unrecoverable structure.

The schema supports these status values, but the current edit path does not perform these transitions automatically.

## Delete behavior

Current deletion or archival removes the event from the active projection but retains its linked preservation rows. Calendar export begins with active projected events and does not pass through preserved top-level `VEVENT` components, so the deleted event is not reintroduced.

Future retention work should choose between tombstoning and removing orphaned preservation rows. This is needed for storage cleanup and unambiguous provenance, not to prevent current export from resurrecting the event.

If the component is part of a recurring series:

- deleting one instance should add or update `EXDATE` or an override according to recurrence policy.
- deleting the whole series currently removes or archives its projected master and overrides; explicit preservation cleanup remains planned.

## Attendee behavior

Ganbaru AI currently has no account identity model. Therefore:

- attendee response status imported from `.ics` is read-only by default.
- the app must not let the user RSVP as another attendee.
- organizer-side edits such as marking a guest optional are allowed only when editing the event is allowed.
- future identity support may allow editing the attendee row that matches the current user.

## Scheduling edits

Offline edits do not notify anyone. If a component has scheduling metadata:

- preserve `METHOD`, `ORGANIZER`, `ATTENDEE`, `REQUEST-STATUS`, and related fields.
- show diagnostics if export may look like an invitation update.
- emit a preserved `METHOD` only when the exported calendar has one distinct preserved method.
- do not imply that attendees were notified.
- require a future transport before sending scheduling messages.

## Planned export warnings

The export UI should eventually warn when:

- any component is `needs-review` or `invalid`.
- unsupported fields were dropped by an accepted regeneration.
- a scheduling component was edited offline.
- a custom timezone could not be interpreted for projection.
- recurrence expansion was capped or partially unsupported.

Warnings should be specific and include component `UID` when available. The full user-visible warning flow is not implemented today.

## Repair behavior

Re-importing the original `.ics` source file is the preferred repair path when a row has no preserved component or when preservation data is known to be incomplete. Unsupported values that were never stored in Ganbaru AI must not be invented from the normalized projection.

Future repair actions must be explicit. If a user accepts regenerated output, diagnostics should make clear that unsupported original fields may be absent from export.
