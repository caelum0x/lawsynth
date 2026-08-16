import {
  createChartModel,
  createScale,
  downsampleForViewport,
  extent,
  padDomain,
  seriesFromAllTrajectoryComponents,
  type ChartModel,
  type Domain,
  type Series,
  type Trajectory,
  type TrajectoryInput,
  normalizeTrajectory,
} from "@lawsynth/chart-core";

export interface TrajectoryView {
  readonly trajectory: Trajectory;
  readonly chart: ChartModel;
  readonly duration: number;
  readonly sampleCount: number;
}

export interface SvgPath {
  readonly id: string;
  readonly label: string;
  readonly color?: string;
  readonly d: string;
}

export interface PlotGeometry {
  readonly paths: readonly SvgPath[];
  readonly xDomain: Domain;
  readonly yDomain: Domain;
  readonly width: number;
  readonly height: number;
}

export function trajectoryView(input: TrajectoryInput | Trajectory, title = "Simulation trajectory"): TrajectoryView {
  const trajectory = "times" in input ? normalizeTrajectory(input) : normalizeTrajectory({
    variables: input.variables,
    times: input.samples.map((sample) => sample.time),
    values: input.samples.map((sample) => sample.values),
    ...(input.metadata === undefined ? {} : { metadata: input.metadata }),
  });
  const series = seriesFromAllTrajectoryComponents(trajectory);
  const chart = createChartModel({ title, series, xLabel: "time", yLabel: "value" });
  const first = trajectory.samples[0]?.time ?? 0;
  const last = trajectory.samples.at(-1)?.time ?? first;
  return Object.freeze({ trajectory, chart, duration: last - first, sampleCount: trajectory.samples.length });
}

function seriesDomain(series: readonly Series[], key: "x" | "y"): Domain {
  const values = series.flatMap((entry) => entry.points.map((point) => point[key]));
  return values.length === 0 ? { min: 0, max: 1 } : padDomain(extent(values), 0.04);
}

export function trajectoryPlotGeometry(
  chart: ChartModel,
  width: number,
  height: number,
  padding = 20,
): PlotGeometry {
  if (![width, height, padding].every(Number.isFinite) || width <= padding * 2 || height <= padding * 2 || padding < 0) {
    throw new RangeError("plot dimensions must leave a positive drawing area");
  }
  const xDomain = seriesDomain(chart.series, "x");
  const yDomain = seriesDomain(chart.series, "y");
  const x = createScale(xDomain, { min: padding, max: width - padding });
  const y = createScale(yDomain, { min: height - padding, max: padding });
  const paths = chart.series.map((series): SvgPath => {
    const points = downsampleForViewport(series.points, width - padding * 2);
    const d = points.map((point, index) => `${index === 0 ? "M" : "L"}${x(point.x).toFixed(2)},${y(point.y).toFixed(2)}`).join(" ");
    return Object.freeze({ id: series.id, label: series.label, ...(series.color === undefined ? {} : { color: series.color }), d });
  });
  return Object.freeze({ paths: Object.freeze(paths), xDomain, yDomain, width, height });
}

export function valueAtTime(trajectory: Trajectory, time: number): Readonly<Record<string, number>> | undefined {
  if (!Number.isFinite(time)) throw new RangeError("time must be finite");
  if (trajectory.samples.length === 0) return undefined;
  let low = 0;
  let high = trajectory.samples.length;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    if (trajectory.samples[middle]!.time < time) low = middle + 1; else high = middle;
  }
  const right = trajectory.samples[low];
  const left = trajectory.samples[low - 1];
  const sample = left === undefined ? right : right === undefined ? left :
    Math.abs(left.time - time) <= Math.abs(right.time - time) ? left : right;
  if (sample === undefined) return undefined;
  return Object.freeze(Object.fromEntries(trajectory.variables.map((variable, index) => [variable, sample.values[index]!])));
}
