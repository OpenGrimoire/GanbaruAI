# Chat teammates and coordination

Ganbaru AI coordinates work through durable conversations, explicit assignments, and review. AI providers are execution mechanisms beneath that model, not organizational identities.

## Participants and teammates

A participant can be the local person, a future human collaborator, or a persistent AI teammate. A teammate has a stable identity and purpose independent of provider and model. Changing its provider does not create a new participant, direct message, or history.

Teammate profiles may define:

- Display identity and purpose.
- Projects and channels where invocation is allowed.
- Default provider, model, interaction mode, and budgets.
- Context policy and authorized working-folder class.
- Review expectations and escalation rules.

Profiles never grant authority by themselves. Effective access is resolved for each assignment.

## Participation and invocation

Ordinary messages do not invoke every visible AI teammate. Work starts through an explicit mention, a typed assignment, a documented workflow, or a user-approved scheduled action.

Mentions distinguish reference from invocation. Quoting a teammate's name in prose is not enough to start work. The UI should make an actionable invocation visible before execution begins.

## Planning and commitment

Conversation can explore ideas without creating durable tasks. A proposal becomes committed work only through an explicit transition that records its objective, project, owner, authority, budget, expected output, and review destination.

Planning may produce tasks, checklists, decisions, or project updates. Generated proposals remain drafts until accepted through the owning feature's typed command. See [Guided project planning](../projects/guided-planning.md).

## Context packages

A context package is a bounded, reproducible selection of information assembled for one assignment. It can include the initiating messages, selected project records, approved Notes, task state, calendar context, attachments, and working-folder references.

Context assembly follows participant access and the run's execution target. It does not scan the complete vault or attach unrelated project history because it might be useful. The recorded package should make later review possible without treating provider memory as canonical.

## Delegation and execution

Delegation creates explicit child assignments with their own objective, target, authority, budget, and result link. A teammate cannot delegate authority it does not hold. Parallel work remains visible as separate assignments or runs rather than becoming hidden provider-native subagents.

The responsible teammate reports material exceptions, decisions, blockers, and results. It does not narrate every tool call into a shared channel. Detailed execution stays available in the run timeline.

## Review and exceptions

Review focuses attention on decisions and risk:

- What changed or was proposed.
- What could not be completed and why.
- Which assumptions or approvals affected the outcome.
- What requires human judgment.
- Which checkpoint or files contain the result.

Approval of one result does not grant standing approval for later work. Rejected or superseded proposals remain understandable in history.

## Anti-burnout constraints

Coordination should reduce work fragmentation rather than create pressure to supervise agents continuously. Defaults should favor bounded work, quiet progress, batch review, explicit deadlines, and visible budgets.

The system must not manufacture urgency, shame inactivity, optimize for message volume, or use teammate activity as proof of human productivity. Scheduled work respects quiet periods and can defer non-urgent notifications.

## Authority and safety

Every assignment resolves access through the canonical [access-control model](../../data/access-control.md). Provider-native trust, a prior run, a teammate profile, or membership in a broad channel cannot widen organizational authority.

Destructive, externally visible, credential-related, security-sensitive, or broad-scope actions require typed boundaries and appropriate confirmation. Revocation stops future reads and context assembly without rewriting legitimate historical messages.

## Human collaboration

Future collaboration roles may include owner, administrator, member, and restricted guest. Encryption, key distribution, revocation, historical visibility, offline copies, and conflict resolution belong to the sync and permission design. Controls remain absent or clearly unavailable until the corresponding behavior is real.
