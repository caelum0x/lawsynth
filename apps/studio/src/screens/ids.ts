/**
 * Screen identifiers, kept dependency-free so lightweight modules (routing,
 * navigation) can validate a screen id without importing the heavier view
 * models that pull in the chart/world packages.
 */
export type ScreenId =
  | "data-lens"
  | "data-prep"
  | "discovery-canvas"
  | "equation-explorer"
  | "structure-map"
  | "regime-timeline"
  | "uncertainty-lens"
  | "world-lab"
  | "scenario-board"
  | "monitor"
  | "export-screen";

export const SCREEN_IDS: readonly ScreenId[] = Object.freeze([
  "data-lens",
  "data-prep",
  "discovery-canvas",
  "equation-explorer",
  "structure-map",
  "regime-timeline",
  "uncertainty-lens",
  "world-lab",
  "scenario-board",
  "monitor",
  "export-screen",
]);

export function isScreenId(value: unknown): value is ScreenId {
  return typeof value === "string" && (SCREEN_IDS as readonly string[]).includes(value);
}
