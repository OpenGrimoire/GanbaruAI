/** Stable Calendar occurrence identity and its eligible execution window. */
export interface FocusOwnershipCandidate {
  id: string;
  startMs: number;
  endMs: number;
  createdAt?: string;
}

/** Keep the eligible owner, then choose the earliest end, creation identity, and occurrence ID. */
export function selectFocusOwner<Candidate extends FocusOwnershipCandidate>(
  candidates: readonly Candidate[],
  nowMs: number,
  currentOwnerId: string | null,
): Candidate | undefined {
  if (!Number.isFinite(nowMs)) return undefined;
  let selected: Candidate | undefined;
  for (const candidate of candidates) {
    if (!Number.isFinite(candidate.startMs) || !Number.isFinite(candidate.endMs)
      || candidate.startMs >= candidate.endMs
      || candidate.startMs > nowMs || nowMs >= candidate.endMs) continue;
    if (candidate.id === currentOwnerId) return candidate;
    if (!selected || compareFocusOwners(candidate, selected) < 0) selected = candidate;
  }
  return selected;
}

function compareFocusOwners(left: FocusOwnershipCandidate, right: FocusOwnershipCandidate): number {
  if (left.endMs !== right.endMs) return left.endMs - right.endMs;
  const leftCreated = left.createdAt ?? "";
  const rightCreated = right.createdAt ?? "";
  if (leftCreated !== rightCreated) return leftCreated < rightCreated ? -1 : 1;
  return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
}
