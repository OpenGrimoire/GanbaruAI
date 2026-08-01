# Agent coordination

Ganbaru AI treats AI as a delegable workforce rather than a collection of isolated assistant chats. Chat is the communication and coordination layer where a person, human collaborators, stable AI roles, and task agents discuss work. Projects remains the canonical work record, Notes remains durable knowledge, Calendar remains the record of time and capacity, and provider sessions remain bounded execution machinery beneath those surfaces.

This is the ideal product direction. The current coding-agent Chat already provides native provider sessions, durable events, approvals, terminals, review, worktrees, and checkpoints. Channels, direct messages, structured planning, automatic delegation, and human collaboration are later layers built on that foundation.

## Product objective

The normal workflow should let a person describe an objective at the level they understand, review the proposed commitments, and return when judgment is needed. It should not require them to create a separate AI chat for every action, repeatedly explain shared context, or supervise every implementation step.

The default loop is:

1. The person describes an objective, problem, observation, or change in a channel or direct message.
2. The AI project manager identifies missing information and asks only questions that materially block planning.
3. The manager proposes structured project changes, including tasks, subtasks, dependencies, acceptance criteria, estimates, deadlines, assignments, reviewers, budgets, risks, and scheduling effects.
4. The person reviews the proposal according to the project's authority policy. Major scope, cost, deadline, security, or destructive changes always remain explicit.
5. Approved work becomes canonical Projects data. Calendar receives proposed or confirmed capacity commitments. Notes receives durable specifications, research, and decisions.
6. Each delegated task starts a narrowly scoped agent run with only the context, tools, workspace, authority, and budget it needs.
7. Runs report concise structured state such as started, blocked, awaiting approval, ready for review, completed, failed, or budget exhausted.
8. Review happens from the task and its deliverables. The person can open the detailed agent timeline when needed, but raw execution chat is not the normal project-management surface.
9. Feedback can correct the deliverable, update the requirement, or create related work without requiring the person to reconstruct context in another isolated conversation.

The user remains accountable for intent and consequential decisions. Ganbaru reduces coordination work without pretending that generated work is automatically correct.

## System boundaries

| Layer | Responsibility | Canonical surface |
|---|---|---|
| Communication | Requests, discussion, questions, decisions in progress, announcements | Chat |
| Work | Tasks, subtasks, dependencies, assignments, reviews, estimates, budgets, deadlines | Projects |
| Knowledge | Specifications, research, explanations, meeting notes, durable decisions | Notes |
| Time and capacity | Scheduled blocks, hard deadlines, availability, workload, focus plans | Calendar |
| Execution | Provider sessions, tools, commands, worktrees, files, checkpoints, usage | Agent runs linked to Chat and Projects |

Chat messages can propose or trigger writes in another layer, but the message is not the only record of a commitment. A created task, accepted requirement change, scheduled block, or durable decision links back to its originating message, participant, reason, and approval when available.

## Organizational model

The Ganbaru AI folder is the local organizational workspace. Project groups and projects provide work context. Chat adds durable communication places without adding another level to the Projects hierarchy.

### Channels

A channel is a durable room with a name, topic, scope, membership, archive state, and chronological communication history. It is organized around a purpose rather than a provider, model, working folder, or individual execution attempt.

Channels can eventually be scoped to the whole Ganbaru AI folder, a project group, or one project. The first channel slice should be project-scoped and create only one default `#general` channel per project. Additional channels such as `#planning`, `#research`, `#decisions`, or `#release` are deliberate user choices or template choices, not automatic copies of every task status or project section.

A channel is permanent even when its AI execution session is not. Ganbaru can compact, fork, replace, or change the backing provider session without changing the channel identity or forcing the person into a new visible conversation.

### Sidebar sections

Sidebar sections organize channels for navigation. They do not change project ownership, task sections, scheduling, permissions, or source-of-truth boundaries. Personal ordering and collapsed state can remain local preferences. Shared section organization may be added later only if it provides clear team value.

### Direct messages

A direct message is a durable conversation with stable participants. A participant can be the local person, a future human collaborator, or a stable AI role such as the Ganbaru manager or a reviewer. Provider and model identity are execution choices, not participant identity. Changing the provider behind the Ganbaru manager does not create a new DM or a new organizational role.

Temporary task agents do not automatically become permanent DM entries. A person can promote a useful role to a stable participant, but ordinary execution stays attached to its task.

### Participation and invocation

A conversation records who authored a message and which participants or roles it addresses. Human communication is persisted and later synchronized without requiring an AI provider. Merely placing an AI role in a channel does not silently send every message, attachment, or linked resource to that role's provider.

The default channel policy is explicit invocation. A person addresses a role, uses a role action, or opens a DM with that role. The Ganbaru manager can be available in a project's `#general` channel once a compatible provider is configured, but ordinary unaddressed messages remain local organizational history. A DM with the manager addresses it by definition.

Later projects can enable bounded subscriptions such as manager triage or a scheduled digest. A subscription states which conversations it observes, what event activates it, which context it may read, its frequency and budget, and whether it can only propose or can perform approved routine actions. Subscription activity is visible and can be paused. There is no hidden always-reading agent.

### Task discussions

Every delegated task can expose a focused discussion containing the task owner, project manager, assigned worker, reviewer, and relevant invited participants. The discussion is reached from the task or a linked channel entry instead of filling the main sidebar with disposable worker chats.

The person can ask the agent that performed the work about a decision, request a small correction, or add the manager to interpret broader consequences. If the native provider session can resume safely, Ganbaru may reuse it. Otherwise, a replacement session receives the task record, approved context package, run summary, artifacts, and relevant discussion. The UI preserves continuity without claiming that a new model session is literally the same mind.

### Replies and execution details

Message replies keep a focused conversational branch inside a channel or DM. They are not provider sessions. Agent-run timelines, tool activity, terminal sessions, review state, and checkpoints are execution details opened from a task, message, or run status entry.

The existing internal term `chat thread` refers to a provider continuation and execution timeline. The durable organizational model must not reuse that internal meaning for a Slack-style reply thread.

## AI roles and identity

An AI role is a stable organizational identity with a purpose, authority policy, context policy, and default execution preferences. It is separate from the provider instance and model used for a particular turn or run.

The first central role is the Ganbaru manager. Projects may later define specialized roles such as researcher, implementer, reviewer, scheduler, or release coordinator. A role is not assumed to be competent merely because it has a name. Its capabilities come from the selected provider, available tools, authorized data, and verified workflow.

Role identity supports consistent communication, but durable memory comes from canonical Ganbaru data and bounded summaries. Ganbaru never treats an unbounded raw conversation transcript as the sole memory of a project.

## Planning and commitment

A manager plan is a reviewable proposal over canonical work. It can include:

- Objective and success criteria.
- Tasks, subtasks, checklists, and milestones.
- Dependencies and critical-path effects.
- Assignee and reviewer.
- Estimate ranges and confidence.
- Start constraints, target dates, and hard deadlines.
- Human time, scheduled time, token, monetary, and iteration budgets.
- Required Notes, files, tools, working folders, and external resources.
- Risks, assumptions, breaking changes, and rollback expectations.
- Review requirements and expected review capacity.
- Reasons for discarded ideas or rejected approaches.

Proposals distinguish suggestions from commitments. Creating a task does not silently schedule it. Estimating a task does not reserve budget. A target date is not a hard deadline unless the project record says so. Each transition is explicit in the structured record and can be controlled by project-level authority preferences.

Requirement changes record what changed, who or what requested it, why, the originating discussion, approval state, and affected tasks, dates, budgets, and deliverables. Breaking changes and discarded approaches remain discoverable so later agents do not repeat rejected work without new evidence.

## Context packages

Each manager action and agent run receives an explicit context package rather than unrestricted access to an entire project or channel history. A context package can contain:

- The objective, task, acceptance criteria, and relevant requirement revisions.
- Direct dependencies and blocking tasks.
- Selected Notes pages, Notes folders, files, and approved excerpts.
- Relevant Calendar constraints and hard deadlines.
- Required project working folder and execution environment.
- Applicable project instructions and provider configuration.
- Recent channel messages or a bounded channel brief.
- Prior run summaries, decisions, review comments, and artifacts.
- Resource budgets and the escalation policy.
- The requesting participant, destination conversation, and effective access scope.

The package is inspectable and records its source identities and revisions. A run does not automatically inherit later edits. Ganbaru either starts a new package revision or explicitly steers the run with the approved change. This prevents silent context drift and makes failures reproducible.

## Delegation and execution

An agent run is a bounded execution record linked to a task or other explicit objective. It records the role, provider, model, working folder, execution environment, context package, authority, budgets, lifecycle state, deliverables, usage, and relevant provider continuation.

Parallel execution is allowed only when workspace ownership, dependencies, budgets, and review capacity permit it. Two agents do not mutate the same execution environment concurrently. Coding work uses isolated worktrees or otherwise separate environments when safe parallelism is required.

The manager monitors exceptions instead of narrating every tool call into a shared channel. Normal channel updates are concise and structured. Detailed provider events remain available in the run timeline for inspection, recovery, and audit.

An addressed AI message first becomes durable organizational history, then starts a manager action or agent run. The room shows one linked run status rather than copying its raw event stream. Questions, approval requests, blockers, and failures become actionable entries. A settled role response is posted to the room under the stable AI-role identity and links to the exact run, provider, model, context package, usage, and artifacts that produced it. Provider startup failure cannot erase the person's message.

Every review-ready run produces a review packet that can be understood without reading the provider transcript. It includes:

- A concise result summary and links to the actual deliverables.
- Each acceptance criterion with supporting evidence or an explicit unmet state.
- Verification performed, including tests, checks, research sources, or manual validation still required.
- Decisions, assumptions, requirement deviations, and breaking changes discovered during execution.
- Changed resources, downstream effects, unresolved risks, and proposed follow-up work.
- Actual token, cost, iteration, and time usage against the applicable budgets.
- The exact context-package and run revisions that produced the result.

The packet is evidence for review, not automatic proof of correctness. A reviewer can open the complete run timeline, artifacts, files, or provider continuation when the summary is insufficient.

## Review and exception management

Delegation is incomplete until its deliverable has the required review. A task can define a human reviewer, AI reviewer, or both, but an AI review never substitutes for a required human approval.

The manager tracks:

- Work waiting for review.
- Reviewer due dates and capacity.
- Review lateness and its recorded reason.
- Requested changes and whether they alter scope.
- Failing acceptance criteria.
- Breaking changes and downstream impact.
- Workspace or dependency conflicts.
- Budget, usage, and deadline risks.
- Runs that are stalled, failed, or repeatedly asking the same question.

Review feedback can create a small correction within the current task, revise the requirement with provenance, or create a related task. The manager proposes the correct category instead of hiding scope growth inside another prompt.

## Anti-burnout constraints

Agent multiplication can move burnout from implementation to supervision. Ganbaru therefore treats human attention and review capacity as constrained resources.

- Projects can set work-in-progress and review-queue limits.
- The manager prefers dependency-aware batches over maximum parallelism.
- Completed work can pause new execution when review debt exceeds the configured limit.
- Routine progress is summarized in digests instead of producing constant notifications.
- Mentions, approvals, hard blockers, deadline risk, and required reviews are separated from informational activity.
- Unread counts do not imply that every message must be read.
- Raw tool logs never become shared-channel noise by default.
- Calendar and recent workload can inform a recommendation, but private productivity signals are not exposed to other participants.

The system optimizes for completed, reviewed work and sustainable attention, not the number of active agents or messages.

## Authority and safety

The manager and task agents act only within explicit authority. Project settings can eventually distinguish actions that are allowed automatically, allowed within a budget, require review, or are always prohibited.

Destructive operations, security changes, credential access, publication, external messages, financial commitments, permission expansion, and broad scope changes require the same or stronger confirmation as the underlying operation. Delegation cannot be used to bypass provider approvals, workspace authorization, or Ganbaru safety boundaries.

Automatic task and calendar writes use typed Ganbaru operations. Agents do not edit the SQLite database directly. Command receipts, originating messages, proposal revisions, approvals, and resulting mutations provide an audit path.

## Future human collaboration and access

Human collaboration is deferred until the local single-user workflow, sync, identity, encryption, and permission model are mature. The current design must still leave room for a person to join:

- A project group and its permitted projects.
- One project without access to the whole group.
- Selected channels inside a project.
- Selected Notes folders or pages.
- Selected tasks or task discussions.
- Explicit project working folders when filesystem access is necessary.

Membership in one scope does not imply access to every resource with the same project id. Notes folders and project working folders are separate resource types and require separate grants. Search, mentions, backlinks, notifications, reports, exports, summaries, and AI context assembly must apply the same effective permission boundary as direct reads.

AI uses the intersection of the requesting participant's authority, the destination conversation's visibility, the selected role's policy, and the agent run's explicit grants. It must not reveal restricted information through a summary, task description, generated report, or explanation of why access was denied.

Future collaboration roles may include owner, administrator, member, and restricted guest. Exact inheritance, revocation, encrypted key distribution, historical-message visibility, and offline access removal belong to the sync and permission design. Revocation must stop future reads and context assembly without rewriting legitimate shared history.

UX can reserve restrained space for future participant avatars, membership, channel visibility, and invitation actions. Until collaboration works, controls are absent or clearly disabled with an explanation. A control never appears functional while doing nothing.

## Delivery sequence

The product should grow in layers:

1. Define channels as durable conversations above the existing provider sessions.
2. Replace working-folder and isolated-chat navigation with channel navigation, while keeping working folder and provider controls available where execution needs them.
3. Add stable participant and AI-role identity, then direct messages and task discussions.
4. Link conversations, messages, tasks, Notes, Calendar events, and execution runs with provenance.
5. Add reviewable manager plans and typed writes to Projects and Calendar.
6. Add context packages, budgets, task-linked agent runs, review queues, and controlled parallel execution.
7. Add permission-aware sync and human collaboration after local behavior and access rules are proven.

Each layer must remain useful without the next one. The initial channel redesign must not pretend to provide autonomous project management, and later autonomy must not make ordinary local Chat depend on sync or a hosted service.
