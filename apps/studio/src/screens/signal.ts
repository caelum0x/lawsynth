import type { TrajectoryInput } from "@lawsynth/chart-core";

/**
 * Deterministic, dependency-free signal transforms shared by the data-quality
 * screens (Data Prep's preparation pipeline and Monitor's residual analysis).
 * Every function is pure: it copies its inputs and returns fresh arrays, so a
 * view model can compose them without hidden state or mutation.
 */

/** Arithmetic mean of a finite sample; `0` for an empty sample. */
export function mean(values: readonly number[]): number {
  if (values.length === 0) return 0;
  let sum = 0;
  for (const value of values) sum += value;
  return sum / values.length;
}

/** Population standard deviation; `0` for a constant or empty sample. */
export function standardDeviation(values: readonly number[]): number {
  if (values.length === 0) return 0;
  const m = mean(values);
  let sum = 0;
  for (const value of values) sum += (value - m) ** 2;
  return Math.sqrt(sum / values.length);
}

export interface Standardized {
  readonly values: readonly number[];
  readonly mean: number;
  readonly std: number;
}

/**
 * Centers a sample on its mean and scales by its standard deviation, yielding
 * z-scores. A constant column (std ≈ 0) standardizes to all-zeros rather than
 * dividing by zero.
 */
export function standardize(values: readonly number[]): Standardized {
  const m = mean(values);
  const std = standardDeviation(values);
  if (std < 1e-12) return { values: values.map(() => 0), mean: m, std: 0 };
  return { values: values.map((value) => (value - m) / std), mean: m, std };
}

/**
 * Centered moving-average smoothing with a shrinking window at the edges so the
 * output stays the same length and no samples are invented. `window <= 1` is an
 * identity transform. Even windows are rounded down to the nearest odd width.
 */
export function movingAverage(values: readonly number[], window: number): readonly number[] {
  const width = Math.max(1, Math.floor(window));
  if (width <= 1 || values.length === 0) return values.slice();
  const half = Math.floor((width - 1) / 2);
  const out: number[] = [];
  for (let i = 0; i < values.length; i += 1) {
    const lo = Math.max(0, i - half);
    const hi = Math.min(values.length - 1, i + half);
    let sum = 0;
    for (let j = lo; j <= hi; j += 1) sum += values[j] ?? 0;
    out.push(sum / (hi - lo + 1));
  }
  return out;
}

/**
 * Removes the ordinary-least-squares linear trend (slope·t + intercept) from a
 * column, leaving the residual about the fitted line. Degenerate time spans
 * (fewer than two points or zero variance in time) return the input unchanged.
 */
export function detrend(times: readonly number[], values: readonly number[]): readonly number[] {
  const n = Math.min(times.length, values.length);
  if (n < 2) return values.slice();
  const tMean = mean(times.slice(0, n));
  const vMean = mean(values.slice(0, n));
  let sTT = 0;
  let sTV = 0;
  for (let i = 0; i < n; i += 1) {
    const dt = (times[i] ?? 0) - tMean;
    sTT += dt * dt;
    sTV += dt * ((values[i] ?? 0) - vMean);
  }
  if (sTT < 1e-12) return values.slice();
  const slope = sTV / sTT;
  const intercept = vMean - slope * tMean;
  return values.map((value, i) => value - (slope * (times[i] ?? 0) + intercept));
}

/** Linearly interpolates a monotonically-timed column at an arbitrary time, clamping outside the sampled span. */
export function interpolateAt(times: readonly number[], values: readonly number[], t: number): number {
  const n = Math.min(times.length, values.length);
  if (n === 0) return 0;
  if (t <= (times[0] ?? 0)) return values[0] ?? 0;
  if (t >= (times[n - 1] ?? 0)) return values[n - 1] ?? 0;
  // Binary search for the bracketing interval [lo, lo+1].
  let lo = 0;
  let hi = n - 1;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if ((times[mid] ?? 0) <= t) lo = mid;
    else hi = mid;
  }
  const t0 = times[lo] ?? 0;
  const t1 = times[lo + 1] ?? t0;
  const v0 = values[lo] ?? 0;
  const v1 = values[lo + 1] ?? v0;
  if (t1 === t0) return v0;
  const ratio = (t - t0) / (t1 - t0);
  return v0 + ratio * (v1 - v0);
}

/** Builds a uniform time grid `[start, start+dt, …]` covering `[start, end]` inclusive. */
export function uniformGrid(start: number, end: number, dt: number): readonly number[] {
  if (!(dt > 0) || !(end > start)) return [start];
  const count = Math.floor((end - start) / dt);
  const grid: number[] = [];
  for (let i = 0; i <= count; i += 1) grid.push(Number((start + i * dt).toFixed(6)));
  const last = grid[grid.length - 1] ?? start;
  if (last < end - 1e-9) grid.push(Number(end.toFixed(6)));
  return grid;
}

/** Resamples every column of a trajectory onto a shared time grid via linear interpolation. */
export function resampleTrajectory(trajectory: TrajectoryInput, grid: readonly number[]): TrajectoryInput {
  const columns = trajectory.variables.map((_, index) => trajectory.times.map((_, row) => trajectory.values[row]?.[index] ?? 0));
  const values = grid.map((t) => columns.map((column) => interpolateAt(trajectory.times, column, t)));
  return { variables: trajectory.variables.slice(), times: grid.slice(), values };
}

/** Drops `count` rows from each end of a trajectory, keeping at least two rows when possible. */
export function trimTrajectory(trajectory: TrajectoryInput, count: number): TrajectoryInput {
  const drop = Math.max(0, Math.floor(count));
  const total = trajectory.times.length;
  if (drop === 0 || total === 0) return trajectory;
  const start = Math.min(drop, Math.max(0, Math.floor((total - 1) / 2)));
  const end = Math.max(start + 1, total - drop);
  return {
    variables: trajectory.variables.slice(),
    times: trajectory.times.slice(start, end),
    values: trajectory.values.slice(start, end).map((row) => row.slice()),
  };
}

/** Applies a per-column map across every row of a trajectory (used for smoothing/detrend). */
export function mapColumns(
  trajectory: TrajectoryInput,
  transform: (column: readonly number[], variable: string, index: number) => readonly number[],
): TrajectoryInput {
  const columns = trajectory.variables.map((variable, index) =>
    transform(trajectory.times.map((_, row) => trajectory.values[row]?.[index] ?? 0), variable, index),
  );
  const values = trajectory.times.map((_, row) => columns.map((column) => column[row] ?? 0));
  return { variables: trajectory.variables.slice(), times: trajectory.times.slice(), values };
}
