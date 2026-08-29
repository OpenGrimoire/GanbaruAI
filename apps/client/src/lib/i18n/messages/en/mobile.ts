export const mobile = {
  primaryNavigation: "Primary navigation",
  createEvent: "Create event",
  theme: "Theme",
  startupFailed: "Ganbaru AI could not open its private mobile data",
  privateDataHeading: "Private mobile storage",
  privateDataDescription:
    "Ganbaru AI keeps the live mobile database in the app's private storage. Theme imports use Android's document picker and exports go to Downloads. Backup and remaining file transfers will use similarly scoped system boundaries.",
  pomodoroInactiveDescription:
    "Pomodoro starts automatically when a Pomodoro-enabled calendar block begins.",
  pomodoroOpenCalendar: "Open calendar",
  focusOnboarding: {
    title: "Keep Focus running",
    description:
      "Review these Android settings so scheduled Focus events can start while Ganbaru AI is closed.",
    notifications: "Notifications",
    notificationsDescription: "Show ongoing Focus progress and phase alerts",
    exactAlarm: "Alarms and reminders",
    exactAlarmDescription: "Start scheduled Focus events at the correct time",
    backgroundRestricted: "Android currently restricts Ganbaru AI in the background",
    review: "Review",
    continue: "Continue",
    continueIn: (seconds: number) => `Continue in ${seconds}s`,
    statusError: "Some Android access status could not be read. You can still review each setting.",
  },
  settings: {
    categoriesLabel: "Settings categories",
    backToCategories: "Back to settings categories",
    projectsHeading: "Project settings stay with each project",
    projectsDescription:
      "Open a project and use its settings panel for workflow, defaults, statuses, priorities, and fields.",
    chatHeading: "Chat communication is available on this device",
    chatDescription:
      "Channels, messages, reply threads, search, drafts, and scheduled messages use the same local Chat model as desktop.",
    chatDetail:
      "Android does not launch local coding-agent processes or workspace tools. Future remote execution will connect agents without replacing this Chat interface.",
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
  },
} as const;
