import type { RunStatus } from "@lawsynth/api-client";
import type { Intervention, WorldDefinition } from "@lawsynth/world-schema";
import {
  parametersForWorld,
  trajectoryPlotGeometry,
  trajectoryView,
  validateParameterOverrides,
  type ParameterRow,
} from "@lawsynth/world-viewer";
import { forwardEuler } from "./simulate.js";
import type { ControlField, Metric, Notice, ScreenModel, ScreenSection } from "./types.js";

export interface WorldLabInput {
  readonly world: WorldDefinition;
  readonly initialState: Readonly<Record<string, number>>;
  readonly overrides: Readonly<Record<string, number>>;
  readonly activeInterventionIds: readonly string[];
  readonly horizon: number;
  readonly step: number;
  readonly width?: number;
  readonly height?: number;
  readonly simulationStatus?: RunStatus;
  readonly running: boolean;
}

function parameterControl(row: ParameterRow, override: number | undefined): ControlField {
  const value = override ?? row.value;
  const hasBounds = row.lower !== undefined && row.upper !== undefined;
  const step = hasBounds ? Number(((row.upper! - row.lower!) / 100).toPrecision(2)) : 0.1;
  return {
    id: `lab:param:${row.id}`,
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

function interventionControl(intervention: Intervention, active: boolean): ControlField {
  const at = intervention.time === undefined ? "" : ` @ t=${intervention.time}`;
  return {
    id: `lab:int:${intervention.id}`,
    label: `${intervention.kind} ${intervention.id}${at}`,
    kind: "toggle",
    value: active,
    ...(intervention.description === undefined ? {} : { help: intervention.description }),
  };
}

/**
 * Simulate/forecast the current world with adjustable parameters and
 * interventions. The control panel reuses `world-viewer` parameter rows; the
 * trajectory is integrated locally by `forwardEuler` and rendered through the
 * `world-viewer` + `chart-core` trajectory geometry pipeline.
 */
export function worldLabModel(input: WorldLabInput): ScreenModel {
  const { world } = input;
  const rows = parametersForWorld(world);
  const activeIds = new Set(input.activeInterventionIds);
  const interventions = world.interventions ?? [];
  const active = interventions.filter((intervention) => activeIds.has(intervention.id));

  const controls: ControlField[] = [
    { id: "lab:horizon", label: "Horizon", kind: "number", value: input.horizon, min: input.step, step: input.step, unit: world.time.unit ?? "" },
    { id: "lab:step", label: "Step", kind: "number", value: input.step, min: 0.001, step: 0.001 },
    ...rows.map((row) => parameterControl(row, input.overrides[row.id])),
    ...interventions.map((intervention) => interventionControl(intervention, activeIds.has(intervention.id))),
  ];

  const sections: ScreenSection[] = [];
  const notices: Notice[] = [];

  const overrideList = Object.entries(input.overrides).map(([id, value]) => ({ id, value }));
  try {
    validateParameterOverrides(world, overrideList);
  } catch (error) {
    notices.push({ tone: "error", message: error instanceof Error ? error.message : "Invalid parameter override." });
  }

  let finalValues: readonly number[] | undefined;
  let sampleCount = 0;
  let variables: readonly string[] = [];
  if (notices.length === 0) {
    try {
      const forecast = forwardEuler(world, {
        horizon: input.horizon,
        step: input.step,
        initialState: input.initialState,
        overrides: input.overrides,
        interventions: active,
      });
      const view = trajectoryView(forecast, "Forecast");
      const geometry = trajectoryPlotGeometry(view.chart, input.width ?? 760, input.height ?? 320);
      sampleCount = view.sampleCount;
      variables = forecast.variables;
      finalValues = forecast.values[forecast.values.length - 1];
      sections.push({ kind: "chart", id: "lab-chart", title: "Forecast trajectory", chart: view.chart, geometry });
    } catch (error) {
      notices.push({ tone: "warning", message: error instanceof Error ? error.message : "Forecast failed." });
    }
  }

  const metrics: readonly Metric[] = [
    { label: "Horizon", value: `${input.horizon} ${world.time.unit ?? ""}`.trim() },
    { label: "Samples", value: String(sampleCount) },
    { label: "Active interventions", value: String(active.length) },
    { label: "Final state", value: finalValues === undefined ? "—" : variables.map((name, i) => `${name}=${(finalValues![i] ?? 0).toFixed(3)}`).join(", ") },
  ];

  const ordered: ScreenSection[] = [];
  if (notices.length > 0) ordered.push({ kind: "notices", id: "lab-notices", notices });
  ordered.push({ kind: "metrics", id: "lab-metrics", title: "Forecast", metrics });
  ordered.push(...sections);
  ordered.push({ kind: "controls", id: "lab-controls", title: "Parameters & interventions", fields: controls });
  ordered.push({
    kind: "actions",
    id: "lab-actions",
    buttons: [
      { id: "lab:simulate", label: input.running ? "Simulating…" : "Run simulation", tone: "success", disabled: input.running },
      { id: "lab:reset", label: "Reset overrides" },
    ],
  });

  return { id: "world-lab", title: "World Lab", subtitle: "Simulate, forecast, and intervene on the current world", sections: ordered };
}
