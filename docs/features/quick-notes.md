# Quick notes

Quick notes are a lightweight, app-wide place for capturing temporary thoughts, reminders, and small pieces of information. They are deliberately separate from the Notes tab. A Quick note has no project membership, page hierarchy, blocks, icons, covers, comments, backlinks, attachments, history, or collaboration state.

## Collection panel

The sticky-note control in every app title bar opens a resident panel aligned to the right edge of the window. The panel and its editor code load with the application shell so the first opening does not wait for a deferred component import. The shell also starts one shared SQLite read for the initial All view and tag list. The panel consumes that resident snapshot when opened, while later mutations invalidate it and canonical reads refresh it. If the user opens the panel before the read finishes, the stable header and creation surface appear immediately without replacing the whole collection with a loading screen. The panel has a default size of 760 by 680 pixels and is clamped to the usable viewport. This width presents three masonry columns at full app size. Its header provides local search, an All view, tag views, Archive, and Trash. Archive and Trash stay aligned to the right, separate from the active-note filters. The active collection begins with a prominent note creation action.

A note can have one optional tag. Users create up to nine tags through a compact inline field placed directly after the final tag in the header. Enter saves the tag, while Escape or moving focus away cancels the draft. Users can select a tag to filter the active collection and assign or clear a tag from a card or editor. View shortcuts follow the same numeric navigation pattern as Calendar and Projects: 1 selects All, keys 2 through 9 select the first eight tags, and 0 selects the ninth tag. Tooltips expose each shortcut. Numeric navigation is disabled while typing or while a nested modal is open.

Active notes are ordered with pinned notes first and recently updated notes first within each group. Cards use a measured shortest-column masonry layout based on the panel's actual width. Card previews keep their natural height up to 320 pixels, then fade rather than letting one long note dominate the collection. Pointer actions can be revealed on hover where hover exists, but remain visible for touch-like input.

Search covers titles and body text through a local SQLite FTS projection. Collection reads use bounded keyset windows of at most 60 cards. Quick notes stay fully offline and publish only local Tauri events to keep open app windows synchronized.

## Editor

Creating or opening a card shows a modal editor above the collection panel. The editor has an optional title and one multiline body. It supports bold, italic, and underline through toolbar controls and Ctrl or Cmd keyboard shortcuts. A collapsed formatting shortcut changes the formatting inherited by subsequent typing. Undo and redo are local to the open editing session.

Pasted content is converted to text, line breaks, bold, italic, and underline. Links, lists, media, scripts, styles, and unsupported formatting are flattened or discarded. Raw HTML is never persisted. Titles are limited to 200 characters and bodies to 65,536 characters.

The note background uses the muted variant of one of the 32 theme-aware event palette slots. New notes default to palette index 30, the same penultimate default used by Calendar events. The picker shows each source palette color clearly, while the card and editor apply the same canvas-blended treatment as past calendar events. The same palette slot follows theme changes. Note titles and bodies share whichever of pure black or pure white has the higher WCAG contrast against the blended background.

Edits autosave after 250 milliseconds and pending writes flush before an editor, panel, detached window, or application closes. Empty new drafts are discarded. Revision checks prevent another window from silently overwriting an edited note. Conflicts can be reloaded or saved as a separate copy, while ordinary failures keep the editor open with a retry action.

## Lifecycle and storage

Active notes can be pinned, archived, or moved to Trash. Archiving and trashing clear the pinned state. Trash preserves whether the note came from the active collection or Archive, so restoring returns it to the correct collection. Trashed notes are read-only, can be restored, and are permanently deleted after seven days. Manual permanent deletion and Empty Trash require confirmation.

Quick notes are structured SQLite data. `quick_note_tags` stores the bounded tag list and its shortcut order. `quick_notes` stores identity, lifecycle, color, optional tag, revision, timestamps, and derived searchable text. `quick_note_text_runs` stores ordered normalized text runs and their supported formatting flags. Mutations replace the run set and derived plain text atomically. Quick notes do not create files in the Ganbaru AI folder and do not participate in Notes import, export, search, history, or collaboration.

## Accessibility and responsive behavior

The title-bar control exposes its dialog state. The panel and editor trap focus while active, close with Escape, and restore prior focus. Cards keep source order in the DOM even though their visual positions use masonry placement. All icon actions have localized names, focus indicators, keyboard activation, and visible touch alternatives. Reduced-motion preferences disable masonry movement animation.

At narrow sizes, the panel and editor fill the available app space, their content scrolls internally, and primary close and lifecycle actions remain reachable. The feature treats the 280 by 180 minimum window as a recovery floor rather than a comfortable editing size.
