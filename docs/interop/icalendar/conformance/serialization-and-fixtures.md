# Serialization and fixtures

This document records parameter, value-type, serialization, and fixture coverage for the [2026-08-30 conformance baseline](./README.md#current-audit-baseline).

## Parameters

Current status:

- `VALUE`: honored for supported dates and date-times and preserved structurally for linked export.
- `TZID`: projected for date-time properties and mapped when possible. The source parameter and related `VTIMEZONE` remain preserved.
- `CN`: projected for organizer and attendee, exported, and tested.
- `ROLE`: projected for attendee, exported, and tested.
- `PARTSTAT`: projected for attendee, exported, and tested.
- `RSVP`: projected for attendee, exported, and tested.
- `LANGUAGE` and `ALTREP`: preserve-only for linked properties.
- `CUTYPE`, `DELEGATED-FROM`, `DELEGATED-TO`, `DIR`, `MEMBER`, and `SENT-BY`: preserve-only and merged for linked people properties.
- `RELTYPE`: preserve-only for linked relationship properties.
- `RANGE`: retained for linked `RECURRENCE-ID` properties.
- `FBTYPE`: preserved in top-level `VFREEBUSY` passthrough.
- `ENCODING`: preserved for inert binary values.
- `FMTTYPE`: preserve-only for linked attachments.
- `RELATED`: preserve-only for linked alarm triggers.
- Unknown `X-*` and registered parameters: retained structurally and merged where their linked property remains in the covered export path.

Parameter preservation is semantic. The relational model retains decoded values and multiplicity, while serialization emits valid quoting and RFC 6868 caret encoding. It does not promise the source file's exact quote placement or caret spelling.

## Value types

Current status:

- `BINARY`: preserved as inert structured data within the inline attachment limit, but not opened or decoded.
- `BOOLEAN`: preserved; selected Google extensions also project as booleans.
- `CAL-ADDRESS`: projected for organizer and attendee addresses, with additional parameters preserved.
- `DATE`: projected for all-day events and selected recurrence values.
- `DATE-TIME`: projected for event times and selected recurrence values.
- `DURATION`: projected for event end calculation and alarm triggers. Linked event export retains imported event `DURATION` shape.
- `FLOAT`: projected for `GEO`.
- `INTEGER`: projected for `PRIORITY` and `SEQUENCE`.
- `PERIOD`: preserved in structured data and top-level `VFREEBUSY` passthrough, but not projected.
- `RECUR`: projected for the supported rule subset.
- `TEXT`: projected for common fields and escaped on export.
- `TIME`: preserved when accepted by `ical.js`, but not projected.
- `URI`: projected for `URL`; attachments and other URI properties remain inert preservation data.
- `UTC-OFFSET`: preserved in timezone definitions.

The app must continue to distinguish floating, UTC, zoned, and date-only values. Structured preservation retains the semantic value type and multi-value shape, not the exact original lexical representation.

## Serialization rules

Current automated evidence:

| Rule | Status | Notes |
| --- | --- | --- |
| CRLF line endings | yes | Serializer tests cover generated output. |
| Content-line folding at 75 octets | yes | Tested with long values. |
| UTF-8-safe folding | yes | Folding avoids splitting encoded characters. |
| TEXT escaping | yes | Backslash, semicolon, comma, newline, and carriage return are covered. |
| RFC 6868 parameter escaping | partial | `CN` and selected cases are covered, not every parameter shape. |
| Exclusive date-only `DTEND` | yes | Parser and serializer tests cover all-day behavior. |
| `RECURRENCE-ID` type matching | partial | UTC, zoned, all-day, and linked range preservation are covered. |
| `EXDATE` and `RDATE` type matching | partial | Date-time coverage is stronger than date and period variants. |
| Unknown properties and parameters in relational storage | yes | Accepted jCal structure is stored as component, property, parameter, and value rows. |
| Unknown linked event data on export | partial | Covered merge paths retain it; object-level merge and orphan event passthrough remain gaps. |
| Stable source bytes | not-applicable | Semantic equivalence, not byte identity, is the contract. |

## Current automated fixture coverage

Current fixture files are listed in [fixtures and clients](../fixtures-and-clients.md#automated-fixture-classes). Current test evidence covers:

- minimal UTC `VEVENT`
- single-day and multi-day all-day events
- imported `DURATION` shape
- floating timed event linked export
- escaped text and UTF-8-safe folding
- categories and geo values
- timed recurrence with `EXDATE`
- yearly all-day recurrence
- recurrence override with `RECURRENCE-ID`
- linked `RANGE=THISANDFUTURE` preservation and cancellation expansion
- date-time `RDATE`, with date-only shape still partial
- preserved custom `VTIMEZONE` export, without custom-rule recurrence math
- basic alarms and preservation of covered unsupported alarm fields
- attendee delegation and organizer `SENT-BY`
- URI and binary attachment preservation
- `VTODO` and `VFREEBUSY` top-level passthrough
- `VJOURNAL` through the shared non-event passthrough path
- mixed-component calendars
- malformed parser input and configured parser safety limits
- zip path, entry, aggregate-size, and import-sanitization protections in the relevant import tests

No row above implies complete coverage of every property allowed on that component.

## Coverage still needed

- date-only and period `RDATE` forms
- broader `EXDATE` parameter and value-type combinations
- explicit `BYSECOND`, `BYMINUTE`, and `BYHOUR` preservation fixtures
- broader supported `BY*` combinations
- custom `VTIMEZONE` transition evaluation across DST
- complete alarm repeat, duration, audio, and email semantics
- object-level RFC 7986 and custom-property export merging
- orphan recurrence overrides and other preserved `VEVENT` components without projections
- broader RFC 6868 quoting and multi-value parameter cases
- manual round trips for all tracked clients beyond the partial Google run

Each new fixture should assert parse diagnostics, relational preservation, projection when applicable, export validity, and semantic equivalence after reparse.

## Comparison policy

Compare semantic component structure and bounded recurrence occurrence sets. Property ordering, case normalization, equivalent escape spelling, and legal line-folding changes are not failures unless they alter meaning or discard unsupported structured data.
