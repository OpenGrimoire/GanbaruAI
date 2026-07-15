import type { LocalRootBinding, MusicInspectorDetail } from "$lib/music/library-contracts";
import type { PlaybackStatus } from "$lib/music/playback";
import { musicReviewSource } from "$lib/music/music-review";
import type { MusicSource } from "$lib/music/sources";
import type { MusicSavedQueueEntry, MusicPlaylistSkipReason } from "$lib/music/music-playlist-playback";
import type { MusicRepeatMode } from "$lib/music/library-contracts";
import { getMusicPlayer, type MusicPlaybackContextOwner } from "$lib/stores/music-player.svelte";

export const MUSIC_CONTEXT_BOUNDARY_EVENT = "ganbaru-ai-music-context-boundary";

interface MusicReviewPlaybackContext {
  source: MusicSource | null;
  positionMs: number;
  status: PlaybackStatus;
  queue: MusicSource[];
  shuffleEnabled: boolean;
  shuffleOrder: number[];
  queueHistory: number[];
  pendingQueueIndex: number | null;
  contextOwner: MusicPlaybackContextOwner;
  activePlaylistId: string | null;
  activePlaylistName: string | null;
  activePlaylistRepeatMode: MusicRepeatMode;
  activeQueueItemIds: string[];
  savedQueueEntries: MusicSavedQueueEntry[];
  savedQueueRecentItemIds: string[];
  savedQueueSkipBreakdown: Record<MusicPlaylistSkipReason, number>;
  volume: number;
  rate: number;
}

export class MusicReviewAuditionController {
  active = $state(false);
  reviewItemId = $state<string | null>(null);
  error = $state<string | null>(null);
  private reviewPositions: Record<string, number> = {};
  private context: MusicReviewPlaybackContext | null = null;
  private readonly player = getMusicPlayer();

  get musicPlayer(): ReturnType<typeof getMusicPlayer> {
    return this.player;
  }

  enter(): void {
    if (this.active) return;
    this.context = {
      source: this.player.currentSource ? { ...this.player.currentSource } : null,
      positionMs: this.player.snapshot.positionMs,
      status: this.player.snapshot.status,
      queue: this.player.queue.map((source) => ({ ...source })),
      shuffleEnabled: this.player.shuffleEnabled,
      shuffleOrder: [...this.player.shuffleOrder],
      queueHistory: [...this.player.queueHistory],
      pendingQueueIndex: this.player.pendingQueueIndex,
      contextOwner: this.player.contextOwner,
      activePlaylistId: this.player.activePlaylistId,
      activePlaylistName: this.player.activePlaylistName,
      activePlaylistRepeatMode: this.player.activePlaylistRepeatMode,
      activeQueueItemIds: [...this.player.activeQueueItemIds],
      savedQueueEntries: [...this.player.savedQueueEntries],
      savedQueueRecentItemIds: [...this.player.savedQueueRecentItemIds],
      savedQueueSkipBreakdown: { ...this.player.savedQueueSkipBreakdown },
      volume: this.player.snapshot.volume,
      rate: this.player.snapshot.rate,
    };
    this.active = true;
  }

  async preview(
    detail: MusicInspectorDetail,
    bindings: readonly LocalRootBinding[],
    autoplay: boolean,
  ): Promise<boolean> {
    if (this.active && this.reviewItemId === detail.item.id) return true;
    this.enter();
    const source = musicReviewSource(detail, bindings);
    if (!source) {
      this.error = "No playable location is available for this item.";
      return false;
    }
    this.error = null;
    try {
      this.player.activePlaylistId = null;
      this.player.activePlaylistName = null;
      this.player.activePlaylistRepeatMode = "off";
      this.player.activeQueueItemIds = [];
      this.player.savedQueueEntries = [];
      await this.player.loadSource(source, { autoplay, resume: false, preserveQueue: true });
      this.player.contextOwner = "review";
      this.reviewItemId = detail.item.id;
      const retainedPosition = this.reviewPositions[detail.item.id] ?? 0;
      if (retainedPosition > 0) await this.player.seekToMs(retainedPosition);
      return true;
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      return false;
    }
  }

  async restore(): Promise<void> {
    const context = this.context;
    this.clearOwnership();
    if (!context) return;
    this.player.queue = context.queue;
    this.player.shuffleEnabled = context.shuffleEnabled;
    this.player.shuffleOrder = context.shuffleOrder;
    this.player.queueHistory = context.queueHistory;
    this.player.pendingQueueIndex = context.pendingQueueIndex;
    this.player.contextOwner = context.contextOwner;
    this.player.activePlaylistId = context.activePlaylistId;
    this.player.activePlaylistName = context.activePlaylistName;
    this.player.activePlaylistRepeatMode = context.activePlaylistRepeatMode;
    this.player.activeQueueItemIds = context.activeQueueItemIds;
    this.player.savedQueueEntries = context.savedQueueEntries;
    this.player.savedQueueRecentItemIds = context.savedQueueRecentItemIds;
    this.player.savedQueueSkipBreakdown = context.savedQueueSkipBreakdown;
    if (!context.source) {
      await this.player.resetPlayer();
      return;
    }
    await this.player.loadSource(context.source, { autoplay: false, resume: false, preserveQueue: true });
    await this.player.setTransientVolume(context.volume);
    await this.player.setTransientRate(context.rate);
    if (context.positionMs > 0) await this.player.seekToMs(context.positionMs);
    if (context.status === "playing") await this.player.playPlayback();
    else if (context.status === "paused") await this.player.pausePlayback();
  }

  keep(): void {
    this.player.contextOwner = "manual";
    this.player.activePlaylistId = null;
    this.player.activePlaylistName = null;
    this.player.activePlaylistRepeatMode = "off";
    this.player.activeQueueItemIds = [];
    this.player.savedQueueEntries = [];
    this.clearOwnership();
  }

  supersedeForBoundary(owner: "calendar-event" | "pomodoro"): void {
    if (this.reviewItemId) this.reviewPositions[this.reviewItemId] = this.player.snapshot.positionMs;
    this.player.contextOwner = owner;
    this.clearOwnership();
  }

  private clearOwnership(): void {
    this.active = false;
    this.reviewItemId = null;
    this.context = null;
    this.error = null;
  }
}

export function createMusicReviewAuditionController(): MusicReviewAuditionController {
  return new MusicReviewAuditionController();
}
