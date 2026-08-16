import type { XYPoint } from "./downsample.js";
import type { Trajectory } from "./trajectory.js";

export interface PhasePortrait { readonly xVariable: string; readonly yVariable: string; readonly points: readonly XYPoint[]; }

export function phasePortrait(trajectory: Trajectory, xVariable: string, yVariable: string): PhasePortrait {
  const xIndex = trajectory.variables.indexOf(xVariable); const yIndex = trajectory.variables.indexOf(yVariable);
  if (xIndex < 0 || yIndex < 0) throw new RangeError("phase portrait variables must exist in trajectory");
  if (xIndex === yIndex) throw new RangeError("phase portrait axes must be distinct");
  return { xVariable, yVariable, points: trajectory.samples.map((sample) => ({ x: sample.values[xIndex]!, y: sample.values[yIndex]! })) };
}
