import type { mobile as enMobile } from "../en/mobile";
import type { MessageShape } from "../types";

export const mobile = {
  primaryNavigation: "Navegación principal",
  createEvent: "Crear evento",
  theme: "Tema",
  startupFailed: "Ganbaru AI no pudo abrir sus datos móviles privados",
  pomodoroInactiveDescription:
    "Pomodoro comienza automáticamente cuando inicia un bloque de calendario con Pomodoro habilitado.",
  pomodoroOpenCalendar: "Abrir calendario",
  vaultSetup: {
    title: "Configura tus datos",
    intro:
      "Empieza con datos nuevos, restaura un archivo .ganbaru-backup o importa una carpeta de Ganbaru AI existente.",
    dataLocation: "Ubicación de datos",
    startFromZero: "Empezar de cero",
    restoreBackupFile: "Restaurar archivo de copia",
    importExistingFolder: "Importar carpeta existente",
  },
  focusOnboarding: {
    title: "Mantén Enfoque en funcionamiento",
    description:
      "Revisa estos ajustes de Android para iniciar eventos de Enfoque programados con Ganbaru AI cerrado.",
    notifications: "Notificaciones",
    notificationsDescription: "Muestra el progreso de Enfoque y las alertas de fase",
    exactAlarm: "Alarmas y recordatorios",
    exactAlarmDescription: "Inicia los eventos de Enfoque programados a la hora correcta",
    backgroundRestricted: "Android actualmente restringe Ganbaru AI en segundo plano",
    review: "Revisar",
    continue: "Continuar",
    continueIn: (seconds: number) => `Continuar en ${seconds} s`,
    statusError: "No se pudo consultar parte del acceso de Android. Aún puedes revisar cada ajuste.",
  },
  settings: {
    categoriesLabel: "Categorías de ajustes",
    backToCategories: "Volver a las categorías de ajustes",
    storageHeading: "Almacenamiento",
    privateStorage: "Privado",
    privateStorageDescription:
      "Android conserva estos datos en el almacenamiento privado de Ganbaru AI. Desinstalar la app los elimina.",
    androidUpdatesDescription: "Android instala actualizaciones desde el origen de distribución de la app",
    viewReleases: "Ver",
    doomscrollingHeading: "Doomscrolling para móvil está planeado",
    doomscrollingDescription:
      "Los controles de sitios web y procesos de escritorio no aplican en Android. La medición de uso y el bloqueo de apps necesitan su propio flujo explícito de acceso al sistema.",
  },
} as const satisfies MessageShape<typeof enMobile>;
