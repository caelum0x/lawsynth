import { categoricalColor, createChartModel, type Series, type TrajectoryInput } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { buildPlot, type PlotLine } from "./geometry.js";
import { forwardEuler } from "./simulate.js";
import { detrend, mapColumns, movingAverage, resampleTrajectory, trimTrajectory, uniformGrid } from "./signal.js";
import type {
  ChartLegendEntry,
  ControlField,
  Metric,
  Notice,
  ScreenModel,
  ScreenSection,
  TableColumn,
  TableRow,
} from "./types.js";

/**
 * The four deterministic preparation knobs. Each maps to one local transform in
 * {@link prepareDataset}; the defaults are a light-touch identity-ish pipeline
 * (a 3-wide smooth, no resample, no detrend, no trim) so the prepared series
 * starts close to the raw one.
 */
export interface DataPrepConfig {
  readonly smoothingWindow: number;
  readonly resampleDt: number;
  readonly detrend: boolean;
  readonly trim: number;
}

export function defaultDataPrepConfig(): DataPrepConfig {
  return Object.freeze({ smoothingWindow: 3, resampleDt: 0, detrend: false, trim: 0 });
}

export interface DataPrepResult {
  readonly variables: readonly string[];
  readonly raw: TrajectoryInput;
  readonly prepared: TrajectoryInput;
  readonly opsApplied: readonly string[];
  readonly rowsIn: number;
  readonly rowsOut: number;
}

/**
 * Runs the local preparation pipeline over a working dataset. The stages are
 * applied in a fixed, explainable order so the result is reproducible:
 *   trim window → uniform resample → moving-average smooth → linear detrend.
 * Every stage is a pure transform from `signal.ts`; the returned `opsApplied`
 * lists exactly which stages ran (a knob at its identity value is skipped).
 */
export function prepareDataset(raw: TrajectoryInput, config: DataPrepConfig): DataPrepResult {
  const ops: string[] = [];
  let prepared: TrajectoryInput = raw;

  if (config.trim > 0 && raw.times.length > 2) {
    prepared = trimTrajectory(prepared, config.trim);
    ops.push(`trim ±${Math.floor(config.trim)} rows`);
  }
  if (config.resampleDt > 0 && prepared.times.length >= 2) {
    const start = prepared.times[0] ?? 0;
    const end = prepared.times[prepared.times.length - 1] ?? start;
    prepared = resampleTrajectory(prepared, uniformGrid(start, end, config.resampleDt));
    ops.push(`resample dt=${config.resampleDt}`);
  }
  if (config.smoothingWindow > 1 && prepared.times.length > 0) {
    const width = Math.floor(config.smoothingWindow);
    prepared = mapColumns(prepared, (column) => movingAverage(column, width));
    ops.push(`smooth window=${width}`);
  }
  if (config.detrend && prepared.times.length >= 2) {
    prepared = mapColumns(prepared, (column) => detrend(prepared.times, column));
    ops.push("detrend (linear)");
  }

  return {
    variables: raw.variables.slice(),
    raw,
    prepared,
    opsApplied: Object.freeze(ops),
    rowsIn: raw.times.length,
    rowsOut: prepared.times.length,
  };
}

export interface DataPrepInput {
  readonly world: WorldDefinition;
  readonly initialState: Readonly<Record<string, number>>;
  readonly config: DataPrepConfig;
  readonly trajectory?: TrajectoryInput;
  readonly applied?: boolean;
  readonly horizon?: number;
  readonly step?: number;
  readonly width?: number;
  readonly height?: number;
}

const RAW_COLOR = "#8a9089";

function resolveWorking(input: DataPrepInput): { trajectory: TrajectoryInput; source: string } {
  if (input.trajectory !== undefined && input.trajectory.times.length > 0) {
    return { trajectory: input.trajectory, source: "working dataset" };
  }
  try {
    return {
      trajectory: forwardEuler(input.world, {
        horizon: input.horizon ?? 12,
        step: input.step ?? 0.1,
        initialState: input.initialState,
      }),
      source: "locally integrated",
    };
  } catch {
    return { trajectory: { variables: [], times: [], values: [] }, source: "unavailable" };
  }
}

/**
 * Data Prep — an interactive data-preparation surface that sits between observe
 * and discover. It profiles the working dataset, runs a deterministic local
 * prep pipeline (smoothing / resample / detrend / trim), and overlays the raw
 * and prepared series per column so the effect of each knob is visible. Pressing
 * "Apply" promotes the prepared dataset to the shared working dataset that the
 * Data Lens and Discovery Canvas then operate on — closing the prep → discover
 * loop.
 */
export function dataPrepModel(input: DataPrepInput): ScreenModel {
  const { world, config } = input;
  const timeSymbol = world.time.symbol ?? "t";
  const timeUnit = world.time.unit ?? "";
  const { trajectory: raw, source } = resolveWorking(input);

  const notices: Notice[] = [];
  const sections: ScreenSection[] = [];

  const controls: readonly ControlField[] = [
    { id: "prep:smooth", label: "Smoothing window", kind: "range", value: config.smoothingWindow, min: 1, max: 15, step: 2, help: "Centered moving average width (1 = off)." },
    { id: "prep:dt", label: "Resample dt", kind: "number", value: config.resampleDt, min: 0, step: 0.05, unit: timeUnit, help: "Uniform time step for linear resampling (0 = keep grid)." },
    { id: "prep:trim", label: "Trim rows (each end)", kind: "number", value: config.trim, min: 0, step: 1, help: "Drop leading/trailing samples before prep." },
    { id: "prep:detrend", label: "Remove linear trend", kind: "toggle", value: config.detrend, help: "Subtract the least-squares line per column." },
  ];

  if (raw.variables.length === 0 || raw.times.length === 0) {
    notices.push({ tone: "info", message: "No dataset is available to prepare for this world." });
    sections.push({ kind: "notices", id: "prep-empty", notices });
    sections.push({ kind: "controls", id: "prep-controls", title: "Preparation", fields: controls });
    return { id: "data-prep", title: "Data Prep", subtitle: "Prepare the working dataset before discovery", sections };
  }

  const result = prepareDataset(raw, config);

  if (input.applied === true) notices.push({ tone: "success", message: "Prepared dataset applied — Data Lens and Discovery now use it." });
  if (result.opsApplied.length === 0) notices.push({ tone: "info", message: "No transforms are active — prepared output equals the raw input." });
  if (result.rowsOut < 3) notices.push({ tone: "warning", message: "Prepared dataset has fewer than three rows — discovery quality will be poor." });
  if (notices.length === 0) notices.push({ tone: "info", message: "Adjust the knobs, then Apply to promote the prepared dataset." });
  sections.push({ kind: "notices", id: "prep-notices", notices });

  const metrics: readonly Metric[] = [
    { label: "Rows in", value: String(result.rowsIn) },
    { label: "Rows out", value: String(result.rowsOut) },
    { label: "Ops applied", value: result.opsApplied.length === 0 ? "none" : String(result.opsApplied.length) },
    { label: "Source", value: source },
  ];
  sections.push({ kind: "metrics", id: "prep-metrics", title: "Preparation summary", metrics });

  // ── Per-column overlay: raw (muted) vs prepared (accented) ────────────────
  const width = input.width ?? 360;
  const height = input.height ?? 150;
  raw.variables.forEach((variable, index) => {
    const preparedColumn = raw.variables.indexOf(variable) < result.prepared.variables.length ? result.prepared.variables.indexOf(variable) : -1;
    const rawPoints = raw.times.map((time, row) => ({ x: time, y: raw.values[row]?.[index] ?? 0 }));
    const prepPoints = result.prepared.times.map((time, row) => ({ x: time, y: result.prepared.values[row]?.[preparedColumn === -1 ? index : preparedColumn] ?? 0 }));
    const preparedColor = categoricalColor(variable);
    const lines: readonly PlotLine[] = [
      { id: `${variable}:raw`, label: "raw", color: RAW_COLOR, points: rawPoints },
      { id: `${variable}:prep`, label: "prepared", color: preparedColor, points: prepPoints },
    ];
    const plot = buildPlot(lines, width, height);
    const series: readonly Series[] = lines.map((line) => ({ id: line.id, label: line.label, points: line.points.map((point) => ({ x: point.x, y: point.y })), ...(line.color === undefined ? {} : { color: line.color }) }));
    const chart = createChartModel({ title: `${variable}: raw vs prepared`, series, xLabel: timeSymbol, yLabel: variable });
    const legend: readonly ChartLegendEntry[] = [
      { id: `${variable}:raw`, label: "raw", color: RAW_COLOR },
      { id: `${variable}:prep`, label: "prepared", color: preparedColor, emphasis: true },
    ];
    sections.push({ kind: "chart", id: `prep-chart-${variable}`, title: `${variable} · raw vs prepared`, chart, geometry: plot.geometry, legend });
  });

  // ── Ops table: which stages ran, in order ─────────────────────────────────
  const opColumns: readonly TableColumn[] = [
    { key: "step", label: "#", align: "end" },
    { key: "op", label: "Operation" },
  ];
  const opRows: readonly TableRow[] = result.opsApplied.map((op, index) => ({ id: `op-${index}`, cells: [String(index + 1), op] }));
  sections.push({ kind: "table", id: "prep-ops", title: "Pipeline", columns: opColumns, rows: opRows, empty: "No operations — prepared output equals the raw input." });

  sections.push({ kind: "controls", id: "prep-controls", title: "Preparation", fields: controls });
  sections.push({
    kind: "actions",
    id: "prep-actions",
    buttons: [
      { id: "prep:apply", label: "Apply to working dataset", tone: "success", disabled: result.rowsOut === 0 },
      { id: "prep:reset", label: "Reset knobs" },
    ],
  });

  return { id: "data-prep", title: "Data Prep", subtitle: "Prepare the working dataset before discovery", sections };
}
