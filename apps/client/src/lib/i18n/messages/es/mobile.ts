import type { mobile as enMobile } from "../en/mobile";
import type { MessageShape } from "../types";

export const mobile = {
  primaryNavigation: "Navegación principal",
  createEvent: "Crear evento",
  theme: "Tema",
  startupFailed: "Ganbaru AI no pudo abrir sus datos móviles privados",
  privateDataHeading: "Almacenamiento móvil privado",
  privateDataDescription:
    "Ganbaru AI guarda la base de datos móvil activa en el almacenamiento privado de la app. Las importaciones de temas usan el selector de documentos de Android y las exportaciones van a Descargas. Las copias de seguridad y transferencias restantes usarán límites del sistema con un alcance similar.",
  pomodoroForegroundOnly:
    "Las alertas del temporizador en segundo plano aún no están disponibles en esta base para Android. Mantén Ganbaru AI abierto mientras usas Pomodoro.",
  pomodoroInactiveDescription:
    "Pomodoro comienza automáticamente cuando inicia un bloque de calendario con Pomodoro habilitado.",
  pomodoroOpenCalendar: "Abrir calendario",
  settings: {
    categoriesLabel: "Categorías de ajustes",
    backToCategories: "Volver a las categorías de ajustes",
    projectsHeading: "Los ajustes de proyecto están dentro de cada proyecto",
    projectsDescription:
      "Abre un proyecto y usa su panel de ajustes para el flujo, valores predeterminados, estados, prioridades y campos.",
    chatHeading: "Los agentes de programación locales requieren una computadora",
    chatDescription:
      "Este dispositivo no puede iniciar procesos locales de Codex, Claude, Cursor, OpenCode u otros agentes de programación.",
    chatDetail:
      "La comunicación móvil de canales aparecerá aquí cuando existan la sincronización cifrada y el límite de comunicación remota.",
    musicHeading: "El soporte multimedia de Android aún no está conectado",
    musicDescription:
      "Los ajustes de música usarán la biblioteca y preferencias compartidas cuando se implementen la reproducción y los controles multimedia nativos de Android.",
    doomscrollingHeading: "Doomscrolling para móvil está planeado",
    doomscrollingDescription:
      "Los controles de sitios web y procesos de escritorio no aplican en Android. La medición de uso y el bloqueo de apps necesitan su propio flujo explícito de acceso al sistema.",
    dataDetail:
      "Desinstalar la app puede eliminar estos datos privados. Las importaciones de temas usan el selector de documentos de Android y las exportaciones van a Descargas. Las copias de seguridad, restauraciones y transferencias restantes usarán límites del sistema con un alcance similar.",
    updatesHeading: "Las actualizaciones siguen el canal de distribución de Android",
    updatesDescription:
      "Las compilaciones de producción se actualizarán mediante la tienda instalada o el canal documentado de APK firmado. El actualizador de escritorio no se ejecuta en Android.",
    calendarTransfersUnavailable:
      "La importación y exportación de archivos de calendario necesitan el selector de documentos de Android. Los datos y la eliminación de calendarios siguen disponibles.",
    notesTransfersUnavailable:
      "La importación y exportación de archivos de notas necesitan el selector de documentos de Android. Las preferencias y retención del historial siguen disponibles.",
    notesNotificationsUnavailable:
      "La entrega de notificaciones de notas necesita el adaptador nativo de Android. Estos controles aparecerán cuando la entrega pueda funcionar de forma confiable.",
    focusHeading: "Pomodoro está disponible en primer plano",
    focusNativeFeaturesUnavailable:
      "La detección de inactividad y las notificaciones de límites en segundo plano necesitan adaptadores nativos de Android. La recuperación de Pomodoro en primer plano sigue disponible.",
  },
} as const satisfies MessageShape<typeof enMobile>;
