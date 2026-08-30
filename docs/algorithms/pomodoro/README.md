# Pomodoro algorithms

Pomodoro behavior is divided into five related specifications so each decision remains reviewable without one monolithic document.

- [Idle detection](idle-detection.md) covers platform activity sources, threshold scheduling, webcam suppression, idle backdating, and resume behavior.
- [State machine](state-machine.md) covers pure transition decisions, side-effect ordering, lifecycle recovery, and calendar boundaries.
- [Plan and history](plan-and-history.md) covers lazy segment persistence, future projection, inheritance, pauses, reconfiguration, and worked examples.
- [Adaptive policy](adaptive-policy.md) covers stable objectives, evidence, boundaries, guardrails, and privacy rules.
- [Adaptive experiments](adaptive-experiments.md) records the seven current bounded experiment lanes and analysis contract.

Durable rows and relationships are summarized in [Pomodoro schema](../../data/schema/pomodoro.md). User-visible behavior remains in the Pomodoro feature documents.

## Shared requirements

- Persist facts about phases that started. Derive future phases.
- Never rewrite completed or interrupted progress after a config change.
- Make transition decisions deterministic from explicit inputs.
- Commit canonical state before native notifications, overlays, media, or window events.
- Treat desktop and mobile recovery evidence separately.
- Record adaptive decisions before the affected phase or run begins.
- Keep one globally active segment at most.

Algorithm names describe responsibilities, not current file or helper names. Source may be reorganized without changing these contracts.
