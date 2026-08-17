import type { ScreenId } from "./ids.js";

export * from "./ids.js";
export * from "./types.js";
export * from "./geometry.js";
export * from "./simulate.js";
export * from "./fixtures.js";
export * from "./discovery-canvas.js";
export * from "./equation-explorer.js";
export * from "./regime-timeline.js";
export * from "./uncertainty-lens.js";
export * from "./world-lab.js";
export * from "./controller.js";
export * from "./render.js";

export interface ScreenDescriptor {
  readonly id: ScreenId;
  readonly title: string;
  readonly subtitle: string;
}

/** Navigable Studio screens in presentation order. */
export const SCREEN_REGISTRY: readonly ScreenDescriptor[] = Object.freeze([
  { id: "discovery-canvas", title: "Discovery Canvas", subtitle: "Configure a run and inspect candidate laws" },
  { id: "equation-explorer", title: "Equation Explorer", subtitle: "Read discovered laws and their terms" },
  { id: "regime-timeline", title: "Regime Timeline", subtitle: "Regime segments over time" },
  { id: "uncertainty-lens", title: "Uncertainty Lens", subtitle: "Confidence bands on a trajectory" },
  { id: "world-lab", title: "World Lab", subtitle: "Simulate, forecast, and intervene" },
]);

export function screenDescriptor(id: ScreenId): ScreenDescriptor {
  const descriptor = SCREEN_REGISTRY.find((entry) => entry.id === id);
  if (descriptor === undefined) throw new RangeError(`unknown screen: ${id}`);
  return descriptor;
}
