import { parameterRange, uncertaintyCoverage } from "../src/uncertainty.js";
import type { WorldDefinition } from "@lawsynth/world-schema";

/** Prepare parameter intervals only after reporting missing uncertainty evidence. */
export function summarizeUncertainty(world: WorldDefinition) {
  const coverage = uncertaintyCoverage(world);
  const ranges = (world.parameters ?? []).map((parameter) => ({ id: parameter.id, range: parameterRange(parameter) }));
  return Object.freeze({ coverage, ranges: Object.freeze(ranges) });
}
