# Chat access control

This document is the normative authorization specification for organizational Chat. It defines who an AI teammate is, how authority enters a run, how channels and folders are disclosed, and how reductions invalidate work that already materialized restricted context. Product presentation is specified in [Chat](../features/chat.md), provider behavior in [AI integration](../features/ai-integration.md), and cross-cutting threats in [Security](security.md).

## Principles

1. Identity is not authority. Creating a teammate does not create a membership, history grant, folder grant, scratch scope, or ambient context.
2. The local owner is the only required seeded participant. The base system has no default or privileged AI teammate.
3. An AI teammate is an ordinary vault-wide identity. Provider, model, role, instructions, and runtime defaults are replaceable configuration.
4. Future default teammates are templates. Instantiating a template creates an ordinary identity with no hidden ID, membership, provider preference, or access-control exemption.
5. Channel presence, readable context, and work resources are separate decisions. A teammate can participate without receiving a folder, shell, persistent scratch, or another channel's history.
6. Access profiles are reusable resource ceilings and defaults. They are configuration, not principals, and cannot grant access without an explicit channel membership and an exact resource grant.
7. Authority is closed world. Missing, stale, fabricated, revoked, or unsupported authority is denied.
8. A reference identifies context. It never embeds or transfers authority.
9. Denial and revocation win over every convenience default.

References to application-owned behavior mean trusted services such as authorization, internal host tools, folder brokering, and Git operations. They never identify an AI teammate.

## Scope and principals

The active vault is the authorization root. Stable principals are the local owner, future invited human participants, and ordinary AI teammates. Conversations and channel memberships are separate from AI-specific execution access so the same membership model can support people without pretending that people and providers have identical runtime controls.

The current delivery scope supports vault-wide AI teammate identities and explicit memberships in project-owned channels. Group channels, workspace channels, invitations, cross-company shared channels, proactive observation, subscriptions, shared memory, planning agents, orchestration, cloud runners, and synchronized access state are later layers. They must build on this model instead of inventing broader implicit authority.

## Effective authority

Every read, dispatch, continuation, host-tool call, scratch reuse, and result publication evaluates the same intersection:

```text
requester authority
intersection destination policy and audience
intersection teammate policy and access-profile ceiling
intersection destination channel membership
intersection referenced source-channel membership
intersection exact folder grants
intersection assignment references and any selected execution target
intersection runtime approval policy
intersection verified provider enforcement
```

The database is canonical. Settings previews, assignment creation, scheduling, provider dispatch, internal host tools, continuations, result publication, and revocation call the same reusable authorization service. A cached in-memory scope may guard lifecycle and concurrency, but it does not replace a database check.

A teammate needs `Participate` in the destination before it can be addressed or publish there. It needs `Read history` in every channel whose messages it may query. A project folder is invisible until an exact folder grant exists. Authority enters a run only through the triggering assignment, structured references, and any explicitly authorized resources. Conversation work does not require a native execution target.

## Channel capabilities

Channel access exposes independent capabilities:

| Preset | Read history | Participate |
|---|---:|---:|
| Use as context | Yes | No |
| Respond when tagged | No | Yes |
| Read and respond | Yes | Yes |

Any other manual combination is `Custom`. `Participate` does not imply history access. `Read history` does not allow assignment or publication.

When history access is enabled, its boundary is either:

- `Entire history`, which is the default.
- `From access grant`, whose lower message ordinal is captured by the backend when the grant is saved.

Moving from `From access grant` to `Entire history` expands authority and requires an impact preview. A normal assignment includes the triggering request, its reply thread, and bounded recent destination messages only when destination history is readable. Messages from other channels enter only through an authorized structured channel reference and a scoped host-tool query.

## Access profiles and revisions

An access profile has a stable identity and immutable revisions. A channel membership links to the stable profile and may only narrow its current revision. Each assignment freezes the exact profile revision used for its authorization decision.

The built-in profiles are localized, immutable, and duplicable:

| Profile | Maximum project-folder capability |
|---|---|
| Conversation only | `none` |
| Read only | `read` |
| Edit files | `edit` |
| Build and test | `execute` |
| Publish changes | `publish` |

No profile is attached automatically when a standalone teammate is created. A channel-scoped creation flow may prepare a conservative proposal containing Conversation only, Respond when tagged, no history, and no project resources. The form shows the exact proposed membership before its final Save. The backend creates the identity first and then saves the reviewed access revision. If access persistence fails, the ordinary identity remains inert instead of receiving partial authority.

Membership overrides may narrow but never widen the linked profile. Reductions apply immediately and interrupt incompatible work. Expansions require impact preview, create a new authorization revision, and affect only new work. Active and queued work never widens automatically. Built-in profiles cannot be edited. A custom profile can be duplicated and revised, but cannot be archived while a membership still uses it.

A profile ceiling is not an effective grant. For example, Publish changes with zero folder grants remains conversation-only access. Provider support is checked against effective grants and the selected native target, not against unused profile headroom.

## Folder capabilities

Folder grants use an ordered capability model:

| Capability | Effective behavior |
|---|---|
| `none` | The folder is undiscoverable and inaccessible. |
| `read` | Application-brokered bounded listing, search, and reads. |
| `edit` | Adds revision-checked creation, patching, replacement, and deletion. Shell remains denied. |
| `execute` | Adds local commands in the one selected execution target. Network remains denied. |
| `publish` | Adds application-brokered Git push and pull-request operations. Arbitrary network remains denied. |

A membership may grant several logical working folders owned by its channel's project. Each grant stores its capability, optional runtime-approval override, optional default-target state, logical repository identity, device binding state, revision, and revocation state. More than one default is invalid. A default is optional because ordinary conversation work has no native target. Read and edit grants may be default native roots when the selected provider proves the matching root enforcement.

Logical folder identity is vault data. External absolute paths, executable paths, and device availability remain device-local and never enter vault SQLite. The UI distinguishes authorization from binding with states such as Ready, Locate, Relink, Clone, Missing, and Unavailable on this device.

## Runtime approvals

Provider CLI permissions and organizational access are separate. Every teammate policy stores the harness permission mode: Ask for approval, Approve for me, Full access, or Custom. The provider adapter maps that mode to Codex, Claude Code, Cursor, OpenCode, or another supported harness. Channel membership does not choose or replace this provider setting.

Organizational runtime policies are:

- `ask`
- `autoApprove`
- `unattended`
- `providerCustom`

Resolution proceeds from folder or scratch override, to channel override, to teammate default, then through any more restrictive provider-enforcement result. Approval behavior controls interaction during an already authorized operation. It never grants a channel, folder, command, or network capability.

Provider-custom behavior is valid only when the adapter proves it cannot widen effective authority. No mode named Full access or equivalent may bypass organizational grants.

## Provider enforcement

Protocol feature support and authority enforcement are different contracts. Each provider probe declares:

```text
ProviderAuthoritySupport {
  isolatedConversation
  internalHostTools
  denyShell
  readOnlyRoot
  writableRoot
  confinedCommands
  networkBoundary
  classifiedPublish
}
```

`isolatedConversation` means the adapter can run an organizational response from an application-managed neutral directory without treating that directory as a granted resource. It does not turn the directory into persistent scratch or a project folder. The provider's native permission mode remains visible and is subject to any more restrictive organizational result.

The settings studio explains unsupported effective resource grants before save. Ordinary channel participation does not show a folder-enforcement error. Dispatch repeats the check and fails before sending a prompt that requires unsupported resources. A provider with a high-capability profile but no corresponding folder grant is not rejected. A granted native target is rejected when its required root or command boundary cannot be proven.

Codex app-server, Claude native, Cursor ACP, and application-owned local OpenCode may receive scoped internal MCP injection through their existing integration paths. External OpenCode cannot receive loopback tokens and is ineligible for work that requires internal channel-history or folder tools.

## Execution targets and secondary folders

An organizational conversation run may have no native execution target. A run that uses native files or commands has exactly one target. Target resolution uses this order only when the assignment requires native work:

1. Retain the locked target during an active compatible continuation.
2. Use the target explicitly selected in assignment review.
3. Infer a target when executable references resolve to one eligible environment.
4. Use the membership's optional default folder.

If none of those choices produces an authorized target, the assignment remains conversation-only. Dispatch does not create scratch automatically.

A channel reference or read-only resource never changes the target. Shell commands run only in the selected target. Existing managed worktrees may be selected, but dispatch never creates one automatically.

Secondary folders remain behind application-brokered tools. Those tools list authorized roots, search paths, read bounded files, apply revision-checked patches, create or replace bounded files, and delete paths with runtime approval. This enables controlled multi-folder work without granting a provider native multi-root filesystem access. Publish actions operate only on the selected target through application Git services.

## Private scratch

Private scratch is an explicit persistent work resource identified by vault, reply thread, and teammate. Provider, model, and provider process are not part of its identity. Folderless conversation does not create scratch.

Scratch is available only after explicit membership and explicit selection for an assignment that needs persistent artifacts. It survives assignment completion, restart, provider replacement, and channel archive. It is private to one teammate in one reply thread, absent from the general resource palette, and deleted only by explicit confirmed cleanup. Artifacts can be promoted only into an authorized project folder or managed attachment.

SQLite stores logical identity, generation, lifecycle, size, and provenance. Bytes live in a vault-namespaced device-local managed directory whose absolute path is not stored in vault data. A missing directory appears as Unavailable on this device.

Each generation records the union of source channels queried while producing it. Reuse and promotion recheck those sources against the current destination audience. When a source constraint no longer holds, the generation is quarantined, its provider continuation is discarded, automatic publication is suppressed, and future work receives a clean generation. Old bytes remain available only for authorized inspection or explicit cleanup.

The archived-storage manager lists logical scopes and generations, derived device availability, bounded size information, and retained source summaries. Its browser accepts only a logical generation ID, normalized relative directory path, opaque cursor, and a page size capped at 50 entries. The application resolves the exact managed root on the current device, rejects traversal, links, reparse points, and special files, and never returns an absolute path. Quarantined generations require an explicit local-owner inspection action. They cannot be reused or promoted.

Artifact promotion is revision checked and idempotent. The request names one source file revision and one exact destination. A project-folder destination must still have an active grant whose capability intersects with its current profile ceiling at edit or higher. A managed attachment is owned by the exact destination channel and passes through the vault-managed Chat attachment boundary without requiring any project-folder grant. Its internally derived project managed folder is only a legacy storage and indexing association. It is never exposed as folder authority and must never become an implicit grant. Promotion rechecks the generation's source constraints before reading and immediately before changing destination bytes, and records the destination channel in its logical audit row.

Cleanup is a separate preview and confirmed operation. The preview returns the optimistic scope revision, bounded size, device availability, active-run count, and whether the final generation would remove the scope. Confirmation is rejected while work is active or when the scope revision changed. A retryable cleanup job is durable before byte deletion begins. Missing device bytes produce `Unavailable on this device`, not a successful deletion. Successful cleanup retains the logical job and generation audit state while removing only the exact validated managed generation directory. The reply-thread inspector links to its own scratch scope, while the Teammates studio links to the complete archived-storage manager. Scratch never appears in the general `@` palette.

## Structured references

Inline references are immutable identity atoms with a stable reference ID, discriminator, stable entity IDs, immutable visible label snapshot, UTF-8 byte range, plain-text projection, and type-specific metadata. They contain no embedded authority.

Reference kinds cover participants, channels, working folders, workspace paths, and execution environments. Same-vault semantic paste retains identity after authorization is rechecked. External or cross-vault paste becomes plain text. Sent references navigate only after a current authorization check.

Typing `@` searches permission-filtered people and project resources. Selecting a teammate outside the current channel preserves the draft and opens the explicit Add to channel flow. Typing `#` searches active channels using Group, Project, and Channel ancestry without previewing messages.

## Strict channel disclosure

A cross-channel reference is valid only when all of these conditions hold:

```text
requester can read source
intersection teammate can read source
intersection teammate can participate in destination
intersection destination read-history audience is a subset of source read-history audience
```

There is no override. Candidate selection, send, scheduled delivery, assignment preflight, every history query, result publication, and destination membership changes repeat the check. A failure preserves the draft and identifies up to three affected people using locale-aware list formatting. The user can remove the reference, review affected members, or manage source access.

Adding a destination reader with earlier-history access is blocked when retained references would disclose a source that reader cannot access. `From access grant` is the safe alternative.

A saved channel reference freezes source, destination, author and message revision, source lower boundary, source message high-water mark, source revision cutoff, and destination audience revision. When the reference enters an assignment, its authorized source record also freezes the assignment authorization revision and scope digest. New source messages and later edits do not silently enter an active assignment. Refreshing the reference creates a new authorization decision.

## Internal host tools

The loopback internal MCP server is application infrastructure, not an AI identity or a general data server. Its run scope contains the provider thread and turn, agent run and assignment, authorization revision, destination, exact channel and folder grants, and a scope digest.

Channel-history tools accept only opaque assignment-scoped source handles. They never accept arbitrary channel IDs and recheck database authorization on every call. A query is 1 to 500 bytes with at most 20 normalized terms. The default page contains 20 messages, the maximum page contains 50, and one response cannot exceed 64 KiB. One authorization revision permits at most 20 calls and 1 MiB of returned text. Results contain normalized Markdown and immutable revision provenance, not attachments, nested resources, local paths, inaccessible counts, or provider metadata.

The server uses an ephemeral 256-bit operating-system CSPRNG bearer, authenticates initialization before a prompt requiring host tools, keeps tokens out of logs, and returns the same generic denial for fabricated, stale, wrong-thread, and revoked handles. Audits record decision categories, bounded query hashes, returned revision IDs and hashes, byte counts, and truncation without duplicating message bodies.

## Continuation and revocation

Provider continuations retain materialized context. Each continuation therefore binds an authorization-scope digest. Reuse is allowed only while every materialized source remains authorized for the same destination.

Access contraction, membership removal, destination change, source-authority loss, and hard revocation:

1. mark affected scopes revoked;
2. interrupt active runs;
3. deny further host-tool calls;
4. stop the provider session;
5. suppress automatic publication;
6. discard the native continuation;
7. require a new authorization revision before resuming; and
8. persist failed cleanup as a retryable job.

An expansion never reactivates an old run, source handle, continuation, or quarantined scratch generation.

## Settings and review

Teammates is the primary complete access editor, but it keeps the same dimensions and visual grammar as every other Settings area. The resident view is a quiet full-width identity directory. Selecting or creating a teammate opens one continuous editor in a focused modal that matches the Access profiles manager rather than reserving a second settings column. The editor leads with the teammate avatar, identity, model, and approval behavior, then presents channel access in the same form. It does not introduce a wider administration surface, nested decorative cards, count pills, or a separate Overview and Access dashboard.

Channel membership and configuration share one flat browser embedded in the teammate editor. Its Group, Project, and Channel columns are visible together, with the first available path open immediately. Clicking a group replaces the Project column, clicking a project replaces the Channel column, and clicking a channel focuses its options without changing authority. The hierarchy uses names and shared checkboxes without decorative resource icons, floating panels, membership cards, or separate removal controls.

A group or project checkbox selects only its current active descendant channels, never channels created later. A channel checkbox creates or removes that exact draft membership. The options below the browser always describe only the focused checked channel, so selecting many channels never creates a long stack of expanded configuration rows. Channel behavior uses Respond when tagged, Read and respond, and Use as context. Folder and history details appear only when the focused membership requires them. This is bulk selection of exact memberships, not inherited group authority.

Tools and folders are optional channel-level details behind the edit control. They are not a second primary control beside every membership. Provider incompatibility appears only when the user selects an effective resource grant that the provider cannot enforce. Conversation-only memberships remain usable. History boundaries and organizational runtime overrides also remain channel-level details. Access-profile management is an advanced teammate utility. Persistent scratch management belongs to the reply-thread or storage lifecycle after scratch has been explicitly created, not to the primary teammate access flow.

One atomic Apply action replaces the complete editable teammate draft: display profile, role, instructions, provider policy, teammate runtime default, channel memberships, folder grants, and scratch overrides. It binds the expected teammate-profile revision, teammate-access revision, and immutable revision of every linked access profile. A concurrent teammate or profile change makes the preview or save stale instead of partially committing or silently widening authority. The studio preserves the local draft, reloads current durable state for comparison, and lets the person explicitly rebase the draft or discard it in favor of the current version. Dirty navigation is protected. Expansions show an impact preview. Reductions identify work that will stop.

The current-channel roster provides a smaller contextual entry point. It shows people and AI teammates, access summaries, provider and device health, and explicit add, create, configure, and remove actions. Removing a membership uses the same preview, optimistic revision, and revocation rules as the full studio.

## Persistence and synchronization boundary

Vault SQLite stores stable identities, memberships, immutable profile revisions, capabilities, history cutoffs, logical folder grants, message references, authorization decisions, authorized sources, host-tool audits, continuation digests, logical scratch scopes, execution environments, revocation records, and cleanup jobs.

Provider health, executable discovery, external paths, local folder bindings, and scratch paths are device-local derived state. A synced grant does not prove that another device has a usable binding or enforcement-capable provider. Until permission-aware encrypted synchronization exists, organizational access is enforced within the active local vault. Future Yjs state must preserve immutable authorization revisions and deny closed while a referenced revision or revocation status is unknown.

## Testing matrix

Authorization tests cover inert creation, explicit channel-scoped save, templates without special authority, profile ceilings, membership narrowing, history expansion, folder tiers, approval precedence, provider rejection, optimistic replacement conflicts, requester and audience intersections, retained-reference disclosure, host-tool bounds and revocation, target inference, secondary-folder traversal and symlink rejection, worktree selection, scratch quarantine and cleanup, live-run interruption, English and Spanish catalog parity, keyboard behavior, and coarse-pointer targets.

Any new context source must add tests showing that direct reads, derived summaries, references, search, exports, scheduling, continuation reuse, and result publication apply the same authority intersection.
