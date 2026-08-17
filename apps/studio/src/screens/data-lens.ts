import { categoricalColor, createChartModel, type Series, type TrajectoryInput } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { buildPlot, type PlotLine } from "./geometry.js";
import { forwardEuler } from "./simulate.js";
import type { Metric, Notice, ScreenModel, ScreenSection, TableColumn, TableRow } from "./types.js";

export interface DataLensInput {
  readonly world: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly initialState: Readonly<Record<string, number>>;
  readonly horizon?: number;
  readonly step?: number;
  readonly width?: number;
  readonly height?: number;
}

interface ColumnStats {
  readonly name: string;
  readonly unit: string;
  readonly min: number;
  readonly max: number;
  readonly mean: number;
  readonly present: number;
  readonly constant: boolean;
}

function column(trajectory: TrajectoryInput, index: number): readonly number[] {
  return trajectory.times.map((_, row) => trajectory.values[row]?.[index] ?? Number.NaN);
}

function statsFor(name: string, unit: string, values: readonly number[]): ColumnStats {
  const finite = values.filter((value) => Number.isFinite(value));
  const min = finite.length === 0 ? Number.NaN : Math.min(...finite);
  const max = finite.length === 0 ? Number.NaN : Math.max(...finite);
  const mean = finite.length === 0 ? Number.NaN : finite.reduce((sum, value) => sum + value, 0) / finite.length;
  return { name, unit, min, max, mean, present: finite.length, constant: finite.length > 0 && max - min < 1e-9 };
}

function fmt(value: number): string {
  return Number.isFinite(value) ? value.toFixed(3) : "—";
}

/** Coefficient-of-variation style spread used to detect irregular time sampling. */
function irregularSampling(times: readonly number[]): boolean {
  if (times.length < 3) return false;
  const deltas: number[] = [];
  for (let i = 1; i < times.length; i += 1) deltas.push((times[i] ?? 0) - (times[i - 1] ?? 0));
  const mean = deltas.reduce((sum, value) => sum + value, 0) / deltas.length;
  if (mean <= 0) return false;
  const variance = deltas.reduce((sum, value) => sum + (value - mean) ** 2, 0) / deltas.length;
  return Math.sqrt(variance) / mean > 0.05;
}

/**
 * Inspects the input dataset the workspace models — before or after discovery.
 * When no observed trajectory is supplied it integrates the world locally
 * (`forwardEuler`) so the lens always has state columns to profile. It reports a
 * per-column profile (min/max/mean/units), small-multiples of each state column
 * over time (via `chart-core`), and basic quality flags (gaps, constant columns).
 */
export function dataLensModel(input: DataLensInput): ScreenModel {
  const { world } = input;
  const timeSymbol = world.time.symbol ?? "t";
  const timeUnit = world.time.unit ?? "";

  let trajectory = input.trajectory;
  let source = "observed dataset";
  if (trajectory === undefined || trajectory.times.length === 0) {
    try {
      trajectory = forwardEuler(world, {
        horizon: input.horizon ?? 12,
        step: input.step ?? 0.1,
        initialState: input.initialState,
      });
      source = "locally integrated";
    } catch {
      trajectory = { variables: [], times: [], values: [] };
    }
  }

  const notices: Notice[] = [];
  const sections: ScreenSection[] = [];

  if (trajectory.variables.length === 0 || trajectory.times.length === 0) {
    notices.push({ tone: "info", message: "No dataset is available to profile for this world." });
    sections.push({ kind: "notices", id: "data-empty", notices });
    return { id: "data-lens", title: "Data Lens", subtitle: "Inspect the input dataset before and after discovery", sections };
  }

  const unitFor = (variable: string): string => world.variables.find((entry) => entry.id === variable)?.unit ?? "";
  const stats: readonly ColumnStats[] = trajectory.variables.map((variable, index) => statsFor(variable, unitFor(variable), column(trajectory!, index)));
  const rows = trajectory.times.length;

  // Quality flags.
  if (irregularSampling(trajectory.times)) notices.push({ tone: "warning", message: "Time column is irregularly sampled (non-uniform steps)." });
  for (const stat of stats) {
    if (stat.constant) notices.push({ tone: "warning", message: `Column "${stat.name}" is constant — it carries no dynamics.` });
    if (stat.present < rows) notices.push({ tone: "warning", message: `Column "${stat.name}" has ${rows - stat.present} missing value(s).` });
  }
  if (rows < 3) notices.push({ tone: "warning", message: "Fewer than three samples: discovery quality will be poor." });
  if (notices.length === 0) notices.push({ tone: "success", message: "No quality issues detected in the profiled columns." });

  const metrics: readonly Metric[] = [
    { label: "Rows", value: String(rows) },
    { label: "State columns", value: String(trajectory.variables.length) },
    { label: "Time span", value: `${fmt(trajectory.times[0] ?? 0)} → ${fmt(trajectory.times[trajectory.times.length - 1] ?? 0)} ${timeUnit}`.trim() },
    { label: "Source", value: source },
  ];
  sections.push({ kind: "metrics", id: "data-metrics", title: "Dataset", metrics });

  sections.push({ kind: "notices", id: "data-quality", notices });

  const profileColumns: readonly TableColumn[] = [
    { key: "column", label: "Column" },
    { key: "unit", label: "Unit" },
    { key: "min", label: "Min", align: "end" },
    { key: "max", label: "Max", align: "end" },
    { key: "mean", label: "Mean", align: "end" },
    { key: "present", label: "Present", align: "end" },
  ];
  const profileRows: readonly TableRow[] = [
    { id: timeSymbol, cells: [timeSymbol, timeUnit || "—", fmt(trajectory.times[0] ?? 0), fmt(trajectory.times[trajectory.times.length - 1] ?? 0), "—", String(rows)], emphasis: true },
    ...stats.map((stat): TableRow => ({ id: stat.name, cells: [stat.name, stat.unit || "—", fmt(stat.min), fmt(stat.max), fmt(stat.mean), String(stat.present)] })),
  ];
  sections.push({ kind: "table", id: "data-profile", title: "Column profile", columns: profileColumns, rows: profileRows });

  // Small multiples: one line chart per state column over time.
  const width = input.width ?? 360;
  const height = input.height ?? 150;
  trajectory.variables.forEach((variable, index) => {
    const points = trajectory!.times.map((time, row) => ({ x: time, y: trajectory!.values[row]?.[index] ?? 0 }));
    const color = categoricalColor(variable);
    const line: PlotLine = { id: variable, label: variable, points, color };
    const plot = buildPlot([line], width, height);
    const series: readonly Series[] = [{ id: variable, label: variable, points: points.map((point) => ({ x: point.x, y: point.y })), color }];
    const chart = createChartModel({ title: `${variable} over ${timeSymbol}`, series, xLabel: timeSymbol, yLabel: unitFor(variable) || variable });
    sections.push({ kind: "chart", id: `data-chart-${variable}`, title: `${variable} over ${timeSymbol}`, chart, geometry: plot.geometry });
  });

  return { id: "data-lens", title: "Data Lens", subtitle: "Inspect the input dataset before and after discovery", sections };
}
