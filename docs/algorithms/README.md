# Algorithms

This directory specifies deterministic domain decisions that need to remain consistent across UI, Rust services, recovery, imports, and tests. Algorithm documents explain inputs, outputs, ordering, edge cases, and rationale. They do not mirror helper functions or persistence DDL.

## Calendar

- [Calendar algorithms](calendar/README.md)
- [Recurrence expansion](calendar/recurrence-expansion.md)
- [Time conflict detection](calendar/time-conflict-detection.md)

## Pomodoro

- [Pomodoro algorithms](pomodoro/README.md)
- [Idle detection](pomodoro/idle-detection.md)
- [State machine](pomodoro/state-machine.md)
- [Plan and history](pomodoro/plan-and-history.md)
- [Adaptive policy](pomodoro/adaptive-policy.md)
- [Adaptive experiments](pomodoro/adaptive-experiments.md)

## Reading algorithm status

These documents describe the intended product contract. A section labeled implementation gap identifies known divergence in current code. Do not remove a desired invariant merely to make documentation match an accidental implementation detail. Resolve the product decision, update code and tests, then remove the gap note.

Data ownership and persistence are indexed in [Data documentation](../data/README.md). Feature documents own user-visible workflows and copy.

## Writing algorithm specifications

A durable algorithm document should include:

- normalized inputs and outputs;
- deterministic ordering and tie-breakers;
- time, timezone, and boundary semantics;
- hard resource bounds;
- persistence effects only where they are part of correctness;
- examples that cover non-obvious cases;
- known implementation or conformance gaps.

Avoid performance claims without measurements and exact source inventories that become stale after ordinary refactors.
