export class MusicFocusRecovery {
  private rememberedKey: string | null = null;

  remember(element: Element | null): void {
    this.rememberedKey = element instanceof HTMLElement
      ? element.dataset.musicFocusKey ?? null
      : null;
  }

  rememberKey(key: string | null): void {
    this.rememberedKey = key;
  }

  async restore(root: ParentNode = document): Promise<boolean> {
    const key = this.rememberedKey;
    if (!key) return false;
    await Promise.resolve();
    const escaped = typeof CSS !== "undefined" && typeof CSS.escape === "function"
      ? CSS.escape(key)
      : key.replace(/["\\]/g, "\\$&");
    const target = root.querySelector<HTMLElement>(`[data-music-focus-key="${escaped}"]`);
    if (!target) return false;
    target.focus({ preventScroll: true });
    return document.activeElement === target;
  }

  clear(): void {
    this.rememberedKey = null;
  }
}

/** Restores focus to a stable builder element after a responsive surface closes. */
export async function restoreMusicFocus(
  key: string,
  root: ParentNode = document,
): Promise<boolean> {
  const recovery = new MusicFocusRecovery();
  recovery.rememberKey(key);
  return recovery.restore(root);
}
