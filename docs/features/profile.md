# Profile

Profile is the folder-local identity reused by communication, assignment, review, and future collaboration surfaces. Settings stores an optional display name, full name, and profile picture. The display name is the primary visible identity. Surfaces that require a visible label fall back to localized contextual copy such as `You`.

The profile picture picker accepts PNG, JPEG, and WebP files up to 3 MB. Ganbaru copies the selected image into `assets/profile/` using a content-derived name and stores only its managed relative path in `config.json`. Replacing or removing the picture deletes the previous managed file. SVG is not accepted because it can contain interactive or external content.

Profile avatars are square with softly rounded corners. When no image is selected, the avatar displays the first character from each of the first two display-name words. The same renderer is used by Chat messages and local Projects assignee or reviewer placeholders so identity remains visually consistent.

The local profile represents a human participant. Persistent AI teammates such as the Ganbaru manager have distinct Agent-labeled identities, and provider accounts or models are execution metadata rather than people. The UI must not let an AI teammate impersonate the local person or a future collaborator.

Future collaboration adds participant and device identity, invitations, membership, and resource grants without turning the local profile into a global hosted Ganbaru account. A collaborator can expose a display identity to authorized rooms and projects while keeping unrelated personal profile data private. Identity visibility follows the same channel, project, Notes, task-discussion, and working-folder access boundaries as content.
