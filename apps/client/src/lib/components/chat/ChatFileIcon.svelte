<script lang="ts">
  import File from "@lucide/svelte/icons/file";
  import FileArchive from "@lucide/svelte/icons/file-archive";
  import FileImage from "@lucide/svelte/icons/file-image";
  import FileText from "@lucide/svelte/icons/file-text";
  import { chatFilePresentation } from "$lib/chat/code-presentation";

  let { path, size = 14 }: { path: string; size?: number } = $props();
  const presentation = $derived(chatFilePresentation(path));
</script>

<span class={`file-icon tone-${presentation.tone}`} aria-hidden="true">
  {#if presentation.kind === "glyph"}<span class="file-glyph">{presentation.glyph}</span>
  {:else if presentation.kind === "image"}<FileImage {size} />
  {:else if presentation.kind === "archive"}<FileArchive {size} />
  {:else if presentation.kind === "text"}<FileText {size} />
  {:else}<File {size} />{/if}
</span>

<style>
  .file-icon { display: inline-grid; width: 1rem; height: 1rem; flex: 0 0 auto; place-items: center; color: var(--muted-foreground); }
  .file-glyph { font-family: "SF Mono", "SFMono-Regular", "JetBrains Mono", "Cascadia Code", Consolas, monospace; font-size: 0.54rem; font-weight: 600; letter-spacing: -0.05em; line-height: 1; }
  .tone-typed { color: color-mix(in srgb, var(--primary) 82%, var(--foreground)); }
  .tone-script { color: color-mix(in srgb, var(--status-tentative) 82%, var(--foreground)); }
  .tone-style { color: color-mix(in srgb, var(--primary) 58%, var(--action-confirm)); }
  .tone-markup { color: color-mix(in srgb, var(--destructive) 72%, var(--foreground)); }
  .tone-data { color: color-mix(in srgb, var(--status-tentative) 68%, var(--foreground)); }
  .tone-systems { color: color-mix(in srgb, var(--destructive) 48%, var(--status-tentative)); }
  .tone-docs { color: color-mix(in srgb, var(--muted-foreground) 88%, var(--foreground)); }
  .tone-config { color: color-mix(in srgb, var(--muted-foreground) 72%, var(--primary)); }
</style>
