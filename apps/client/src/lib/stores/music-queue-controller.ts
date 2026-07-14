import { nextShuffleIndex } from "$lib/music/playback";
import type { MusicSource } from "$lib/music/sources";

export interface MusicQueueState {
  currentSource: MusicSource | null;
  queue: MusicSource[];
  shuffleEnabled: boolean;
  shuffleExplicit: boolean;
  shuffleOrder: number[];
  queueHistory: number[];
  pendingQueueIndex: number | null;
}

interface MusicQueueControllerContext {
  state: MusicQueueState;
  isBusy(): boolean;
  loadSource(source: MusicSource): Promise<void>;
  persistSettings(): void;
  updateExternalControls(): void;
  updateTray(): void;
}

export interface MusicQueueController {
  currentIndex(): number;
  highlightedIndex(): number;
  canPlayPrevious(): boolean;
  canPlayNext(): boolean;
  reset(): void;
  toggleShuffle(): void;
  playItem(index: number): Promise<void>;
  playNext(): Promise<void>;
  playPrevious(): Promise<void>;
}

/** Owns queue history, shuffle selection, and track navigation. */
export function createMusicQueueController(
  context: MusicQueueControllerContext,
): MusicQueueController {
  const state = context.state;

  function currentIndex(): number {
    const source = state.currentSource;
    if (!source) return -1;
    return state.queue.findIndex((item) => item.identity === source.identity);
  }

  function highlightedIndex(): number {
    if (
      state.pendingQueueIndex !== null
      && state.queue[state.pendingQueueIndex]
    ) return state.pendingQueueIndex;
    return currentIndex();
  }

  function canPlayPrevious(): boolean {
    if (!state.currentSource) return false;
    return state.queueHistory.length > 0 || currentIndex() > 0;
  }

  function canPlayNext(): boolean {
    if (!state.currentSource) return false;
    if (state.shuffleEnabled) return state.queue.length > 1;
    const index = currentIndex();
    return index >= 0 && index < state.queue.length - 1;
  }

  function reset(): void {
    state.queue = [];
    state.shuffleOrder = [];
    state.queueHistory = [];
    state.pendingQueueIndex = null;
  }

  function toggleShuffle(): void {
    state.shuffleEnabled = !state.shuffleEnabled;
    state.shuffleExplicit = true;
    state.shuffleOrder = [];
    state.queueHistory = [];
    context.persistSettings();
    context.updateExternalControls();
    context.updateTray();
  }

  async function loadIndex(index: number, rememberCurrent: boolean): Promise<void> {
    const source = state.queue[index];
    if (!source) return;
    const activeIndex = currentIndex();
    if (rememberCurrent && activeIndex >= 0 && activeIndex !== index) {
      state.queueHistory = [...state.queueHistory, activeIndex];
    }
    state.shuffleOrder = state.shuffleOrder.filter((item) => item !== index);
    state.pendingQueueIndex = index;
    await context.loadSource(source);
  }

  async function playItem(index: number): Promise<void> {
    if (context.isBusy() || currentIndex() === index) return;
    await loadIndex(index, true);
  }

  async function playNext(): Promise<void> {
    if (context.isBusy() || state.queue.length === 0) return;
    const activeIndex = currentIndex();
    let nextIndex: number | null = null;
    if (state.shuffleEnabled) {
      const selection = nextShuffleIndex(
        state.queue.length,
        activeIndex,
        state.shuffleOrder,
      );
      nextIndex = selection.index;
      state.shuffleOrder = selection.remainingOrder;
    } else if (activeIndex < 0) {
      nextIndex = 0;
    } else if (activeIndex < state.queue.length - 1) {
      nextIndex = activeIndex + 1;
    }
    if (nextIndex === null || !state.queue[nextIndex]) return;
    await loadIndex(nextIndex, true);
  }

  async function playPrevious(): Promise<void> {
    if (context.isBusy() || !state.currentSource) return;
    const activeIndex = currentIndex();
    if (state.queueHistory.length > 0) {
      const history = [...state.queueHistory];
      const previousIndex = history.pop();
      state.queueHistory = history;
      if (previousIndex !== undefined && state.queue[previousIndex]) {
        state.pendingQueueIndex = previousIndex;
        await context.loadSource(state.queue[previousIndex]);
      }
      return;
    }
    const previousIndex = activeIndex - 1;
    if (previousIndex >= 0 && state.queue[previousIndex]) {
      state.pendingQueueIndex = previousIndex;
      await context.loadSource(state.queue[previousIndex]);
    }
  }

  return {
    currentIndex,
    highlightedIndex,
    canPlayPrevious,
    canPlayNext,
    reset,
    toggleShuffle,
    playItem,
    playNext,
    playPrevious,
  };
}
