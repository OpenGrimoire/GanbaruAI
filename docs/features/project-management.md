# Project management

Project management is Ganbaru AI's guided system for turning intent into reviewed, scheduled, and completed work. It combines a flexible lifecycle framework with the canonical task layer in Projects, durable knowledge in Notes, capacity in Calendar, and communication plus delegation in Chat.

The feature is not a rigid wizard and Chat is not a second task database. A person can begin with an incomplete idea, discuss it with an explicitly configured planning teammate or collaborators, approve a structured plan, and then manage the resulting commitments from Projects. See [Agent coordination](agent-coordination.md) for the teammate, channel, context-package, agent-run, and review model. No planning teammate is seeded or privileged by the base system.

## Product objective

The system should reduce the mental work of repeatedly decomposing, explaining, assigning, monitoring, and rescheduling a project. It should preserve human control over scope and consequential decisions while allowing routine coordination to become explicit and eventually automated.

A successful project flow answers:

- What outcome are we trying to create, and how will we know it succeeded?
- What information, decisions, tasks, and dependencies are missing?
- Who or what owns each task, and who reviews it?
- What human time, Calendar capacity, money, tokens, tools, and files are available?
- Which dates are targets, which are hard deadlines, and what happens if something slips?
- Which assumptions, risks, breaking changes, rejected approaches, and unresolved questions exist?
- What work is ready, blocked, running, waiting for review, accepted, or no longer relevant?
- What changed, who requested it, why, and which commitments are affected?

## Canonical boundaries

Projects owns plans once they become commitments: tasks, subtasks, dependencies, milestones, assignments, reviews, estimates, budgets, dates, status, requirement revisions, and execution links. Notes owns specifications, research, meeting notes, and other durable documents. Calendar owns reserved time and hard scheduling constraints. Chat owns the discussion and provenance that led to each proposed or accepted change.

A conversational plan is a proposal until accepted. A channel message is not a task merely because it sounds actionable. The manager can suggest structured changes, but Ganbaru records the transition from suggestion to commitment and the authority under which it occurred.

## Quest chain structure

The project lifecycle is presented as a sequential set of phases. NPCs in the eventual visual layer can guide first-time users through each phase, while returning users can jump, skip, reorder, or replace phases. The framework helps work start; it does not force every project through the same ceremony.

### Phase 1: Genesis (brainstorming)

Free-form ideation with structured prompts: write ideas, choose the best ones or combine them, iterate until a sufficiently attractive idea emerges, and preserve why alternatives were discarded. Discard reasons can include outside current capabilities, too expensive, already exists, poor fit, weak demand, excessive risk, wrong timing, or replaced by a stronger approach.

Ideas remain Notes or structured candidates until the person promotes one into a project objective. An AI teammate can research and compare candidates, but it does not silently discard a user-authored idea or turn it into committed work.

### Phase 2: Forging the idea (evaluation)

Evaluate each candidate against three criteria:

1. **Want:** whether the person or team genuinely wants to do it.
2. **Can:** whether it is achievable with available knowledge, authority, time, money, tools, and risk tolerance.
3. **Need:** whether it addresses a real problem or desire. Revenue or another sustainability mechanism can strengthen this dimension.

A candidate that scores well on all three is a strong project. Weak scores are explicit risks rather than reasons for the manager to manufacture confidence. Evaluation records evidence, assumptions, uncertainty, and the decision that follows.

### Phase 3: The journey ahead (planning and execution)

The most substantial phase is divided into planning, minimum viable product, execution, and post-execution.

**3.1 Planning** covers deep brainstorming, market analysis, competitor research, specification, resource estimation, execution constraints, and a final viability review. Each area has a structured template and can create Notes, decisions, risks, tasks, or follow-up research.

**3.2 Minimum viable product** covers recruiting collaborators, proof of concept, MVP development, updates from MVP feedback, and optional funding preparation.

**3.3 Execution** covers the detailed execution plan, alpha and beta work, launch preparation, polish, release, and the review and acceptance needed at each boundary.

**3.4 Post-execution** covers contingency handling, stakeholder updates, expansion to new platforms or regions, maintenance, and product evolution.

## Manager planning flow

An explicitly authorized planning teammate converts an objective into a reviewable proposal. A proposal can include:

- Objective, scope, exclusions, and success criteria.
- Tasks, subtasks, checklists, milestones, and acceptance criteria.
- Dependencies, critical-path effects, and parallelism opportunities.
- Assignee, reviewer, and required approval type.
- Estimate range, confidence, and estimation assumptions.
- Start constraints, target dates, hard deadlines, and scheduling options.
- Human-effort, scheduled-time, monetary, token, and iteration budgets.
- Required Notes, project files, working folders, external resources, and tools.
- Risks, unknowns, breaking changes, rollback expectations, and escalation conditions.
- Review effort and the effect of the review queue on completion.

The proposal shows which parts were inferred. The person can accept all, accept selected changes, edit them, request another plan, or reject them with a reason. Acceptance creates canonical Projects records through typed commands. It never depends on parsing the manager's prose later.

Project settings eventually define bounded authority for routine changes. Creating or changing hard deadlines, financial commitments, publication, credentials, security settings, destructive work, permission grants, or broad project scope always requires explicit authorization regardless of automation preferences.

## Tasks and subtasks

Use checklist items for simple steps that only need done or not done. Use subtasks when work needs separate ownership, review, status, dates, scheduling, dependencies, Notes, a context package, or an agent run.

An agent-ready task should have a clear objective, acceptance criteria, relevant inputs, deliverable type, allowed tools, required working folder, budget, and escalation policy. The manager can identify that a task is not ready and create prerequisite research or decision tasks instead of sending an underspecified request to a worker.

Task state distinguishes execution from acceptance. A worker marking output complete can move a task to review-ready, but only the required review can move it to accepted or a terminal Done status. Projects with simpler workflows can map review-ready directly to Done when no separate review is required.

Review-ready work includes a structured packet with deliverables, acceptance-criteria evidence, verification, decisions, deviations, breaking changes, unresolved risks, follow-up proposals, budget usage, and the context and run revisions that produced it. This is the default review surface. The raw provider transcript remains available for investigation but is not required reading.

## Assignment and review

Assignments can target the local person, a future human collaborator, a persistent AI teammate, or a bounded agent run. Reviewer assignment is separate from worker assignment. A task can require a human reviewer, an AI reviewer, both in sequence, or no separate review.

The manager tracks work waiting for review as capacity, not as an invisible final step. It records a review target date, current reviewer, late state, and reason when known. Reasons can include reviewer unavailable, requested changes, failing acceptance criteria, dependency changed, breaking change discovered, missing evidence, environment failure, or scope expansion.

Review feedback has three explicit outcomes:

- **Correction:** the deliverable does not satisfy the existing requirement and returns to the current task.
- **Requirement revision:** the expected result changed, so the task history records the new requirement, requester, reason, approval, and downstream effects.
- **Related work:** the deliverable is acceptable for the current scope, but the feedback creates a new linked task.

This prevents scope growth from hiding inside repeated prompts to the same agent.

## Budgets and sustainable capacity

Projects can eventually define project, milestone, task, and run budgets. Budget kinds include human effort, Calendar time, money, tokens, provider cost, and bounded retry or iteration counts. Estimates and budgets are separate: an estimate predicts consumption, while a budget constrains or authorizes it.

The manager considers provider quotas, execution environments, task dependencies, available Calendar blocks, work-in-progress limits, and review capacity before starting parallel work. More agents are not automatically better. When review debt exceeds the configured limit, Ganbaru can pause new execution, recommend a smaller batch, or escalate the deadline risk.

Private productivity data can inform a person's own scheduling suggestions. Future team planning receives only coarse, privacy-safe availability or capacity signals, never another person's focus, idle, break, or diary details.

## Requirement and decision history

Every meaningful change to a task's description, acceptance criteria, scope, assignment, reviewer, estimate, budget, dates, dependencies, or execution authority creates a timestamped revision or change event. It records:

- What changed.
- When it changed.
- Who or what requested it.
- Why it changed.
- The originating message, plan, review, or external reference when available.
- Who approved it or which bounded policy allowed it.
- Which tasks, dates, budgets, Notes, deliverables, and agent runs are affected.

History is permanently attached to the relevant project records and remains searchable and exportable. User-authored reasons are preserved. Generated explanations are labeled as AI-authored and cannot overwrite the requester's explanation.

Decisions and discarded approaches are also durable. A later manager or agent can see that an approach was rejected and why, without treating the rejection as an eternal prohibition. New evidence can reopen the decision through another recorded revision.

## Date cascade and replanning

When scope, estimates, assignments, reviewer availability, or dependencies change, the system calculates downstream effects. It distinguishes a proposed target shift from a hard-deadline conflict and identifies the reason for each affected task or Calendar block.

Replanning is reviewable. Ganbaru can propose moving work, reducing scope, adding capacity, changing parallelism, or accepting deadline risk. It does not silently move protected Calendar history or reinterpret a hard deadline as flexible. Approved schedule changes use normal Calendar and task-event-link commands.

## Automatic reports and attention views

Project status reports can be generated from task state, Calendar plans and actuals, Pomodoro history, requirement changes, milestones, budgets, agent runs, review queues, risks, and decisions. Reports are read-only projections in Markdown or PDF; editing a generated report does not alter canonical data.

Chat and Projects expose attention views for approvals, blockers, review-ready work, late reviews, budget risk, deadline risk, and failed runs. Routine progress belongs in digests rather than an interrupting message stream. Unread communication and actionable attention are separate concepts.

## Methodology templates

Beyond the lifecycle, the system includes guided forms for established methodologies such as reverse brainstorming, value proposition canvas, business model canvas, SWOT analysis, and market research frameworks. Each is structured and actionable rather than a static template that the user has to interpret.

Templates can suggest channels, Notes, teammates, tasks, review requirements, and project defaults, but they create only the minimum useful structure. They do not create a channel for every phase, task, or status.

## Software repository integration

When a project uses a software repository, the `ganbaru-ai` CLI exports repo-facing Markdown views such as `KANBAN.md` and generated reports. This makes approved context available to collaborators who do not run Ganbaru AI and to agents that read the repository natively. The export is a view; SQLite remains the source of truth. Changes to exported Markdown can be imported only when that export type has an explicit validated import path.

Agent runs link to exact project tasks, authorized working folders, context-package revisions, branches or worktrees, checkpoints, reviews, and deliverables. A commit, pull request, or file change is evidence of work, not proof that its task is accepted.

## Future collaboration and permissions

Human collaboration is deferred. The design still allows a person to join a project group, one project, selected channels, selected Notes folders or pages, selected tasks or task discussions, and explicitly granted project working folders.

Project membership does not imply access to every project resource. A restricted participant sees only authorized tasks and derived summaries. Manager plans, reports, search, notifications, and AI context assembly apply the same permissions. Inviting someone never retroactively exposes private history without a clear history-visibility choice.

Until collaboration is implemented, participant controls are absent or clearly disabled as planned functionality. Assignee and reviewer fields may use the local profile, but they do not pretend that multi-person assignment already works.

## Linkage to other systems

- **Chat:** captures discussion, proposals, approvals, status updates, task discussions, and review conversations.
- **Projects:** owns the canonical task graph, assignments, reviews, budgets, changes, and execution links.
- **Calendar:** owns scheduling, hard time constraints, capacity reservations, and proposed cascades.
- **Notes:** owns specifications, research, decisions, and working documents.
- **Work environments:** activates the tools and context needed for scheduled human work.
- **AI integration:** supplies provider-native planning actions and agent runs through bounded context and authority. A planning role is an ordinary configured teammate or workflow, never a privileged built-in identity.
- **Gamification:** can guide phase transitions in the eventual visual layer without changing the underlying project model.
