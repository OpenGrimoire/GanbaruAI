<script lang="ts">
  import { onMount, tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    getNotesDataSourceSchema,
    updateNotesDataSourceSchema,
  } from "$lib/api/notes";
  import NotesDataSourceRows from "./NotesDataSourceRows.svelte";
  import {
    createNotesDataSourcePropertyDraft,
    defaultNotesDataSourcePropertyName,
    notesDataSourceSchemaDraftFromDto,
    notesDataSourceSchemaUpdateFromDraft,
    type NotesDataSourceSchemaOptionDraft,
    type NotesDataSourceSchemaPropertyDraft,
  } from "$lib/notes/data-source-schema";
  import {
    NOTES_DATA_SOURCE_NUMBER_FORMATS,
    NOTES_DATA_SOURCE_PROPERTY_TYPES,
    NOTES_DATA_SOURCE_SELECT_COLORS,
    NOTES_DATA_SOURCE_STATUS_GROUPS,
    type NotesChildDatabaseBlock,
    type NotesDataSourceNumberFormat,
    type NotesDataSourcePropertyType,
    type NotesDataSourceSchema,
    type NotesDataSourceSelectColor,
    type NotesDataSourceStatusGroup,
  } from "$lib/notes/types";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Database from "@lucide/svelte/icons/database";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    block,
    focusBlockId,
    focusRequestId,
    onFocusBlock,
    onKeydown,
    onSelectPage,
  }: {
    block: NotesChildDatabaseBlock;
    focusBlockId: string | null;
    focusRequestId: number;
    onFocusBlock: (blockId: string) => void;
    onKeydown: (event: KeyboardEvent) => void;
    onSelectPage: (pageId: string) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;

  let button: HTMLButtonElement | null = $state(null);
  let expanded = $state(false);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let saved = $state(false);
  let dirty = $state(false);
  let schema = $state<NotesDataSourceSchema | null>(null);
  let properties = $state<NotesDataSourceSchemaPropertyDraft[]>([]);
  let newPropertyType = $state<NotesDataSourcePropertyType>("rich_text");

  const title = $derived(block.child_database.title.trim());
  const dataSourceId = $derived(block.child_database.data_source_id ?? null);
  const localDatabase = $derived(
    block.child_database.database_id !== undefined
      && block.child_database.data_source_id !== undefined
      && block.child_database.view_id !== undefined,
  );
  const propertyCount = $derived(properties.length);

  onMount(() => {
    if (localDatabase) void loadSchema();
  });

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    void tick().then(() => button?.focus());
  });

  async function loadSchema(): Promise<void> {
    if (!dataSourceId) return;
    loading = true;
    error = null;
    try {
      const loaded = await getNotesDataSourceSchema(dataSourceId);
      schema = loaded;
      properties = notesDataSourceSchemaDraftFromDto(loaded.data_source, loaded.view);
      dirty = false;
      saved = false;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      loading = false;
    }
  }

  async function saveSchema(): Promise<void> {
    if (!dataSourceId) return;
    saving = true;
    error = null;
    try {
      const update = notesDataSourceSchemaUpdateFromDraft(properties);
      const updated = await updateNotesDataSourceSchema(dataSourceId, update);
      schema = updated;
      properties = notesDataSourceSchemaDraftFromDto(updated.data_source, updated.view);
      dirty = false;
      saved = true;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      saving = false;
    }
  }

  function markDirty(next: NotesDataSourceSchemaPropertyDraft[]): void {
    properties = next;
    dirty = true;
    saved = false;
  }

  function updateProperty(
    propertyId: string,
    patch: Partial<NotesDataSourceSchemaPropertyDraft>,
  ): void {
    markDirty(properties.map((property) => {
      if (property.id !== propertyId) return property;
      const nextType = patch.type ?? property.type;
      const next: NotesDataSourceSchemaPropertyDraft = {
        ...property,
        ...patch,
        type: nextType,
      };
      if (nextType === "status" && next.options.length === 0) {
        next.options = createNotesDataSourcePropertyDraft("status", "Status").options;
      }
      if (nextType !== "select" && nextType !== "multi_select" && nextType !== "status") {
        next.options = [];
      }
      if (nextType === "title") next.hidden = false;
      return next;
    }));
  }

  function addProperty(): void {
    const name = defaultNotesDataSourcePropertyName(newPropertyType, properties);
    const property = createNotesDataSourcePropertyDraft(newPropertyType, name);
    markDirty([...properties, property]);
  }

  function deleteProperty(propertyId: string): void {
    markDirty(properties.filter((property) => property.id !== propertyId || property.type === "title"));
  }

  function moveProperty(propertyId: string, direction: -1 | 1): void {
    const index = properties.findIndex((property) => property.id === propertyId);
    if (index < 0) return;
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= properties.length) return;
    const next = [...properties];
    const [property] = next.splice(index, 1);
    if (!property) return;
    next.splice(nextIndex, 0, property);
    markDirty(next);
  }

  function addOption(propertyId: string): void {
    const property = properties.find((item) => item.id === propertyId);
    if (!property) return;
    const option: NotesDataSourceSchemaOptionDraft = {
      id: crypto.randomUUID(),
      name: `Option ${property.options.length + 1}`,
      color: "default",
      group: "To-do",
    };
    updateProperty(propertyId, { options: [...property.options, option] });
  }

  function updateOption(
    propertyId: string,
    optionId: string,
    patch: Partial<NotesDataSourceSchemaOptionDraft>,
  ): void {
    const property = properties.find((item) => item.id === propertyId);
    if (!property) return;
    updateProperty(propertyId, {
      options: property.options.map((option) =>
        option.id === optionId ? { ...option, ...patch } : option,
      ),
    });
  }

  function deleteOption(propertyId: string, optionId: string): void {
    const property = properties.find((item) => item.id === propertyId);
    if (!property) return;
    updateProperty(propertyId, {
      options: property.options.filter((option) => option.id !== optionId),
    });
  }

  function propertyTypeLabel(type: NotesDataSourcePropertyType): string {
    switch (type) {
      case "title":
        return t("notes.databaseSchemaPropertyType.title");
      case "rich_text":
        return t("notes.databaseSchemaPropertyType.richText");
      case "number":
        return t("notes.databaseSchemaPropertyType.number");
      case "select":
        return t("notes.databaseSchemaPropertyType.select");
      case "multi_select":
        return t("notes.databaseSchemaPropertyType.multiSelect");
      case "status":
        return t("notes.databaseSchemaPropertyType.status");
      case "date":
        return t("notes.databaseSchemaPropertyType.date");
      case "checkbox":
        return t("notes.databaseSchemaPropertyType.checkbox");
      case "url":
        return t("notes.databaseSchemaPropertyType.url");
      case "email":
        return t("notes.databaseSchemaPropertyType.email");
      case "phone_number":
        return t("notes.databaseSchemaPropertyType.phoneNumber");
      case "files":
        return t("notes.databaseSchemaPropertyType.files");
      case "people":
        return t("notes.databaseSchemaPropertyType.people");
      case "created_time":
        return t("notes.databaseSchemaPropertyType.createdTime");
      case "created_by":
        return t("notes.databaseSchemaPropertyType.createdBy");
      case "last_edited_time":
        return t("notes.databaseSchemaPropertyType.lastEditedTime");
      case "last_edited_by":
        return t("notes.databaseSchemaPropertyType.lastEditedBy");
      case "unique_id":
        return t("notes.databaseSchemaPropertyType.uniqueId");
      case "place":
        return t("notes.databaseSchemaPropertyType.place");
    }
  }

  function statusMessage(): string {
    if (loading) return t("notes.databaseSchemaLoading");
    if (saving) return t("notes.databaseSchemaSave");
    if (dirty) return t("notes.databaseSchemaUnsaved");
    if (saved) return t("notes.databaseSchemaSaved");
    return t("notes.databaseSchemaPropertyCount", propertyCount);
  }
</script>

<section
  class="my-1 min-w-0 rounded-md border border-border bg-background/70 p-2"
  aria-label={t("notes.blockType.childDatabase")}
>
  <div class="flex min-w-0 items-start gap-2">
    <div
      class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
      aria-hidden="true"
    >
      <Database class="size-4" />
    </div>
    <button
      bind:this={button}
      type="button"
      class="flex min-h-8 min-w-0 flex-1 flex-col gap-0.5 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onkeydown={onKeydown}
      onclick={() => onFocusBlock(block.id)}
    >
      <span class="min-w-0 truncate text-[0.866667rem] font-medium text-foreground">
        {title || t("notes.untitled")}
      </span>
      <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground">
        {localDatabase ? t("notes.childDatabaseLocal") : t("notes.childDatabasePreserved")}
      </span>
    </button>
    {#if localDatabase}
      <button
        type="button"
        class="inline-flex h-8 shrink-0 items-center gap-1 rounded-md px-2 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-expanded={expanded}
        onclick={() => {
          expanded = !expanded;
        }}
      >
        {t("notes.databaseSchemaToggle")}
      </button>
    {/if}
  </div>

  {#if localDatabase && expanded}
    <div class="mt-3 min-w-0 space-y-3 border-t border-border pt-3">
      <div class="flex min-w-0 flex-wrap items-center gap-2 text-[0.8rem] text-muted-foreground">
        <span class="min-w-0 flex-1 truncate" role="status">
          {#if error}
            {schema
              ? t("notes.databaseSchemaSaveFailed", error)
              : t("notes.databaseSchemaLoadFailed", error)}
          {:else}
            {statusMessage()}
          {/if}
        </span>
        <button
          type="button"
          class="inline-flex h-8 items-center gap-1 rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
          disabled={loading || saving}
          aria-label={t("notes.databaseSchemaReload")}
          title={t("notes.databaseSchemaReload")}
          onclick={() => {
            void loadSchema();
          }}
        >
          <RefreshCw class="size-3.5" aria-hidden="true" />
        </button>
        <button
          type="button"
          class="inline-flex h-8 items-center gap-1 rounded-md bg-primary px-2 text-primary-foreground disabled:pointer-events-none disabled:opacity-50"
          disabled={loading || saving || !dirty}
          onclick={() => {
            void saveSchema();
          }}
        >
          <Save class="size-3.5" aria-hidden="true" />
          <span>{t("notes.databaseSchemaSave")}</span>
        </button>
      </div>

      <div class="space-y-2">
        {#each properties as property, index (property.id)}
          <div class="grid min-w-0 gap-2 rounded-md border border-border p-2 @container">
            <div class="grid min-w-0 gap-2 @lg:grid-cols-[minmax(7rem,1fr)_minmax(7rem,12rem)_auto]">
              <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                <span class="mb-1 block">{t("notes.databaseSchemaName")}</span>
                <input
                  class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-70"
                  value={property.name}
                  disabled={saving}
                  oninput={(event) => {
                    updateProperty(property.id, {
                      name: event.currentTarget.value,
                    });
                  }}
                />
              </label>
              <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                <span class="mb-1 block">{t("notes.databaseSchemaType")}</span>
                <select
                  class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-70"
                  value={property.type}
                  disabled={saving || property.type === "title"}
                  onchange={(event) => {
                    updateProperty(property.id, {
                      type: event.currentTarget.value as NotesDataSourcePropertyType,
                    });
                  }}
                >
                  {#each NOTES_DATA_SOURCE_PROPERTY_TYPES as type}
                    <option value={type}>{propertyTypeLabel(type)}</option>
                  {/each}
                </select>
              </label>
              <div class="flex min-w-0 items-end justify-end gap-1">
                <button
                  type="button"
                  class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                  disabled={saving || index === 0}
                  aria-label={t("notes.databaseSchemaMoveUp")}
                  title={t("notes.databaseSchemaMoveUp")}
                  onclick={() => moveProperty(property.id, -1)}
                >
                  <ArrowUp class="size-3.5" aria-hidden="true" />
                </button>
                <button
                  type="button"
                  class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                  disabled={saving || index === properties.length - 1}
                  aria-label={t("notes.databaseSchemaMoveDown")}
                  title={t("notes.databaseSchemaMoveDown")}
                  onclick={() => moveProperty(property.id, 1)}
                >
                  <ArrowDown class="size-3.5" aria-hidden="true" />
                </button>
                <button
                  type="button"
                  class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                  disabled={saving || property.type === "title"}
                  aria-label={property.hidden ? t("notes.databaseSchemaShow") : t("notes.databaseSchemaHide")}
                  title={property.hidden ? t("notes.databaseSchemaShow") : t("notes.databaseSchemaHide")}
                  onclick={() => updateProperty(property.id, { hidden: !property.hidden })}
                >
                  {#if property.hidden}
                    <EyeOff class="size-3.5" aria-hidden="true" />
                  {:else}
                    <Eye class="size-3.5" aria-hidden="true" />
                  {/if}
                </button>
                <button
                  type="button"
                  class="inline-flex size-8 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-40"
                  disabled={saving || property.type === "title"}
                  aria-label={t("notes.databaseSchemaDelete")}
                  title={t("notes.databaseSchemaDelete")}
                  onclick={() => deleteProperty(property.id)}
                >
                  <Trash2 class="size-3.5" aria-hidden="true" />
                </button>
              </div>
            </div>

            <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
              <span class="mb-1 block">{t("notes.databaseSchemaDescription")}</span>
              <input
                class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-70"
                value={property.description}
                placeholder={t("notes.databaseSchemaDescriptionPlaceholder")}
                disabled={saving}
                oninput={(event) => {
                  updateProperty(property.id, {
                    description: event.currentTarget.value,
                  });
                }}
              />
            </label>

            {#if property.type === "number"}
              <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                <span class="mb-1 block">{t("notes.databaseSchemaNumberFormat")}</span>
                <select
                  class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={property.numberFormat}
                  onchange={(event) => {
                    updateProperty(property.id, {
                      numberFormat: event.currentTarget.value as NotesDataSourceNumberFormat,
                    });
                  }}
                >
                  {#each NOTES_DATA_SOURCE_NUMBER_FORMATS as format}
                    <option value={format}>{format}</option>
                  {/each}
                </select>
              </label>
            {:else if property.type === "unique_id"}
              <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                <span class="mb-1 block">{t("notes.databaseSchemaUniquePrefix")}</span>
                <input
                  class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={property.uniquePrefix}
                  placeholder={t("notes.databaseSchemaUniquePrefixPlaceholder")}
                  oninput={(event) => {
                    updateProperty(property.id, {
                      uniquePrefix: event.currentTarget.value,
                    });
                  }}
                />
              </label>
            {:else if property.type === "select" || property.type === "multi_select" || property.type === "status"}
              <div class="space-y-2">
                {#each property.options as option (option.id)}
                  <div class="grid min-w-0 gap-2 @lg:grid-cols-[minmax(7rem,1fr)_minmax(7rem,10rem)_minmax(7rem,10rem)_auto]">
                    <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                      <span class="mb-1 block">{t("notes.databaseSchemaOptionName")}</span>
                      <input
                        class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        value={option.name}
                        oninput={(event) => {
                          updateOption(property.id, option.id, {
                            name: event.currentTarget.value,
                          });
                        }}
                      />
                    </label>
                    <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                      <span class="mb-1 block">{t("notes.databaseSchemaOptionColor")}</span>
                      <select
                        class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        value={option.color}
                        onchange={(event) => {
                          updateOption(property.id, option.id, {
                            color: event.currentTarget.value as NotesDataSourceSelectColor,
                          });
                        }}
                      >
                        {#each NOTES_DATA_SOURCE_SELECT_COLORS as color}
                          <option value={color}>{color}</option>
                        {/each}
                      </select>
                    </label>
                    {#if property.type === "status"}
                      <label class="min-w-0 text-[0.733333rem] text-muted-foreground">
                        <span class="mb-1 block">{t("notes.databaseSchemaOptionGroup")}</span>
                        <select
                          class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                          value={option.group}
                          onchange={(event) => {
                            updateOption(property.id, option.id, {
                              group: event.currentTarget.value as NotesDataSourceStatusGroup,
                            });
                          }}
                        >
                          {#each NOTES_DATA_SOURCE_STATUS_GROUPS as group}
                            <option value={group}>{group}</option>
                          {/each}
                        </select>
                      </label>
                    {/if}
                    <div class="flex items-end justify-end">
                      <button
                        type="button"
                        class="inline-flex size-8 items-center justify-center rounded-md text-destructive hover:bg-destructive/10"
                        aria-label={t("notes.databaseSchemaDeleteOption")}
                        title={t("notes.databaseSchemaDeleteOption")}
                        onclick={() => deleteOption(property.id, option.id)}
                      >
                        <Trash2 class="size-3.5" aria-hidden="true" />
                      </button>
                    </div>
                  </div>
                {/each}
                <button
                  type="button"
                  class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-[0.8rem] hover:bg-accent"
                  onclick={() => addOption(property.id)}
                >
                  <Plus class="size-3.5" aria-hidden="true" />
                  <span>{t("notes.databaseSchemaAddOption")}</span>
                </button>
              </div>
            {:else if property.type === "title"}
              <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseSchemaReadOnlyTitle")}</p>
            {:else}
              <p class="text-[0.8rem] text-muted-foreground">
                {t("notes.databaseSchemaReadonlyPlaceholder")}
              </p>
            {/if}
          </div>
        {/each}
      </div>

      <div class="flex min-w-0 flex-wrap items-center gap-2 border-t border-border pt-3">
        <select
          class="h-8 min-w-36 rounded-md border border-input bg-background px-2 text-[0.866667rem] outline-none focus-visible:ring-2 focus-visible:ring-ring"
          value={newPropertyType}
          onchange={(event) => {
            newPropertyType = event.currentTarget.value as NotesDataSourcePropertyType;
          }}
        >
          {#each NOTES_DATA_SOURCE_PROPERTY_TYPES.filter((type) => type !== "title") as type}
            <option value={type}>{propertyTypeLabel(type)}</option>
          {/each}
        </select>
        <button
          type="button"
          class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-[0.8rem] hover:bg-accent"
          onclick={addProperty}
        >
          <Plus class="size-3.5" aria-hidden="true" />
          <span>{t("notes.databaseSchemaAddProperty")}</span>
        </button>
      </div>

      {#if dataSourceId}
        <NotesDataSourceRows {dataSourceId} {onSelectPage} />
      {/if}
    </div>
  {/if}
</section>
