# Work environments

A work environment is a saved configuration that the calendar activates automatically when a session block starts. It covers what apps to open, what browser tabs to load, what playlist to play, what blocker rules to enforce, and what project, task, Notes, or Chat context to suggest. The user defines environments once during planning; the app applies them throughout the week.

This doc is a placeholder. Deeper design comes in a later pass.

## What an environment contains

| Setting | Examples |
|---------|----------|
| Apps to open | VSCode, Figma, terminal, Slack |
| Apps to close or minimize | Discord, browser windows from a different env |
| Browser tabs | A list of URLs opened in a designated browser window or group |
| Music playlist | Local file, YouTube playlist, or environment-default playlist |
| Blocker rules | Which sites and apps are off-limits (see `features/doomscrolling.md`) |
| Edge panel context | Which project context to display |
| Ganbaru context | Project plus optional task, Notes page, or Chat channel to surface |

## How activation works

When a calendar session block reaches its start time:

1. The calendar reads the block's environment assignment.
2. The work environment manager closes apps not in the new environment (or minimizes them, configurable).
3. Opens apps specified in the new environment.
4. Opens the correct browser tabs via the browser extension.
5. Starts the assigned playlist.
6. Updates the blocker rules.
7. Updates the edge panel context.
8. Selects the linked project and offers the task, Notes page, or Chat channel needed for the session.

This eliminates the daily friction of arranging the workspace manually. The user decides once during weekly planning; the app applies it throughout the week. Context activation must preserve drafts, open review state, and active execution. It can suggest or open a linked channel or task, but it never retargets a running provider continuation or silently discards the person's current UI state.

## Manual activation

Environments can also be activated manually from the edge panel or settings, independent of any calendar event. This is useful for ad-hoc work where the user has not planned a calendar block.

## Templates and inheritance

Environments are templates: "Deep Work: Project X," "Admin," "Creative Writing." A session block references a template; if the template changes, all blocks using it pick up the change on next activation.

Per-block overrides are possible (e.g., the same template but a different playlist for one specific block). Overrides are stored on the block, not the template.

## Why desktop-only

Environment management requires controlling other applications (opening, closing, focusing windows). Mobile sandboxing prohibits this. On mobile, Doomscrolling's app-level blocking is the closest equivalent, but it does not orchestrate apps the way a desktop environment does.

## Linkage to other systems

- **Calendar:** the source of activation triggers.
- **Music:** plays the assigned playlist.
- **Doomscrolling:** enforces the assigned rules.
- **Edge panel:** displays the environment name and the relevant project context.
- **Pomodoro:** runs alongside; the timer is independent of the environment but typically pairs with one.
- **Projects and Notes:** surface the linked task and supporting knowledge for scheduled human work.
- **Chat:** surfaces the linked channel, task discussion, or attention item without turning the environment into a provider-session owner.

A human work environment and an AI execution environment are related but distinct. Calendar activation may surface a channel or folder for the person, but it never grants a teammate access or changes an active run. An AI run selects one authorized current-folder, worktree, or scratch execution environment under [Chat access control](../data/access-control.md). Other granted folders remain secondary application-brokered resources.
