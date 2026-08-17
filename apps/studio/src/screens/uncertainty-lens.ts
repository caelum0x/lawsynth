import { categoricalColor, createChartModel, type Series, type TrajectoryInput } from "@lawsynth/chart-core";
import type { TrajectoryBand, WorldDefinition } from "@lawsynth/world-schema";
import { bandPolygonPoints, uncertaintySummary } from "@lawsynth/world-viewer";
import { buildPlot, linePath, type PlotLine, type PlotPoint } from "./geometry.js";
import type { BandOverlay, ControlField, Metric, Notice, ScreenModel, ScreenSection, TableRow } from "./types.js";

export interface UncertaintyLensInput {
  readonly world: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly selectedVariable?: string;
  readonly width?: number;
  readonly height?: number;
}

function trajectoryBands(world: WorldDefinition): readonly TrajectoryBand[] {
  const entries = world.uncertainty?.entries ?? [];
  return entries.flatMap((entry) => (entry.level === "trajectory" ? entry.bands : []));
}

function observedPoints(trajectory: TrajectoryInput | undefined, variable: string): readonly PlotPoint[] {
  if (trajectory === undefined) return [];
  const index = trajectory.variables.indexOf(variable);
  if (index < 0) return [];
  return trajectory.times.map((time, row) => ({ x: time, y: trajectory.values[row]?.[index] ?? 0 }));
}

/**
 * Overlays an uncertainty band (from the world's `trajectory` uncertainty) on a
 * line trajectory. The band ring comes from `world-viewer.bandPolygonPoints` and
 * is mapped through the same `chart-core` scales as the median/observed lines so
 * the shaded area stays aligned with the traces.
 */
export function uncertaintyLensModel(input: UncertaintyLensInput): ScreenModel {
  const { world } = input;
  const summary = uncertaintySummary(world.uncertainty);
  const bands = trajectoryBands(world);
  const sections: ScreenSection[] = [];

  const metrics: readonly Metric[] = [
    { label: "Trajectory bands", value: String(summary.counts.trajectory) },
    { label: "Parameter", value: String(summary.counts.parameter) },
    { label: "Data / structural", value: `${summary.counts.data} / ${summary.counts.structural}` },
    { label: "Method", value: summary.method ?? "—" },
  ];
  sections.push({ kind: "metrics", id: "uncertainty-metrics", title: "Coverage", metrics });

  if (bands.length === 0) {
    const notices: readonly Notice[] = [{ tone: "info", message: "No trajectory uncertainty bands are recorded for this world." }];
    sections.push({ kind: "notices", id: "no-bands", notices });
    return { id: "uncertainty-lens", title: "Uncertainty Lens", subtitle: "Overlay confidence bands on a trajectory", sections };
  }

  const selected = bands.find((band) => band.variable === input.selectedVariable) ?? bands[0]!;
  const width = input.width ?? 760;
  const height = input.height ?? 320;

  const upperPts: readonly PlotPoint[] = selected.times.map((time, i) => ({ x: time, y: selected.upper[i] ?? 0 }));
  const lowerPts: readonly PlotPoint[] = selected.times.map((time, i) => ({ x: time, y: selected.lower[i] ?? 0 }));
  const medianPts: readonly PlotPoint[] = (selected.median ?? []).map((value, i) => ({ x: selected.times[i] ?? 0, y: value }));
  const observedPts = observedPoints(input.trajectory, selected.variable);

  const lines: PlotLine[] = [
    { id: "band-upper", label: "upper", points: upperPts },
    { id: "band-lower", label: "lower", points: lowerPts },
  ];
  const seriesColor = categoricalColor(selected.variable);
  if (medianPts.length > 0) lines.push({ id: "median", label: `${selected.variable} median`, points: medianPts, color: seriesColor });
  if (observedPts.length > 0) lines.push({ id: "observed", label: `${selected.variable} observed`, points: observedPts, color: categoricalColor(`${selected.variable}:obs`) });

  const plot = buildPlot(lines, width, height);
  const ring = bandPolygonPoints(selected).map((point) => ({ x: plot.toX(point.time), y: plot.toY(point.value) }));
  const polygon = ring.length === 0 ? "" : `M${ring.map((point) => `${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(" L")} Z`;

  const overlay: BandOverlay = {
    id: `band-${selected.variable}`,
    label: `${Math.round(selected.confidence * 100)}% band`,
    variable: selected.variable,
    confidence: selected.confidence,
    polygon,
    color: seriesColor,
    ...(medianPts.length > 0 ? { medianPath: linePath(medianPts, plot.toX, plot.toY) } : {}),
  };

  const visibleIds = new Set(["median", "observed"]);
  const chartSeries: readonly Series[] = lines
    .filter((line) => visibleIds.has(line.id))
    .map((line) => ({ id: line.id, label: line.label, points: line.points.map((point) => ({ x: point.x, y: point.y })), ...(line.color === undefined ? {} : { color: line.color }) }));
  const chart = createChartModel({
    title: `Uncertainty · ${selected.variable}`,
    series: chartSeries,
    xLabel: world.time.symbol ?? "t",
    yLabel: selected.variable,
  });
  const geometry = { ...plot.geometry, paths: plot.geometry.paths.filter((path) => visibleIds.has(path.id)) };

  const controls: readonly ControlField[] = [
    {
      id: "unc:variable",
      label: "Band variable",
      kind: "select",
      value: selected.variable,
      options: bands.map((band) => ({ value: band.variable, label: band.variable })),
    },
  ];
  sections.push({ kind: "controls", id: "uncertainty-controls", title: "Band", fields: controls });
  sections.push({ kind: "chart", id: "uncertainty-chart", title: "Trajectory with band", chart, geometry, bands: [overlay] });

  const rows: readonly TableRow[] = bands.map((band) => ({
    id: band.variable,
    selected: band.variable === selected.variable,
    cells: [band.variable, `${Math.round(band.confidence * 100)}%`, String(band.times.length)],
  }));
  sections.push({
    kind: "table",
    id: "bands",
    title: "Bands",
    columns: [
      { key: "variable", label: "Variable" },
      { key: "confidence", label: "Confidence", align: "end" },
      { key: "points", label: "Points", align: "end" },
    ],
    rows,
  });

  return { id: "uncertainty-lens", title: "Uncertainty Lens", subtitle: "Overlay confidence bands on a trajectory", sections };
}
