<script module lang="ts">
  export interface ChatCodeSelection {
    text: string;
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { basicSetup, EditorView } from "codemirror";

  let {
    text,
    language,
    readOnly = false,
    onChange = () => {},
    onSelectionChange = () => {},
    onSave = () => {},
  }: {
    text: string;
    language: string | null;
    readOnly?: boolean;
    onChange?: (text: string) => void;
    onSelectionChange?: (selection: ChatCodeSelection) => void;
    onSave?: () => void;
  } = $props();

  let host: HTMLDivElement | undefined = $state();
  let editor: EditorView | null = null;
  let applyingExternalText = false;

  onMount(() => {
    if (!host) return;
    host.addEventListener("keydown", handleKeydown);
    editor = new EditorView({
      doc: text,
      parent: host,
      extensions: [
        basicSetup,
        EditorView.editable.of(!readOnly),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged && !applyingExternalText) onChange(update.state.doc.toString());
          if (update.selectionSet || update.docChanged) {
            const range = update.state.selection.main;
            const start = update.state.doc.lineAt(range.from);
            const end = update.state.doc.lineAt(range.to);
            onSelectionChange({
              text: update.state.sliceDoc(range.from, range.to),
              startLine: start.number,
              startColumn: range.from - start.from + 1,
              endLine: end.number,
              endColumn: range.to - end.from + 1,
            });
          }
        }),
        EditorView.theme({
          "&": {
            height: "100%",
            color: "var(--foreground)",
            backgroundColor: "var(--cal-bg)",
            fontSize: "0.733333rem",
          },
          ".cm-scroller": {
            fontFamily: '"SF Mono", "SFMono-Regular", "JetBrains Mono", "Cascadia Code", Consolas, "Liberation Mono", Menlo, monospace',
            lineHeight: "21px",
          },
          ".cm-content": { caretColor: "var(--foreground)" },
          ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--foreground)" },
          ".cm-gutters": {
            color: "var(--muted-foreground)",
            backgroundColor: "var(--cal-bg)",
            borderRight: "1px solid var(--border)",
          },
          ".cm-activeLine, .cm-activeLineGutter": {
            backgroundColor: "color-mix(in srgb, var(--accent) 58%, transparent)",
          },
          ".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection": {
            backgroundColor: "color-mix(in srgb, var(--primary) 28%, transparent) !important",
          },
          ".cm-panels": {
            color: "var(--foreground)",
            backgroundColor: "var(--background)",
          },
          ".cm-panels input": {
            color: "var(--foreground)",
            backgroundColor: "var(--cal-bg)",
            border: "1px solid var(--border)",
          },
        }),
      ],
    });
    return () => {
      host?.removeEventListener("keydown", handleKeydown);
      editor?.destroy();
      editor = null;
    };
  });

  $effect(() => {
    const nextText = text;
    if (!editor || editor.state.doc.toString() === nextText) return;
    applyingExternalText = true;
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: nextText } });
    applyingExternalText = false;
  });

  function handleKeydown(event: KeyboardEvent): void {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
      event.preventDefault();
      onSave();
    }
  }
</script>

<div
  bind:this={host}
  class="code-editor"
  data-language={language ?? "text"}
></div>

<style>
  .code-editor { min-width: 0; min-height: 0; flex: 1; overflow: hidden; }
  .code-editor :global(.cm-editor) { height: 100%; }
</style>
