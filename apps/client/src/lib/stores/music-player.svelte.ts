import { invoke } from "@tauri-apps/api/core";
import {
  type LocalBackendKind,
  type LocalPlayerSnapshot,
} from "$lib/api/media-player";
import {
  clampRate,
  clampVolume,
  formatRateLabel,
  formatVolumePercent,
  shouldUseWebviewLocalVideo,
  MAX_VOLUME,
  type PlaybackSnapshot,
} from "$lib/music/playback";
import {
  formatSourceKind,
  isYouTubeSource,
  sourceDisplayLabel,
  type MusicSource,
} from "$lib/music/sources";
import type { SchedulerRunContext } from "$lib/scheduling/lifecycle-scheduler";
import {
  initialMusicSnapshot,
  loadMusicPlayerSettings,
  persistMusicPlayerSettings,
} from "./music-player-settings";
import {
  type YouTubeCommandAction,
} from "./music-player-youtube-host";
import { MusicLoadRuntime } from "./music-load-runtime";
import { createMusicQueueController } from "./music-queue-controller";
import {
  createMusicHostedMediaController,
  type MusicStaleVisual,
} from "./music-hosted-media-controller";
import { createMusicExternalControls } from "./music-external-controls";
import { createMusicPlaybackRuntime } from "./music-playback-runtime";
import { createMusicYouTubeAdapter } from "./music-youtube-adapter";
import { createMusicNativeLocalAdapter } from "./music-native-local-adapter";
import { createMusicWebviewLocalAdapter } from "./music-webview-local-adapter";
import {
  createMusicSourceController,
  type LoadSourceOptions,
} from "./music-source-controller";

export type { MusicStaleVisual } from "./music-hosted-media-controller";

const progressMaxFallback = 1;

const initialPlayerSettings = loadMusicPlayerSettings();

class MusicPlayerStore {
  sourceInput = $state("");
  currentSource = $state<MusicSource | null>(null);
  parseError = $state<string | null>(null);
  playerError = $state<string | null>(null);
  snapshot = $state<PlaybackSnapshot>(initialMusicSnapshot(initialPlayerSettings));
  queue = $state<MusicSource[]>([]);
  folderScanTruncated = $state(false);
  shuffleEnabled = $state(initialPlayerSettings.shuffleEnabled);
  shuffleExplicit = $state(initialPlayerSettings.shuffleExplicit);
  muted = $state(initialPlayerSettings.muted);
  shuffleOrder = $state<number[]>([]);
  queueHistory = $state<number[]>([]);
  pendingQueueIndex = $state<number | null>(null);
  youtubeHostUrl = $state<string | null>(null);
  youtubeFrame = $state<HTMLIFrameElement | null>(null);
  youtubeHostToken = $state<string | null>(null);
  youtubeHostReady = $state(false);
  localMediaElement = $state<HTMLMediaElement | null>(null);
  localMediaSrc = $state<string | null>(null);
  localHasVideo = $state(false);
  localBackendKind = $state<LocalBackendKind>("none");
  localVideoReady = $state(false);
  currentArtworkUrl = $state<string | null>(null);
  staleVisual = $state<MusicStaleVisual | null>(null);
  surfaceElement = $state<HTMLElement | null>(null);
  sourceActionBusy = $state(false);
  volumeFeedbackId = $state(0);

  private readonly loadRuntime = new MusicLoadRuntime();
  private lastTraySignature = "";
  private readonly queueController = createMusicQueueController({
    state: this,
    isBusy: () => this.isBusy,
    loadSource: (source) => this.loadSource(source, {
      autoplay: true,
      resume: false,
      preserveQueue: true,
    }),
    persistSettings: () => this.persistPlayerSettings(),
    updateExternalControls: () => this.updateNativeMediaControls(),
    updateTray: () => this.updateMusicTray(),
  });
  private readonly hostedMediaController = createMusicHostedMediaController({
    state: this,
    loadedTitle: () => this.loadedTitle,
  });
  private readonly youtubeAdapter = createMusicYouTubeAdapter({
    state: this,
    loadRuntime: this.loadRuntime,
    effectiveVolume: () => this.effectiveYouTubeVolume(),
    loadSource: (source, autoplay) => this.loadSource(source, {
      autoplay,
      resume: false,
      preserveQueue: true,
    }),
    persist: (force) => this.persistCurrentPlaybackState(force),
    updateExternalControls: () => this.updateSystemMediaControls(),
    updateTray: () => this.updateMusicTray(),
    canPlayNext: () => this.canPlayNextTrack,
    playNext: () => this.playNextTrack(),
  });
  private readonly nativeLocalAdapter = createMusicNativeLocalAdapter({
    state: this,
    updateExternalControls: () => this.updateSystemMediaControls(),
    updateTray: () => this.updateMusicTray(),
    persist: (force) => this.persistCurrentPlaybackState(force),
    playNext: () => this.playNextTrack(),
  });
  private readonly webviewLocalAdapter = createMusicWebviewLocalAdapter({
    state: this,
    effectiveVolume: () => this.effectiveVolume(),
    updateExternalControls: () => this.updateSystemMediaControls(),
    updateNativeControls: () => this.updateNativeMediaControls(),
    updateTray: () => this.updateMusicTray(),
    persist: (force) => this.persistCurrentPlaybackState(force),
    playNext: () => this.playNextTrack(),
  });
  private readonly externalControls = createMusicExternalControls({
    currentSource: () => this.currentSource,
    snapshot: () => this.snapshot,
    title: () => this.loadedTitle,
    sourceKindLabel: () => this.sourceKindLabel,
    artworkUrl: () => this.currentArtworkUrl,
    isBusy: () => this.isBusy,
    canPrevious: () => this.canPlayPreviousTrack,
    canNext: () => this.canPlayNextTrack,
    volume: () => this.volumeControlValue,
    muted: () => this.muted,
    shuffleEnabled: () => this.shuffleEnabled,
    play: () => this.playPlayback(),
    pause: () => this.pausePlayback(),
    togglePlay: () => this.togglePlay(),
    stop: () => this.stopPlayback(),
    previous: () => this.playPreviousTrack(),
    next: () => this.playNextTrack(),
    seekBy: (deltaMs) => this.seekByMs(deltaMs),
    seekTo: (positionMs) => this.seekToMs(positionMs),
    setVolume: (volume) => this.setVolume(volume),
    setRate: (rate) => this.setRate(rate),
    toggleShuffle: () => this.toggleShuffle(),
    handleWindowMessage: this.youtubeAdapter.handleMessage,
  });
  private readonly playbackRuntime = createMusicPlaybackRuntime({
    loadRuntime: this.loadRuntime,
    currentSource: () => this.currentSource,
    snapshot: () => this.snapshot,
    isInitialized: this.externalControls.isInitialized,
    usesNativeLocalBackend: () => this.usesNativeLocalBackend(),
    postYouTubeSnapshot: () => this.postYouTubeCommand({ action: "snapshot" }),
    refreshNativeLocalSnapshot: (context) =>
      this.refreshNativeLocalSnapshot(context),
  });
  private readonly sourceController = createMusicSourceController({
    state: this,
    loadRuntime: this.loadRuntime,
    hostedMedia: this.hostedMediaController,
    destroyYouTube: this.youtubeAdapter.destroy,
    clearYouTubePlaylist: this.youtubeAdapter.clearPlaylistResolution,
    resetLocalPlayback: () => this.resetLocalPlayback(),
    shouldUseVideoElement: (snapshot) => this.shouldUseVideoElement(snapshot),
    prepareWebview: this.webviewLocalAdapter.prepare,
    configureWebview: this.webviewLocalAdapter.configure,
    playWebview: this.webviewLocalAdapter.play,
    playNative: this.nativeLocalAdapter.play,
    applyNativeSnapshot: this.nativeLocalAdapter.applySnapshot,
    loadYouTube: this.youtubeAdapter.load,
    setLoadedPlaybackState: this.playbackRuntime.setLoadedState,
    persistPlayback: () => this.persistCurrentPlaybackState(),
    persistSettings: () => this.persistPlayerSettings(),
    updateExternalControls: () => this.updateSystemMediaControls(),
    updateTray: () => this.updateMusicTray(),
  });

  get isBusy(): boolean {
    return this.snapshot.status === "loading";
  }

  get isPlaying(): boolean {
    return this.snapshot.status === "playing";
  }

  get durationMs(): number {
    return this.snapshot.durationMs ?? 0;
  }

  get progressMax(): number {
    return this.durationMs > 0 ? this.durationMs : progressMaxFallback;
  }

  get progressValue(): number {
    if (this.durationMs <= 0) return 0;
    return Math.min(this.snapshot.positionMs, this.progressMax);
  }

  get loadedTitle(): string {
    return this.currentSource ? sourceDisplayLabel(this.currentSource) : "Nothing loaded";
  }

  get sourceKindLabel(): string {
    return this.currentSource ? formatSourceKind(this.currentSource.kind) : "No source";
  }

  get queuePositionLabel(): string {
    const index = this.currentQueueIndex;
    if (index < 0 || this.queue.length === 0) return "No playlist";
    return `${index + 1} of ${this.queue.length}`;
  }

  get currentQueueIndex(): number {
    return this.queueController.currentIndex();
  }

  get highlightedQueueIndex(): number {
    return this.queueController.highlightedIndex();
  }

  get canPlayPreviousTrack(): boolean {
    return this.queueController.canPlayPrevious();
  }

  get canPlayNextTrack(): boolean {
    return this.queueController.canPlayNext();
  }

  get isLocalVideoActive(): boolean {
    return this.currentSource?.kind === "local-file" && this.localHasVideo;
  }

  get volumeMax(): number {
    return MAX_VOLUME;
  }

  get volumeControlValue(): number {
    return clampVolume(this.snapshot.volume);
  }

  get volumePercentLabel(): string {
    return formatVolumePercent(this.volumeControlValue);
  }

  get volumeFeedbackLabel(): string {
    return `Sound ${formatVolumePercent(this.muted ? 0 : this.volumeControlValue)}`;
  }

  get speedLabel(): string {
    return formatRateLabel(this.snapshot.rate);
  }

  get isYouTubeActive(): boolean {
    return this.currentSource ? isYouTubeSource(this.currentSource) : false;
  }

  init(): void {
    this.externalControls.init();
    this.updateMusicTray();
  }

  destroy(): void {
    this.externalControls.destroy();
    this.youtubeAdapter.destroy();
    this.playbackRuntime.stopScheduler();
    this.hostedMediaController.destroy();
  }

  setSurfaceElement(element: HTMLElement | null): void {
    this.surfaceElement = element;
  }

  registerYouTubeFrame(frame: HTMLIFrameElement | null): void {
    this.youtubeAdapter.registerFrame(frame);
  }

  resumeSnapshotScheduler(): void {
    this.playbackRuntime.resumeScheduler();
  }

  registerLocalMedia(element: HTMLMediaElement | null): void {
    this.webviewLocalAdapter.register(element);
  }

  async loadFromInput(): Promise<void> {
    await this.sourceController.loadFromInput();
  }
  async loadFolder(): Promise<void> {
    await this.sourceController.loadFolder();
  }
  async loadSource(
    source: MusicSource,
    options: LoadSourceOptions = {},
  ): Promise<void> {
    await this.sourceController.loadSource(source, options);
  }
  async togglePlay(): Promise<void> {
    if (!this.currentSource || this.snapshot.status === "loading") return;
    if (this.snapshot.status === "playing") {
      await this.pausePlayback();
      return;
    }
    await this.playPlayback();
  }

  async playPlayback(): Promise<void> {
    if (!this.currentSource) return;
    this.playerError = null;
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.playNativeLocalMedia();
      } else {
        await this.playLocalMediaElement();
      }
      await this.persistCurrentPlaybackState();
      this.updateMusicTray();
      return;
    }
    this.youtubeAdapter.clearOptimisticPause();
    this.postYouTubeCommand({ action: "play", volume: this.effectiveYouTubeVolume() });
    this.snapshot = { ...this.snapshot, status: "playing" };
    this.updateSystemMediaControls();
    void this.persistCurrentPlaybackState();
    this.updateMusicTray();
  }

  async pausePlayback(): Promise<void> {
    if (!this.currentSource) return;
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.pauseNativeLocalMedia();
      } else {
        this.pauseLocalMediaElement();
      }
    } else {
      this.youtubeAdapter.beginOptimisticPause();
      this.postYouTubeCommand({ action: "pause", volume: this.effectiveYouTubeVolume() });
      this.snapshot = { ...this.snapshot, status: "paused" };
      this.updateSystemMediaControls();
    }
    void this.persistCurrentPlaybackState();
    this.updateMusicTray();
  }

  async stopPlayback(): Promise<void> {
    if (!this.currentSource) return;
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.stopNativeLocalMedia();
      } else {
        this.stopLocalMediaElement();
      }
    } else {
      this.postYouTubeCommand({ action: "stop" });
      this.snapshot = { ...this.snapshot, status: "idle", positionMs: 0 };
    }
    await this.persistCurrentPlaybackState(true);
    this.updateMusicTray();
  }

  async seekToMs(positionMs: number): Promise<void> {
    if (!this.currentSource) return;
    const bounded = Math.max(0, Math.min(positionMs, this.progressMax));
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.seekNativeLocalMedia(bounded);
      } else {
        this.seekLocalMediaElement(bounded);
      }
    } else {
      this.postYouTubeCommand({ action: "seek", positionMs: bounded });
      this.snapshot = { ...this.snapshot, positionMs: bounded };
    }
    this.updateSystemMediaControls();
    await this.persistCurrentPlaybackState(true);
  }

  async setVolume(value: number): Promise<void> {
    const volume = clampVolume(value);
    this.snapshot = { ...this.snapshot, volume };
    this.muted = false;
    this.announceVolumeFeedback();
    this.persistPlayerSettings();
    if (!this.currentSource) {
      this.updateNativeMediaControls();
      return;
    }
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.applyNativeLocalVolume();
      } else {
        this.applyLocalVolume();
      }
    } else {
      this.postYouTubeCommand({ action: "volume", volume: clampVolume(volume) });
    }
    this.updateNativeMediaControls();
    await this.persistCurrentPlaybackState();
    this.updateMusicTray();
  }

  async setTransientVolume(value: number): Promise<void> {
    const volume = clampVolume(value);
    this.snapshot = { ...this.snapshot, volume };
    if (!this.currentSource) {
      this.updateNativeMediaControls();
      return;
    }
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.applyNativeLocalTransientVolume();
      } else {
        this.applyLocalVolume();
      }
    } else {
      this.postYouTubeCommand({ action: "volume", volume: this.effectiveYouTubeVolume() });
    }
    this.updateNativeMediaControls();
    this.updateMusicTray();
  }

  async toggleMute(): Promise<void> {
    this.muted = !this.muted;
    this.announceVolumeFeedback();
    this.persistPlayerSettings();
    if (!this.currentSource) {
      this.updateNativeMediaControls();
      return;
    }
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.applyNativeLocalMute();
      } else {
        this.applyLocalVolume();
      }
    } else {
      this.postYouTubeCommand({ action: "volume", volume: this.effectiveYouTubeVolume() });
    }
    this.updateNativeMediaControls();
    await this.persistCurrentPlaybackState();
  }

  async adjustVolume(delta: number): Promise<void> {
    await this.setVolume(this.snappedVolume(this.volumeControlValue + delta, Math.abs(delta)));
  }

  private snappedVolume(value: number, step: number): number {
    if (!Number.isFinite(value) || !Number.isFinite(step) || step <= 0) return this.volumeControlValue;
    return Number((Math.round(value / step) * step).toFixed(2));
  }

  private announceVolumeFeedback(): void {
    this.volumeFeedbackId += 1;
  }

  async seekByMs(deltaMs: number): Promise<void> {
    await this.seekToMs(this.snapshot.positionMs + deltaMs);
  }

  handleVolumeWheel(event: WheelEvent): void {
    if (event.ctrlKey) return;
    const target = event.target;
    if (target instanceof HTMLElement) {
      if (target.closest("[data-music-scrollable='true']")) return;
      const editable = target.closest("input, textarea, [contenteditable='true']");
      if (editable && !target.closest("[data-music-volume-control='true']")) return;
    }
    event.preventDefault();
    event.stopPropagation();
    const delta = event.deltaY === 0 ? -event.deltaX : event.deltaY;
    if (delta === 0) return;
    const step = 0.05;
    const direction = delta > 0 ? -1 : 1;
    void this.setVolume(this.snappedVolume(this.volumeControlValue + direction * step, step));
  }

  async setRate(value: number): Promise<void> {
    const rate = clampRate(value);
    this.snapshot = { ...this.snapshot, rate };
    this.persistPlayerSettings();
    if (!this.currentSource) {
      this.updateNativeMediaControls();
      return;
    }
    if (this.currentSource.kind === "local-file") {
      if (this.usesNativeLocalBackend()) {
        await this.nativeLocalAdapter.applyRate(rate);
      } else if (this.localMediaElement) {
        this.localMediaElement.playbackRate = rate;
      }
    } else {
      this.postYouTubeCommand({ action: "rate", rate });
    }
    this.updateSystemMediaControls();
    await this.persistCurrentPlaybackState();
  }

  async resetPlayer(): Promise<void> {
    await this.sourceController.resetPlayer();
  }
  toggleShuffle(): void {
    this.queueController.toggleShuffle();
  }

  async playQueueItem(index: number): Promise<void> {
    await this.queueController.playItem(index);
  }

  async playNextTrack(): Promise<void> {
    await this.queueController.playNext();
  }

  async playPreviousTrack(): Promise<void> {
    await this.queueController.playPrevious();
  }

  handleLocalLoadedMetadata(event: Event): void {
    this.webviewLocalAdapter.handleLoadedMetadata(event);
  }

  handleLocalLoadedData(event: Event): void {
    this.webviewLocalAdapter.handleLoadedData(event);
  }

  handleLocalTimeUpdate(event: Event): void {
    this.webviewLocalAdapter.handleTimeUpdate(event);
  }

  handleLocalPlay(event: Event): void {
    this.webviewLocalAdapter.handlePlay(event);
  }

  handleLocalPause(event: Event): void {
    this.webviewLocalAdapter.handlePause(event);
  }

  async handleLocalEnded(event: Event): Promise<void> {
    await this.webviewLocalAdapter.handleEnded(event);
  }

  handleLocalError(event: Event): void {
    this.webviewLocalAdapter.handleError(event);
  }

  handleArtworkLoaded(): void {
    this.hostedMediaController.finishVisualTransitionAfterPaint();
  }

  handleArtworkError(): void {
    this.currentArtworkUrl = null;
    this.hostedMediaController.syncRetention();
  }

  private applyLocalSnapshot(localSnapshot: LocalPlayerSnapshot): void {
    this.nativeLocalAdapter.applySnapshot(localSnapshot);
  }

  private async playNativeLocalMedia(): Promise<void> {
    await this.nativeLocalAdapter.play();
  }

  private async pauseNativeLocalMedia(): Promise<void> {
    await this.nativeLocalAdapter.pause();
  }

  private async stopNativeLocalMedia(): Promise<void> {
    await this.nativeLocalAdapter.stop();
  }

  private async seekNativeLocalMedia(positionMs: number): Promise<void> {
    await this.nativeLocalAdapter.seek(positionMs);
  }

  private async applyNativeLocalVolume(): Promise<void> {
    await this.nativeLocalAdapter.applyVolume(true);
  }

  private async applyNativeLocalTransientVolume(): Promise<void> {
    await this.nativeLocalAdapter.applyVolume(false);
  }

  private async applyNativeLocalMute(): Promise<void> {
    await this.nativeLocalAdapter.applyMute();
  }

  private async refreshNativeLocalSnapshot(context?: SchedulerRunContext): Promise<void> {
    await this.nativeLocalAdapter.refresh(context);
  }

  private configureLocalMediaElement(): void {
    this.webviewLocalAdapter.configure();
  }

  private async playLocalMediaElement(): Promise<void> {
    await this.webviewLocalAdapter.play();
  }

  private pauseLocalMediaElement(): void {
    this.webviewLocalAdapter.pause();
  }

  private stopLocalMediaElement(): void {
    this.webviewLocalAdapter.stop();
  }

  private seekLocalMediaElement(positionMs: number): void {
    this.webviewLocalAdapter.seek(positionMs);
  }

  private shouldUseVideoElement(localSnapshot: LocalPlayerSnapshot): boolean {
    return shouldUseWebviewLocalVideo(localSnapshot);
  }

  private usesNativeLocalBackend(): boolean {
    return this.nativeLocalAdapter.isActive();
  }

  private async resetLocalPlayback(): Promise<void> {
    await this.nativeLocalAdapter.reset();
    this.resetLocalMediaElement();
  }

  private resetLocalMediaElement(): void {
    this.webviewLocalAdapter.reset();
  }

  private postYouTubeCommand(payload: Record<string, unknown> & { action: YouTubeCommandAction | "snapshot" }): void {
    this.youtubeAdapter.post(payload);
  }

  private async persistCurrentPlaybackState(force = false): Promise<void> {
    await this.playbackRuntime.persist(force);
  }

  private destroyYouTubePlayer(): void {
    this.youtubeAdapter.destroy();
  }

  private persistPlayerSettings(): void {
    persistMusicPlayerSettings({
      volume: this.snapshot.volume,
      rate: this.snapshot.rate,
      shuffleEnabled: this.shuffleEnabled,
      shuffleExplicit: this.shuffleExplicit,
      muted: this.muted,
    });
  }

  private applyLocalVolume(): void {
    this.webviewLocalAdapter.applyVolume();
  }

  private effectiveVolume(): number {
    return this.muted ? 0 : this.volumeControlValue;
  }

  private effectiveYouTubeVolume(): number {
    return this.muted ? 0 : clampVolume(this.snapshot.volume);
  }

  private updateSystemMediaControls(): void {
    this.externalControls.update();
  }

  private updateNativeMediaControls(): void {
    this.externalControls.updateNative();
  }

  private updateMusicTray(): void {
    this.playbackRuntime.syncScheduler();
    const signature = [
      this.currentSource?.identity ?? "none",
      this.currentSource ? this.loadedTitle : "none",
      this.snapshot.status,
      this.canPlayPreviousTrack ? "prev" : "no-prev",
      this.canPlayNextTrack ? "next" : "no-next",
    ].join("|");
    if (signature === this.lastTraySignature) return;
    this.lastTraySignature = signature;
    invoke("update_music_tray", {
      update: {
        status: this.snapshot.status,
        title: this.currentSource ? this.loadedTitle : null,
        canPlayPause: Boolean(this.currentSource) && !this.isBusy,
        canPrevious: this.canPlayPreviousTrack,
        canNext: this.canPlayNextTrack,
      },
    }).catch(() => {});
  }
}

let store: MusicPlayerStore | null = null;

export function getMusicPlayer(): MusicPlayerStore {
  if (!store) store = new MusicPlayerStore();
  return store;
}
