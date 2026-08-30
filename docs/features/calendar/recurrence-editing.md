# Calendar recurrence editing

Recurrence preview is a non-mutating projection of the current Save result. It shows what the visible calendar will contain if the current draft and scope are saved, but it never detaches, splits, collapses, deletes, archives, transfers sessions, or writes data before Save.

## Product contract

- Preview and Save use the same semantic plan.
- Scope switching reprojects one shared draft immediately.
- Missing fields and explicitly cleared fields are distinct.
- The selected occurrence stays visible unless the selected operation truly removes it.
- Save commits the calendar mutation and Pomodoro reference transfers atomically.
- Successful Save refreshes the visible window from canonical persisted expansion.
- Preview contours refer only to rendered event identities and always clear on close or completion.
- A visually absent block never retains an invisible hit area.

## Ownership

The frontend owns pure draft normalization, edit planning, preview projection, affected-set presentation, and active-event interaction restrictions. These operations remain synchronous enough for typing, scope changes, drag, and resize.

The backend owns durable validation, protected-history invariants, and atomic persistence. Persisted window expansion may be backend-owned, but canonical expansion and frontend preview must agree for supported recurrence rules.

Live preview should not require an asynchronous round trip. A future shared recurrence library is appropriate only if it preserves immediate deterministic projection across frontend and backend callers.

## Edit-session input

One edit session contains:

- The selected template or occurrence.
- The source template for a generated occurrence.
- The selected recurrence date.
- Baseline event values.
- Normalized draft field operations.
- Selected scope.
- One captured edit time used by preview and Save.
- Current visible window.
- Relevant active Pomodoro metadata.

Changing scope changes only the projection parameter. It does not create a separate draft or reset explicit edits.

## Field operations

Nullable and structurally meaningful fields use explicit operations:

- **Unchanged:** the user left the baseline value unchanged.
- **Set:** the user supplied a new value.
- **Cleared:** the user explicitly removed the value.

For recurrence, a missing property never means that repeat was turned off. Turning repeat off and back on resolves from the final visible value compared with the baseline.

## Scope selector

The selector appears only when the event belonged to a saved recurring series when the session opened. It appears for the template occurrence and generated occurrences.

Adding recurrence to a non-recurring event does not show a scope selector. The event becomes one recurring template while retaining its base identity.

The selector is hidden for the selected active occurrence, and the effective scope is `Only this`.

## Projection output

Given the same saved rows, selected occurrence, normalized draft, scope, active-session metadata, captured edit time, and visible window, projection returns the same:

- Visible events for the window.
- Preview contour identities.
- Editing identity used to anchor the panel.
- Semantic commit plan.
- Canonical-refresh requirement.

Unrelated events remain unchanged. Virtual preview events use stable collision-free identities. Contour identities are always a subset of rendered identities.

Delete and archive previews use the same affected-set model. Protected rows can retain their current geometry while a contour explains that the operation will archive them.

## Save

Save uses the current projection's semantic plan or recomputes the same plan from identical normalized input. It does not reinterpret the draft independently.

The panel can close and the submitted projection can remain visually stable while persistence completes. That display state is not a mutation. One backend batch applies template updates, exceptions, detachment, splitting, materialization, archive operations, and active-run reference transfers.

After success, the app clears preview state and reloads the visible range from canonical expansion. Older foreground or prefetched loads cannot replace that result. After failure, the user receives a retryable error and no partial recurrence mutation remains.

## Create and non-recurring edits

Creating a recurring event creates one template. Preview expands it only in the visible window.

Adding recurrence to a saved non-recurring event converts that row into a template and preserves its ID as the first occurrence. Clearing recurrence on a non-recurring event is a no-op.

## Only this

If recurrence is unchanged or cleared, the selected occurrence detaches as a non-recurring standalone and the source template gains an exception for that date.

If recurrence is set to a different rule, the selected occurrence becomes an independent recurring template and the source still gains one exception.

Preview shows the source without the selected occurrence plus the detached result and any visible expansion of its independent rule.

## Following

The old template ends before the selected occurrence. If recurrence remains set, a new template begins at the selected occurrence. If recurrence is cleared, only one standalone survivor remains there.

Exceptions relevant to the new side transfer so an occurrence previously detached, archived, or deleted cannot regenerate after a split.

## All

Without protected history, the template can be updated directly. With protected history, the old template is capped at the protected boundary and a new template begins at the first mutable occurrence. Individual protected occurrences detach only when a historical template cannot preserve them safely.

Clearing recurrence collapses only the mutable side. The selected occurrence is the survivor even when it is not the original template date.

## Active Pomodoro sessions

An active run survives every recurrence edit. When the active occurrence receives a new event identity, run and segment references transfer atomically. Original historical identity remains preserved.

An active selected occurrence cannot edit the repeat chain or its recorded start. A different active occurrence in the affected series stays unchanged on the protected side or is materialized before the edit.

## Rendering

Preview affects only occurrences from the edited series and current window. The calendar rail, event status, and active-session presentation use the projected identities and geometry without writing history.

After commit, rendering comes from canonical persistence. A preview or cache never becomes a second source of truth.

## Quality requirements

- Planning and projection remain pure and deterministic.
- Protected-history decisions consider the complete affected range, not only visible dates.
- Malformed or unsupported recurrence that cannot be enumerated safely stops with a diagnostic rather than rewriting history.
- Preview and commit plans have focused parity tests.
- Backend batches prove rollback across calendar and Pomodoro reference changes.
- Required user scenarios live in [Calendar recurrence testing](../../testing/calendar-recurrence.md).
