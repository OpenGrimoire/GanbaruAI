import { invoke } from "@tauri-apps/api/core";
import { onDestroy, untrack } from "svelte";
import { getLocalization } from "$lib/i18n/translator.svelte";
import { getTheme } from "$lib/stores/theme.svelte";

type ThemeStore = ReturnType<typeof getTheme>;
type Translator = ReturnType<typeof getLocalization>["t"];

export interface ThemeJsonControllerContext {
  store: ThemeStore;
  themeId: () => string;
  translate: Translator;
  reportError: (message: string, error: unknown) => void;
}

/** Own the editable JSON draft and asynchronous import/export feedback. */
export class ThemeJsonController {
  draft = $state("");
  dirty = $state(false);
  errors = $state<string[]>([]);
  notice = $state<string | undefined>(undefined);
  #noticeTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly context: ThemeJsonControllerContext) {
    this.draft = untrack(() => context.store.exportTheme(context.themeId()) ?? "");
    $effect(() => {
      const next = context.store.exportTheme(context.themeId()) ?? "";
      if (!this.dirty) this.draft = next;
    });
    onDestroy(() => {
      if (this.#noticeTimer) clearTimeout(this.#noticeTimer);
    });
  }

  #flash(message: string): void {
    this.notice = message;
    if (this.#noticeTimer) clearTimeout(this.#noticeTimer);
    this.#noticeTimer = setTimeout(() => { this.notice = undefined; }, 1_800);
  }

  copy = async (): Promise<void> => {
    try {
      await navigator.clipboard.writeText(this.draft);
      this.#flash(this.context.translate("settings.theme.editor.jsonCopied"));
    } catch (error) {
      this.context.reportError("clipboard write failed", error);
      this.#flash(this.context.translate("settings.theme.editor.jsonCopyFailed"));
    }
  };

  save = async (): Promise<void> => {
    try {
      const saved = await invoke<boolean>("vault_pick_and_write_theme_json", {
        defaultName: `${this.context.themeId()}.json`,
        contents: this.draft,
      });
      if (saved) this.#flash(this.context.translate("settings.theme.editor.jsonSaved"));
    } catch (error) {
      this.context.reportError("save dialog failed", error);
      this.#flash(this.context.translate("settings.theme.editor.jsonSaveFailed"));
    }
  };

  apply = async (): Promise<void> => {
    const result = await this.context.store.replaceTheme(
      this.context.themeId(),
      this.draft,
    );
    if (!result.ok) {
      this.errors = result.errors;
      return;
    }
    this.errors = [];
    this.dirty = false;
    this.#flash(this.context.translate("settings.theme.editor.jsonUpdated"));
  };

  reset = (): void => {
    this.draft = this.context.store.exportTheme(this.context.themeId()) ?? "";
    this.dirty = false;
    this.errors = [];
  };

  input = (event: Event): void => {
    this.draft = (event.currentTarget as HTMLTextAreaElement).value;
    this.dirty = true;
    this.errors = [];
  };
}
