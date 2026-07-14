# Hot SQLite query plans

This document records the expected access path for latency-sensitive SQLite reads. Tests use `EXPLAIN QUERY PLAN` against the migrated schema. An index is added only when an exact production query demonstrates a high-cardinality table scan or avoidable temporary sort.

| Domain | Hot read | Expected access path |
| --- | --- | --- |
| Projects | Task counts grouped by status for one project | `idx_project_tasks_project_status` |
| Projects | Dependency lookup from a blocked task | `idx_project_task_dependencies_blocked` |
| Notes | Active sidebar page order | `idx_notes_pages_active` |
| Notes | FTS keyset result page | FTS virtual-table match plus the existing search projection indexes, as asserted in `notes/tests/search.rs` |
| Notes | Visible child blocks ordered within a parent | `idx_notes_blocks_parent_block` |
| Calendar | Event window overlap | Existing start and end range indexes, as asserted in `calendar_reads.rs` |
| Calendar | Notification, EXDATE, RDATE, override, and attendee hydration | Existing event foreign-key and sort indexes, as asserted in `calendar_reads.rs` |
| Music | Playback resume state by stable source identity | Primary-key autoindex on `music_playback_states.source_identity` |
| Doomscrolling | Local-date usage window ordered by source | `idx_doomscrolling_usage_samples_date_source` |
| Pomodoro | Open run lookup | `idx_pomodoro_runs_open` and the single-open partial unique invariant |
| Pomodoro | Active segment lookup | `idx_pomodoro_segments_single_active` |

The 2026-07-11 audit found no demonstrated plan gap, so it adds no migration. In particular, it does not add speculative overlapping indexes or modify the baseline schema. The focused plan tests fail if these reads regress to a full scan or temporary sort where the expected index should satisfy the predicate and ordering.
