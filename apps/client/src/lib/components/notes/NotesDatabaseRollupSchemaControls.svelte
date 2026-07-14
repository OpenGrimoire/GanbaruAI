<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type {
    NotesDataSourceRollupTargetOption,
    NotesDataSourceSchemaPropertyDraft,
  } from "$lib/notes/data-source-schema";
  import {
    NOTES_DATA_SOURCE_ROLLUP_FUNCTIONS,
    type NotesDataSourceRollupFunction,
  } from "$lib/notes/types";

  let {
    property,
    relations,
    targets,
    saving,
    onRelationChange,
    onTargetChange,
    onFunctionChange,
  }: {
    property: NotesDataSourceSchemaPropertyDraft;
    relations: readonly NotesDataSourceSchemaPropertyDraft[];
    targets: readonly NotesDataSourceRollupTargetOption[];
    saving: boolean;
    onRelationChange: (relationId: string) => void;
    onTargetChange: (targetId: string) => void;
    onFunctionChange: (rollupFunction: NotesDataSourceRollupFunction) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="space-y-2">
  <div class="grid min-w-0 gap-2 @lg:grid-cols-3">
    <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
      <span class="mb-1 block">{t("notes.databaseSchemaRollupRelation")}</span>
      <select
        class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-70"
        value={property.rollupRelationPropertyId}
        disabled={saving || relations.length === 0}
        onchange={(event) => {
          onRelationChange(event.currentTarget.value);
        }}
      >
        {#if property.rollupRelationPropertyId && !relations.some((relation) => relation.id === property.rollupRelationPropertyId)}
          <option value={property.rollupRelationPropertyId}>
            {property.rollupRelationPropertyName || property.rollupRelationPropertyId}
          </option>
        {/if}
        {#each relations as relation (relation.id)}
          <option value={relation.id}>{relation.name}</option>
        {/each}
      </select>
    </label>
    <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
      <span class="mb-1 block">{t("notes.databaseSchemaRollupTargetProperty")}</span>
      <select
        class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-70"
        value={property.rollupPropertyId}
        disabled={saving || targets.length === 0}
        onchange={(event) => {
          onTargetChange(event.currentTarget.value);
        }}
      >
        {#if property.rollupPropertyId && !targets.some((target) => target.id === property.rollupPropertyId)}
          <option value={property.rollupPropertyId}>
            {property.rollupPropertyName || property.rollupPropertyId}
          </option>
        {/if}
        {#each targets as target (target.id)}
          <option value={target.id}>{target.name}</option>
        {/each}
      </select>
    </label>
    <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
      <span class="mb-1 block">{t("notes.databaseSchemaRollupFunction")}</span>
      <select
        class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
        value={property.rollupFunction}
        disabled={saving}
        onchange={(event) => {
          onFunctionChange(event.currentTarget.value as NotesDataSourceRollupFunction);
        }}
      >
        {#each NOTES_DATA_SOURCE_ROLLUP_FUNCTIONS as rollupFunction}
          <option value={rollupFunction}>
            {t("notes.databaseSchemaRollupFunctionLabel", rollupFunction)}
          </option>
        {/each}
      </select>
    </label>
  </div>
  {#if relations.length === 0}
    <p class="text-[0.8rem] text-muted-foreground">
      {t("notes.databaseSchemaRollupNoRelations")}
    </p>
  {/if}
</div>
