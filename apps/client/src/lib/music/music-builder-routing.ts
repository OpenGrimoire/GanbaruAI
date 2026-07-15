export type MusicBuilderDestination =
  | { kind: "review" }
  | { kind: "playlists" }
  | { kind: "playlist"; playlistId: string }
  | { kind: "library" }
  | { kind: "sources" }
  | { kind: "issues" }
  | { kind: "soundscapes" };

export interface MusicBuilderRoute {
  destination: MusicBuilderDestination;
  inspectorItemId: string | null;
}

export interface MusicBuilderHistory {
  current: MusicBuilderRoute;
  backStack: MusicBuilderRoute[];
}

export interface MusicBuilderRouteContext {
  playlistIds: ReadonlySet<string>;
  itemIds?: ReadonlySet<string>;
}

export function initialMusicBuilderRoute(
  unreviewedCount: number,
  remembered: MusicBuilderDestination | null,
  context: MusicBuilderRouteContext,
): MusicBuilderHistory {
  const fallback: MusicBuilderDestination = unreviewedCount > 0 ? { kind: "review" } : { kind: "playlists" };
  const destination = remembered && isValidDestination(remembered, context) ? remembered : fallback;
  return { current: { destination, inspectorItemId: null }, backStack: [] };
}

export function pushMusicBuilderRoute(
  history: MusicBuilderHistory,
  route: MusicBuilderRoute,
  context: MusicBuilderRouteContext,
): MusicBuilderHistory {
  const valid = repairMusicBuilderRoute(route, context);
  if (routeKey(valid) === routeKey(history.current)) return history;
  return {
    current: valid,
    backStack: [...history.backStack, history.current].slice(-40),
  };
}

export function backMusicBuilderRoute(
  history: MusicBuilderHistory,
  context: MusicBuilderRouteContext,
): MusicBuilderHistory | null {
  if (history.backStack.length === 0) return null;
  const backStack = [...history.backStack];
  while (backStack.length > 0) {
    const candidate = repairMusicBuilderRoute(backStack.pop()!, context);
    if (isValidDestination(candidate.destination, context)) {
      return { current: candidate, backStack };
    }
  }
  return null;
}

export function repairMusicBuilderHistory(
  history: MusicBuilderHistory,
  context: MusicBuilderRouteContext,
): MusicBuilderHistory {
  return {
    current: repairMusicBuilderRoute(history.current, context),
    backStack: history.backStack.map((route) => repairMusicBuilderRoute(route, context)),
  };
}

export function routeKey(route: MusicBuilderRoute): string {
  const destination = route.destination.kind === "playlist"
    ? `playlist:${route.destination.playlistId}`
    : route.destination.kind;
  return `${destination}:inspector:${route.inspectorItemId ?? "none"}`;
}

function repairMusicBuilderRoute(
  route: MusicBuilderRoute,
  context: MusicBuilderRouteContext,
): MusicBuilderRoute {
  const destination = isValidDestination(route.destination, context)
    ? route.destination
    : { kind: "playlists" } as const;
  const inspectorItemId = route.inspectorItemId
    && (!context.itemIds || context.itemIds.has(route.inspectorItemId))
    ? route.inspectorItemId
    : null;
  return { destination, inspectorItemId };
}

function isValidDestination(
  destination: MusicBuilderDestination,
  context: MusicBuilderRouteContext,
): boolean {
  return destination.kind !== "playlist" || context.playlistIds.has(destination.playlistId);
}
