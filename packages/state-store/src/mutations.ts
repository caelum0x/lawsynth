import { UnknownEventError } from "./errors.js";
import type { StateEvent } from "./events.js";
import type { StudioState } from "./store.js";
import { setPanel } from "./panels.js";
import { updatePreferences } from "./preferences.js";
import { clearSelection, select, setHovered, toggleSelection } from "./selection.js";
import { clearWorkspace, updateWorkspace } from "./workspace.js";

/** Pure event reducer. It does not mutate its input or invoke I/O. */
export function reduceState(current: StudioState, event: StateEvent): StudioState {
  switch (event.type) {
    case "workspace.updated": return { ...current, workspace: updateWorkspace(current.workspace, event.patch) };
    case "workspace.cleared": return { ...current, workspace: clearWorkspace(current.workspace), selection: clearSelection(current.selection) };
    case "selection.set": return { ...current, selection: select(event.ids, event.primaryId) };
    case "selection.toggled": return { ...current, selection: toggleSelection(current.selection, event.id) };
    case "selection.hovered": return { ...current, selection: setHovered(current.selection, event.id) };
    case "panel.updated": return { ...current, panels: setPanel(current.panels, event.id, event.patch) };
    case "preferences.updated": return { ...current, preferences: updatePreferences(current.preferences, event.patch) };
    default: throw new UnknownEventError((event as { type: string }).type);
  }
}

export function reduceEvents(initial: StudioState, events: readonly StateEvent[]): StudioState {
  return events.reduce(reduceState, initial);
}
