import type { EventDraft } from "./events.js";
import type { PanelId, PanelSide } from "./panels.js";
import type { PreferencesState } from "./preferences.js";
import type { WorkspacePatch } from "./workspace.js";

export type Command =
  | { readonly kind: "workspace.update"; readonly patch: WorkspacePatch }
  | { readonly kind: "workspace.clear" }
  | { readonly kind: "selection.set"; readonly ids: readonly string[]; readonly primaryId?: string }
  | { readonly kind: "selection.toggle"; readonly id: string }
  | { readonly kind: "selection.hover"; readonly id?: string }
  | { readonly kind: "panel.update"; readonly id: PanelId; readonly patch: Partial<{ readonly open: boolean; readonly side: PanelSide; readonly size: number }> }
  | { readonly kind: "preferences.update"; readonly patch: Partial<PreferencesState> };

export function commandEvent(command: Command): EventDraft {
  switch (command.kind) {
    case "workspace.update": return { type: "workspace.updated", patch: command.patch };
    case "workspace.clear": return { type: "workspace.cleared" };
    case "selection.set": return command.primaryId === undefined ? { type: "selection.set", ids: command.ids } : { type: "selection.set", ids: command.ids, primaryId: command.primaryId };
    case "selection.toggle": return { type: "selection.toggled", id: command.id };
    case "selection.hover": return command.id === undefined ? { type: "selection.hovered" } : { type: "selection.hovered", id: command.id };
    case "panel.update": return { type: "panel.updated", id: command.id, patch: command.patch };
    case "preferences.update": return { type: "preferences.updated", patch: command.patch };
  }
}
