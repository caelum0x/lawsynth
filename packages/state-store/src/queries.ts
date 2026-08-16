import type { PanelId } from "./panels.js";
import type { StudioState } from "./store.js";

export const activeProjectId = (state: StudioState): string | undefined => state.workspace.projectId;
export const activeWorldId = (state: StudioState): string | undefined => state.workspace.worldId;
export const activeRunId = (state: StudioState): string | undefined => state.workspace.runId;
export const selectedIds = (state: StudioState): readonly string[] => state.selection.ids;
export const primarySelection = (state: StudioState): string | undefined => state.selection.primaryId;
export const isSelected = (state: StudioState, id: string): boolean => state.selection.ids.includes(id);
export const isPanelOpen = (state: StudioState, id: PanelId): boolean => state.panels[id].open;
export const openPanels = (state: StudioState): readonly PanelId[] => (Object.keys(state.panels) as PanelId[]).filter((id) => state.panels[id].open);
