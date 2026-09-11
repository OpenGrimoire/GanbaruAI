# Theme authoring

## Theme selection

Appearance settings list quick-toggle choices and all registered themes. Built-ins appear before user themes. The quick toggle can use selected custom light and dark targets rather than hardcoding the built-in pair.

A keyboard-accessible theme picker previews highlighted themes. Confirm applies the highlighted theme; cancel restores the theme active before the picker opened.

## Editor session

Opening a theme creates an in-memory authoring buffer and preserves the pre-edit snapshot. Every source, token, isolation, palette, metadata, Calendar default, reset, or rebake edit updates the buffer and live preview.

The app underneath remains inspectable where space permits so the user can test real surfaces. Compact layouts use a sheet or full-screen editor while keeping Save, Cancel, and scrolling reachable.

The editor is responsive by available container space rather than one fixed desktop size. Draggable presentation remains within the visible viewport and has keyboard-reachable controls.

## Save, cancel, and reset

**Save and apply** validates and persists the complete buffer in one operation, keeps the edited theme active, and exits only after success. A failed save leaves the session retryable.

**Cancel** restores the exact pre-edit registry snapshot and previously active theme. If the session created a new duplicate, Cancel removes that unsaved theme.

**Reset all to seed** replaces the current buffer with the theme's seed snapshot. It remains unsaved until Save.

Closing the application during an active session uses the same dirty-session confirmation. Since streaming edits are not persisted, canceling the close or discarding the buffer remains truthful.

## Authoring organization

The editor groups source colors, app canvas, Calendar surface, event palette, Calendar details, event panel, text, actions, and semantic status colors in a stable human-readable order. Stored rows are keyed by identity, not by presentation position.

Source controls explain which token families they drive. Individual token rows expose resolved color and isolation. Runtime-only colors do not appear as fake editable rows.

## Event palette editor

All 32 event slots remain visible and retain stable identity. Editing a slot previews existing events that reference it. The editor distinguishes slot selection from reordering UI controls, because persisted event identity must never change from visual reordering.

Fallback and blend behavior are explained where users can observe their result.

## Contrast feedback

Warnings update with the live theme and link to the affected control where possible. They identify text and surface pairs, event label problems, semantic actions, and focus visibility.

The editor does not block every low-contrast save because themes are user-authored, but it does not hide the risk or label a failing result accessible.

## Typography and density

Font family and font scale are Appearance preferences rather than duplicated theme-token fields. They apply with the active theme but retain independent portable preference identity.

Layout density is not currently a theme format dimension. A future density system requires explicit component and accessibility contracts rather than arbitrary spacing tokens in imported color themes.

## Accessibility

Every color field has a text value and keyboard operation. Isolation, warning, selected state, built-in protection, and dirty state are communicated without color alone. Live preview never steals focus from the active editor control.
