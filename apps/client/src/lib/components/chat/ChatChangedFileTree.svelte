<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Folder from "@lucide/svelte/icons/folder";
  import { buildChangedFileTree, type ChatChangedFileTreeNode } from "$lib/chat/inspector-model";
  import type { ChatChangedFileRead } from "$lib/chat/contracts";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    files,
    selectedFile,
    onSelect,
  }: {
    files: ChatChangedFileRead[];
    selectedFile: string | null;
    onSelect: (file: ChatChangedFileRead) => void;
  } = $props();

  const { t } = getLocalization();
  let collapsed = $state<string[]>([]);
  const tree = $derived(buildChangedFileTree(files));

  function toggle(path: string): void {
    collapsed = collapsed.includes(path)
      ? collapsed.filter((entry) => entry !== path)
      : [...collapsed, path];
  }

  function statusLabel(file: ChatChangedFileRead): string {
    return file.status.slice(0, 1).toUpperCase();
  }
</script>

{#snippet nodes(entries: ChatChangedFileTreeNode[], depth: number)}
  {#each entries as node (node.kind + node.relativePath)}
    {#if node.kind === "directory"}
      <button type="button" class="tree-row" style={`padding-left:${depth * 0.5 + 0.25}rem`} onclick={() => toggle(node.relativePath)}>
        {#if collapsed.includes(node.relativePath)}<ChevronRight size={12} />{:else}<ChevronDown size={12} />{/if}
        <Folder size={12} /><span class="truncate">{node.name}</span>
      </button>
      {#if !collapsed.includes(node.relativePath)}{@render nodes(node.children, depth + 1)}{/if}
    {:else if node.file}
      <button type="button" class="tree-row" class:selected={selectedFile === node.relativePath} style={`padding-left:${depth * 0.5 + 1.5}rem`} onclick={() => node.file && onSelect(node.file)} title={node.relativePath}>
        <span class="w-3 font-mono text-[0.583333rem]">{statusLabel(node.file)}</span>
        <span class="min-w-0 flex-1 truncate text-left">{node.name}</span>
        {#if node.file.previousRelativePath}<span class="max-w-20 truncate text-[0.583333rem] text-muted-foreground" title={node.file.previousRelativePath}>← {node.file.previousRelativePath}</span>{/if}
        <span class="flex gap-0.5 text-[0.5rem]" aria-label={t("chat.inspector.changeSources")}>
          {#if node.file.providerReported}<span class="source provider" title={t("chat.inspector.providerSource")}>P</span>{/if}
          {#if node.file.gitObserved}<span class="source git" title={t("chat.inspector.gitSource")}>G</span>{/if}
        </span>
        {#if node.file.binary}<span class="text-[0.583333rem]">B</span>{/if}
        {#if node.file.additions !== null}<span class="text-[0.583333rem] text-emerald-600">+{node.file.additions}</span>{/if}
        {#if node.file.deletions !== null}<span class="text-[0.583333rem] text-destructive">−{node.file.deletions}</span>{/if}
      </button>
    {/if}
  {/each}
{/snippet}

{@render nodes(tree, 0)}

<style>
  .tree-row { display: flex; width: 100%; min-height: 1.75rem; align-items: center; gap: 0.25rem; border-radius: 0.25rem; padding-block: 0.25rem; padding-right: 0.3rem; font-size: 0.75rem; }
  .tree-row:hover, .tree-row.selected { background: var(--accent); }
  .source { border-radius: 0.15rem; padding: 0.05rem 0.18rem; font-weight: 700; }
  .source.provider { background: color-mix(in srgb, var(--primary) 15%, transparent); color: var(--primary); }
  .source.git { background: color-mix(in srgb, var(--success, #16a34a) 15%, transparent); color: var(--success, #16a34a); }
</style>
