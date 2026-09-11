import type { ChatChangeNotification } from "./contracts";

export interface ChatSequenceState {
  lastSequence: number;
  revision: number;
}

export type ChatSequenceDecision =
  | { kind: "applied"; state: ChatSequenceState; changedProjectionKeys: string[] }
  | { kind: "duplicate"; state: ChatSequenceState }
  | { kind: "gap"; state: ChatSequenceState; replayAfterSequence: number }
  | { kind: "stale_revision"; state: ChatSequenceState };

export function applyChatChangeNotification(
  state: ChatSequenceState,
  notification: ChatChangeNotification,
): ChatSequenceDecision {
  if (notification.sequence <= state.lastSequence) {
    return { kind: "duplicate", state };
  }
  if (notification.sequence !== state.lastSequence + 1) {
    return { kind: "gap", state, replayAfterSequence: state.lastSequence };
  }
  if (notification.revision <= state.revision) {
    return { kind: "stale_revision", state };
  }
  return {
    kind: "applied",
    state: {
      lastSequence: notification.sequence,
      revision: notification.revision,
    },
    changedProjectionKeys: [...new Set(notification.changedProjectionKeys)],
  };
}

export function reconcileChatSequence(
  local: ChatSequenceState,
  durable: ChatSequenceState,
): ChatSequenceDecision {
  if (durable.lastSequence < local.lastSequence || durable.revision < local.revision) {
    return { kind: "stale_revision", state: local };
  }
  if (durable.lastSequence === local.lastSequence) {
    return durable.revision === local.revision
      ? { kind: "duplicate", state: local }
      : { kind: "stale_revision", state: local };
  }
  return { kind: "gap", state: local, replayAfterSequence: local.lastSequence };
}
