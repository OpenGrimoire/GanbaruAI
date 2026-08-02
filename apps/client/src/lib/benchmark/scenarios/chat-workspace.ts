import { invoke } from "@tauri-apps/api/core";
import * as chatApi from "$lib/api/chat";
import { getChatBenchmarkHandle } from "$lib/components/chat/benchmark-handle.svelte";
import { getNavigation } from "$lib/stores/navigation.svelte";
import { getChat } from "$lib/stores/chat.svelte";
import {
  DEFAULT_BENCHMARK_DATASET,
  type BenchmarkDatasetProfile,
  type BenchmarkMetric,
  type BenchmarkScenario,
  type BenchmarkScenarioContext,
  type BenchmarkSeedHandle,
} from "../types";
import { timingStatsMetric, waitForFrames } from "./calendar-utils";
import { ensureBenchmarkDbReady, invokeDb, measureMs, throwIfAborted } from "./operation-utils";

const QUERY_RUNS = 20;
const THREAD_SWITCH_RUNS = 20;
const STREAM_FRAME_RUNS = 120;
const PROVIDER_STOP_RUNS = 5;
const IDLE_CPU_WINDOW_MS = 2_000;

interface DenseChatFixtureSummary {
  profile: string;
  channelCount: number;
  threadCount: number;
  eventCount: number;
}

function waitForChatHandle(timeoutMs = 10_000): Promise<void> {
  const deadline = performance.now() + timeoutMs;
  const handle = getChatBenchmarkHandle();
  return new Promise((resolve, reject) => {
    function check(): void {
      if (handle.available) {
        resolve();
      } else if (performance.now() >= deadline) {
        reject(new Error("Chat benchmark route activation timed out"));
      } else {
        requestAnimationFrame(check);
      }
    }
    check();
  });
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds));
}

async function measureIdleCpuPercent(): Promise<number> {
  const startCpuMs = await invoke<number>("benchmark_read_process_cpu_ms");
  const started = performance.now();
  await delay(IDLE_CPU_WINDOW_MS);
  const wallMs = performance.now() - started;
  const endCpuMs = await invoke<number>("benchmark_read_process_cpu_ms");
  return Math.max(0, endCpuMs - startCpuMs) / wallMs * 100;
}

export const chatWorkspaceScenario: BenchmarkScenario = {
  id: "chat-workspace",
  label: "Project Chat",
  description:
    "Measures dense Chat route activation, channel switching, paged reads, search, streamed paints, idle process CPU, memory, and owned process stop.",
  workload: {
    kind: "stress-memory",
    question: "Does dense local Chat remain responsive, bounded, and idle when no provider is running?",
    label: "dense project working-folder Chat interactions",
    durationMs: 0,
    memoryMode: "post-workload",
  },
  defaultDataset: DEFAULT_BENCHMARK_DATASET,
  runMode: "dense-only",

  async setup(_context: BenchmarkScenarioContext): Promise<void> {
    await ensureBenchmarkDbReady();
    getNavigation().navigate("calendar");
    await waitForFrames(2);
  },

  async runWorkload(signal: AbortSignal): Promise<BenchmarkMetric[]> {
    throwIfAborted(signal);
    const activationStarted = performance.now();
    getNavigation().navigate("chat");
    await waitForChatHandle();
    const handle = getChatBenchmarkHandle();
    await handle.waitUntilUsable();
    await waitForFrames(1);
    const routeActivationMs = performance.now() - activationStarted;
    const channelIds = handle.channelIds();
    if (channelIds.length < 2) throw new Error("Chat benchmark requires at least two channels");

    const switchSamples: number[] = [];
    for (let index = 0; index < THREAD_SWITCH_RUNS; index++) {
      throwIfAborted(signal);
      const channelId = channelIds[index % 2];
      switchSamples.push(await measureMs(() => handle.switchChannel(channelId)));
    }

    const denseChannelId = channelIds[0];
    await handle.switchChannel(denseChannelId);
    const pageReadSamples: number[] = [];
    for (let index = 0; index < QUERY_RUNS; index++) {
      throwIfAborted(signal);
      pageReadSamples.push(await measureMs(async () => {
        const page = await chatApi.readChatChannelPage(denseChannelId, null, 100);
        if (page.messages.length !== 100) {
          throw new Error(`Dense Chat latest page returned ${page.messages.length} messages`);
        }
      }));
    }

    const localSearchSamples: number[] = [];
    for (let index = 0; index < QUERY_RUNS; index++) {
      const started = performance.now();
      const query = ["general", "planning", "implementation", "review"][index % 4];
      const matchCount = handle.localSearch(query);
      localSearchSamples.push(performance.now() - started);
      if (matchCount === 0) throw new Error("Dense Chat local rail search returned no match");
    }

    const indexedSearchSamples: number[] = [];
    for (let index = 0; index < QUERY_RUNS; index++) {
      throwIfAborted(signal);
      indexedSearchSamples.push(await measureMs(async () => {
        const projectId = getChat().selectedChannel?.projectId;
        if (!projectId) throw new Error("Dense Chat channel has no project");
        const query = ["general", "planning", "implementation", "review"][index % 4];
        const matches = await chatApi.searchChatMessages(query, projectId, 20);
        if (matches.length === 0) throw new Error("Dense Chat message search returned no match");
      }));
    }

    const streamFrameSamples = await handle.streamFrames(STREAM_FRAME_RUNS);
    const idleCpuPercent = await measureIdleCpuPercent();
    const providerStopSamples: number[] = [];
    for (let index = 0; index < PROVIDER_STOP_RUNS; index++) {
      throwIfAborted(signal);
      providerStopSamples.push(await invoke<number>("benchmark_measure_chat_provider_stop"));
    }

    return [
      timingStatsMetric("Chat route activation", [routeActivationMs]),
      timingStatsMetric("recent channel switch", switchSamples),
      timingStatsMetric("latest page SQLite read and projection", pageReadSamples),
      timingStatsMetric("loaded rail search", localSearchSamples),
      timingStatsMetric("indexed rail search", indexedSearchSamples),
      timingStatsMetric("streamed paint cadence", streamFrameSamples),
      {
        label: "idle app process CPU",
        unit: "percent",
        value: idleCpuPercent,
        details: { windowMs: IDLE_CPU_WINDOW_MS },
      },
      timingStatsMetric("owned provider process stop", providerStopSamples),
    ];
  },

  async seed(
    dataset: BenchmarkDatasetProfile,
    _context: BenchmarkScenarioContext,
  ): Promise<BenchmarkSeedHandle> {
    await ensureBenchmarkDbReady();
    const summary = await invokeDb<DenseChatFixtureSummary>("benchmark_seed_dense_chat_workspace");
    if (summary.profile !== "dense-chat-v1" || summary.channelCount !== 80 || summary.eventCount !== 10_000) {
      throw new Error("Dense Chat benchmark fixture did not match version 1");
    }
    return {
      calendarId: "benchmark-chat",
      eventCount: summary.eventCount,
      datasetId: summary.profile,
      dataset,
    };
  },

  async cleanup(_seedHandle: { calendarId: string }): Promise<void> {
    // The isolated benchmark database is deleted after the run.
  },
};
