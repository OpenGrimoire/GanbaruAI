# Feature documentation

Feature documents describe the intended product behavior and the reasons behind it. They are normative unless a section is explicitly labeled as current implementation status.

Implementation status uses four terms:

- **Implemented:** the current source provides the documented core behavior.
- **Partial:** a useful slice exists, but a documented part remains incomplete or unvalidated.
- **Planned:** the behavior is product direction and has no complete implementation yet.
- **Deferred:** the behavior is intentionally outside the active delivery sequence.

Source paths, schema inventories, cache tuning, device test matrices, and delivery history belong in architecture, data, performance, testing, or roadmap documentation. Feature specs keep security bounds and limits when those bounds affect user expectations or protect durable data.

## Current and active domains

| Domain | Status | Documentation |
| --- | --- | --- |
| Calendar | Partial | [Calendar](calendar/README.md) |
| Pomodoro | Partial | [Pomodoro](pomodoro/README.md) |
| Projects | Partial | [Projects](projects/README.md) |
| Notes | Partial | [Notes](notes/README.md) |
| Chat | Partial | [Chat](chat/README.md) |
| AI integrations | Partial | [AI integrations](ai/README.md) |
| Doomscrolling | Partial | [Doomscrolling](doomscrolling/README.md) |
| Music | Partial | [Music](music/README.md) |
| Themes | Implemented | [Themes](themes/README.md) |
| Localization | Partial | [Localization](localization.md) |
| Quick notes | Implemented | [Quick notes](quick-notes.md) |
| Profile | Implemented | [Profile](profile.md) |

## Planned product surfaces

- [Diary](diary.md)
- [Edge panel](edge-panel.md)
- [Sleep alarm](sleep-alarm.md)
- [Work environments](work-environments.md)

## Deferred product surfaces

- [Gamification](gamification.md)

Platform-specific behavior lives under [platform documentation](../platforms/README.md). Data ownership and persistence rules live under [data documentation](../data/README.md). Pure logic that can be specified independently of UI or persistence lives under `docs/algorithms/`.
