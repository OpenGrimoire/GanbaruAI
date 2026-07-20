<script lang="ts">
  import Network from "@lucide/svelte/icons/network";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  let { onlineCount, onLocalOnly, onContinue, onClose }: { onlineCount: number; onLocalOnly: () => void; onContinue: () => void; onClose: () => void } = $props();
  const { t } = getLocalization();
</script>

<div class="absolute inset-0 z-50 grid place-items-center bg-background/60 p-2 backdrop-blur-sm">
  <button type="button" class="absolute inset-0" onclick={onClose} aria-label={t("music.builder.close")}></button>
  <div use:containMusicDialogFocus={{ onEscape: onClose }} class="relative flex max-h-full w-[min(27rem,100%)] flex-col overflow-hidden rounded-2xl border border-border/70 bg-popover shadow-2xl" role="dialog" aria-modal="true" aria-label={t("music.builder.refreshRequiresNetworkTitle")} tabindex="-1">
    <div class="min-h-0 overflow-y-auto p-4"><span class="grid h-10 w-10 place-items-center rounded-xl bg-primary/10 text-primary"><Network size={19} /></span><h2 class="mt-3 text-sm font-semibold">{t("music.builder.refreshRequiresNetworkTitle")}</h2><p class="mt-1.5 text-[0.7rem] leading-relaxed text-muted-foreground">{t("music.builder.refreshRequiresNetworkDescription", onlineCount)}</p></div>
    <div class="flex shrink-0 flex-wrap justify-end gap-2 border-t border-border/60 p-3"><button type="button" onclick={onClose} class="h-8 rounded-lg px-2.5 text-[0.68rem] font-semibold text-muted-foreground hover:bg-accent">{t("music.builder.cancel")}</button><button type="button" onclick={onLocalOnly} class="h-8 rounded-lg bg-secondary px-2.5 text-[0.68rem] font-semibold text-secondary-foreground">{t("music.builder.localOnly")}</button><button type="button" onclick={onContinue} class="h-8 rounded-lg bg-primary px-2.5 text-[0.68rem] font-semibold text-primary-foreground">{t("music.builder.continueOnline")}</button></div>
  </div>
</div>
