# Calendar recurrence

Recurring events use a template-plus-occurrence model. The template is canonical structured data; visible occurrences are expanded for the requested window.

## Supported rules

Ganbaru AI supports daily, weekly, monthly, and yearly frequency with interval, weekday selection, month-day selection, ordinal weekday patterns, count, end date, and exception dates. Advanced imported rule parts are preserved according to the [iCalendar interoperability contract](../../interop/icalendar/README.md), even when the local editor cannot produce or fully interpret them.

The editor offers common presets and a bounded advanced editor. It must not silently simplify an imported rule in a way that changes future occurrences.

## Identity model

The persisted template row is also the first occurrence and uses the template UUID. Later generated occurrences use a deterministic identity:

```text
<template-id>::YYYY-MM-DD
```

Generated occurrence identity is stable for a template and recurrence date. Each occurrence has independent Pomodoro history. Work recorded on one occurrence never carries into another occurrence automatically.

Pomodoro records retain both a live canonical event reference where possible and the exact original occurrence identity. Archiving can clear the live reference while preserving historical joins through the original identity.

## Protected occurrences

An occurrence is protected when it has started by the captured edit time or has a run, segment, override, exception, active session, or another durable reference. Protection is evaluated across the affected recurrence range, not only the visible window.

Structural edits preserve protected meaning through one of these forms:

- A historical template capped at the protected boundary.
- A detached standalone event with its own identity.
- An archived historical event when it should no longer appear as active calendar data.

Future untracked occurrences remain mutable.

## Structural operations

### Only this

The selected occurrence is excluded from the source template and becomes a standalone event or an independent recurring template. Runs and segments for that occurrence transfer atomically when identity changes.

Use this for a one-off variation.

### Following

The old template is capped before the selected occurrence. If recurrence remains enabled, a new template begins at the selected occurrence. If recurrence is cleared, the selected occurrence becomes one standalone survivor and later generated occurrences stop.

Use this for a permanent change from a selected point forward.

### All

If no protected history exists, the template may be updated directly. Otherwise the old template preserves the protected side and a new mutable template begins at the first mutable occurrence.

If recurrence is cleared, protected history remains and the selected occurrence becomes the one non-recurring survivor on the mutable side. Other mutable occurrences stop expanding.

## Delete and archive scope

Deleting or archiving a future occurrence can remove mutable future expansion after preserving any protected occurrences in range. Starting from an already started occurrence is history-only: affected started occurrences archive, while later mutable occurrences remain in the repeat chain unless a future structural boundary is explicitly selected.

Every scoped operation is one semantic plan and one atomic backend transaction. The calendar does not archive or remove occurrences one at a time in a visibly inconsistent sequence.

## Active sessions

The selected active occurrence can only use `Only this`; the scope selector is hidden. It may change a valid end boundary but cannot move or rewrite its recorded start.

If another occurrence in the affected series is active, it remains on the protected side or is materialized unchanged before the structural edit. The run reference transfers in the same transaction as the calendar mutation.

## Time shifts

Template-wide time changes apply only to mutable occurrences after the protection boundary. Historical blocks retain their original time so Pomodoro segments remain visually and analytically aligned.

See [Recurrence editing](recurrence-editing.md), [recurrence expansion](../../algorithms/calendar/recurrence-expansion.md), and [data invariants](../../data/invariants.md).
