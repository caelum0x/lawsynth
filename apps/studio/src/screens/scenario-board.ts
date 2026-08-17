import { categoricalColor, createChartModel, type Series, type TrajectoryInput } from "@lawsynth/chart-core";
import type { Intervention, WorldDefinition } from "@lawsynth/world-schema";
import {
  parametersForWorld,
  validateParameterOverrides,
  type ParameterRow,
} from "@lawsynth/world-viewer";
import { buildPlot, type PlotLine, type PlotPoint } from "./geometry.js";
import { forwardEuler } from "./simulate.js";
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
 * A named what-if for the current world: a set of parameter overrides and a
 * selection of the world's interventions to activate. Definitions are immutable
 * and owned by the controller (mirroring the World Lab's lab state); the board
 * simulates each one against an implicit unperturbed baseline.
 */
export interface ScenarioDefinition {
  readonly id: string;
  readonly name: string;
  readonly overrides: Readonly<Record<string, number>>;
  readonly activeInterventionIds: readonly string[];
}

/** The in-progress scenario being composed via the parameter/intervention controls. */
export interface ScenarioDraft {
  readonly name: string;
  readonly overrides: Readonly<Record<string, number>>;
  readonly activeInterventionIds: readonly string[];
}

export interface ScenarioBoardInput {
  readonly world: WorldDefinition;
  readonly initialState: Readonly<Record<string, number>>;
  readonly scenarios: readonly ScenarioDefinition[];
  readonly draft: ScenarioDraft;
  readonly horizon: number;
  readonly step: number;
  readonly focusVariableId?: string;
  readonly selectedScenarioId?: string;
  readonly width?: number;
  readonly height?: number;
}

/** Stable identifier + color for the implicit baseline row/series. */
export const BASELINE_SCENARIO_ID = "baseline";

interface ScenarioRun {
  readonly id: string;
  readonly name: string;
  readonly isBaseline: boolean;
  readonly color: string;
  readonly trajectory: TrajectoryInput;
  readonly finalState: Readonly<Record<string, number>>;
  readonly divergence: number;
}

function scenarioColor(id: string, isBaseline: boolean): string {
  return categoricalColor(isBaseline ? "scenario:baseline" : `scenario:${id}`);
}

function finalStateOf(trajectory: TrajectoryInput): Readonly<Record<string, number>> {
  const last = trajectory.values[trajectory.values.length - 1] ?? [];
  const state: Record<string, number> = {};
  trajectory.variables.forEach((name, index) => (state[name] = last[index] ?? 0));
  return Object.freeze(state);
}

function euclideanDivergence(
  a: Readonly<Record<string, number>>,
  b: Readonly<Record<string, number>>,
  variables: readonly string[],
): number {
  let sum = 0;
  for (const name of variables) {
    const delta = (a[name] ?? 0) - (b[name] ?? 0);
    sum += delta * delta;
  }
  return Math.sqrt(sum);
}

function activeInterventions(world: WorldDefinition, ids: readonly string[]): readonly Intervention[] {
  const set = new Set(ids);
  return (world.interventions ?? []).filter((intervention) => set.has(intervention.id));
}

/**
 * Suggests up to two illustrative scenarios by nudging the world's adjustable
 * parameters toward their bounds, so a freshly loaded world already has
 * something to compare. Returns `[]` when every parameter is fixed.
 */
export function defaultScenarios(world: WorldDefinition): readonly ScenarioDefinition[] {
  const adjustable = parametersForWorld(world).filter((row) => !row.fixed);
  const scenarios: ScenarioDefinition[] = [];
  const high = adjustable[0];
  if (high !== undefined) {
    const value = high.upper !== undefined ? Number(((high.value + high.upper) / 2).toPrecision(4)) : high.value * 1.5;
    scenarios.push({ id: `high-${high.id}`, name: `High ${high.id}`, overrides: Object.freeze({ [high.id]: value }), activeInterventionIds: [] });
  }
  const low = adjustable[1] ?? adjustable[0];
  if (low !== undefined) {
    const value = low.lower !== undefined ? Number(((low.value + low.lower) / 2).toPrecision(4)) : low.value * 0.5;
    scenarios.push({ id: `low-${low.id}`, name: `Low ${low.id}`, overrides: Object.freeze({ [low.id]: value }), activeInterventionIds: [] });
  }
  return Object.freeze(scenarios);
}

function draftParameterControl(row: ParameterRow, override: number | undefined): ControlField {
  const value = override ?? row.value;
  const hasBounds = row.lower !== undefined && row.upper !== undefined;
  const step = hasBounds ? Number(((row.upper! - row.lower!) / 100).toPrecision(2)) : 0.1;
  return {
    id: `board:param:${row.id}`,
    label: row.description ? `${row.id} — ${row.description}` : row.id,
    kind: hasBounds ? "range" : "number",
    value,
    disabled: row.fixed,
    step,
    ...(row.lower === undefined ? {} : { min: row.lower }),
    ...(row.upper === undefined ? {} : { max: row.upper }),
    ...(row.unit === undefined ? {} : { unit: row.unit }),
  };
}

function draftInterventionControl(intervention: Intervention, active: boolean): ControlField {
  const at = intervention.time === undefined ? "" : ` @ t=${intervention.time}`;
  return {
    id: `board:int:${intervention.id}`,
    label: `${intervention.kind} ${intervention.id}${at}`,
    kind: "toggle",
    value: active,
    ...(intervention.description === undefined ? {} : { help: intervention.description }),
  };
}

/**
 * Scenario Board — a visual decision surface for comparing what-if scenarios.
 *
 * Each named scenario (plus an implicit unperturbed baseline) is integrated
 * locally by `forwardEuler` over the world's continuous laws. A multi-series
 * chart overlays one line per scenario for the focused state variable, and a
 * comparison table reports each scenario's final state and its divergence from
 * baseline, with the selected scenario highlighted as the current choice.
 */
export function scenarioBoardModel(input: ScenarioBoardInput): ScreenModel {
  const { world } = input;
  const rows = parametersForWorld(world);
  const interventions = world.interventions ?? [];
  const draftActive = new Set(input.draft.activeInterventionIds);
  const sections: ScreenSection[] = [];
  const notices: Notice[] = [];

  // Validate every scenario's overrides up front so an invalid what-if surfaces
  // a message instead of throwing during integration.
  for (const scenario of input.scenarios) {
    try {
      validateParameterOverrides(world, Object.entries(scenario.overrides).map(([id, value]) => ({ id, value })));
    } catch (error) {
      notices.push({ tone: "error", message: `${scenario.name}: ${error instanceof Error ? error.message : "invalid override"}` });
    }
  }

  const simulate = (overrides: Readonly<Record<string, number>>, active: readonly string[]): TrajectoryInput =>
    forwardEuler(world, {
      horizon: input.horizon,
      step: input.step,
      initialState: input.initialState,
      overrides,
      interventions: activeInterventions(world, active),
    });

  const runs: ScenarioRun[] = [];
  let variables: readonly string[] = [];
  let baselineFinal: Readonly<Record<string, number>> = {};

  if (notices.length === 0) {
    try {
      const baselineTrajectory = simulate({}, []);
      variables = baselineTrajectory.variables;
      baselineFinal = finalStateOf(baselineTrajectory);
      runs.push({ id: BASELINE_SCENARIO_ID, name: "Baseline", isBaseline: true, color: scenarioColor(BASELINE_SCENARIO_ID, true), trajectory: baselineTrajectory, finalState: baselineFinal, divergence: 0 });
      for (const scenario of input.scenarios) {
        const trajectory = simulate(scenario.overrides, scenario.activeInterventionIds);
        const finalState = finalStateOf(trajectory);
        runs.push({
          id: scenario.id,
          name: scenario.name,
          isBaseline: false,
          color: scenarioColor(scenario.id, false),
          trajectory,
          finalState,
          divergence: euclideanDivergence(finalState, baselineFinal, variables),
        });
      }
    } catch (error) {
      notices.push({ tone: "warning", message: error instanceof Error ? error.message : "Scenario simulation failed." });
    }
  }

  const focus = variables.includes(input.focusVariableId ?? "") ? input.focusVariableId! : variables[0] ?? "";
  const selectedId = runs.some((run) => run.id === input.selectedScenarioId) ? input.selectedScenarioId : undefined;

  const metrics: readonly Metric[] = [
    { label: "Scenarios", value: String(input.scenarios.length) },
    { label: "Compared", value: String(runs.length) },
    { label: "Baseline final", value: variables.length === 0 ? "—" : variables.map((name) => `${name}=${(baselineFinal[name] ?? 0).toFixed(3)}`).join(", ") },
    { label: "Selected", value: selectedId === undefined ? "—" : runs.find((run) => run.id === selectedId)?.name ?? "—" },
  ];

  // ── Multi-series overlay: one line per scenario for the focused state ──────
  if (runs.length > 0 && focus !== "") {
    const lines: PlotLine[] = runs.map((run) => {
      const index = run.trajectory.variables.indexOf(focus);
      const points: readonly PlotPoint[] = run.trajectory.times.map((time, i) => ({ x: time, y: run.trajectory.values[i]?.[index] ?? 0 }));
      return { id: run.id, label: run.name, color: run.color, points };
    });
    const width = input.width ?? 760;
    const height = input.height ?? 320;
    const plot = buildPlot(lines, width, height);
    const chartSeries: readonly Series[] = lines.map((line) => ({
      id: line.id,
      label: line.label,
      points: line.points.map((point) => ({ x: point.x, y: point.y })),
      ...(line.color === undefined ? {} : { color: line.color }),
    }));
    const chart = createChartModel({ title: `Scenarios · ${focus}`, series: chartSeries, xLabel: world.time.symbol ?? "t", yLabel: focus });
    const legend: readonly ChartLegendEntry[] = runs.map((run) => ({ id: run.id, label: run.name, color: run.color, ...(run.id === selectedId ? { emphasis: true } : {}) }));
    sections.push({ kind: "chart", id: "board-chart", title: `Overlay · ${focus}`, chart, geometry: plot.geometry, legend });
  }

  // ── Comparison table: final state per variable + divergence from baseline ──
  const columns: TableColumn[] = [
    { key: "scenario", label: "Scenario" },
    ...variables.map((name): TableColumn => ({ key: `final:${name}`, label: `${name} final`, align: "end" })),
    { key: "divergence", label: "Δ baseline", align: "end" },
  ];
  const tableRows: readonly TableRow[] = runs.map((run) => ({
    id: run.id,
    selected: run.id === selectedId,
    emphasis: run.isBaseline,
    cells: [
      run.name,
      ...variables.map((name) => (run.finalState[name] ?? 0).toFixed(3)),
      run.isBaseline ? "—" : run.divergence.toFixed(3),
    ],
  }));

  // ── Controls: focus + horizon/step + the draft scenario being composed ────
  const controls: ControlField[] = [
    ...(variables.length > 0
      ? [{ id: "board:focus", label: "Focus state", kind: "select" as const, value: focus, options: variables.map((name) => ({ value: name, label: name })) }]
      : []),
    { id: "board:horizon", label: "Horizon", kind: "number", value: input.horizon, min: input.step, step: input.step, unit: world.time.unit ?? "" },
    { id: "board:step", label: "Step", kind: "number", value: input.step, min: 0.001, step: 0.001 },
    { id: "board:name", label: "New scenario name", kind: "text", value: input.draft.name },
    ...rows.map((row) => draftParameterControl(row, input.draft.overrides[row.id])),
    ...interventions.map((intervention) => draftInterventionControl(intervention, draftActive.has(intervention.id))),
  ];

  const ordered: ScreenSection[] = [];
  if (notices.length > 0) ordered.push({ kind: "notices", id: "board-notices", notices });
  ordered.push({ kind: "metrics", id: "board-metrics", title: "Decision surface", metrics });
  ordered.push(...sections);
  ordered.push({ kind: "table", id: "scenarios", title: "Comparison", columns, rows: tableRows, empty: "No scenarios defined yet — compose one below and add it." });
  ordered.push({ kind: "controls", id: "board-controls", title: "Define a scenario", fields: controls });
  ordered.push({
    kind: "actions",
    id: "board-actions",
    buttons: [
      { id: "board:add", label: "Add scenario", tone: "success" },
      { id: "board:remove", label: "Remove selected", disabled: selectedId === undefined || selectedId === BASELINE_SCENARIO_ID },
      { id: "board:reset", label: "Reset draft" },
    ],
  });

  return { id: "scenario-board", title: "Scenario Board", subtitle: "Compare what-if scenarios against a baseline", sections: ordered };
}
