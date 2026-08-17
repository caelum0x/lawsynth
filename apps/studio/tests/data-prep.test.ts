import { dataPrepModel, defaultDataPrepConfig, prepareDataset } from "../src/screens/index.js";
import type { TrajectoryInput } from "@lawsynth/chart-core";
import { equal, world } from "./support.js";

function ramp(): TrajectoryInput {
  const times = [0, 1, 2, 3, 4, 5];
  // Two columns: a noisy ramp and a spike, so smoothing + detrend have an effect.
  const values = times.map((t) => [t + (t % 2 === 0 ? 0.5 : -0.5), Math.sin(t)]);
  return { variables: ["prey", "predator"], times, values };
}

export async function dataPrepTests(): Promise<void> {
  // prepareDataset applies the requested stages in order and reports them.
  const result = prepareDataset(ramp(), { smoothingWindow: 3, resampleDt: 0, detrend: true, trim: 1 });
  equal(result.rowsIn, 6);
  equal(result.rowsOut, 4, "trim drops one row from each end"); // 6 - 2 = 4
  equal(result.opsApplied.length, 3, "trim + smooth + detrend ran, resample skipped");
  // Detrend removes the linear trend => prepared 'prey' column sums to ~0.
  const preyIndex = result.prepared.variables.indexOf("prey");
  const sum = result.prepared.values.reduce((acc, row) => acc + (row[preyIndex] ?? 0), 0);
  equal(Math.abs(sum) < 1e-6, true, "detrended column is mean-centered");

  // Identity config (window 1, no resample/detrend/trim) leaves rows unchanged.
  const identity = prepareDataset(ramp(), { smoothingWindow: 1, resampleDt: 0, detrend: false, trim: 0 });
  equal(identity.opsApplied.length, 0);
  equal(identity.rowsOut, identity.rowsIn);

  // The screen model overlays raw vs prepared per column (2 series + legend each).
  const model = dataPrepModel({ world, initialState: { prey: 1, predator: 1 }, config: defaultDataPrepConfig(), trajectory: ramp() });
  equal(model.id, "data-prep");
  const charts = model.sections.filter((section) => section.kind === "chart");
  equal(charts.length, 2, "one overlay chart per column");
  const first = charts[0];
  if (first?.kind === "chart") {
    equal(first.chart.series.length, 2);
    equal(first.legend?.length, 2);
  }
  // Apply/reset actions are present so the loop can be closed from the UI.
  const actions = model.sections.find((section) => section.kind === "actions");
  equal(actions?.kind, "actions");
  if (actions?.kind === "actions") equal(actions.buttons.some((button) => button.id === "prep:apply"), true);
}
