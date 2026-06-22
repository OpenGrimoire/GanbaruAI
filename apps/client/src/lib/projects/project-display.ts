import { getEventColor } from "$lib/components/calendar/utils";
import type { EventColor } from "$lib/components/calendar/types";
import type { Translate } from "$lib/i18n/translator.svelte";
import type { Theme } from "$lib/stores/themes";
import type {
  ProjectCustomFieldType,
  ProjectLifecycleStatus,
  ProjectPriority,
  ProjectStatus,
  ProjectTask,
  ProjectTaskChangeEvent,
  ProjectTaskType,
} from "./types";

export function projectPriorityLabel(priority: ProjectPriority, t: Translate): string {
  if (priority === "low") return t("projects.priority.low");
  if (priority === "high") return t("projects.priority.high");
  if (priority === "urgent") return t("projects.priority.urgent");
  return t("projects.priority.normal");
}

export function projectLifecycleLabel(status: ProjectLifecycleStatus, t: Translate): string {
  if (status === "hidden") return t("projects.lifecycle.hidden");
  if (status === "archived") return t("projects.lifecycle.archived");
  return t("projects.lifecycle.active");
}

export function projectLifecycleBadgeClass(status: ProjectLifecycleStatus): string {
  if (status === "hidden") return "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300";
  if (status === "archived") return "border-muted-foreground/30 bg-muted/50 text-muted-foreground";
  return "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
}

export function projectStatusBadgeClass(status: ProjectStatus | undefined): string {
  if (status?.category === "done") return "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
  if (status?.category === "blocked") return "border-destructive/40 bg-destructive/10 text-destructive";
  if (status?.category === "active") return "border-sky-500/40 bg-sky-500/10 text-sky-700 dark:text-sky-300";
  return "border-border bg-muted/50 text-muted-foreground";
}

export function projectTaskArchivedBadgeClass(task: Pick<ProjectTask, "archivedAt">): string {
  return task.archivedAt
    ? "border-muted-foreground/30 bg-muted/50 text-muted-foreground"
    : "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
}

export function projectPriorityBadgeClass(priority: ProjectPriority): string {
  if (priority === "urgent") return "border-destructive/40 bg-destructive/10 text-destructive";
  if (priority === "high") return "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300";
  if (priority === "low") return "border-muted-foreground/20 bg-muted/30 text-muted-foreground";
  return "border-border bg-background/70 text-foreground";
}

export function projectLabelColorDotStyle(color: EventColor | undefined, theme: Theme): string {
  if (color === undefined) return "";
  return `background-color: ${getEventColor(color, theme).bg};`;
}

export function projectLabelColorSwatchClass(color: EventColor | undefined): string {
  return color === undefined ? "border-border bg-muted/50" : "border-transparent";
}

export function projectPersonInitials(name: string): string {
  const initials = name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => Array.from(part)[0] ?? "")
    .join("")
    .toLocaleUpperCase();
  return initials || "?";
}

export function projectTaskTypeLabel(taskType: ProjectTaskType, t: Translate): string {
  if (taskType === "milestone") return t("projects.taskType.milestone");
  if (taskType === "bug") return t("projects.taskType.bug");
  if (taskType === "habit") return t("projects.taskType.habit");
  return t("projects.taskType.task");
}

export function projectCustomFieldTypeLabel(fieldType: ProjectCustomFieldType, t: Translate): string {
  if (fieldType === "number") return t("projects.customFields.typeNumber");
  if (fieldType === "date") return t("projects.customFields.typeDate");
  if (fieldType === "select") return t("projects.customFields.typeSelect");
  if (fieldType === "multi_select") return t("projects.customFields.typeMultiSelect");
  if (fieldType === "checkbox") return t("projects.customFields.typeCheckbox");
  if (fieldType === "url") return t("projects.customFields.typeUrl");
  return t("projects.customFields.typeText");
}

export function projectTaskHistoryFieldLabel(fieldName: string, t: Translate): string {
  if (fieldName.startsWith("custom_field:")) return fieldName.slice("custom_field:".length);
  if (fieldName === "title") return t("projects.history.fields.title");
  if (fieldName === "description") return t("projects.history.fields.description");
  if (fieldName === "status") return t("projects.history.fields.status");
  if (fieldName === "section") return t("projects.history.fields.section");
  if (fieldName === "parent") return t("projects.history.fields.parent");
  if (fieldName === "priority") return t("projects.history.fields.priority");
  if (fieldName === "type") return t("projects.history.fields.type");
  if (fieldName === "estimate") return t("projects.history.fields.estimate");
  if (fieldName === "due_date") return t("projects.history.fields.dueDate");
  if (fieldName === "due_time") return t("projects.history.fields.dueTime");
  if (fieldName === "start_date") return t("projects.history.fields.startDate");
  if (fieldName === "start_time") return t("projects.history.fields.startTime");
  if (fieldName === "target_date") return t("projects.history.fields.targetDate");
  if (fieldName === "archived_at") return t("projects.history.fields.archiveState");
  if (fieldName === "blocker_reason") return t("projects.history.fields.blockerReason");
  if (fieldName === "milestone") return t("projects.history.fields.milestone");
  if (fieldName === "checklist") return t("projects.history.fields.checklist");
  if (fieldName === "event_id") return t("projects.history.fields.event");
  if (fieldName === "blocking_task_id") return t("projects.history.fields.dependency");
  return fieldName;
}

export function projectTaskHistoryEventLabel(event: ProjectTaskChangeEvent, t: Translate): string {
  const field = event.fieldName ? projectTaskHistoryFieldLabel(event.fieldName, t) : "";
  const oldValue = event.oldValue ?? t("projects.history.emptyValue");
  const newValue = event.newValue ?? t("projects.history.emptyValue");
  if (event.eventType === "created") return t("projects.history.created");
  if (event.eventType === "scheduled") return t("projects.history.scheduled");
  if (event.eventType === "event_unlinked") return t("projects.history.eventUnlinked");
  if (event.eventType === "dependency_added") return t("projects.history.dependencyAdded", newValue);
  if (event.eventType === "dependency_removed") return t("projects.history.dependencyRemoved", oldValue);
  if (event.eventType === "completed") return t("projects.history.completed", oldValue, newValue);
  if (event.eventType === "reopened") return t("projects.history.reopened", oldValue, newValue);
  if (event.eventType === "archived") return t("projects.history.archived");
  if (event.fieldName) return t("projects.history.fieldChanged", field, oldValue, newValue);
  return t("projects.history.updated");
}
