import type { ScreenId } from "./ids.js";

export * from "./ids.js";
export * from "./types.js";
export * from "./geometry.js";
export * from "./simulate.js";
export * from "./fixtures.js";
export * from "./export-format.js";
export * from "./signal.js";
export * from "./data-lens.js";
export * from "./data-prep.js";
export * from "./discovery-canvas.js";
export * from "./equation-explorer.js";
export * from "./structure-map.js";
export * from "./regime-timeline.js";
export * from "./uncertainty-lens.js";
export * from "./world-lab.js";
export * from "./scenario-board.js";
export * from "./monitor.js";
export * from "./export-screen.js";
export * from "./controller.js";
export * from "./render.js";

export interface ScreenDescriptor {
  readonly id: ScreenId;
  readonly title: string;
  readonly subtitle: string;
}

/** Navigable Studio screens in the product's observe → discover → understand → use → share order. */
export const SCREEN_REGISTRY: readonly ScreenDescriptor[] = Object.freeze([
  { id: "data-lens", title: "Data Lens", subtitle: "Profile the input dataset and its quality" },
  { id: "data-prep", title: "Data Prep", subtitle: "Smooth, resample, detrend, and trim the working dataset" },
  { id: "discovery-canvas", title: "Discovery Canvas", subtitle: "Configure a run and inspect candidate laws" },
  { id: "equation-explorer", title: "Equation Explorer", subtitle: "Read discovered laws and their terms" },
  { id: "structure-map", title: "Structure Map", subtitle: "Variable coupling graph from law dependencies" },
  { id: "regime-timeline", title: "Regime Timeline", subtitle: "Regime segments over time" },
  { id: "uncertainty-lens", title: "Uncertainty Lens", subtitle: "Confidence bands on a trajectory" },
  { id: "world-lab", title: "World Lab", subtitle: "Simulate, forecast, and intervene" },
  { id: "scenario-board", title: "Scenario Board", subtitle: "Compare what-if scenarios against a baseline" },
  { id: "monitor", title: "Monitor", subtitle: "Model-based anomaly detection on new data" },
  { id: "export-screen", title: "Export", subtitle: "Equations, LaTeX, Python, and the raw World IR" },
]);

export function screenDescriptor(id: ScreenId): ScreenDescriptor {
  const descriptor = SCREEN_REGISTRY.find((entry) => entry.id === id);
  if (descriptor === undefined) throw new RangeError(`unknown screen: ${id}`);
  return descriptor;
}
