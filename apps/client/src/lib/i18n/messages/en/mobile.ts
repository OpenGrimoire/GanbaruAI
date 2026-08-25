export const mobile = {
  primaryNavigation: "Primary navigation",
  createEvent: "Create event",
  theme: "Theme",
  startupFailed: "Ganbaru AI could not open its private mobile data",
  privateDataHeading: "Private mobile storage",
  privateDataDescription:
    "Ganbaru AI keeps the live mobile database in the app's private storage. Theme imports use Android's document picker and exports go to Downloads. Backup and remaining file transfers will use similarly scoped system boundaries.",
  pomodoroForegroundOnly:
    "Background timer alerts are not available in this Android foundation yet. Keep Ganbaru AI open while using Pomodoro.",
  pomodoroInactiveDescription:
    "Pomodoro starts automatically when a Pomodoro-enabled calendar block begins.",
  pomodoroOpenCalendar: "Open calendar",
  settings: {
    categoriesLabel: "Settings categories",
    backToCategories: "Back to settings categories",
    projectsHeading: "Project settings stay with each project",
    projectsDescription:
      "Open a project and use its settings panel for workflow, defaults, statuses, priorities, and fields.",
    chatHeading: "Local coding agents require a desktop",
    chatDescription:
      "This device cannot launch Codex, Claude, Cursor, OpenCode, or other local coding-agent processes.",
    chatDetail:
      "Mobile channel communication will appear here after the encrypted sync and remote communication boundary exists.",
    musicHeading: "Android media support is not connected yet",
    musicDescription:
      "Music settings will use the shared library and preferences after native Android playback and media controls are implemented.",
    doomscrollingHeading: "Mobile Doomscrolling is planned",
    doomscrollingDescription:
      "Website and desktop-process controls do not apply to Android. Mobile usage awareness and app enforcement need their own explicit system access flow.",
    dataDetail:
      "Uninstalling the app can remove this private data. Theme imports use Android's document picker and exports go to Downloads. Backup, restore, and remaining transfers will use similarly scoped system boundaries.",
    updatesHeading: "Updates follow the Android distribution channel",
    updatesDescription:
      "Production builds will update through the installed store or the documented signed-APK channel. The desktop updater does not run on Android.",
    calendarTransfersUnavailable:
      "Calendar file import and export need the Android document picker. Calendar data and deletion remain available.",
    notesTransfersUnavailable:
      "Notes file import and export need the Android document picker. Notes preferences and version-history retention remain available.",
    notesNotificationsUnavailable:
      "Notes notification delivery needs the native Android notification adapter. These controls will appear when delivery can work reliably.",
    focusHeading: "Foreground Pomodoro is available",
    focusNativeFeaturesUnavailable:
      "Idle detection and background boundary notifications need native Android adapters. Foreground Pomodoro recovery remains available.",
  },
} as const;
