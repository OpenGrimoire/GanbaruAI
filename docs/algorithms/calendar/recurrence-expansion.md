# Recurrence expansion

Recurrence expansion converts one normalized calendar template into bounded concrete occurrences for an inclusive civil-date window. TypeScript uses the result for calendar projection. Rust uses the same domain contract for backend reads and commands. Both consumers must produce equivalent identities and dates for the supported rule subset.

## Inputs and output

Inputs are:

- a template event with stable identity, start, end, and optional home zone;
- an optional normalized recurrence rule;
- exception dates, additional dates, and occurrence overrides;
- an inclusive window start and end as civil dates.

The output contains concrete events whose date span overlaps the requested window. An event may begin before the window and still be returned when its end reaches the window.

Non-recurring events pass through unchanged when they overlap. A template with additional dates but no recurrence rule produces its original occurrence plus eligible additional occurrences.

## Supported normalized rules

Current expansion applies:

- daily, weekly, monthly, and yearly frequency;
- positive interval;
- weekday selection for weekly rules;
- ordinal weekday selection for supported monthly and yearly rules;
- month-day and month selection for supported monthly and yearly rules;
- count or until termination;
- exception dates;
- additional dates;
- occurrence overrides and cancellation from a recurrence date.

Import may preserve more iCalendar properties than the expander applies. Preservation is not support. Current conformance gaps are listed below and should remain visible until interoperability fixtures and both expanders agree.

## Civil dates and instants

Recurrence walks local civil dates, not fixed UTC durations. TypeScript uses Temporal.PlainDate. Rust uses chrono's date-only representation. This keeps a weekly 09:00 event at 09:00 in its home zone across offset changes.

After selecting a civil occurrence date, the application combines it with the stored local time and home-zone policy to obtain instants where an instant is needed. Duration across a DST transition follows those instants. The recurrence date itself remains the local identity.

All-day occurrences use date spans directly. Timed multi-day occurrences preserve the template's civil day span and local time components.

## Window semantics

Both window bounds are inclusive civil dates. An occurrence overlaps when:

- its end date is on or after the window start; and
- its start date is on or before the window end.

Generation may fast-forward to the first date that could overlap, but it must preserve the same logical occurrence count as walking from the template. Fast-forward is an optimization, not a different recurrence rule.

## Expansion order

For one template:

1. Validate the date range, recurrence shape, positive interval, and bounds.
2. Preserve the original template ID for its first occurrence. Emit it when it overlaps and is not excluded or cancelled.
3. Walk or fast-forward the recurrence cursor according to the supported normalized rule.
4. Stop at the count limit, until date, cancellation-from boundary, window end, or hard iteration guard.
5. For each eligible recurrence date, apply exclusion and cancellation, preserve the template day span, then apply an exact-date override.
6. Add distinct additional dates that are not already generated, excluded, or cancelled.
7. Return deterministic concrete events. Callers apply their required display ordering.

An override changes the concrete occurrence fields while retaining original-series and recurrence-date provenance. A moved override remains associated with the recurrence instant it replaced.

## Concrete identity

The original occurrence keeps the template ID. A generated occurrence uses a deterministic identity derived from template ID and recurrence civil date. It also records the template as its recurring parent.

Identity does not depend on title, display time, current device zone, or expansion window. Expanding the same occurrence through a different window yields the same ID.

Persisted override rows have their own durable storage identity, but projections retain the generated occurrence and original-series relationships required by edits, project links, and Pomodoro history.

## Invalid dates

Rules such as day 31 do not silently clamp to day 30 in a month that has no 31st. The supported normalized rule either skips that candidate or advances according to its explicit ordinal logic. Leap-day behavior follows the same principle.

An ordinal weekday is calculated within its requested month. If the requested ordinal does not exist, that month contributes no occurrence.

Parser normalization and expander behavior must agree. A parser must not normalize an unsupported form into a superficially similar rule with different meaning.

## Exceptions, additional dates, and overrides

Exception dates remove an occurrence by recurrence civil date. Additional dates add an occurrence with the template's time and day span unless an override replaces its fields.

The same civil date appears at most once. A normal recurrence date and an additional date deduplicate. Exclusion or cancellation wins over ordinary emission.

An exact override is applied after base occurrence construction. A series cancellation-from boundary stops later generation. These precedence rules must be identical in TypeScript and Rust.

## Bounds and failure behavior

Expansion is bounded by the requested window and a hard 10,000-iteration guard per template. Imports and reads also bound the number of input events and recurrence data before expansion.

Invalid normalized data returns a controlled error in Rust or is rejected before frontend use. It must not hang, allocate without bound, or generate occurrences outside the requested window merely to satisfy a malformed count.

Do not document sub-millisecond performance without a benchmark. The durable requirement is bounded work proportional to the relevant window, with fast-forward for common long-running rules.

## Examples

### Daily count

A template beginning June 1 with a daily interval and count 3 has recurrence dates June 1, June 2, and June 3 before exclusions. A June 2 exception affects which concrete events are returned. The precise interaction between COUNT and EXDATE is an unresolved conformance issue described below.

### Multi-day overlap

An occurrence spanning June 1 through June 3 overlaps a June 3 through June 7 window even though its start is before the window. It is returned.

### Weekly DST change

A 09:00 event in America/New_York remains at 09:00 on each selected weekday. Its UTC instant changes when the home-zone offset changes.

### Generated identity

If template event-1 produces June 8, the concrete projection uses the deterministic event-1 plus June 8 identity regardless of whether the caller expanded June alone or the entire quarter.

## Current implementation gaps

### Preserved but unsupported rule parts

BYSETPOS, BYWEEKNO, BYYEARDAY, and WKST are not applied by either current expander. A TypeScript source comment previously claimed BYSETPOS support, but the algorithm does not implement it. Import and export may preserve these values; feature and interoperability documentation must not call them supported until both implementations and conformance fixtures agree.

The current weekly walk effectively uses its built-in week convention rather than an imported WKST value.

### COUNT and EXDATE

Generated excluded dates after the original currently advance the cursor without advancing the generated count. The original date is initialized as the first count position even when it is excluded. This is internally inconsistent and may differ from RFC recurrence-set semantics, where the rule produces its limited set before EXDATE subtraction.

Choose the interoperability contract, update TypeScript and Rust together, and add fixtures covering an excluded original, an excluded later occurrence, and multiple exclusions near COUNT.

### RDATE and RRULE termination

Current code applies the RRULE until date to additional RDATE values. In iCalendar recurrence-set semantics, RDATE is generally an independent inclusion. Confirm the intended interoperability behavior and update both expanders and fixtures together.

### Dual implementation equivalence

The two implementations use different date libraries and duplicated control flow. Equivalence is a required contract, not an assumption. Shared fixtures must cover fast-forward, multi-day overlap, exceptions, additional dates, overrides, cancellation, invalid month days, DST boundaries, and the hard guard.
