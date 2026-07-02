import type { common as enCommon } from "../en/common";
import type { MessageShape } from "../types";

export const common = {
  appName: "Ganbaru AI",
  cancel: "Cancelar",
  close: "Cerrar",
  confirm: "Confirmar",
  save: "Guardar",
  reset: "Restablecer",
  delete: "Eliminar",
  archive: "Archivar",
  edit: "Editar",
  done: "Listo",
  loading: "Cargando",
  none: "Ninguno",
  disabled: "Desactivado",
  enabled: "Activado",
  system: "Sistema",
  english: "Inglés",
  spanish: "Español",
  yesShortcut: "Sí (Enter)",
  noShortcut: "No (Esc)",
  cancelShortcut: "Cancelar (Esc)",
  minimize: "Minimizar",
  maximize: "Maximizar",
  restore: "Restaurar",
} as const satisfies MessageShape<typeof enCommon>;
