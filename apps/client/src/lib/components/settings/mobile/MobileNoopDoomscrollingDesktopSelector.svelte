<script lang="ts" module>
  export interface DoomscrollingAppSelection {
    name: string;
    matchNames: readonly string[];
  }

  export type DoomscrollingAppSelectorMode = "multi" | "single";
</script>

<script lang="ts">
  let {
    title: _title,
    mode: _mode = "multi",
    existingNames: _existingNames,
    protectAppSelf: _protectAppSelf = false,
    onAdd: _onAdd,
    onRemove: _onRemove,
    onSelect: _onSelect,
    onCancel: _onCancel,
  }: {
    title: string;
    mode?: DoomscrollingAppSelectorMode;
    existingNames: readonly string[];
    protectAppSelf?: boolean;
    onAdd?: (app: DoomscrollingAppSelection) => boolean;
    onRemove?: (name: string) => void;
    onSelect?: (app: DoomscrollingAppSelection) => void;
    onCancel: () => void;
  } = $props();
</script>
