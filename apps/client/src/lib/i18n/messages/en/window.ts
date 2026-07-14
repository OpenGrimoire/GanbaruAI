export const window = {
  close: "Close",
  closeWindowWithShortcut: (shortcut: string) => `Close window (${shortcut})`,
  closeAppWithShortcut: (shortcut: string) => `Close app (${shortcut})`,
} as const;
