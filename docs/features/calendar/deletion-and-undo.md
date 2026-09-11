# Calendar deletion and undo

Ganbaru AI does not provide a general Calendar undo and redo stack. Creates, edits, moves, resizes, and recurrence changes commit directly. Delete and archive are the reversible Calendar actions exposed through a short-lived undo opportunity.

## Delete versus archive

Future untracked events may be hard deleted. Events that started, have timer history, are active, or own another durable reference are archived instead. A scoped recurrence operation can combine archive of protected occurrences with removal of mutable future expansion.

The confirmation explains the effective result. Active events require the run to end before delete or archive becomes available.

## Pending and completed state

When deletion or archive begins, the event panel closes and a status toast reports the pending operation. The calendar applies the semantic plan's final visible projection so one scoped operation does not disappear occurrence by occurrence.

After persistence succeeds, the toast reports whether events were deleted, archived, or both and offers `Undo` for five seconds. It also has a dismiss control.

Only the latest completed delete or archive operation has an undo opportunity. Starting another operation finalizes the previous one.

## Undo scope

Undo restores the snapshot and recurrence structure captured by the successful operation:

- A standalone delete restores the event.
- `Only this` restores the prior exception and occurrence state.
- `Following` restores the prior recurrence boundary and any detached survivor.
- `All` restores the prior historical and mutable series state.
- Archive undo restores archived records to their valid active representation.

If Undo expires, is dismissed, or the app closes, the operation remains committed. Durable recovery of archived history remains available through archive surfaces where supported; a hard-deleted future event does not gain indefinite recovery from the toast.

## Pomodoro interaction

If an operation stops an active Pomodoro run, undoing the Calendar mutation restores calendar data only. It does not restart the run, delete interruption history, or rewrite completed segments. This preserves append-only focus history.

## Failure behavior

The backend applies a scoped plan atomically. A failed operation restores the canonical visible window and reports an error rather than leaving a partial split, archive, exception, or run-reference transfer.

See [Recurrence editing](recurrence-editing.md) and [data invariants](../../data/invariants.md).
