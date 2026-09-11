import type { CalendarEvent } from "./types";
import type { EditSessionState, PanelAnchor } from "./edit-session.svelte";
import { PENDING_CREATE_ID } from "./display-events";

interface CalendarDragSession {
  readonly state: EditSessionState;
  readonly dirty: boolean;
  openEdit(
    event: CalendarEvent,
    anchor: PanelAnchor,
    fullEvent?: CalendarEvent,
    detailsLoaded?: boolean,
  ): void;
  updateChanges(changes: Partial<CalendarEvent>): void;
}

export interface CalendarViewDragControllerOptions {
  session: CalendarDragSession;
  isCommitHidden: () => boolean;
  editingId: () => string | undefined;
  visibleEvents: () => CalendarEvent[];
  isRecurring: (event: CalendarEvent) => boolean;
  isActivePomodoroEvent: (event: CalendarEvent) => boolean;
  panelAnchor: (eventId: string) => PanelAnchor;
  loadPanel: () => Promise<void>;
  confirmDiscard: (action: () => Promise<void>) => void;
  getTemplate: (event: CalendarEvent) => CalendarEvent | undefined;
  updateBlock: (event: CalendarEvent) => Promise<void>;
  now?: () => number;
}

/** Coordinates calendar drag and resize commits with open edit sessions. */
export class CalendarViewDragController {
  lastDragEndTime = 0;

  constructor(private readonly options: CalendarViewDragControllerOptions) {}

  markInteractionEnd(): void {
    this.lastDragEndTime = (this.options.now ?? Date.now)();
  }

  async handle(event: CalendarEvent): Promise<void> {
    if (this.options.isCommitHidden()) return;
    this.markInteractionEnd();

    if (event.id === PENDING_CREATE_ID) {
      if (this.options.session.state.mode === "create") this.applyTimes(event);
      return;
    }

    const state = this.options.session.state;
    if (state.mode === "edit"
      && (state.originalEvent.id === event.id || this.options.editingId() === event.id)) {
      this.applyTimes(event);
      return;
    }

    if (this.options.isActivePomodoroEvent(event) || this.options.isRecurring(event)) {
      const original = this.options.visibleEvents().find((candidate) => candidate.id === event.id);
      if (!original) return;
      const open = async () => {
        await this.options.loadPanel();
        this.options.session.openEdit(original, this.options.panelAnchor(event.id), original);
        this.applyTimes(event);
      };
      if (this.options.session.dirty) {
        this.options.confirmDiscard(open);
        return;
      }
      await open();
      return;
    }

    const template = this.options.getTemplate(event);
    if (template && template.start === event.start && template.end === event.end) return;
    await this.options.updateBlock(event);
  }

  private applyTimes(event: CalendarEvent): void {
    this.options.session.updateChanges({ start: event.start, end: event.end });
  }
}
