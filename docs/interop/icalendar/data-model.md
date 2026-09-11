# Data model

This document describes the implemented relational preservation schema and its boundary with normalized calendar rows. The schema preserves accepted iCalendar structure semantically. It does not promise byte-for-byte reproduction of source files.

## Principles

- Do not add one column per RFC property.
- Keep the current normalized calendar rows as the render and editing projection.
- Store accepted imported iCalendar structure in relational preservation tables.
- Link preserved components to projected rows when they map to app concepts.
- Keep raw or structured preserved data lazy-loaded.
- Make migrations idempotent and avoid modifying user-authored data unless the mapping is certain.

## Existing projection tables

The existing tables remain the app-facing model:

- `calendar_events`
- `calendar_event_attendees`
- `calendar_event_alarms`
- `calendar_event_overrides`
- future task, journal, and free/busy projection tables if those UI features ship

These tables are optimized for visible-window queries, recurrence expansion, editing, pomodoro, and notifications.

## Preservation tables

Preservation uses `icalendar_objects`, `icalendar_components`, and relational property/value child tables. It stores imported iCalendar structure without JSON columns. Supported projections link back to preserved components.

### `icalendar_objects`

One row per imported or generated top-level iCalendar object.

Implemented fields:

- `id`: primary key.
- `calendar_id`: owning Ganbaru AI calendar row.
- `source_kind`: `import-file`, `import-zip-entry`, `local-export-base`, or `subscription`.
- `source_name`: file basename, zip entry name, URL label, or user-visible origin.
- `source_fingerprint`: hash of source text or normalized object for dedupe and diagnostics.
- `prodid`: preserved `PRODID` when present.
- `version`: preserved `VERSION`, normally `2.0`.
- `method`: preserved `METHOD`, if present.
- `calendar_scale`: preserved `CALSCALE`, normally `GREGORIAN`.
- `created_at`: row creation time.
- `updated_at`: last update time.

Parser notes live in `icalendar_object_diagnostics` as message rows ordered by `sort_order`.

Indexes:

- `(calendar_id)`
- `(calendar_id, source_kind, source_name)`
- `(source_fingerprint)`

### `icalendar_components`

One row per component, including the `VCALENDAR` root and nested components.

Implemented fields:

- `id`: primary key.
- `object_id`: parent `icalendar_objects` row.
- `parent_component_id`: parent component for recursive structure, or null for the object root.
- `calendar_id`: owning calendar for simpler queries.
- `component_type`: lowercase component type such as `vevent`, `vtodo`, `vjournal`, `vfreebusy`, `vtimezone`, `valarm`.
- `uid`: `UID` value when the component type has one.
- `recurrence_id`: normalized recurrence identity when present.
- `recurrence_id_value_type`: `date`, `date-time`, or other exact value type.
- `sequence`: parsed `SEQUENCE`, if present.
- `dtstart_key`: normalized start key for lookup and ordering.
- `projected_kind`: currently `event`, `alarm`, or null; future projections may add more kinds.
- `projected_id`: linked row in the projection table, such as `calendar_events.id`.
- `preservation_status`: `lossless`, `partial`, `unsupported`, `needs-review`, `regenerated`, or `invalid`.
- `sort_order`: component order inside its parent.
- `created_at`: row creation time.
- `updated_at`: last update time.

Component properties are stored in `icalendar_component_properties`; property parameters are stored in `icalendar_property_parameters`; property and parameter values are stored as recursive rows in `icalendar_value_nodes`; projection notes are stored in `icalendar_component_projection_warnings`.

Indexes:

- `(object_id)`
- `(calendar_id, component_type)`
- `(calendar_id, uid)`
- `(calendar_id, uid, recurrence_id)`
- `(projected_kind, projected_id)`
- `(preservation_status)`

### Implemented child tables

The relational component tree uses these child tables:

- `icalendar_component_properties`: property name, value type, and order for one component.
- `icalendar_property_parameters`: parameter name and order for one property.
- `icalendar_value_nodes`: recursive arrays, objects, scalar values, and their order for a property or parameter.
- `icalendar_object_diagnostics`: ordered parser diagnostics for one object.
- `icalendar_component_projection_warnings`: ordered projection warnings for one component.

Properties, parameters, and values are ordered by `sort_order`. Nested value nodes use `parent_node_id`, and object members use `object_key`.

### Possible future lookup tables

An `icalendar_timezones` lookup table may be justified if resolving `VTIMEZONE` definitions requires loading too many component rows. It is not implemented today. Until measurements show that need, timezone definitions remain ordinary preserved components.

An `icalendar_component_links` join table may be justified if the current direct projection links become insufficient. It is not implemented today. Current master events, override events, and alarms use `icalendar_components.projected_kind` and `projected_id`; projected rows also carry direct component IDs.

## Preservation format

RFC 7265 jCal is the in-memory parser and serializer shape, while SQLite stores the same semantic structure relationally. Within parser safety limits, the representation retains:

- component names
- property names
- parameter names and values
- value types
- multi-value structure, including jCal array and object values
- nested components
- extension fields

Export is generated from structured relational data so current projected fields can be overlaid safely. Original property order, name case, quoting, escaping spelling, and line folding are not compatibility guarantees.

## Projection mapping

Projection creates or updates current app rows:

- `VEVENT` maps to `calendar_events`.
- `VALARM` under `VEVENT` maps to `calendar_event_alarms` when supported.
- `ATTENDEE` maps to `calendar_event_attendees` while preserving unsupported attendee parameters in `icalendar_property_parameters` and `icalendar_value_nodes`.
- recurring override `VEVENT`s map to `calendar_event_overrides`.
- `VTODO`, `VJOURNAL`, and `VFREEBUSY` are currently preservation-only. Future app surfaces may add projections without replacing the source representation.

Every projected row created from preserved data should be traceable back to its component. `calendar_events`, `calendar_event_overrides`, `calendar_event_attendees`, and `calendar_event_alarms` carry nullable `icalendar_component_id` columns. Attendees also store `icalendar_property_index`, because `ATTENDEE` is a property on a `VEVENT` rather than its own component.

The `icalendar_components.projected_kind` and `projected_id` reverse link is used where one component maps to one projected row: master events, override events, and alarms. Attendee rows keep their direct link on the projected row so multiple attendees can reference the same preserved `VEVENT` without overwriting the component's reverse link.

Full-event loads reconstruct linked `VEVENT` structures for the serializer. The preserved component remains import provenance. Export overlays regenerated supported fields onto that reconstructed component instead of mutating its relational rows during each edit. This merge retains unsupported event properties, unsupported parameters, inert URI attachments, imported `DURATION` shape, floating date-time shape, `RECURRENCE-ID;RANGE=THISANDFUTURE`, and unsupported alarm fields where covered by the merge path.

Preserved `VTIMEZONE` components are loaded separately for calendar export and emitted before generated timezone stubs. Preserved top-level non-event components such as `VTODO`, `VJOURNAL`, `VFREEBUSY`, and custom components pass through while they have no app projection. Top-level preserved `VEVENT` components are not passed through independently of projected rows, so deleting or archiving the projected row prevents that event from reappearing in export even though its preservation rows remain.

Imported event rows with a source UID but no `icalendar_component_id` derive a `regenerated` iCalendar preservation state so diagnostics and export behavior make clear that no original component is available.

## Current re-import identity

Current bulk import uses two related identities:

- Within one parsed file, duplicate master `VEVENT` components are grouped by `UID`. The parser keeps the highest `SEQUENCE`, then the newest `LAST-MODIFIED`, `DTSTAMP`, or `CREATED` revision key.
- Against stored projected rows, import looks up `calendar_id` plus `source_uid`. A lower incoming `SEQUENCE` is skipped; an equal or higher sequence updates the row.
- Preservation objects are replaced by `calendar_id`, `source_kind`, and `source_name` only when none of the imported events is an older revision.
- Recurrence overrides remain children of the selected master event payload. `component_type` and `RECURRENCE-ID` are not independent bulk-upsert keys today.
- `source_fingerprint` is stored for provenance and diagnostics, not used as the projected event upsert key.

The broader source, component type, UID, recurrence identity, and sequence key remains a possible future model if non-event components gain independent projections. It must not be described as implemented today.

## Data retention

Deleting an imported calendar removes its projected rows before deleting the calendar. Foreign-key cascades then remove:

- preserved iCalendar objects
- preserved components
- diagnostics and links

Current single-event deletion or archival does not remove or tombstone the linked preservation rows. Export starts from projected events and excludes top-level preserved `VEVENT` passthrough, so the deleted event does not silently reappear. The source component remains until its owning imported calendar or preservation object is removed.

An explicit tombstone or cleanup policy is still needed if retained orphaned source components become a storage, diagnostics, or future re-import problem. That is an implementation gap, not current behavior.
