# Native blocking work

Tauri asynchronous command executors must not perform unbounded synchronous work. Database transactions stay on their asynchronous SQL path. Filesystem traversal, process observation, blocking platform APIs, image and media probing, and archive generation run on blocking workers when their cost depends on user input or external state.

Current replaceable worker boundaries include:

- Music folder scans, with generation replacement, traversal bounds, deterministic ordering, and explicit truncation.
- Doomscrolling application discovery, foreground observation, process enumeration, and graceful close operations.
- Project icon network resolution and bounded remote image processing.
- Notes import and export file, archive, and rendering stages, with atomic output publication.

Worker code must not hold a SQLite transaction or a shared player mutex while awaiting a blocking task. Cancellation returns no partial authoritative result. Atomic writers publish only complete output. A replaced request cannot publish results after the newer generation has become authoritative.
