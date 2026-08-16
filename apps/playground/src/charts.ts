import {
  createChartModel,
  normalizeTrajectory,
  seriesFromAllTrajectoryComponents,
  type ChartModel,
  type TrajectoryInput,
} from "@lawsynth/chart-core";

export interface ChartSelection {
  readonly visibleVariables?: readonly string[];
  readonly title?: string;
  readonly xLabel?: string;
  readonly yLabel?: string;
}

export interface TrajectoryChartSet {
  readonly combined: ChartModel;
  readonly individual: readonly ChartModel[];
  readonly sampleCount: number;
  readonly duration: number;
}

export function trajectoryChartSet(input: TrajectoryInput, selection: ChartSelection = {}): TrajectoryChartSet {
  const trajectory = normalizeTrajectory(input);
  const requested = selection.visibleVariables === undefined
    ? new Set(trajectory.variables)
    : new Set(selection.visibleVariables);

  for (const variable of requested) {
    if (!trajectory.variables.includes(variable)) throw new RangeError(`unknown trajectory variable: ${variable}`);
  }

  const series = seriesFromAllTrajectoryComponents(trajectory).filter((entry) => requested.has(entry.id));
  if (series.length === 0) throw new RangeError("at least one trajectory variable must be visible");

  const combined = createChartModel({
    title: selection.title ?? "Simulation trajectory",
    series,
    xLabel: selection.xLabel ?? "time",
    yLabel: selection.yLabel ?? "value",
  });
  const individual = series.map((entry) => createChartModel({
    title: entry.label,
    series: [entry],
    xLabel: selection.xLabel ?? "time",
    yLabel: entry.unit ?? selection.yLabel ?? "value",
  }));
  const first = trajectory.samples[0]?.time ?? 0;
  const last = trajectory.samples.at(-1)?.time ?? first;

  return Object.freeze({
    combined,
    individual: Object.freeze(individual),
    sampleCount: trajectory.samples.length,
    duration: last - first,
  });
}

export function chartsForTrajectory(input: TrajectoryInput): readonly ChartModel[] {
  return trajectoryChartSet(input).individual;
}

export function combinedTrajectoryChart(input: TrajectoryInput): ChartModel {
  return trajectoryChartSet(input).combined;
}
