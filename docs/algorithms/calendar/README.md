# Calendar algorithms

Calendar algorithms turn recurrence rules and overlapping schedules into deterministic concrete events and one Pomodoro ownership decision. They operate on normalized domain values and leave persistence to calendar services.

- [Recurrence expansion](recurrence-expansion.md) defines bounded generation of concrete occurrences, exception and override handling, IDs, windows, and timezone behavior.
- [Time conflict detection](time-conflict-detection.md) defines active ownership and records the current divergence between auto-start and timeline display.

Interoperability scope and fixtures live under [iCalendar interoperability](../../interop/icalendar/). Calendar feature behavior lives in the feature specifications. Durable event identity and archive rules are in [Calendar schema](../../data/schema/calendar.md).

## Shared requirements

- Use explicit inclusive or exclusive boundary semantics.
- Preserve stable event and recurrence identity.
- Resolve equal candidates deterministically.
- Bound expansion and scanning before processing untrusted imports.
- Keep local civil recurrence intent separate from elapsed instant arithmetic.
- Apply the same desired ownership policy to automation and visual projections.

Current recurrence conformance and ownership divergences are documented in the detailed pages rather than hidden behind a claim that all consumers are equivalent.
