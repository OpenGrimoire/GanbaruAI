<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { getMusicContextAssignments, getMusicPlaylistSummaries } from "$lib/api/music-library";
  import type { CalendarEvent } from "$lib/components/calendar/types";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import { loadContextMusicPlaylist } from "$lib/music/music-context-playlist-loader";
  import {
    resolveMusicContextAssignment,
    type MusicActivityPhase,
    type MusicAssignmentSource,
    type MusicContextAssignment,
  } from "$lib/music/music-context-assignment";
  import {
    MusicPhaseAutomationPlanner,
    nextMusicEventBoundaryMs,
    selectActiveMusicEvent,
    type MusicPhaseActivation,
  } from "$lib/music/music-phase-automation";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    getMusicPlayer,
    type MusicContextPlayback,
    type MusicContextPlaybackIssue,
  } from "$lib/stores/music-player.svelte";
  import { getPomodoro } from "$lib/stores/pomodoro.svelte";

  const calendar = getCalendar();
  const player = getMusicPlayer();
  const pomodoro = getPomodoro();
  const { t } = getLocalization();
  const planner = new MusicPhaseAutomationPlanner();
  let boundaryPulse = $state(0);
  let activationGeneration = 0;
  let processedRetryRequest = 0;

  function calendarWindowEvents(): CalendarEvent[] {
    const today = Temporal.Now.plainDateISO();
    return calendar.eventsInWindow(today.subtract({ days: 1 }), today.add({ days: 2 }));
  }

  function activityPhase(event: CalendarEvent): MusicActivityPhase {
    if (!event.pomodoroConfig || pomodoro.activeBlockId === null) return "focus";
    if (pomodoro.phase === "short_break") return "short-break";
    if (pomodoro.phase === "long_break") return "long-break";
    return "focus";
  }

  function phaseToken(event: CalendarEvent, phase: MusicActivityPhase): string {
    if (!event.pomodoroConfig || pomodoro.activeBlockId === null) return event.start;
    const activeSegment = pomodoro.segments.find((segment) => segment.status === "active");
    return activeSegment?.id
      ?? `${pomodoro.activeRunId ?? event.id}:${pomodoro.currentRhythmPosition}:${phase}`;
  }

  async function allPlaylistSummaries(): Promise<MusicPlaylistSummary[]> {
    const limit = 500;
    const playlists: MusicPlaylistSummary[] = [];
    for (let offset = 0; ; offset += limit) {
      const page = await getMusicPlaylistSummaries(Date.now(), offset, limit);
      playlists.push(...page);
      if (page.length < limit) return playlists;
    }
  }

  function phaseAssignment(
    assignments: readonly MusicContextAssignment[],
    phase: MusicActivityPhase,
  ): MusicContextAssignment | null {
    return assignments.find((assignment) => assignment.phase === phase) ?? null;
  }

  function playbackContext(
    activation: MusicPhaseActivation,
    event: CalendarEvent,
    behavior: MusicContextAssignment["behavior"],
    assignmentSource: MusicAssignmentSource,
    playlist: MusicPlaylistSummary | null,
    state: MusicContextPlayback["state"],
    issue: MusicContextPlaybackIssue = null,
  ): MusicContextPlayback {
    const eventTitle = event.title.trim() || t("music.assignment.context.untitledEvent");
    return {
      owner: event.pomodoroConfig ? "pomodoro" : "calendar-event",
      activationKey: activation.key,
      eventId: event.id,
      eventTitle,
      displayLabel: t(
        "music.assignment.context.selectedBy",
        t(`music.assignment.phase.${activation.phase}`),
        eventTitle,
      ),
      phase: activation.phase,
      behavior,
      assignmentSource,
      playlistId: playlist?.id ?? null,
      playlistName: playlist?.name ?? null,
      state,
      issue,
    };
  }

  async function pauseForUnavailable(context: MusicContextPlayback): Promise<void> {
    player.setContextPlayback(context);
    if (player.isPlaying) await player.pausePlayback("context");
  }

  async function activate(activation: MusicPhaseActivation, event: CalendarEvent): Promise<void> {
    const generation = ++activationGeneration;
    const manualActionVersion = player.manualPlaybackActionVersion;
    const ownerId = event.recurringParentId ?? event.id.split("::")[0];
    try {
      const [overrides, snapshots, environment, playlists] = await Promise.all([
        getMusicContextAssignments("event-override", ownerId),
        getMusicContextAssignments("event-snapshot", ownerId),
        event.environmentId
          ? getMusicContextAssignments("work-environment", event.environmentId)
          : Promise.resolve([]),
        allPlaylistSummaries(),
      ]);
      if (
        generation !== activationGeneration
        || !planner.isCurrent(activation.key)
        || player.manualPlaybackActionVersion !== manualActionVersion
      ) return;
      const resolved = resolveMusicContextAssignment({
        phase: activation.phase,
        eventOverride: phaseAssignment(overrides, activation.phase),
        environmentAssignment: phaseAssignment(environment, activation.phase),
        projectSnapshot: phaseAssignment(snapshots, activation.phase),
        availablePlaylistIds: new Set(playlists.map((playlist) => playlist.id)),
        availableSoundscapeIds: new Set(),
        timedEvent: !event.allDay,
        pomodoroEnabled: Boolean(event.pomodoroConfig),
        activation: activation.activation,
        consecutiveEvent: activation.consecutiveEvent,
      });
      const assignment = resolved.assignment;
      if (!assignment || resolved.availability === "no-assignment" || resolved.availability === "not-applicable") {
        if (player.contextPlayback?.activationKey !== activation.key) player.clearContextPlayback();
        return;
      }
      const playlist = assignment.playlistId
        ? playlists.find((candidate) => candidate.id === assignment.playlistId) ?? null
        : null;
      const soundscapeIssue: MusicContextPlaybackIssue = resolved.soundscapeAvailability === "missing-soundscape"
        ? "deleted-soundscape"
        : null;
      if (resolved.availability === "missing-playlist") {
        await pauseForUnavailable(playbackContext(
          activation,
          event,
          assignment.behavior,
          resolved.source,
          null,
          "unavailable",
          "missing-playlist",
        ));
        return;
      }
      if (assignment.behavior === "keep-current-music") {
        player.setContextPlayback(playbackContext(activation, event, assignment.behavior, resolved.source, playlist, "kept", soundscapeIssue));
        return;
      }
      if (assignment.behavior === "pause-music") {
        player.setContextPlayback(playbackContext(activation, event, assignment.behavior, resolved.source, playlist, "paused", soundscapeIssue));
        if (player.isPlaying) await player.pausePlayback("context");
        return;
      }
      if (!playlist) return;
      const autoplay = assignment.behavior === "play-automatically";
      const context = playbackContext(
        activation,
        event,
        assignment.behavior,
        resolved.source,
        playlist,
        "prepared",
        soundscapeIssue,
      );
      const previousItemId = player.currentSavedItemId;
      const result = await loadContextMusicPlaylist(playlist, context, autoplay, previousItemId);
      if (generation !== activationGeneration || !planner.isCurrent(activation.key)) return;
      if (result.loaded) {
        if (player.manualPlaybackActionVersion !== manualActionVersion) return;
        if (autoplay) {
          await player.playPlayback("context");
          if (
            generation === activationGeneration
            && planner.isCurrent(activation.key)
            && player.manualPlaybackActionVersion === manualActionVersion
          ) player.setContextPlayback({ ...context, state: "playing" });
        }
        return;
      }
      const issue: MusicContextPlaybackIssue = !player.online && result.projection.skipped.offline > 0
        ? "offline-only"
        : "no-eligible-items";
      player.setContextPlayback({ ...context, state: "unavailable", issue });
    } catch (error) {
      if (generation !== activationGeneration || !planner.isCurrent(activation.key)) return;
      console.warn("Failed to activate the event soundtrack:", error);
      const current = player.contextPlayback;
      if (current?.activationKey === activation.key) {
        player.setContextPlayback({ ...current, state: "unavailable", issue: "no-eligible-items" });
      } else {
        player.setContextPlayback(playbackContext(
          activation,
          event,
          "inherit",
          "none",
          null,
          "unavailable",
          "activation-failed",
        ));
      }
    }
  }

  $effect(() => {
    void calendar.indexVersion;
    void boundaryPulse;
    const retryRequest = player.contextRetryRequest;
    const loaded = calendar.loaded;
    if (!loaded) return;
    const events = calendarWindowEvents();
    const nowMs = Date.now();
    const event = selectActiveMusicEvent(events, {
      nowMs,
      activeBlockId: pomodoro.activeBlockId,
    });
    const phase = event ? activityPhase(event) : "focus";
    const activation = planner.observe(event ? {
      eventId: event.id,
      phase,
      phaseToken: phaseToken(event, phase),
      suspended: Boolean(pomodoro.suspendedAway),
    } : null);

    let requestedActivation = activation;
    if (retryRequest !== processedRetryRequest) {
      processedRetryRequest = retryRequest;
      const currentKey = player.contextPlayback?.activationKey;
      if (currentKey && player.contextPlayback?.state === "unavailable") {
        requestedActivation = planner.retry(currentKey);
      }
    }
    if (requestedActivation && event) void activate(requestedActivation, event);
    if (!event && player.contextPlayback) {
      activationGeneration += 1;
      player.clearContextPlayback();
    }

    const nextBoundary = nextMusicEventBoundaryMs(events, nowMs);
    if (nextBoundary === null) return;
    const timeoutId = window.setTimeout(() => {
      boundaryPulse += 1;
    }, Math.max(1, nextBoundary - Date.now() + 25));
    return () => window.clearTimeout(timeoutId);
  });
</script>
