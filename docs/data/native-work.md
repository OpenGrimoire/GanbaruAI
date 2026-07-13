# Native blocking work

Tauri async command executors must not perform unbounded synchronous work. Database transactions stay on their async SQL path. Filesystem traversal, process observation, blocking platform APIs, image and media probing, and archive generation run on blocking workers when their cost depends on user input or external state.

The current replaceable worker boundaries are:

- Music folder scans run on a blocking worker, use a generation token, stop at 5,000 tracks, skip symlinks, and return deterministic truncated results when replaced.
- Doomscrolling installed-app discovery, foreground observation, and process enumeration run on blocking workers. New observations replace older generations. Linux process termination and platform foreground-close calls also run off the async executor because graceful termination can wait up to 800 milliseconds.
- Project icon DNS resolution and bounded remote image processing run outside the async executor and retain their byte and address validation.
- Notes import and export commands keep SQLite work asynchronous. Their synchronous file, archive, and rendering stages are bounded by existing import, export, decompression, row, and byte limits. Atomic export writers create a temporary file and rename only after a complete successful write.

Worker code must not hold a SQLite transaction or shared player mutex while awaiting `spawn_blocking`. Cancellation returns no partial authoritative result. Atomic writers retain or remove their temporary output on failure, and a newer replaceable request cannot publish the older generation's result.
