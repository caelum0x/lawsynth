import type { PanelId, PanelSide } from "./panels.js";
import type { PreferencesState } from "./preferences.js";
import type { WorkspacePatch } from "./workspace.js";

export interface StateEventBase { readonly eventId: string; readonly at: number; readonly type: string; }
export type StateEvent =
  | (StateEventBase & { readonly type: "workspace.updated"; readonly patch: WorkspacePatch })
  | (StateEventBase & { readonly type: "workspace.cleared" })
  | (StateEventBase & { readonly type: "selection.set"; readonly ids: readonly string[]; readonly primaryId?: string })
  | (StateEventBase & { readonly type: "selection.toggled"; readonly id: string })
  | (StateEventBase & { readonly type: "selection.hovered"; readonly id?: string })
  | (StateEventBase & { readonly type: "panel.updated"; readonly id: PanelId; readonly patch: Partial<{ readonly open: boolean; readonly side: PanelSide; readonly size: number }> })
  | (StateEventBase & { readonly type: "preferences.updated"; readonly patch: Partial<PreferencesState> });

type WithoutEnvelope<Event> = Event extends StateEventBase ? Omit<Event, "eventId" | "at"> : never;
export type EventDraft = WithoutEnvelope<StateEvent>;

export function createEvent(draft: EventDraft, eventId: string, at: number): StateEvent {
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$/u.test(eventId)) throw new TypeError("Event id is invalid");
  if (!Number.isSafeInteger(at) || at < 0) throw new RangeError("Event timestamp must be a non-negative integer");
  return Object.freeze({ ...draft, eventId, at }) as StateEvent;
}

export function compareEvents(left: StateEvent, right: StateEvent): number {
  return left.at - right.at || left.eventId.localeCompare(right.eventId);
}

export function isStateEvent(value: unknown): value is StateEvent {
  return typeof value === "object" && value !== null && typeof (value as { type?: unknown }).type === "string" && typeof (value as { eventId?: unknown }).eventId === "string" && Number.isSafeInteger((value as { at?: unknown }).at);
}
