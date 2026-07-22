<script lang="ts">
  import Bot from "@lucide/svelte/icons/bot";
  import Braces from "@lucide/svelte/icons/braces";
  import CodeXml from "@lucide/svelte/icons/code-xml";
  import MousePointer2 from "@lucide/svelte/icons/mouse-pointer-2";
  import Sparkles from "@lucide/svelte/icons/sparkles";

  let {
    familyId,
    label,
    accentColor = null,
    size = 15,
  }: {
    familyId: string;
    label: string;
    accentColor?: string | null;
    size?: number;
  } = $props();
</script>

<span class="provider-icon" style:color={accentColor ?? "currentColor"} title={label} aria-hidden="true">
  {#if familyId === "codex"}<Braces {size} />
  {:else if familyId === "claude"}<Sparkles {size} />
  {:else if familyId === "cursor"}<MousePointer2 {size} />
  {:else if familyId === "opencode"}<CodeXml {size} />
  {:else}<Bot {size} />{/if}
</span>

<style>
  .provider-icon { display: inline-grid; width: 1.25rem; height: 1.25rem; flex: 0 0 auto; place-items: center; }
</style>
