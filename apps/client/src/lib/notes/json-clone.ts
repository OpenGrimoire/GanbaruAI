/** Clone a Notes DTO through JSON so reactive proxies are detached before IPC and history storage. */
export function cloneNotesJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
