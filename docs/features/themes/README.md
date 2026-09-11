# Themes

Themes define Ganbaru AI's application shell, Calendar surfaces, event palette, typography context, and contrast behavior through a validated registry. A theme is not a light/dark boolean.

## Current scope

| Capability | Status |
| --- | --- |
| Immutable built-in light and dark themes | Implemented |
| User themes with full resolved snapshots and source palettes | Implemented |
| Import, export, duplicate, edit, seed reset, and engine rebake | Implemented |
| Stable 32-slot event palette | Implemented |
| Live preview, token isolation, contrast warnings, and responsive editor | Implemented |
| Font family and scale preferences | Implemented |

## Theme model

A theme contains:

- Stable identity and display name.
- Resolved app and Calendar color snapshots.
- Source colors used by the derivation engine.
- Per-token isolation state for authored overrides.
- A 32-slot event palette plus fallback and blend canvas.
- Calendar default-bundle choice.
- Derivation engine version and seed snapshot metadata.

Built-in themes are code-owned and immutable. User themes are normalized SQLite data. The active theme and quick-toggle preferences remain active-vault configuration.

## Principles

- Stored event colors use stable slot identities, not copied hex values.
- Imported themes cannot shadow built-in identities.
- Effective canvas luminance determines runtime light or dark treatment.
- Source edits can derive coherent families without erasing isolated authored values.
- Engine upgrades never silently rebake a user theme.
- Contrast feedback is visible and understandable, but the app does not falsely certify an inaccessible palette.
- Save and Cancel have complete snapshot semantics.

## Documentation map

- [Format and persistence](format-and-persistence.md)
- [Color engine](color-engine.md)
- [Theme authoring](authoring.md)
- [Calendar event colors](../calendar/README.md)
- [Data architecture](../../data/architecture.md)
