import type {
  ChatPromptCatalogEntry,
  ChatSendKey,
  ProjectWorkingFolderPathRead,
  InteractionMode,
  JsonValue,
  ModelOptionDefinition,
  ModelOptionSelection,
  ProviderCapabilities,
  ProviderModel,
  ProviderSessionState,
  SafetyMode,
  UserInputQuestion,
  VersionedJson,
} from "$lib/chat/contracts";

export const CHAT_IMAGE_LIMIT = 8;
export const CHAT_IMAGE_BYTE_LIMIT = 20 * 1024 * 1024;
export const CHAT_IMAGE_TOTAL_BYTE_LIMIT = 50 * 1024 * 1024;

export type ComposerPrimaryAction = "send" | "stop" | "stopping" | "resolve_request";
export type ComposerFollowupAction = "steer" | "queue" | "retain" | null;

export interface ComposerActionState {
  primary: ComposerPrimaryAction;
  followup: ComposerFollowupAction;
  sendEnabled: boolean;
}

export interface ComposerSelections {
  workingFolderId: string | null;
  providerInstanceId: string | null;
  modelId: string | null;
  providerManagedModel: boolean;
  safetyMode: SafetyMode | null;
  interactionMode: InteractionMode | null;
  fullAccessTrusted: boolean;
}

export interface ComposerSelectionError {
  field: "workspace" | "provider" | "model" | "safety" | "interaction" | "trust";
  message: string;
}

export function interactionModeForPrompt(
  text: string,
  current: InteractionMode | null,
): InteractionMode {
  return /^\/plan(?:\s|$)/i.test(text.trimStart()) ? "plan" : current ?? "build";
}

export interface ComposerTokenTrigger {
  kind: "mention" | "skill" | "command";
  query: string;
  start: number;
  end: number;
}

export interface ContextMeterRead {
  usedTokens: number;
  maximumTokens: number | null;
  ratio: number | null;
  warning: boolean;
}

export interface ApprovalChoiceRead {
  id: string;
  label: string;
  decisionKind: "allow_once" | "allow_session" | "deny" | "cancel";
  description: string | null;
}

const ACTIVE_SESSION_STATES: ProviderSessionState[] = [
  "active",
  "waiting_for_approval",
  "waiting_for_user_input",
];

export function composerActionState(
  sessionState: ProviderSessionState,
  capabilities: ProviderCapabilities,
  hasDraft: boolean,
  hasPendingRequest: boolean,
): ComposerActionState {
  if (hasPendingRequest) return { primary: "resolve_request", followup: null, sendEnabled: false };
  if (sessionState === "stopping") return { primary: "stopping", followup: null, sendEnabled: false };
  if (ACTIVE_SESSION_STATES.includes(sessionState)) {
    const followup = !hasDraft
      ? null
      : supports(capabilities, "steering")
        ? "steer"
        : supports(capabilities, "queued_follow_up")
          ? "queue"
          : "retain";
    return { primary: "stop", followup, sendEnabled: false };
  }
  return { primary: "send", followup: null, sendEnabled: hasDraft && sessionState !== "starting" };
}

export function queuedFollowupDispatchReady(
  sessionState: ProviderSessionState,
  latestTurnState: string | null,
): boolean {
  if (sessionState === "ready") return true;
  return sessionState === "stopped"
    && !["pending", "dispatching", "active", "waiting_for_approval", "waiting_for_user_input"]
      .includes(latestTurnState ?? "");
}

export function validateComposerSelections(
  selections: ComposerSelections,
  capabilities: ProviderCapabilities,
): ComposerSelectionError[] {
  const errors: ComposerSelectionError[] = [];
  if (!selections.workingFolderId) errors.push({ field: "workspace", message: "Choose a workspace" });
  if (!selections.providerInstanceId) errors.push({ field: "provider", message: "Choose a provider" });
  if (!selections.modelId && !selections.providerManagedModel) errors.push({ field: "model", message: "Choose a model or provider-managed model" });
  if (!selections.safetyMode) errors.push({ field: "safety", message: "Choose a safety mode" });
  if (!selections.interactionMode) errors.push({ field: "interaction", message: "Choose Build or Plan" });
  if (selections.interactionMode === "plan" && !supports(capabilities, "native_plan")) {
    errors.push({ field: "interaction", message: "This provider does not support native Plan mode" });
  }
  if ((selections.safetyMode === "full_access" || selections.safetyMode === "custom") && !selections.fullAccessTrusted) {
    errors.push({ field: "trust", message: "Confirm broad permission trust for this provider and workspace" });
  }
  return errors;
}

export function composerTokenTrigger(text: string, cursor: number): ComposerTokenTrigger | null {
  const before = text.slice(0, Math.max(0, Math.min(cursor, text.length)));
  const match = /(^|\s)([@$/])([^\s@$]*)$/.exec(before);
  if (!match) return null;
  const prefix = match[2];
  return {
    kind: prefix === "@" ? "mention" : prefix === "$" ? "skill" : "command",
    query: match[3],
    start: before.length - match[3].length - 1,
    end: before.length,
  };
}

export function replaceComposerToken(text: string, trigger: ComposerTokenTrigger, value: string): string {
  return `${text.slice(0, trigger.start)}${value} ${text.slice(trigger.end)}`;
}

export function filterPromptCatalog(
  entries: ChatPromptCatalogEntry[],
  kind: "skill" | "command",
  query: string,
): ChatPromptCatalogEntry[] {
  const normalized = query.trim().toLowerCase();
  return entries
    .filter((entry) => entry.kind === kind && fuzzyMatch(`${entry.label} ${entry.value}`, normalized))
    .sort((left, right) => fuzzyRank(left.label, normalized) - fuzzyRank(right.label, normalized)
      || left.label.localeCompare(right.label));
}

export function filterWorkspacePaths(entries: ProjectWorkingFolderPathRead[], query: string): ProjectWorkingFolderPathRead[] {
  const normalized = query.trim().toLowerCase();
  return [...entries]
    .filter((entry) => fuzzyMatch(entry.relativePath, normalized))
    .sort((left, right) => fuzzyRank(left.relativePath, normalized) - fuzzyRank(right.relativePath, normalized)
      || left.relativePath.localeCompare(right.relativePath));
}

export function rankedModels(
  models: ProviderModel[],
  favoriteIds: string[],
  recentIds: string[],
  query: string,
): ProviderModel[] {
  const favorite = new Map(favoriteIds.map((id, index) => [id, index]));
  const recent = new Map(recentIds.map((id, index) => [id, index]));
  const normalized = query.trim().toLowerCase();
  return models
    .filter((model) => fuzzyMatch(`${model.displayName} ${model.id}`, normalized))
    .sort((left, right) => {
      const leftGroup = favorite.has(left.id) ? 0 : recent.has(left.id) ? 1 : 2;
      const rightGroup = favorite.has(right.id) ? 0 : recent.has(right.id) ? 1 : 2;
      return leftGroup - rightGroup
        || (favorite.get(left.id) ?? recent.get(left.id) ?? Number.MAX_SAFE_INTEGER)
          - (favorite.get(right.id) ?? recent.get(right.id) ?? Number.MAX_SAFE_INTEGER)
        || fuzzyRank(`${left.displayName} ${left.id}`, normalized) - fuzzyRank(`${right.displayName} ${right.id}`, normalized)
        || left.displayName.localeCompare(right.displayName);
    });
}

export function validateModelOptions(
  definitions: ModelOptionDefinition[],
  selections: ModelOptionSelection[],
): string[] {
  const definitionsByKey = new Map(definitions.map((definition) => [definition.key, definition]));
  const errors: string[] = [];
  for (const selection of selections) {
    const definition = definitionsByKey.get(selection.key);
    if (!definition || definition.kind === "unknown") continue;
    if (!optionMatchesDefinition(definition, selection)) errors.push(selection.key);
  }
  return errors;
}

export function composerModelSelection(
  modelId: string | null,
  providerManaged: boolean,
  options: ModelOptionSelection[],
): VersionedJson {
  return {
    schemaVersion: 1,
    value: { modelId, providerManaged, options: options as unknown as JsonValue },
  };
}

export function readComposerModelSelection(value: VersionedJson | null): {
  modelId: string | null;
  providerManaged: boolean;
  options: ModelOptionSelection[];
} {
  if (!value || typeof value.value !== "object" || value.value === null || Array.isArray(value.value)) {
    return { modelId: null, providerManaged: false, options: [] };
  }
  const modelId = typeof value.value.modelId === "string" ? value.value.modelId : null;
  const providerManaged = value.value.providerManaged === true;
  const options = Array.isArray(value.value.options) ? value.value.options as unknown as ModelOptionSelection[] : [];
  return { modelId, providerManaged, options };
}

export function contextMeter(usedTokens: number | null, maximumTokens: number | null): ContextMeterRead | null {
  if (usedTokens === null || !Number.isSafeInteger(usedTokens) || usedTokens < 0) return null;
  const maximum = maximumTokens !== null && Number.isSafeInteger(maximumTokens) && maximumTokens > 0
    ? maximumTokens
    : null;
  const ratio = maximum === null ? null : Math.min(1, usedTokens / maximum);
  return { usedTokens, maximumTokens: maximum, ratio, warning: ratio !== null && ratio >= 0.85 };
}

export function parseApprovalChoices(value: VersionedJson): ApprovalChoiceRead[] {
  if (!Array.isArray(value.value)) return [];
  return value.value.flatMap((entry) => {
    if (typeof entry !== "object" || entry === null || Array.isArray(entry)) return [];
    const id = entry.id;
    const label = entry.label;
    const decisionKind = entry.decisionKind;
    const description = entry.description;
    if (typeof id !== "string" || typeof label !== "string") return [];
    if (!["allow_once", "allow_session", "deny", "cancel"].includes(String(decisionKind))) return [];
    if (description !== null && typeof description !== "string") return [];
    return [{ id, label, decisionKind: decisionKind as ApprovalChoiceRead["decisionKind"], description }];
  });
}

export function parseUserInputQuestions(value: VersionedJson): UserInputQuestion[] {
  if (!Array.isArray(value.value)) return [];
  return value.value.flatMap((entry) => {
    if (typeof entry !== "object" || entry === null || Array.isArray(entry)) return [];
    if (typeof entry.id !== "string" || typeof entry.question !== "string" || !Array.isArray(entry.options)) return [];
    const options = entry.options.flatMap((option) => {
      if (typeof option !== "object" || option === null || Array.isArray(option)) return [];
      if (typeof option.id !== "string" || typeof option.label !== "string") return [];
      return [{ id: option.id, label: option.label, description: typeof option.description === "string" ? option.description : null }];
    });
    return [{
      id: entry.id,
      header: typeof entry.header === "string" ? entry.header : null,
      question: entry.question,
      options,
      multiple: entry.multiple === true,
      freeFormAllowed: entry.freeFormAllowed === true,
      required: entry.required === true,
    }];
  });
}

export function shouldSendComposerKey(event: Pick<KeyboardEvent, "key" | "shiftKey" | "ctrlKey" | "metaKey" | "isComposing">, sendKey: ChatSendKey): boolean {
  if (event.isComposing || event.shiftKey || event.key !== "Enter") return false;
  const modified = event.ctrlKey || event.metaKey;
  return sendKey === "enter" ? !modified : modified;
}

export function autosizeComposerHeight(scrollHeight: number, lineHeight: number): number {
  const minimum = Math.max(64, lineHeight * 3 + 16);
  const maximum = Math.max(minimum, lineHeight * 10 + 16);
  return Math.min(maximum, Math.max(minimum, scrollHeight));
}

export function validateImageFiles(
  files: Pick<File, "name" | "size" | "type">[],
  existingCount: number,
  existingBytes = 0,
): string | null {
  if (existingCount + files.length > CHAT_IMAGE_LIMIT) return `Attach up to ${CHAT_IMAGE_LIMIT} images`;
  const unsupported = files.find((file) => !["image/png", "image/jpeg", "image/gif", "image/webp"].includes(file.type));
  if (unsupported) return `${unsupported.name} is not a supported image`;
  const oversized = files.find((file) => file.size > CHAT_IMAGE_BYTE_LIMIT);
  if (oversized) return `${oversized.name} exceeds the 20 MiB limit`;
  const totalBytes = files.reduce((total, file) => total + file.size, existingBytes);
  if (totalBytes > CHAT_IMAGE_TOTAL_BYTE_LIMIT) return "Chat images must total 50 MiB or less";
  return null;
}

export function validateUserInputAnswers(
  questions: UserInputQuestion[],
  answers: import("$lib/chat/contracts").UserInputAnswer[],
): string[] {
  const byQuestion = new Map(answers.map((answer) => [answer.questionId, answer]));
  const invalid: string[] = [];
  for (const question of questions) {
    const answer = byQuestion.get(question.id);
    const selected = answer?.selectedOptionIds ?? [];
    const freeForm = answer?.freeFormText?.trim() ?? "";
    const optionIds = new Set(question.options.map((option) => option.id));
    const validSelection = selected.every((optionId) => optionIds.has(optionId))
      && (question.multiple || selected.length <= 1);
    const validFreeForm = question.freeFormAllowed || freeForm.length === 0;
    const complete = !question.required || selected.length > 0 || freeForm.length > 0;
    if (!validSelection || !validFreeForm || !complete) invalid.push(question.id);
  }
  return invalid;
}

function supports(capabilities: ProviderCapabilities, capability: string): boolean {
  return capabilities.entries.some((entry) => entry.capability === capability && entry.supported);
}

function fuzzyMatch(value: string, query: string): boolean {
  if (!query) return true;
  const normalized = value.toLowerCase();
  if (normalized.includes(query)) return true;
  let index = 0;
  for (const character of normalized) if (character === query[index]) index += 1;
  return index === query.length;
}

function fuzzyRank(value: string, query: string): number {
  if (!query) return 0;
  const normalized = value.toLowerCase();
  if (normalized === query) return 0;
  if (normalized.startsWith(query)) return 1;
  const index = normalized.indexOf(query);
  return index === -1 ? 1_000 + normalized.length : 10 + index;
}

function optionMatchesDefinition(definition: ModelOptionDefinition, selection: ModelOptionSelection): boolean {
  switch (definition.kind) {
    case "boolean": return selection.value.kind === "boolean";
    case "choice": return selection.value.kind === "choice" && definition.options.some((option) => option.value === selection.value.value);
    case "multiple_choice": return selection.value.kind === "multiple_choice" && selection.value.value.every((value) => definition.options.some((option) => option.value === value));
    case "integer_range": return selection.value.kind === "integer"
      && selection.value.value >= definition.minimum
      && selection.value.value <= definition.maximum
      && (selection.value.value - definition.minimum) % definition.step === 0;
    case "text": return selection.value.kind === "text" && (definition.allowEmpty || selection.value.value.length > 0);
    case "unknown": return true;
  }
}
