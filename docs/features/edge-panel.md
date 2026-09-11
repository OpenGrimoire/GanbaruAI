# Edge panel

The Edge panel is a planned desktop glance and quick-action surface for the current focus context. It should reduce window switching without reproducing Calendar, Projects, Notes, Chat, or Music.

## Product goal

A user can check time remaining, control Music, capture a task, or open the current work context with one deliberate action. The panel remains quiet and peripheral when unused.

## Reveal and placement

The final reveal mechanism must work across supported desktops, Wayland and X11 differences, multiple monitors, touch, keyboard-only use, and accessibility tools. Possible adapters include a configured screen edge, tray or title-bar action, global shortcut where supported, or an optional pinned window.

The spec does not require a universal right-edge mouse hook or a particular low-level input dependency. Users choose placement and reveal behavior where the platform permits it.

## Contents

| Section | Purpose |
| --- | --- |
| Focus | Remaining opportunity and applicable pause, resume, or transition action |
| Current context | Active Calendar block and linked project, task, Note, or Chat conversation |
| Quick task capture | Create a bounded task draft in the selected project |
| Attention | Urgent approval, blocker, or review for the current context only |
| Music | Play, pause, previous, next, and volume through the canonical player |
| Navigation | Open the relevant full feature surface |

The panel does not show a general unread feed, productivity score, complete task list, or provider event stream.

## Safety and state

Opening context never discards a draft, retargets a running provider continuation, changes Chat authority, or silently selects a working folder. Quick actions use the same typed feature commands and confirmation rules as their owning surfaces.

## Platform boundary

The always-available panel is desktop-specific. Mobile equivalents use the app top bar, notifications, and future operating-system widgets. Portable current-context data can be shared even though the window and input adapter are desktop-only.

See [Pomodoro progress](pomodoro/progress-displays.md), [Music](music/README.md), [Projects](projects/README.md), and [Work environments](work-environments.md).
