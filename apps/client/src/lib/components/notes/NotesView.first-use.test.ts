// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

const backend = vi.hoisted(() => {
  const promise = new Promise<never>(() => undefined);
  let notesCalls = 0;
  let projectsCalls = 0;
  return {
    promise,
    get notesCalls() {
      return notesCalls;
    },
    get projectsCalls() {
      return projectsCalls;
    },
    recordNotesCall() {
      notesCalls += 1;
    },
    recordProjectsCall() {
      projectsCalls += 1;
    },
  };
});

vi.mock("@tauri-apps/api/window", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/window")>();
  return {
    ...actual,
    getCurrentWindow: () => ({ label: "main" }),
  };
});

vi.mock("$lib/api/notes", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/notes")>();
  const unresolved = () => {
    backend.recordNotesCall();
    return backend.promise;
  };
  return {
    ...actual,
    listNotesSidebarPages: unresolved,
    listNotesPages: unresolved,
    listNotesFolders: unresolved,
    listNotesPageTemplates: unresolved,
    getNotesLocalUser: unresolved,
  };
});

vi.mock("$lib/api/projects", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/projects")>();
  return {
    ...actual,
    loadProjectsSnapshot: () => {
      backend.recordProjectsCall();
      return backend.promise;
    },
  };
});

vi.mock("$lib/api/notes-project-history", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/notes-project-history")>();
  return {
    ...actual,
    flushDueNotesProjectHistory: () => Promise.resolve(),
    initializeNotesProjectHistory: () => Promise.resolve(),
  };
});

describe("NotesView first use", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    vi.unstubAllGlobals();
    component = undefined;
    target = undefined;
  });

  it("renders its workspace shell while initial backend promises are unresolved", async () => {
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    });
    target = document.createElement("div");
    document.body.append(target);
    const { default: NotesView } = await import("./NotesView.svelte");

    component = mount(NotesView, { target });
    await tick();

    expect(backend.notesCalls).toBe(3);
    expect(backend.projectsCalls).toBe(1);
    expect(target.querySelector('[data-first-use-shell="notes"]')).not.toBeNull();
    expect(target.querySelector("[data-notes-workspace-header]")).not.toBeNull();
    expect(target.querySelector("[data-notes-first-use-state]")?.getAttribute("aria-busy"))
      .toBe("true");
  }, 15_000);
});
