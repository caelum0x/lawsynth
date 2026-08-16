import type { XYPoint } from "./downsample.js";
import { trajectoryComponent, type Trajectory } from "./trajectory.js";

export interface Series {
  readonly id: string;
  readonly label: string;
  readonly points: readonly XYPoint[];
  readonly unit?: string;
  readonly color?: string;
}

export interface SeriesOptions {
  readonly label?: string;
  readonly unit?: string;
  readonly color?: string;
}

/** Converts one named trajectory component into plot coordinates. */
export function seriesFromTrajectory(trajectory: Trajectory, variable: string, options: SeriesOptions = {}): Series {
  const points = trajectoryComponent(trajectory, variable).map(([x, y]) => ({ x, y }));
  return {
    id: variable,
    label: options.label ?? variable,
    points,
    ...(options.unit === undefined ? {} : { unit: options.unit }),
    ...(options.color === undefined ? {} : { color: options.color }),
  };
}

/** Converts every component while preserving declaration order. */
export function seriesFromAllTrajectoryComponents(trajectory: Trajectory): Series[] {
  return trajectory.variables.map((variable) => seriesFromTrajectory(trajectory, variable));
}

/**
 * Validates a series and returns a copied representation. X values must be
 * monotonic, which lets decimators and hit-testing use binary search safely.
 */
export function normalizeSeries(input: Series): Series {
  if (!input.id.trim() || !input.label.trim()) throw new TypeError("series id and label must be non-empty");
  let previous = -Infinity;
  const points = input.points.map((point, index) => {
    if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) throw new RangeError(`series point ${index} must be finite`);
    if (point.x < previous) throw new RangeError("series x values must be monotonic");
    previous = point.x;
    return { x: point.x, y: point.y };
  });
  return { id: input.id, label: input.label, points, ...(input.unit === undefined ? {} : { unit: input.unit }), ...(input.color === undefined ? {} : { color: input.color }) };
}

/** Finds the nearest data sample in x-space using a binary search. */
export function nearestPoint(series: Series, x: number): XYPoint | undefined {
  if (!Number.isFinite(x)) throw new RangeError("x must be finite");
  if (series.points.length === 0) return undefined;
  let lo = 0; let hi = series.points.length;
  while (lo < hi) { const mid = Math.floor((lo + hi) / 2); if (series.points[mid]!.x < x) lo = mid + 1; else hi = mid; }
  const right = series.points[lo]; const left = series.points[lo - 1];
  if (left === undefined) return right === undefined ? undefined : { ...right };
  if (right === undefined) return { ...left };
  return Math.abs(left.x - x) <= Math.abs(right.x - x) ? { ...left } : { ...right };
}
