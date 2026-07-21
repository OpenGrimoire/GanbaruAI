import {
  ACTIVITY_STATUSES,
  CANONICAL_ITEM_KINDS,
  CONTENT_STREAM_KINDS,
  MESSAGE_STREAMING_STATES,
} from "./contracts";
import type {
  ActivityStatus,
  CanonicalItemKind,
  CanonicalStoredEvent,
  ChangedFileSummary,
  ChatTimelineItemRead,
  ChatTimelineTurnRead,
  ChatTurnId,
  ChatTurnState,
  ContentDeltaEvent,
  ContentStreamKind,
  ItemLifecycleEvent,
  ModelId,
  PlanStep,
  ProposedPlanDeltaEvent,
  RequestResolutionState,
  ThreadUsageUpdatedEvent,
  TurnModeSnapshot,
  UtcTimestamp,
  UserInputQuestion,
  VersionedJson,
} from "./contracts";

export type TimelineMessageState = "pending" | "streaming" | "complete" | "interrupted" | "failed";

export interface TimelineAssistantMetadata {
  durationMs: number | null;
  modelId: ModelId | null;
  usage: ThreadUsageUpdatedEvent | null;
  changedFiles: ChangedFileSummary[];
  diffSources: string[];
}

export interface TimelineAttachmentSummary {
  id: string | null;
  displayName: string;
  kind: string | null;
  byteSize: number | null;
  status: string | null;
}

export interface TimelineMentionSummary {
  relativePath: string;
  kind: string | null;
}

export interface TimelineUserContext {
  attachments: TimelineAttachmentSummary[];
  mentions: TimelineMentionSummary[];
  terminalContext: string[];
  preCheckpointId: string | null;
}

export interface TimelineMessageRow {
  id: string;
  kind: "message";
  role: "user" | "assistant";
  turnId: ChatTurnId | null;
  sequence: number;
  createdAt: UtcTimestamp;
  markdown: string;
  state: TimelineMessageState;
  userContext: TimelineUserContext | null;
  metadata: TimelineAssistantMetadata | null;
}

export interface TimelineActivityRow {
  id: string;
  kind: "activity";
  turnId: ChatTurnId | null;
  sequence: number;
  createdAt: UtcTimestamp;
  activityKind: CanonicalItemKind | ContentStreamKind | "task" | "hook" | "tool" | "mcp" | "notice";
  status: ActivityStatus;
  title: string;
  detail: string | null;
  metadata: VersionedJson | null;
}

export interface TimelinePlanRow {
  id: string;
  kind: "plan";
  turnId: ChatTurnId | null;
  sequence: number;
  createdAt: UtcTimestamp;
  planKind: "structured" | "proposed";
  markdown: string;
  steps: PlanStep[];
  state: "streaming" | "complete";
}

export type TimelineRow = TimelineMessageRow | TimelineActivityRow | TimelinePlanRow;

export interface TimelinePendingRequest {
  id: string;
  turnId: ChatTurnId | null;
  sequence: number;
  createdAt: UtcTimestamp;
  kind: "approval" | "user_input";
  title: string;
  detail: string | null;
  questions: UserInputQuestion[];
  state: RequestResolutionState;
}

export interface TimelineTurn {
  id: ChatTurnId;
  state: ChatTurnState;
  startedAt: UtcTimestamp;
  completedAt: UtcTimestamp | null;
  durationMs: number | null;
  modes: TurnModeSnapshot;
  modelId: ModelId | null;
  effectiveModelId: ModelId | null;
  usage: ThreadUsageUpdatedEvent | null;
  changedFiles: ChangedFileSummary[];
  diffSources: string[];
  stopReason: string | null;
  recoverable: boolean;
}

export interface TimelineProjection {
  rows: TimelineRow[];
  turns: TimelineTurn[];
  pendingRequests: TimelinePendingRequest[];
  threadUsage: ThreadUsageUpdatedEvent | null;
  ignoredDuplicateEventIds: string[];
}

export interface TimelineTurnFoldRow {
  id: string;
  kind: "turn_fold";
  turnId: ChatTurnId;
  sequence: number;
  createdAt: UtcTimestamp;
  state: "completed" | "interrupted" | "failed";
  durationMs: number | null;
  hiddenRows: TimelineActivityRow[];
  expanded: boolean;
}

export interface TimelineActivityGroupRow {
  id: string;
  kind: "activity_group";
  turnId: ChatTurnId | null;
  sequence: number;
  createdAt: UtcTimestamp;
  latest: TimelineActivityRow;
  earlierRows: TimelineActivityRow[];
  expanded: boolean;
}

export type TimelineDisplayRow = TimelineRow | TimelineTurnFoldRow | TimelineActivityGroupRow;

interface MutableStreamRow {
  row: TimelineMessageRow | TimelineActivityRow | TimelinePlanRow;
  parts: Map<number, string>;
}

const DEFAULT_MODES: TurnModeSnapshot = {
  safetyMode: "supervised",
  interactionMode: "build",
};

/**
 * Projects validated canonical events into stable, provider-neutral timeline state.
 *
 * @param events Canonical events from one thread, in any sequence order.
 * @returns Stable timeline rows, turn summaries, and unresolved requests.
 */
export function projectCanonicalTimeline(events: readonly CanonicalStoredEvent[]): TimelineProjection {
  const rows = new Map<string, TimelineRow>();
  const streams = new Map<string, MutableStreamRow>();
  const turns = new Map<ChatTurnId, TimelineTurn>();
  const requests = new Map<string, TimelinePendingRequest>();
  const seenEventIds = new Set<string>();
  const ignoredDuplicateEventIds: string[] = [];
  let threadUsage: ThreadUsageUpdatedEvent | null = null;

  const orderedEvents = [...events].sort((left, right) => left.sequence - right.sequence);
  for (const event of orderedEvents) {
    if (seenEventIds.has(event.eventId)) {
      ignoredDuplicateEventIds.push(event.eventId);
      continue;
    }
    seenEventIds.add(event.eventId);

    switch (event.event.type) {
      case "turn_started": {
        if (!event.turnId) break;
        turns.set(event.turnId, {
          id: event.turnId,
          state: event.event.payload.state,
          startedAt: event.createdAt,
          completedAt: null,
          durationMs: null,
          modes: event.event.payload.modes,
          modelId: event.event.payload.modelId,
          effectiveModelId: event.event.payload.modelId,
          usage: null,
          changedFiles: [],
          diffSources: [],
          stopReason: null,
          recoverable: false,
        });
        break;
      }
      case "turn_completed": {
        const turn = ensureTurn(turns, event);
        if (!turn) break;
        turn.state = event.event.payload.state;
        turn.completedAt = event.createdAt;
        turn.durationMs = elapsedMilliseconds(turn.startedAt, event.createdAt);
        turn.usage = event.event.payload.usage;
        turn.changedFiles = mergeChangedFiles(turn.changedFiles, event.event.payload.changedFiles);
        turn.stopReason = event.event.payload.stopReason;
        break;
      }
      case "turn_aborted": {
        const turn = ensureTurn(turns, event);
        if (!turn) break;
        turn.state = event.event.payload.state;
        turn.completedAt = event.createdAt;
        turn.durationMs = elapsedMilliseconds(turn.startedAt, event.createdAt);
        turn.stopReason = event.event.payload.reason;
        turn.recoverable = event.event.payload.recoverable;
        break;
      }
      case "thread_usage_updated":
        threadUsage = event.event.payload;
        break;
      case "thread_reverted":
        upsertActivity(
          rows,
          event,
          `activity:restore:${event.eventId}`,
          "notice",
          "completed",
          "thread_reverted",
          null,
          {
            schemaVersion: 1,
            value: {
              checkpointId: event.event.payload.checkpointId,
              revertedTurnIds: [...event.event.payload.revertedTurnIds],
              providerHistoryAction: event.event.payload.providerHistoryAction,
            },
          },
        );
        break;
      case "model_rerouted": {
        const turn = ensureTurn(turns, event);
        if (turn) turn.effectiveModelId = event.event.payload.effectiveModelId;
        upsertActivity(rows, event, `model:${event.sequence}`, "notice", "completed", "Model rerouted", event.event.payload.reason, null);
        break;
      }
      case "diff_updated": {
        const turn = ensureTurn(turns, event);
        if (!turn) break;
        turn.changedFiles = mergeChangedFiles(turn.changedFiles, event.event.payload.files);
        if (!turn.diffSources.includes(event.event.payload.source)) turn.diffSources.push(event.event.payload.source);
        break;
      }
      case "content_delta":
        appendContentDelta(rows, streams, event, event.event.payload);
        break;
      case "item_started":
      case "item_updated":
      case "item_completed":
        upsertLifecycleItem(rows, event, event.event.payload);
        break;
      case "plan_updated":
        rows.set(`plan:structured:${event.turnId ?? "thread"}`, {
          id: `plan:structured:${event.turnId ?? "thread"}`,
          kind: "plan",
          turnId: event.turnId,
          sequence: event.sequence,
          createdAt: event.createdAt,
          planKind: "structured",
          markdown: event.event.payload.markdown,
          steps: event.event.payload.steps,
          state: "complete",
        });
        break;
      case "proposed_plan_delta":
        appendPlanDelta(rows, streams, event, event.event.payload);
        break;
      case "proposed_plan_completed": {
        const id = `plan:proposed:${event.event.payload.planId}`;
        rows.set(id, {
          id,
          kind: "plan",
          turnId: event.turnId,
          sequence: rows.get(id)?.sequence ?? event.sequence,
          createdAt: rows.get(id)?.createdAt ?? event.createdAt,
          planKind: "proposed",
          markdown: event.event.payload.markdown,
          steps: [],
          state: "complete",
        });
        break;
      }
      case "request_opened":
        requests.set(event.event.payload.requestId, {
          id: event.event.payload.requestId,
          turnId: event.turnId,
          sequence: event.sequence,
          createdAt: event.createdAt,
          kind: "approval",
          title: event.event.payload.title,
          detail: event.event.payload.detail,
          questions: [],
          state: "open",
        });
        break;
      case "user_input_requested":
        requests.set(event.event.payload.requestId, {
          id: event.event.payload.requestId,
          turnId: event.turnId,
          sequence: event.sequence,
          createdAt: event.createdAt,
          kind: "user_input",
          title: event.event.payload.questions[0]?.question ?? "Provider question",
          detail: null,
          questions: event.event.payload.questions,
          state: "open",
        });
        break;
      case "request_resolved":
      case "user_input_resolved": {
        const request = requests.get(event.event.payload.requestId);
        if (request) request.state = event.event.payload.state;
        break;
      }
      case "task_lifecycle":
        upsertActivity(rows, event, `task:${event.event.payload.taskId}`, "task", event.event.payload.status, event.event.payload.title, event.event.payload.detail, event.event.payload.safeMetadata);
        break;
      case "hook_lifecycle":
        upsertActivity(rows, event, `hook:${event.event.payload.hookId}`, "hook", event.event.payload.status, event.event.payload.title, event.event.payload.detail, null);
        break;
      case "tool_progress":
        upsertActivity(rows, event, `tool:${event.event.payload.toolId}`, "tool", event.event.payload.status, event.event.payload.title, event.event.payload.summary, null);
        break;
      case "mcp_status":
        upsertActivity(rows, event, `mcp:${event.event.payload.serverId}`, "mcp", event.event.payload.status, event.event.payload.serverId, event.event.payload.detail, null);
        break;
      case "mcp_oauth_completed":
        upsertActivity(rows, event, `mcp-oauth:${event.event.payload.serverId}:${event.sequence}`, "mcp", event.event.payload.successful ? "completed" : "failed", event.event.payload.serverId, event.event.payload.detail, null);
        break;
      case "configuration_warning":
      case "deprecation_notice":
      case "runtime_warning":
        upsertActivity(rows, event, `notice:${event.eventId}`, "notice", "unknown", event.event.payload.title, event.event.payload.detail, null);
        break;
      case "runtime_error":
        upsertActivity(rows, event, `error:${event.eventId}`, "error", "failed", event.event.payload.message, event.event.payload.code, event.event.payload.safeDetails);
        break;
      case "unknown":
        upsertActivity(rows, event, `unknown:${event.eventId}`, "unknown", "unknown", event.event.payload.summary, event.event.payload.sourceType, event.event.payload.safePayload);
        break;
      default:
        break;
    }
  }

  attachPendingTurnRows(rows, turns);
  attachTerminalAssistantMetadata(rows, turns);
  return {
    rows: [...rows.values()].sort(compareRows),
    turns: [...turns.values()].sort((left, right) => left.startedAt.localeCompare(right.startedAt)),
    pendingRequests: [...requests.values()].sort((left, right) => left.sequence - right.sequence),
    threadUsage,
    ignoredDuplicateEventIds,
  };
}

/**
 * Converts validated paged SQLite projection DTOs into the shared timeline model.
 */
export function projectTimelineReadModel(
  items: readonly ChatTimelineItemRead[],
  turnReads: readonly ChatTimelineTurnRead[],
): Pick<TimelineProjection, "rows" | "turns"> {
  const turns = new Map<ChatTurnId, TimelineTurn>();
  for (const turn of turnReads) {
    turns.set(turn.turnId, {
      id: turn.turnId,
      state: turn.state,
      startedAt: turn.startedAt ?? turn.completedAt ?? "1970-01-01T00:00:00.000Z",
      completedAt: turn.completedAt,
      durationMs: turn.startedAt && turn.completedAt ? elapsedMilliseconds(turn.startedAt, turn.completedAt) : null,
      modes: turn.modes,
      modelId: turn.modelId,
      effectiveModelId: turn.modelId,
      usage: turn.usage,
      changedFiles: turn.changedFiles,
      diffSources: [],
      stopReason: turn.stopReason,
      recoverable: false,
    });
  }
  const rows = new Map<string, TimelineRow>();
  for (const item of items) {
    const data = jsonRecord(item.data.value);
    if (!data) continue;
    const createdAt = jsonString(data.createdAt);
    if (!createdAt) continue;
    if (item.kind === "message") {
      const role = data.role === "user" || data.role === "assistant" ? data.role : null;
      const markdown = jsonString(data.markdown);
      const state = jsonString(data.streamingState);
      if (!role || markdown === null || !state || !isKnownValue(MESSAGE_STREAMING_STATES, state)) continue;
      rows.set(item.activityId, {
        id: item.activityId,
        kind: "message",
        role,
        turnId: item.turnId,
        sequence: item.sequenceAnchor,
        createdAt,
        markdown,
        state,
        userContext: role === "user" ? parseTimelineUserContext(data.metadata) : null,
        metadata: null,
      });
      continue;
    }
    if (item.kind === "activity") {
      const activityKind = jsonString(data.activityKind);
      const status = jsonString(data.status);
      const title = jsonString(data.title);
      if (!activityKind || !status || title === null || !isKnownValue(ACTIVITY_STATUSES, status)) continue;
      const knownKind = isKnownValue(CANONICAL_ITEM_KINDS, activityKind) || isKnownValue(CONTENT_STREAM_KINDS, activityKind)
        ? activityKind
        : "unknown";
      rows.set(item.activityId, {
        id: item.activityId,
        kind: "activity",
        turnId: item.turnId,
        sequence: item.sequenceAnchor,
        createdAt,
        activityKind: knownKind,
        status,
        title,
        detail: jsonNullableString(data.detail),
        metadata: data.metadata === undefined ? null : { schemaVersion: item.data.schemaVersion, value: data.metadata },
      });
      continue;
    }
    if (item.kind === "plan") {
      const markdown = jsonString(data.markdown);
      if (markdown === null) continue;
      rows.set(item.activityId, {
        id: item.activityId,
        kind: "plan",
        turnId: item.turnId,
        sequence: item.sequenceAnchor,
        createdAt,
        planKind: "proposed",
        markdown,
        steps: parseProjectedPlanSteps(data.steps),
        state: data.state === "proposed" ? "streaming" : "complete",
      });
    }
  }
  attachPendingTurnRows(rows, turns);
  attachTerminalAssistantMetadata(rows, turns);
  return { rows: [...rows.values()].sort(compareRows), turns: [...turns.values()] };
}

/**
 * Converts projected rows into the compact display model used by the timeline.
 *
 * @param rows Stable projected timeline rows.
 * @param turns Turn summaries associated with the rows.
 * @param expandedTurnIds Turn folds expanded in the current window session.
 * @param expandedGroupIds Activity groups expanded in the current window session.
 * @returns Folded and grouped display rows with stable identifiers.
 */
export function buildTimelineDisplayRows(
  rows: readonly TimelineRow[],
  turns: readonly TimelineTurn[],
  expandedTurnIds: ReadonlySet<ChatTurnId> = new Set(),
  expandedGroupIds: ReadonlySet<string> = new Set(),
): TimelineDisplayRow[] {
  const terminalTurns = new Map(
    turns
      .filter((turn): turn is TimelineTurn & { state: "completed" | "interrupted" | "failed" } => isTerminalTurn(turn.state))
      .map((turn) => [turn.id, turn]),
  );
  const activitiesByTurn = new Map<ChatTurnId, TimelineActivityRow[]>();
  for (const row of rows) {
    if (row.kind !== "activity" || !row.turnId || !terminalTurns.has(row.turnId)) continue;
    const activities = activitiesByTurn.get(row.turnId) ?? [];
    activities.push(row);
    activitiesByTurn.set(row.turnId, activities);
  }

  const emittedFolds = new Set<ChatTurnId>();
  const folded: TimelineDisplayRow[] = [];
  for (const row of rows) {
    if (row.kind !== "activity" || !row.turnId || !terminalTurns.has(row.turnId)) {
      folded.push(row);
      continue;
    }
    const turn = terminalTurns.get(row.turnId);
    if (!turn) continue;
    const expanded = expandedTurnIds.has(row.turnId);
    if (!emittedFolds.has(row.turnId)) {
      const hiddenRows = activitiesByTurn.get(row.turnId) ?? [];
      folded.push({
        id: `turn-fold:${row.turnId}`,
        kind: "turn_fold",
        turnId: row.turnId,
        sequence: hiddenRows[0]?.sequence ?? row.sequence,
        createdAt: hiddenRows[0]?.createdAt ?? row.createdAt,
        state: turn.state,
        durationMs: turn.durationMs,
        hiddenRows,
        expanded,
      });
      emittedFolds.add(row.turnId);
    }
    if (expanded) folded.push(row);
  }

  return groupSettledActivities(folded, expandedGroupIds);
}

function ensureTurn(turns: Map<ChatTurnId, TimelineTurn>, event: CanonicalStoredEvent): TimelineTurn | null {
  if (!event.turnId) return null;
  const existing = turns.get(event.turnId);
  if (existing) return existing;
  const turn: TimelineTurn = {
    id: event.turnId,
    state: "active",
    startedAt: event.createdAt,
    completedAt: null,
    durationMs: null,
    modes: DEFAULT_MODES,
    modelId: null,
    effectiveModelId: null,
    usage: null,
    changedFiles: [],
    diffSources: [],
    stopReason: null,
    recoverable: false,
  };
  turns.set(event.turnId, turn);
  return turn;
}

function appendContentDelta(
  rows: Map<string, TimelineRow>,
  streams: Map<string, MutableStreamRow>,
  event: CanonicalStoredEvent,
  payload: ContentDeltaEvent,
): void {
  const assistant = payload.streamKind === "assistant_text";
  const id = assistant ? `message:${payload.itemId}` : `activity:${payload.itemId}`;
  let stream = streams.get(id);
  if (!stream) {
    const row: TimelineMessageRow | TimelineActivityRow = assistant
      ? { id, kind: "message", role: "assistant", turnId: event.turnId, sequence: event.sequence, createdAt: event.createdAt, markdown: "", state: "streaming", userContext: null, metadata: null }
      : { id, kind: "activity", turnId: event.turnId, sequence: event.sequence, createdAt: event.createdAt, activityKind: payload.streamKind, status: "active", title: activityTitle(payload.streamKind), detail: null, metadata: null };
    stream = { row, parts: new Map() };
    streams.set(id, stream);
    rows.set(id, row);
  }
  stream.parts.set(payload.contentIndex, `${stream.parts.get(payload.contentIndex) ?? ""}${payload.delta}`);
  const content = [...stream.parts.entries()].sort(([left], [right]) => left - right).map(([, value]) => value).join("");
  if (stream.row.kind === "message") stream.row.markdown = content;
  else if (stream.row.kind === "activity") stream.row.detail = content;
}

function appendPlanDelta(
  rows: Map<string, TimelineRow>,
  streams: Map<string, MutableStreamRow>,
  event: CanonicalStoredEvent,
  payload: ProposedPlanDeltaEvent,
): void {
  const id = `plan:proposed:${payload.planId}`;
  let stream = streams.get(id);
  if (!stream) {
    const row: TimelinePlanRow = { id, kind: "plan", turnId: event.turnId, sequence: event.sequence, createdAt: event.createdAt, planKind: "proposed", markdown: "", steps: [], state: "streaming" };
    stream = { row, parts: new Map() };
    streams.set(id, stream);
    rows.set(id, row);
  }
  stream.parts.set(payload.contentIndex, `${stream.parts.get(payload.contentIndex) ?? ""}${payload.delta}`);
  if (stream.row.kind === "plan") {
    stream.row.markdown = [...stream.parts.entries()].sort(([left], [right]) => left - right).map(([, value]) => value).join("");
  }
}

function upsertLifecycleItem(
  rows: Map<string, TimelineRow>,
  event: CanonicalStoredEvent,
  payload: ItemLifecycleEvent,
): void {
  if (payload.kind === "assistant_message" || payload.kind === "user_message") {
    const id = `message:${payload.itemId}`;
    const current = rows.get(id);
    const role = payload.kind === "user_message" ? "user" : "assistant";
    rows.set(id, {
      id,
      kind: "message",
      role,
      turnId: event.turnId,
      sequence: current?.sequence ?? event.sequence,
      createdAt: current?.createdAt ?? event.createdAt,
      markdown: current?.kind === "message" && current.markdown ? current.markdown : payload.detail ?? "",
      state: messageState(payload.status),
      userContext: role === "user"
        ? parseTimelineUserContext(payload.safeMetadata?.value)
        : null,
      metadata: current?.kind === "message" ? current.metadata : null,
    });
    return;
  }
  upsertActivity(rows, event, `activity:${payload.itemId}`, payload.kind, payload.status, payload.title ?? activityTitle(payload.kind), payload.detail, payload.safeMetadata);
}

function upsertActivity(
  rows: Map<string, TimelineRow>,
  event: CanonicalStoredEvent,
  id: string,
  activityKind: TimelineActivityRow["activityKind"],
  status: ActivityStatus,
  title: string,
  detail: string | null,
  metadata: VersionedJson | null,
): void {
  const current = rows.get(id);
  rows.set(id, {
    id,
    kind: "activity",
    turnId: event.turnId,
    sequence: current?.sequence ?? event.sequence,
    createdAt: current?.createdAt ?? event.createdAt,
    activityKind,
    status,
    title,
    detail,
    metadata,
  });
}

function attachTerminalAssistantMetadata(rows: Map<string, TimelineRow>, turns: Map<ChatTurnId, TimelineTurn>): void {
  for (const turn of turns.values()) {
    if (!isTerminalTurn(turn.state)) continue;
    const assistantRows = [...rows.values()].filter((row): row is TimelineMessageRow => row.kind === "message" && row.role === "assistant" && row.turnId === turn.id);
    const assistant = assistantRows.sort((left, right) => right.sequence - left.sequence)[0];
    if (!assistant) continue;
    assistant.state = turn.state === "completed" ? "complete" : turn.state === "interrupted" ? "interrupted" : "failed";
    assistant.metadata = {
      durationMs: turn.durationMs,
      modelId: turn.effectiveModelId,
      usage: turn.usage,
      changedFiles: turn.changedFiles,
      diffSources: turn.diffSources,
    };
  }
}

function attachPendingTurnRows(rows: Map<string, TimelineRow>, turns: Map<ChatTurnId, TimelineTurn>): void {
  for (const turn of turns.values()) {
    if (turn.state !== "pending" && turn.state !== "dispatching" && turn.state !== "active") continue;
    const turnRows = [...rows.values()].filter((row) => row.turnId === turn.id);
    if (!turnRows.some((row) => row.kind === "message" && row.role === "user")) continue;
    const hasProviderOutput = turnRows.some((row) => row.kind !== "message" || row.role === "assistant");
    if (hasProviderOutput) continue;
    const latest = turnRows.sort((left, right) => right.sequence - left.sequence)[0];
    rows.set(`turn-pending:${turn.id}`, {
      id: `turn-pending:${turn.id}`,
      kind: "activity",
      turnId: turn.id,
      sequence: (latest?.sequence ?? 0) + 1,
      createdAt: latest?.createdAt ?? turn.startedAt,
      activityKind: "notice",
      status: "active",
      title: "Working",
      detail: null,
      metadata: null,
    });
  }
}

function mergeChangedFiles(current: readonly ChangedFileSummary[], incoming: readonly ChangedFileSummary[]): ChangedFileSummary[] {
  const files = new Map(current.map((file) => [file.relativePath, file]));
  for (const file of incoming) files.set(file.relativePath, file);
  return [...files.values()];
}

function elapsedMilliseconds(start: UtcTimestamp, end: UtcTimestamp): number | null {
  const elapsed = Date.parse(end) - Date.parse(start);
  return Number.isFinite(elapsed) && elapsed >= 0 ? elapsed : null;
}

function messageState(status: ActivityStatus): TimelineMessageState {
  if (status === "completed") return "complete";
  if (status === "interrupted") return "interrupted";
  if (status === "failed") return "failed";
  return "streaming";
}

function isTerminalTurn(state: ChatTurnState): boolean {
  return state === "completed" || state === "interrupted" || state === "failed";
}

function activityTitle(kind: CanonicalItemKind | ContentStreamKind): string {
  return kind.replaceAll("_", " ");
}

function compareRows(left: TimelineRow, right: TimelineRow): number {
  return left.sequence - right.sequence || left.id.localeCompare(right.id);
}

function jsonRecord(value: VersionedJson["value"] | undefined): Record<string, VersionedJson["value"]> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value
    : null;
}

function jsonString(value: VersionedJson["value"] | undefined): string | null {
  return typeof value === "string" ? value : null;
}

function jsonNullableString(value: VersionedJson["value"] | undefined): string | null {
  return value === null || value === undefined ? null : jsonString(value);
}

function isKnownValue<const Values extends readonly string[]>(values: Values, value: string): value is Values[number] {
  return values.some((candidate) => candidate === value);
}

function parseProjectedPlanSteps(value: VersionedJson["value"] | undefined): PlanStep[] {
  if (!Array.isArray(value)) return [];
  const steps: PlanStep[] = [];
  for (const candidate of value) {
    const record = jsonRecord(candidate);
    if (!record) continue;
    const id = jsonString(record.id);
    const text = jsonString(record.text);
    const status = jsonString(record.status);
    if (!id || text === null || !status || !isKnownValue(ACTIVITY_STATUSES, status)) continue;
    steps.push({ id, text, status });
  }
  return steps;
}

/** Parses bounded, presentation-safe context metadata attached to a user message. */
export function parseTimelineUserContext(value: VersionedJson["value"] | undefined): TimelineUserContext | null {
  const record = jsonRecord(value);
  if (!record) return null;
  const attachments = parseUserAttachments(record.attachments).slice(0, 20);
  const mentions = parseUserMentions(record.mentions).slice(0, 100);
  const terminalContext = parseTerminalContext(record.terminalContext).slice(0, 20);
  const preCheckpointId = jsonString(record.preCheckpointId) ?? jsonString(record.checkpointId);
  if (attachments.length === 0 && mentions.length === 0 && terminalContext.length === 0 && !preCheckpointId) return null;
  return { attachments, mentions, terminalContext, preCheckpointId };
}

function parseUserAttachments(value: VersionedJson["value"] | undefined): TimelineAttachmentSummary[] {
  if (!Array.isArray(value)) return [];
  const attachments: TimelineAttachmentSummary[] = [];
  for (const candidate of value) {
    const record = jsonRecord(candidate);
    const displayName = record ? jsonString(record.displayName) ?? jsonString(record.filename) : null;
    if (!record || !displayName) continue;
    const byteSize = typeof record.byteSize === "number" && Number.isFinite(record.byteSize) && record.byteSize >= 0
      ? record.byteSize
      : null;
    attachments.push({
      id: jsonString(record.attachmentId) ?? jsonString(record.id),
      displayName,
      kind: jsonString(record.kind),
      byteSize,
      status: jsonString(record.status),
    });
  }
  return attachments;
}

function parseUserMentions(value: VersionedJson["value"] | undefined): TimelineMentionSummary[] {
  if (!Array.isArray(value)) return [];
  const mentions: TimelineMentionSummary[] = [];
  for (const candidate of value) {
    const record = jsonRecord(candidate);
    const relativePath = record ? jsonString(record.relativePath) : null;
    if (!record || !relativePath) continue;
    mentions.push({ relativePath, kind: jsonString(record.kind) });
  }
  return mentions;
}

function parseTerminalContext(value: VersionedJson["value"] | undefined): string[] {
  if (!Array.isArray(value)) return [];
  const context: string[] = [];
  for (const candidate of value) {
    if (typeof candidate === "string") {
      if (candidate.trim()) context.push(candidate.slice(0, 240));
      continue;
    }
    const record = jsonRecord(candidate);
    const label = record ? jsonString(record.label) ?? jsonString(record.command) ?? jsonString(record.text) : null;
    if (label?.trim()) context.push(label.slice(0, 240));
  }
  return context;
}

function groupSettledActivities(
  rows: readonly TimelineDisplayRow[],
  expandedGroupIds: ReadonlySet<string>,
): TimelineDisplayRow[] {
  const result: TimelineDisplayRow[] = [];
  const expandedFoldTurnIds = new Set(
    rows
      .filter((row): row is TimelineTurnFoldRow => row.kind === "turn_fold" && row.expanded)
      .map((row) => row.turnId),
  );
  let run: TimelineActivityRow[] = [];
  const flush = (): void => {
    if (run.length < 2) {
      result.push(...run);
      run = [];
      return;
    }
    const first = run[0];
    const latest = run.at(-1);
    if (!first || !latest) {
      run = [];
      return;
    }
    const id = `activity-group:${first.id}:${latest.id}`;
    result.push({
      id,
      kind: "activity_group",
      turnId: first.turnId,
      sequence: first.sequence,
      createdAt: first.createdAt,
      latest,
      earlierRows: run.slice(0, -1),
      expanded: expandedGroupIds.has(id),
    });
    run = [];
  };

  for (const row of rows) {
    if (row.kind === "activity" && !expandedFoldTurnIds.has(row.turnId ?? "") && row.status !== "active" && row.status !== "pending" && row.status !== "waiting") {
      if (run.length === 0 || run[0]?.turnId === row.turnId) run.push(row);
      else {
        flush();
        run.push(row);
      }
      continue;
    }
    flush();
    result.push(row);
  }
  flush();
  return result;
}
