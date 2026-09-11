<script lang="ts">
  import CloudRain from "@lucide/svelte/icons/cloud-rain";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { pickSoundscapeFile, revealLocalFile } from "$lib/api/music";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicSoundscapeDefinition } from "$lib/music/soundscape-contracts";
  import { getSoundscapeStore } from "$lib/stores/soundscape.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";

  const { t } = getLocalization();
  const soundscape = getSoundscapeStore();
  let editingId = $state<string | null>(null);
  let nameDraft = $state("");
  let pendingDelete = $state<MusicSoundscapeDefinition | null>(null);
  let {
    filter = "all",
    compact = false,
    addRequest = 0,
    onPlaybackStart = () => undefined,
  }: {
    filter?: "all" | "generated" | "local";
    compact?: boolean;
    addRequest?: number;
    onPlaybackStart?: () => void;
  } = $props();
  let handledAddRequest = $state(0);

  const generated = $derived(soundscape.definitions.filter((entry) => entry.sourceKind === "generated-noise"));
  const local = $derived(soundscape.definitions.filter((entry) => entry.sourceKind === "local-loop"));

  function isPlaying(definition: MusicSoundscapeDefinition): boolean {
    return soundscape.snapshot.status === "playing" && soundscape.snapshot.sourceId === definition.id;
  }

  function displayName(definition: MusicSoundscapeDefinition): string {
    return definition.generatedKind ? t(`music.soundscape.generatedName.${definition.generatedKind}`) : definition.name;
  }

  async function addLoop(replace?: MusicSoundscapeDefinition): Promise<void> {
    const path = await pickSoundscapeFile();
    if (!path || !soundscape.deviceId) return;
    const filename = path.split(/[\\/]/).pop() ?? t("music.soundscape.localLoop");
    const name = replace?.name ?? filename.replace(/\.[^.]+$/, "");
    await soundscape.saveDefinition({
      id: replace?.id ?? crypto.randomUUID(),
      sourceKind: "local-loop",
      generatedKind: null,
      bundledIdentity: null,
      name,
      localPath: path,
      expectedVersion: replace?.version ?? null,
      updatedAt: Date.now(),
    });
  }

  $effect(() => {
    if (addRequest <= handledAddRequest) return;
    handledAddRequest = addRequest;
    void addLoop();
  });

  function beginRename(definition: MusicSoundscapeDefinition): void {
    editingId = definition.id;
    nameDraft = definition.name;
  }

  async function saveRename(definition: MusicSoundscapeDefinition): Promise<void> {
    const name = nameDraft.trim();
    if (!name || !definition.localPath) return;
    await soundscape.saveDefinition({
      id: definition.id,
      sourceKind: definition.sourceKind,
      generatedKind: definition.generatedKind,
      bundledIdentity: definition.bundledIdentity,
      name,
      localPath: definition.localPath,
      expectedVersion: definition.version,
      updatedAt: Date.now(),
    });
    editingId = null;
  }
</script>

<div class="h-full min-h-0 overflow-y-auto p-3" aria-busy={soundscape.loading || soundscape.saving}>
  {#if !compact}<header class="mb-3 rounded-xl border border-border/60 bg-card/60 p-3">
    <h2 class="text-sm font-semibold">{t("music.soundscape.title")}</h2>
    <p class="mt-1 text-xs leading-relaxed text-muted-foreground">{t("music.soundscape.oneLayerExplanation")}</p>
  </header>{/if}

  {#if soundscape.error}
    <div class="mb-3 flex items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 p-2 text-xs" role="alert">
      <span>{t("music.soundscape.genericError")}</span><button type="button" class="rounded-md px-2 py-1 font-medium hover:bg-accent" onclick={() => { void soundscape.recover(); }}>{t("music.soundscape.retry")}</button>
    </div>
  {/if}

  {#if filter !== "local"}<section aria-labelledby="generated-soundscapes">
    <h3 id="generated-soundscapes" class="mb-2 text-xs font-semibold text-muted-foreground">{t("music.soundscape.generated")}</h3>
    <div class="grid grid-cols-[repeat(auto-fit,minmax(min(13rem,100%),1fr))] gap-2.5">
      {#each generated as definition (definition.id)}
        <article class:active-card={soundscape.snapshot.sourceId === definition.id} class="soundscape-card">
          <span class="soundscape-icon"><CloudRain size={19} strokeWidth={1.45} /></span>
          <span class="min-w-0 flex-1"><strong class="block text-xs font-semibold">{displayName(definition)}</strong><span class="mt-1 block text-[0.68rem] leading-relaxed text-muted-foreground">{t(`music.soundscape.description.${definition.generatedKind ?? "white"}`)}</span></span>
          <button type="button" class="soundscape-action" aria-label={isPlaying(definition) ? t("music.soundscape.pause") : t("music.soundscape.play", displayName(definition))} onclick={() => { if (!isPlaying(definition)) onPlaybackStart(); void (isPlaying(definition) ? soundscape.pause() : soundscape.play(definition.id)); }}>
            {#if isPlaying(definition)}<Pause size={15} />{:else}<Play size={15} />{/if}
          </button>
        </article>
      {/each}
    </div>
  </section>{/if}

  {#if filter !== "generated"}<section class:mt-5={filter === "all"} aria-labelledby="local-soundscapes">
    <div class="mb-2 flex items-center justify-between gap-3"><h3 id="local-soundscapes" class="text-xs font-semibold text-muted-foreground">{t("music.soundscape.localLoops")}</h3>{#if !compact}<button type="button" class="inline-flex h-8 items-center gap-1.5 rounded-md bg-secondary px-2.5 text-xs font-medium hover:bg-accent" onclick={() => { void addLoop(); }}><Plus size={14} />{t("music.soundscape.addLoop")}</button>{/if}</div>
    {#if local.length === 0}
      <button type="button" class="flex w-full flex-col items-center justify-center rounded-xl border border-dashed border-border p-5 text-center hover:bg-accent/30" onclick={() => { void addLoop(); }}>
        <FolderOpen size={22} class="text-muted-foreground" /><strong class="mt-2 text-xs">{t("music.soundscape.addFirstLoop")}</strong><span class="mt-1 max-w-md text-[0.68rem] leading-relaxed text-muted-foreground">{t("music.soundscape.filesStayInPlace")}</span>
      </button>
    {:else}
      <div class="space-y-2">
        {#each local as definition (definition.id)}
          <article class:active-card={soundscape.snapshot.sourceId === definition.id} class="soundscape-card items-center">
            <span class="soundscape-icon"><FolderOpen size={18} /></span>
            <span class="min-w-0 flex-1">
              {#if editingId === definition.id}
                <form class="flex gap-1.5" onsubmit={(event) => { event.preventDefault(); void saveRename(definition); }}><input class="h-8 min-w-0 flex-1 rounded-md border border-input bg-background px-2 text-xs" bind:value={nameDraft} aria-label={t("music.soundscape.name")} /><button class="rounded-md bg-primary px-2 text-xs text-primary-foreground">{t("music.soundscape.saveName")}</button></form>
              {:else}
                <button type="button" class="block max-w-full truncate text-left text-xs font-semibold hover:underline" onclick={() => beginRename(definition)}>{definition.name}</button>
                <span class="mt-1 block truncate text-[0.65rem] text-muted-foreground">{definition.availability === "available" ? t("music.soundscape.ready") : t("music.soundscape.needsRepair")}</span>
              {/if}
            </span>
            {#if definition.availability === "available"}
              <button type="button" class="soundscape-action" aria-label={isPlaying(definition) ? t("music.soundscape.pause") : t("music.soundscape.play", displayName(definition))} onclick={() => { if (!isPlaying(definition)) onPlaybackStart(); void (isPlaying(definition) ? soundscape.pause() : soundscape.play(definition.id)); }}>{#if isPlaying(definition)}<Pause size={15} />{:else}<Play size={15} />{/if}</button>
              <button type="button" class="soundscape-action" aria-label={t("music.soundscape.showFile")} title={t("music.soundscape.showFile")} onclick={() => definition.localPath && void revealLocalFile(definition.localPath)}><FolderOpen size={15} /></button>
            {:else}<button type="button" class="rounded-md bg-secondary px-2 py-1.5 text-xs" onclick={() => { void addLoop(definition); }}>{t("music.soundscape.repair")}</button>{/if}
            <button type="button" class="soundscape-action text-destructive" aria-label={t("music.soundscape.remove")} title={t("music.soundscape.remove")} onclick={() => { pendingDelete = definition; }}><Trash2 size={15} /></button>
          </article>
        {/each}
      </div>
    {/if}
  </section>{/if}
</div>

{#if pendingDelete}
  <ConfirmDialog title={t("music.soundscape.removeTitle")} message={t("music.soundscape.removeMessage", pendingDelete.name)} confirmLabel={t("music.soundscape.remove")} cancelLabel={t("common.cancel")} onConfirm={() => { const definition = pendingDelete; pendingDelete = null; if (definition) void soundscape.removeDefinition(definition); }} onCancel={() => { pendingDelete = null; }} />
{/if}

<style>
  .soundscape-card { display: flex; min-width: 0; align-items: flex-start; gap: 0.7rem; border: 1px solid color-mix(in srgb, var(--border) 65%, transparent); border-radius: 0.85rem; background: color-mix(in srgb, var(--card) 78%, transparent); padding: 0.75rem; transition: border-color 130ms ease, background-color 130ms ease; }
  .active-card { border-color: color-mix(in srgb, var(--primary) 45%, var(--border)); background: color-mix(in srgb, var(--primary) 6%, var(--card)); }
  .soundscape-icon { display: grid; height: 2.25rem; width: 2.25rem; flex: none; place-items: center; border-radius: 0.7rem; background: color-mix(in srgb, var(--primary) 10%, var(--secondary)); color: var(--muted-foreground); }
  .soundscape-action { display: inline-grid; height: 2rem; width: 2rem; flex: none; place-items: center; border-radius: 0.45rem; background: var(--secondary); transition: background-color 120ms ease; }
  .soundscape-action:hover { background: var(--accent); }
  @media (prefers-reduced-motion: reduce) { .soundscape-card, .soundscape-action { transition: none; } }
</style>
