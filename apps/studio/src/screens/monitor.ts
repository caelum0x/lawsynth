import { categoricalColor, createChartModel, type Series, type TrajectoryInput } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { shockDataset } from "./fixtures.js";
import { bandPolygon, buildPlot, type PlotLine, type PlotPoint } from "./geometry.js";
import { forwardEuler } from "./simulate.js";
import { interpolateAt, standardize } from "./signal.js";
import type {
  BandOverlay,
  ChartLegendEntry,
  ControlField,
  Metric,
  Notice,
  NoticeTone,
  ScreenModel,
  ScreenSection,
  TableColumn,
  TableRow,
} from "./types.js";

export type MonitorSource = "seeded" | "workspace";

export const MONITOR_SOURCES: readonly MonitorSource[] = Object.freeze(["seeded", "workspace"]);

/** Threshold slider + comparison-window knobs for the anomaly monitor. */
export interface MonitorConfig {
  readonly threshold: number;
  readonly step: number;
  readonly source: MonitorSource;
}

export function defaultMonitorConfig(): MonitorConfig {
  return Object.freeze({ threshold: 3, step: 0.1, source: "seeded" });
}

export interface MonitorInput {
  readonly world: WorldDefinition;
  readonly initialState: Readonly<Record<string, number>>;
  readonly config: MonitorConfig;
  readonly observed?: TrajectoryInput;
  readonly horizon?: number;
  readonly selectedAnomalyId?: string;
  readonly width?: number;
  readonly height?: number;
}

/** One flagged timestamp: the state whose standardized residual most exceeds the threshold there. */
export interface AnomalyFlag {
  readonly id: string;
  readonly rowIndex: number;
  readonly time: number;
  readonly variable: string;
  readonly z: number;
  readonly residual: number;
}

type Verdict = "in-control" | "watch" | "drift";

const RESIDUAL_BAND_ID = "__thr";

function columnOf(trajectory: TrajectoryInput, index: number): readonly number[] {
  return trajectory.times.map((_, row) => trajectory.values[row]?.[index] ?? 0);
}

function verdictTone(verdict: Verdict): NoticeTone {
  return verdict === "in-control" ? "success" : verdict === "watch" ? "warning" : "error";
}

function longestRun(flags: readonly boolean[]): number {
  let best = 0;
  let current = 0;
  for (const flag of flags) {
    current = flag ? current + 1 : 0;
    if (current > best) best = current;
  }
  return best;
}

/**
 * Monitor — the "is my system still behaving?" screen. It answers that question
 * with the model itself: it integrates the current world forward with
 * `forwardEuler` and compares the prediction against a comparison dataset (the
 * shared working dataset, or a seeded shock-injected fixture). For every state
 * it aligns prediction to the observed grid by linear interpolation, forms the
 * residual (observed − predicted), and standardizes it into a z-score. Rows
 * whose worst |z| exceeds the threshold slider are flagged; a run of consecutive
 * flags escalates the verdict from in-control to drift.
 */
export function monitorModel(input: MonitorInput): ScreenModel {
  const { world, config } = input;
  const timeSymbol = world.time.symbol ?? "t";
  const notices: Notice[] = [];
  const sections: ScreenSection[] = [];

  const controls: readonly ControlField[] = [
    { id: "mon:threshold", label: "Anomaly threshold", kind: "range", value: config.threshold, min: 1, max: 6, step: 0.25, unit: "σ", help: "Flag rows whose standardized residual exceeds this." },
    { id: "mon:source", label: "Comparison data", kind: "select", value: config.source, options: [{ value: "seeded", label: "Seeded (shock-injected)" }, { value: "workspace", label: "Working dataset" }], help: "Which observations to score against the model." },
    { id: "mon:step", label: "Prediction step", kind: "number", value: config.step, min: 0.001, step: 0.001 },
  ];

  // ── Prediction: integrate the world over the comparison window ────────────
  let predicted: TrajectoryInput;
  try {
    const horizon = input.horizon ?? (input.observed !== undefined && input.observed.times.length > 1
      ? (input.observed.times[input.observed.times.length - 1] ?? 12) - (input.observed.times[0] ?? 0)
      : 12);
    predicted = forwardEuler(world, { horizon: horizon > 0 ? horizon : 12, step: config.step, initialState: input.initialState });
  } catch (error) {
    notices.push({ tone: "error", message: `Model could not be simulated: ${error instanceof Error ? error.message : "unknown error"}.` });
    sections.push({ kind: "notices", id: "mon-notices", notices });
    sections.push({ kind: "controls", id: "mon-controls", title: "Monitoring", fields: controls });
    return { id: "monitor", title: "Monitor", subtitle: "Model-based anomaly detection on new data", sections };
  }

  // ── Observed comparison dataset ───────────────────────────────────────────
  const workspaceReady = input.observed !== undefined && input.observed.times.length > 0;
  const useWorkspace = config.source === "workspace" && workspaceReady;
  const observed: TrajectoryInput = useWorkspace ? input.observed! : shockDataset(predicted, { variableIndex: 0 });
  if (config.source === "workspace" && !workspaceReady) notices.push({ tone: "warning", message: "No working dataset available — scoring against the seeded shock fixture instead." });

  const states = predicted.variables.filter((variable) => observed.variables.includes(variable));
  if (states.length === 0 || observed.times.length === 0) {
    notices.push({ tone: "info", message: "No shared state columns to monitor between the model and the comparison data." });
    sections.push({ kind: "notices", id: "mon-notices", notices });
    sections.push({ kind: "controls", id: "mon-controls", title: "Monitoring", fields: controls });
    return { id: "monitor", title: "Monitor", subtitle: "Model-based anomaly detection on new data", sections };
  }

  const times = observed.times;
  // Predicted aligned onto the observed grid + standardized residuals, per state.
  const predictedAligned = new Map<string, readonly number[]>();
  const observedByVar = new Map<string, readonly number[]>();
  const residualZ = new Map<string, readonly number[]>();
  for (const variable of states) {
    const predColumn = columnOf(predicted, predicted.variables.indexOf(variable));
    const obsColumn = columnOf(observed, observed.variables.indexOf(variable));
    const aligned = times.map((t) => interpolateAt(predicted.times, predColumn, t));
    const residual = obsColumn.map((value, i) => value - (aligned[i] ?? 0));
    predictedAligned.set(variable, aligned);
    observedByVar.set(variable, obsColumn);
    residualZ.set(variable, standardize(residual).values);
  }

  // ── Flag the worst state per timestamp beyond the threshold ───────────────
  const rowFlagged: boolean[] = [];
  const anomalies: AnomalyFlag[] = [];
  let maxAbsZ = 0;
  times.forEach((time, row) => {
    let worst = 0;
    let worstVar = states[0]!;
    for (const variable of states) {
      const z = residualZ.get(variable)?.[row] ?? 0;
      if (Math.abs(z) > Math.abs(worst)) { worst = z; worstVar = variable; }
    }
    maxAbsZ = Math.max(maxAbsZ, Math.abs(worst));
    const flagged = Math.abs(worst) > config.threshold;
    rowFlagged.push(flagged);
    if (flagged) {
      const observedValue = observedByVar.get(worstVar)?.[row] ?? 0;
      const predictedValue = predictedAligned.get(worstVar)?.[row] ?? 0;
      anomalies.push({ id: `t${row}`, rowIndex: row, time, variable: worstVar, z: worst, residual: observedValue - predictedValue });
    }
  });

  const run = longestRun(rowFlagged);
  const verdict: Verdict = anomalies.length === 0 ? "in-control" : run >= 3 ? "drift" : "watch";
  const selectedId = anomalies.some((flag) => flag.id === input.selectedAnomalyId) ? input.selectedAnomalyId : undefined;

  if (verdict === "in-control") notices.push({ tone: "success", message: "System is in control — no residuals exceed the threshold." });
  else if (verdict === "watch") notices.push({ tone: "warning", message: `${anomalies.length} isolated anomaly(ies) detected — watch for drift.` });
  else notices.push({ tone: "error", message: `Drift detected — ${run} consecutive flagged samples.` });
  sections.push({ kind: "notices", id: "mon-notices", notices });

  const metrics: readonly Metric[] = [
    { label: "Verdict", value: verdict, tone: verdictTone(verdict) },
    { label: "Anomalies", value: String(anomalies.length) },
    { label: "Max |z|", value: maxAbsZ.toFixed(2) },
    { label: "Threshold", value: `${config.threshold.toFixed(2)} σ` },
    { label: "Samples", value: String(times.length) },
  ];
  sections.push({ kind: "metrics", id: "mon-metrics", title: "Health", metrics });

  // ── Observed vs predicted overlay, per state ──────────────────────────────
  const width = input.width ?? 420;
  const height = input.height ?? 170;
  for (const variable of states) {
    const observedPoints: readonly PlotPoint[] = times.map((time, row) => ({ x: time, y: observedByVar.get(variable)?.[row] ?? 0 }));
    const predictedPoints: readonly PlotPoint[] = times.map((time, row) => ({ x: time, y: predictedAligned.get(variable)?.[row] ?? 0 }));
    const observedColor = categoricalColor(variable);
    const lines: readonly PlotLine[] = [
      { id: "predicted", label: "predicted", color: "#8a9089", points: predictedPoints },
      { id: "observed", label: "observed", color: observedColor, points: observedPoints },
    ];
    const plot = buildPlot(lines, width, height);
    const series: readonly Series[] = lines.map((line) => ({ id: line.id, label: line.label, points: line.points.map((point) => ({ x: point.x, y: point.y })), ...(line.color === undefined ? {} : { color: line.color }) }));
    const chart = createChartModel({ title: `${variable}: observed vs predicted`, series, xLabel: timeSymbol, yLabel: variable });
    const legend: readonly ChartLegendEntry[] = [
      { id: "predicted", label: "predicted", color: "#8a9089" },
      { id: "observed", label: "observed", color: observedColor, emphasis: true },
    ];
    sections.push({ kind: "chart", id: `mon-overlay-${variable}`, title: `${variable} · observed vs predicted`, chart, geometry: plot.geometry, legend });
  }

  // ── Residual strip: standardized residual per state + ±threshold band ─────
  const residualLines: PlotLine[] = states.map((variable) => ({
    id: `res:${variable}`,
    label: variable,
    color: categoricalColor(variable),
    points: times.map((time, row) => ({ x: time, y: residualZ.get(variable)?.[row] ?? 0 })),
  }));
  const tMin = times[0] ?? 0;
  const tMax = times[times.length - 1] ?? tMin + 1;
  // A transparent envelope forces the plot domain to include ±threshold, then is
  // stripped from the rendered paths — only the residual lines and shaded band draw.
  const envelope: PlotLine = { id: RESIDUAL_BAND_ID, label: "", color: "transparent", points: [{ x: tMin, y: config.threshold }, { x: tMax, y: -config.threshold }] };
  const stripPlot = buildPlot([...residualLines, envelope], width, height);
  const stripGeometry = { ...stripPlot.geometry, paths: stripPlot.geometry.paths.filter((path) => path.id !== RESIDUAL_BAND_ID) };
  const upper: readonly PlotPoint[] = [{ x: tMin, y: config.threshold }, { x: tMax, y: config.threshold }];
  const lower: readonly PlotPoint[] = [{ x: tMin, y: -config.threshold }, { x: tMax, y: -config.threshold }];
  const band: BandOverlay = { id: "thr-band", label: `±${config.threshold}σ`, variable: "", confidence: 0, polygon: bandPolygon(upper, lower, stripPlot.toX, stripPlot.toY), color: "#c58a1e" };
  const residualSeries: readonly Series[] = residualLines.map((line) => ({ id: line.id, label: line.label, points: line.points.map((point) => ({ x: point.x, y: point.y })), ...(line.color === undefined ? {} : { color: line.color }) }));
  const residualChart = createChartModel({ title: "Standardized residuals", series: residualSeries, xLabel: timeSymbol, yLabel: "z-score" });
  const residualLegend: readonly ChartLegendEntry[] = residualLines.map((line) => ({ id: line.id, label: line.label, color: line.color ?? "#000000" }));
  sections.push({ kind: "chart", id: "mon-residuals", title: "Residual strip (in-control band shaded)", chart: residualChart, geometry: stripGeometry, bands: [band], legend: residualLegend });

  // ── Flagged anomalies list (selectable → shared store) ────────────────────
  const columns: readonly TableColumn[] = [
    { key: "time", label: timeSymbol, align: "end" },
    { key: "variable", label: "State" },
    { key: "z", label: "z", align: "end" },
    { key: "residual", label: "Residual", align: "end" },
  ];
  const rows: readonly TableRow[] = anomalies.map((flag) => ({
    id: flag.id,
    selected: flag.id === selectedId,
    cells: [flag.time.toFixed(3), flag.variable, flag.z.toFixed(2), flag.residual.toFixed(3)],
  }));
  sections.push({ kind: "table", id: "mon-anomalies", title: "Flagged anomalies", columns, rows, empty: "No anomalies flagged at the current threshold." });

  sections.push({ kind: "controls", id: "mon-controls", title: "Monitoring", fields: controls });

  return { id: "monitor", title: "Monitor", subtitle: "Model-based anomaly detection on new data", sections };
}
