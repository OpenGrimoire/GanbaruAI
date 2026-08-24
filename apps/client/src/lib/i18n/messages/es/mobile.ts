import type { mobile as enMobile } from "../en/mobile";
import type { MessageShape } from "../types";

export const mobile = {
  primaryNavigation: "Navegación principal",
  createEvent: "Crear evento",
  theme: "Tema",
  startupFailed: "Ganbaru AI no pudo abrir sus datos móviles privados",
  privateDataHeading: "Almacenamiento móvil privado",
  privateDataDescription:
    "Ganbaru AI guarda la base de datos móvil activa en el almacenamiento privado de la app. La importación, exportación y copia de seguridad usarán el selector de documentos de Android cuando ese puente esté disponible.",
  pomodoroForegroundOnly:
    "Las alertas del temporizador en segundo plano aún no están disponibles en esta base para Android. Mantén Ganbaru AI abierto mientras usas Pomodoro.",
  pomodoroInactiveDescription:
    "Pomodoro comienza automáticamente cuando inicia un bloque de calendario con Pomodoro habilitado.",
  pomodoroOpenCalendar: "Abrir calendario",
} as const satisfies MessageShape<typeof enMobile>;
