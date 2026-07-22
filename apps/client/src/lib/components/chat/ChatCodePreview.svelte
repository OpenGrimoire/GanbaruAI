<script lang="ts">
  import { onMount } from "svelte";
  import { chatCodeColumns, highlightChatCode } from "$lib/chat/code-presentation";
  import { chatVirtualRange } from "$lib/chat/file-tree-model";

  let {
    text,
    language,
  }: {
    text: string;
    language: string | null;
  } = $props();

  const ROW_HEIGHT = 21;
  let scroller: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let viewportHeight = $state(420);
  let scrollFrame: number | null = null;
  const lines = $derived(highlightChatCode(text, language));
  const columns = $derived(chatCodeColumns(text));
  const range = $derived(chatVirtualRange(lines.length, scrollTop, viewportHeight, ROW_HEIGHT, 12));
  const visibleLines = $derived(lines.slice(range.start, range.end));

  onMount(() => {
    const observer = new ResizeObserver(([entry]) => {
      if (entry) viewportHeight = entry.contentRect.height;
    });
    if (scroller) observer.observe(scroller);
    return () => {
      observer.disconnect();
      if (scrollFrame !== null) window.cancelAnimationFrame(scrollFrame);
    };
  });

  function handleScroll(event: Event): void {
    const target = event.currentTarget;
    if (!(target instanceof HTMLDivElement) || scrollFrame !== null) return;
    scrollFrame = window.requestAnimationFrame(() => {
      scrollTop = target.scrollTop;
      scrollFrame = null;
    });
  }
</script>

<div bind:this={scroller} class="code-scroller" onscroll={handleScroll}>
  <div
    class="code-canvas"
    style={`--code-columns:${columns};height:${range.totalSize}px;`}
  >
    <div class="code-window" style={`transform:translateY(${range.offset}px);`}>
      {#each visibleLines as line, visibleIndex (range.start + visibleIndex)}
        <div class="code-line">
          <span class="line-number">{range.start + visibleIndex + 1}</span>
          <code>{#each line.tokens as token}<span class={`syntax-${token.kind}`}>{token.text}</span>{/each}</code>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .code-scroller { min-width: 0; min-height: 0; flex: 1; overflow: auto; background: var(--cal-bg); font-family: "SF Mono", "SFMono-Regular", "JetBrains Mono", "Cascadia Code", Consolas, "Liberation Mono", Menlo, monospace; font-size: 0.733333rem; line-height: 21px; tab-size: 2; }
  .code-canvas { position: relative; width: max(100%, calc((var(--code-columns) + 7) * 1ch)); min-width: max-content; }
  .code-window { position: absolute; inset-inline: 0; top: 0; }
  .code-line { display: grid; height: 21px; grid-template-columns: 3.4rem minmax(0, 1fr); }
  .code-line code { display: block; white-space: pre; padding-inline: 0.8rem 1.25rem; color: var(--foreground); }
  .line-number { position: sticky; left: 0; z-index: 1; user-select: none; border-right: 1px solid color-mix(in srgb, var(--border) 70%, transparent); background: var(--cal-bg); padding-right: 0.65rem; text-align: right; color: color-mix(in srgb, var(--muted-foreground) 68%, transparent); }
  .syntax-keyword { color: color-mix(in srgb, var(--destructive) 72%, var(--foreground)); }
  .syntax-string { color: color-mix(in srgb, var(--action-confirm) 82%, var(--foreground)); }
  .syntax-number { color: color-mix(in srgb, var(--primary) 82%, var(--foreground)); }
  .syntax-comment { color: color-mix(in srgb, var(--muted-foreground) 76%, transparent); font-style: italic; }
  .syntax-type { color: color-mix(in srgb, var(--status-tentative) 78%, var(--foreground)); }
  .syntax-function { color: color-mix(in srgb, var(--primary) 72%, var(--foreground)); }
  .syntax-property, .syntax-attribute { color: color-mix(in srgb, var(--primary) 58%, var(--foreground)); }
  .syntax-operator { color: color-mix(in srgb, var(--muted-foreground) 88%, var(--foreground)); }
  .syntax-tag { color: color-mix(in srgb, var(--destructive) 72%, var(--foreground)); }
  .syntax-variable { color: color-mix(in srgb, var(--status-tentative) 72%, var(--foreground)); }
</style>
