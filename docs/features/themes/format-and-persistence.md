# Theme format and persistence

## Registered format

Every theme passes the same validation boundary before it enters the registry. The format includes identity, display metadata, resolved token snapshots, source values, isolation flags, Calendar defaults, event palette, blend canvas, and derivation engine version.

Unknown fields may be ignored only under a documented forward-compatible rule. Missing required values, invalid hex colors, unknown token identities, malformed slot data, duplicate identity, or unsupported format version fail with actionable diagnostics.

## Token snapshots

The current editable catalog contains 36 app tokens and 7 Calendar tokens. The token identities are the compatibility contract; the numeric count is only a current fact and must change with the authoritative catalogs.

User themes store every current editable token as a resolved hex snapshot. Runtime-only implementation colors derive from stored values and are not exported as editable rows.

Sources are stored alongside snapshots for future source-driven edits. Paint uses the resolved snapshot so later engine changes cannot silently alter an existing theme.

## Event palette

Every theme owns exactly 32 event-color slots with stable numeric identity. Calendar events store the slot index. Changing a theme recolors an event without rewriting event records.

Invalid or unknown event slot values fall back to the theme's fallback slot. Palette evolution preserves slot identity. Reordering visible color controls does not renumber persisted slots.

The blend canvas controls dimmed event variants and normally follows the Calendar canvas unless the relevant token is isolated.

## Persistence ownership

User-authored themes and their seed snapshots live in normalized SQLite records. Built-in light and dark themes stay code-pinned and never become database rows.

The active theme ID, quick-toggle light and dark choices, font family, and font scale live in active-vault `config.json` as portable preferences. Theme content does not live in that config blob.

Desktop and mobile platform bootstraps load vault configuration and hydrate user themes before mounting their application shell. The shared `main.ts` platform selector does not own this boot sequence.

## Built-in protection

Reserved built-in IDs cannot be inserted through user import or persistence. Unknown active IDs fall back to the default built-in theme. Built-in definitions also pass validation as defense against source drift.

Editing a built-in creates an editable user copy or previews the built-in without persisting changes to its definition.

## Seed snapshots

A user theme retains a seed snapshot that supports `Reset all to seed`. Duplicating a theme creates a new independent seed. Import establishes the validated imported content as the initial seed unless the import format explicitly carries a compatible seed policy.

Reset replaces the editable theme content in the current authoring buffer only. Save is still required to persist it.

## Import and export

Export is deterministic and versioned. It includes authored identity, snapshots, sources, isolation, event palette, Calendar defaults, blend behavior, and engine version without including device-local selection preferences.

Import validates the complete object before registration. It never partially applies token rows. Identity collisions require an explicit new identity or replacement flow; built-in collision is always rejected.

Imported older engine versions remain paintable from their resolved snapshots and receive an explicit rebake opportunity.

## Schema evolution

Adding a new editable token requires a default for built-ins, user-theme upgrade behavior, seed behavior, import handling, editor placement, and persistence migration. Removing or renaming a token requires an explicit drop or mapping rule so dead persistent values do not remain hidden.

Exact tables and migrations belong in [data documentation](../../data/README.md).
