# Calendar recurrence testing

Calendar recurrence testing must prove that preview, commit planning, backend persistence, canonical expansion, protected history, and active Pomodoro references agree.

## Automated coverage

Current frontend unit tests exercise normalized field operations, scope switching, projection identities, protected boundaries, plan construction, and executor command sequencing with mocked command boundaries. Current Rust tests independently exercise recurrence batch transactions and Calendar lifecycle protections. This is layered coverage, not an end-to-end proof that a frontend projection matches committed persistence.

Cross-boundary tests must still be added to commit a plan, reload it through the canonical backend expansion path, and compare that result with the projected visible result for each supported recurrence family.

## Required scenario matrix

Each scenario is tested for the template's first occurrence and a generated occurrence where both are valid:

| Draft operation | Scope | Required result |
| --- | --- | --- |
| Non-recurring event gains repeat | No scope selector | Existing row becomes the first template occurrence and keeps its identity. |
| Fields change, recurrence unchanged | Only this | Selected occurrence detaches; source series continues with one exception. |
| Recurrence changes | Only this | Selected occurrence becomes an independent recurring template. |
| Fields or recurrence change | Following | Old template caps before selection and new template starts at selection. |
| Repeat clears | Following | Old template caps and one standalone survivor remains. |
| Recurrence changes, no protected history | All | Template may update directly. |
| Recurrence changes with protected history | All | Historical template remains unchanged through the boundary; mutable template carries the new rule. |
| Repeat clears with protected history | All | Protected history remains and selected mutable occurrence becomes the sole survivor. |

## Protection scenarios

Tests must cover:

- Adding an exception for started, tracked, overridden, active, and future untracked occurrences.
- Moving an end date before protected occurrences.
- Reducing count below protected occurrences.
- Changing a rule so protected dates no longer match.
- Time shifts that would otherwise disconnect event geometry from recorded segments.
- Same-day occurrences on both sides of the captured edit time.
- Exceptions that must transfer across a following split.
- Unsupported or malformed imported rules that cannot be enumerated safely.

## Active-session scenarios

- Adding recurrence to an active non-recurring event retains the base identity.
- Editing the selected active occurrence forces `Only this` and protects its start.
- A following edit across a later active occurrence materializes it unchanged before splitting.
- An all edit from another occurrence leaves the active protected occurrence unchanged.
- Run and segment transfer rolls back with the Calendar transaction on failure.

## Delete and archive scenarios

- Only the selected future untracked occurrence hard deletes.
- Protected selected occurrences archive.
- Following from a started occurrence affects started history without deleting later mutable occurrences.
- Following from a future occurrence preserves protected history and removes the mutable future chain.
- All on a future-only untracked series can delete the template.
- All with protected history preserves the historical side and removes only the intended mutable side.
- Undo restores the complete prior recurrence structure and does not restart Pomodoro history.

## Manual acceptance

For representative daily, weekly, monthly, and advanced rules:

1. Open an occurrence and change scope without editing fields. Verify only the affected contour changes.
2. Edit fields, clear and restore repeat, and move between all scopes. Verify the same draft is retained.
3. Save and confirm the immediate view matches a restart and fresh window load.
4. Repeat with started history and an active occurrence elsewhere in the series.
5. Trigger a persistence failure and confirm no partial structure or stale preview remains.
