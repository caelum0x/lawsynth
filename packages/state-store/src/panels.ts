import { InvariantError } from "./errors.js";

export type PanelId = "inspector" | "navigator" | "timeline" | "console" | "candidates";
export type PanelSide = "left" | "right" | "bottom";

export interface PanelState { readonly id: PanelId; readonly side: PanelSide; readonly open: boolean; readonly size: number; }
export type PanelsState = Readonly<Record<PanelId, PanelState>>;

const panel = (id: PanelId, side: PanelSide, size: number): PanelState => Object.freeze({ id, side, open: true, size });
export const DEFAULT_PANELS: PanelsState = Object.freeze({
  navigator: panel("navigator", "left", 320), inspector: panel("inspector", "right", 360), timeline: panel("timeline", "bottom", 280), console: panel("console", "bottom", 280), candidates: panel("candidates", "bottom", 320),
});

export function setPanel(panels: PanelsState, id: PanelId, patch: Partial<Pick<PanelState, "open" | "side" | "size">>): PanelsState {
  const current = panels[id];
  const next: PanelState = { ...current, ...patch };
  if (!Number.isFinite(next.size) || next.size < 160 || next.size > 1_600) throw new InvariantError("Panel size must be in 160..=1600 pixels");
  if (next.id !== id) throw new InvariantError("Panel id cannot be changed");
  if (next.open === current.open && next.side === current.side && next.size === current.size) return panels;
  return Object.freeze({ ...panels, [id]: Object.freeze(next) });
}

export function validatePanels(value: PanelsState): PanelsState {
  let current = DEFAULT_PANELS;
  for (const id of Object.keys(DEFAULT_PANELS) as PanelId[]) {
    const candidate = value[id];
    if (candidate !== undefined) current = setPanel(current, id, candidate);
  }
  return current;
}
