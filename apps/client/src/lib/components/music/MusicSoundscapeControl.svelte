<script lang="ts">
  import AudioLines from "@lucide/svelte/icons/audio-lines";
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import { onMount } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getSoundscapeStore } from "$lib/stores/soundscape.svelte";
  import { cn } from "$lib/utils";

  let { onOpenSoundscapes }: { onOpenSoundscapes: () => void } = $props();
  const { t } = getLocalization();
  const soundscape = getSoundscapeStore();
  let root = $state<HTMLElement | null>(null);
  let open = $state(false);
  const active = $derived(soundscape.activeDefinition);
  const activeName = $derived(active?.generatedKind ? t(`music.soundscape.generatedName.${active.generatedKind}`) : active?.name ?? "");

  onMount(() => {
    const close = (event: PointerEvent) => { if (open && root && !root.contains(event.target as Node)) open = false; };
    window.addEventListener("pointerdown", close, true);
    return () => window.removeEventListener("pointerdown", close, true);
  });
</script>

<div bind:this={root} class="relative">
  <button type="button" class={cn("relative inline-flex h-9 w-9 items-center justify-center rounded-md bg-secondary text-secondary-foreground transition-colors hover:bg-accent", soundscape.snapshot.status === "playing" && "text-primary")} aria-haspopup="dialog" aria-expanded={open} aria-label={t("music.soundscape.controls")} title={active ? t("music.soundscape.active", activeName) : t("music.soundscape.controls")} onclick={() => open = !open}>
    <AudioLines size={14} strokeWidth={1.4} />
    {#if soundscape.snapshot.status === "playing"}<span class="absolute bottom-1 right-1 h-1.5 w-1.5 rounded-full bg-primary" aria-hidden="true"></span>{/if}
  </button>
  {#if open}
    <div class="absolute bottom-full right-0 z-40 mb-2 max-h-[calc(100vh-1rem)] w-[min(17rem,calc(100vw-1rem))] overflow-y-auto rounded-xl border border-border bg-popover p-2.5 text-popover-foreground shadow-xl" role="dialog" aria-label={t("music.soundscape.controls")}>
      <div class="flex items-center gap-2"><span class="grid h-8 w-8 place-items-center rounded-lg bg-secondary"><AudioLines size={16} /></span><span class="min-w-0 flex-1"><strong class="block truncate text-xs">{active ? activeName : t("music.soundscape.noneSelected")}</strong><span class="text-[0.65rem] text-muted-foreground">{soundscape.snapshot.status === "playing" ? t("music.soundscape.playing") : t("music.soundscape.notPlaying")}</span></span>{#if active}<button type="button" class="grid h-8 w-8 place-items-center rounded-md bg-primary text-primary-foreground" onclick={() => { void (soundscape.snapshot.status === "playing" ? soundscape.pause() : soundscape.snapshot.status === "paused" ? soundscape.resume() : soundscape.play(active.id)); }}>{#if soundscape.snapshot.status === "playing"}<Pause size={14} />{:else}<Play size={14} />{/if}</button>{/if}</div>
      <label class="mt-3 block text-[0.68rem] font-medium" for="soundscape-volume">{t("music.soundscape.volume")}: {Math.round((soundscape.persisted?.volume ?? 0.35) * 100)}%</label>
      <input id="soundscape-volume" class="mt-1.5 block w-full accent-primary" type="range" min="0" max="1" step="0.05" value={soundscape.persisted?.volume ?? 0.35} oninput={(event) => { void soundscape.setVolume(Number(event.currentTarget.value)); }} />
      <div class="mt-2 grid grid-cols-3 gap-1.5">
        {#each soundscape.definitions.filter((entry) => entry.sourceKind === "generated-noise") as definition (definition.id)}
          <button type="button" class={cn("truncate rounded-md bg-secondary px-2 py-1.5 text-[0.68rem] hover:bg-accent", active?.id === definition.id && "ring-1 ring-primary")} onclick={() => { void soundscape.play(definition.id); }}>{definition.generatedKind ? t(`music.soundscape.generatedName.${definition.generatedKind}`) : definition.name}</button>
        {/each}
      </div>
      {#if soundscape.definitions.some((entry) => entry.sourceKind === "local-loop")}
        <div class="mt-1.5 space-y-1">
          {#each soundscape.definitions.filter((entry) => entry.sourceKind === "local-loop") as definition (definition.id)}
            <button type="button" disabled={definition.availability !== "available"} class={cn("flex h-8 w-full items-center justify-between gap-2 rounded-md bg-secondary px-2 text-left text-[0.68rem] hover:bg-accent disabled:opacity-50", active?.id === definition.id && "ring-1 ring-primary")} onclick={() => { void soundscape.play(definition.id); }}><span class="truncate">{definition.name}</span><span class="shrink-0 text-[0.6rem] text-muted-foreground">{definition.availability === "available" ? t("music.soundscape.ready") : t("music.soundscape.needsRepair")}</span></button>
          {/each}
        </div>
      {/if}
      <button type="button" class="mt-2 h-8 w-full rounded-md text-xs font-medium text-primary hover:bg-accent" onclick={() => { open = false; onOpenSoundscapes(); }}>{t("music.soundscape.openBuilder")}</button>
    </div>
  {/if}
</div>
