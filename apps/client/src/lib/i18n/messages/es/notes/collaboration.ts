import type { collaboration as enCollaboration } from "../../en/notes/collaboration";
import type { MessageShape } from "../../types";

export const collaboration = {
  commentsCount: (count: number) => `${count} ${count === 1 ? "comentario" : "comentarios"}`,
  commentsCountWithUnread: (count: number, unread: number) =>
    `${count} ${count === 1 ? "comentario" : "comentarios"}, ${unread} sin leer`,
  loadingComments: "Cargando comentarios",
  loadCommentsFailed: (message: string) => `No se pudieron cargar los comentarios: ${message}`,
  noComments: "No hay comentarios",
  pageDiscussion: "Discusión de página",
  blockComment: "Comentario de bloque",
  blockCommentOn: (text: string) => `Bloque: ${text}`,
  inlineCommentOn: (text: string) => `Texto: ${text}`,
  inlineCommentAnchorMissing: "No se encontró el texto anclado en el bloque actual",
  showResolvedComments: "Mostrar resueltos",
  commentPlaceholder: "Agregar un comentario...",
  replyPlaceholder: "Responder...",
  commentInput: "Texto del comentario",
  replyCommentInput: "Texto de respuesta",
  editCommentInput: "Editar texto del comentario",
  addComment: "Agregar comentario",
  reply: "Responder",
  commentBlock: "Comentar",
  unreadCommentThread: "Sin leer",
  unreadCommentShortCount: (count: number) => `${count} sin leer`,
  blockCommentsCount: (count: number) =>
    `${count} ${count === 1 ? "comentario de bloque" : "comentarios de bloque"}`,
  blockUnreadCommentsCount: (unread: number, count: number) =>
    `${unread} sin leer de ${count} ${count === 1 ? "comentario de bloque" : "comentarios de bloque"
    }`,
  resolveCommentThread: "Resolver",
  reopenCommentThread: "Reabrir",
  suggestionsCount: (count: number) =>
    `${count} ${count === 1 ? "sugerencia" : "sugerencias"}`,
  loadingSuggestions: "Cargando sugerencias",
  loadSuggestionsFailed: (message: string) =>
    `No se pudieron cargar las sugerencias: ${message}`,
  noSuggestions: "No hay sugerencias",
  suggestedEdits: "Ediciones sugeridas",
  showDecidedSuggestions: "Mostrar decididas",
  newSuggestion: "Nueva sugerencia",
  originalText: "Original",
  proposedText: "Propuesto",
  createSuggestion: "Crear sugerencia",
  acceptSuggestion: "Aceptar",
  rejectSuggestion: "Rechazar",
  suggestionStatusOpen: "Abierta",
  suggestionStatusAccepted: "Aceptada",
  suggestionStatusRejected: "Rechazada",
  suggestionTargetMissing: "No se encontró el texto sugerido en el bloque actual"
} as const satisfies MessageShape<typeof enCollaboration>;
