import type {
  focusDialog as enFocusDialog,
  pomodoroNotification as enPomodoroNotification,
  pomodoroOverlay as enPomodoroOverlay,
} from "../en/focus";
import type { MessageShape } from "../types";

export const focusDialog = {
  stopTitle: "¿Detener la sesión de enfoque?",
  stopMessage: "Todas las funciones de enfoque se detendrán",
  stopSession: "Detener sesión",
  undoChanges: "Deshacer cambios",
  resumeTitle: "¿Reanudar sesión de enfoque?",
  awayMessage: (duration: string) => `Estuviste ausente durante ${duration}`,
  resume: "Reanudar",
  stopSessionCancel: "Detener sesión",
  awayHoursMinutes: (hours: number, minutes: number) => `${hours} h ${minutes} min`,
  awayHours: (hours: number) => `${hours} h`,
  awayMinutes: (minutes: number) => `${minutes} min`,
  awaySeconds: (seconds: number) => `${seconds} s`,
} as const satisfies MessageShape<typeof enFocusDialog>;

export const pomodoroOverlay = {
  focusPausedTitle: "Sesión de enfoque pausada",
  focusPausedBody: "No se detectó actividad. Regresa para reanudar tu sesión.",
  focusFailedTitle: "La sesión de enfoque falló",
  breakCompleteTitle: "Descanso completado",
  eventFinishedTitle: "Evento terminado",
  dayCompletedTitle: "Día completado",
  workweekCompletedTitle: "Semana laboral completada",
  pressAnyKey: "pulsa cualquier tecla para continuar",
  resumeFocus: "reanudar enfoque",
  restartFocus: "reiniciar enfoque",
  extendBreak: "extender el descanso",
  endBreakNow: "terminar tu descanso ahora",
  press: "Pulsa",
  to: "para",
  keyHint: (key: string, label: string) => `Pulsa ${key} para ${label}`,
  extendBreakHint: (shortcut: string) => `Pulsa ${shortcut} para extender el descanso`,
  endBreakHint: (shortcut: string) => `Pulsa ${shortcut} para terminar tu descanso ahora`,
} as const satisfies MessageShape<typeof enPomodoroOverlay>;

export const pomodoroNotification = {
  channelName: "Sesiones de enfoque",
  channelDescription: "Progreso persistente de las sesiones de enfoque activas",
  alertsChannelName: "Alertas de enfoque",
  alertsChannelDescription: "Alertas cuando terminan las fases de enfoque y descanso",
  focusTitle: "Enfoque",
  shortBreakTitle: "Descanso corto",
  longBreakTitle: "Descanso largo",
  pausedText: "En pausa",
  focusCompleteTitle: "Enfoque completado",
  breakCompleteTitle: "Descanso completado",
  sessionCompleteText: "Sesión completada",
} as const satisfies MessageShape<typeof enPomodoroNotification>;
