import { monitorModel, shockDataset } from "../src/screens/index.js";
import { forwardEuler } from "../src/screens/index.js";
import { equal, world } from "./support.js";

export async function monitorTests(): Promise<void> {
  // shockDataset injects a decaying bump into one state, leaving the grid intact.
  const baseline = forwardEuler(world, { horizon: 6, step: 0.5, initialState: { prey: 1 } });
  const shocked = shockDataset(baseline, { variableIndex: 0, startFraction: 0.5, magnitude: 2 });
  equal(shocked.times.length, baseline.times.length);
  const lastRow = shocked.values.length - 1;
  equal((shocked.values[lastRow]?.[0] ?? 0) > (baseline.values[lastRow]?.[0] ?? 0), true, "shock raises the state late in the window");

  // Seeded monitor: model vs shock-injected data flags anomalies and verdicts drift.
  const model = monitorModel({ world, initialState: { prey: 1 }, config: { threshold: 2, step: 0.5, source: "seeded" } });
  equal(model.id, "monitor");

  const anomalies = model.sections.find((section) => section.kind === "table" && section.id === "mon-anomalies");
  equal(anomalies?.kind, "table");
  if (anomalies?.kind === "table") equal(anomalies.rows.length > 0, true, "the seeded shock produces flagged anomalies");

  // Health metrics carry a verdict that is not in-control once a shock is present.
  const metrics = model.sections.find((section) => section.kind === "metrics");
  if (metrics?.kind === "metrics") {
    const verdict = metrics.metrics.find((metric) => metric.label === "Verdict");
    equal(verdict?.value !== "in-control", true, "a shocked system is flagged");
  }

  // Residual strip is a chart carrying the ±threshold band overlay.
  const residual = model.sections.find((section) => section.kind === "chart" && section.id === "mon-residuals");
  equal(residual?.kind, "chart");
  if (residual?.kind === "chart") equal((residual.bands?.length ?? 0) > 0, true, "residual strip shades the in-control band");

  // Observed-vs-predicted overlay: two series (observed + predicted) with a legend.
  const overlay = model.sections.find((section) => section.kind === "chart" && section.id === "mon-overlay-prey");
  equal(overlay?.kind, "chart");
  if (overlay?.kind === "chart") {
    equal(overlay.chart.series.length, 2);
    equal(overlay.legend?.length, 2);
  }

  // A clean run (predicted == observed) is in-control with no anomalies.
  const clean = monitorModel({ world, initialState: { prey: 1 }, config: { threshold: 3, step: 0.5, source: "workspace" }, observed: baseline });
  const cleanMetrics = clean.sections.find((section) => section.kind === "metrics");
  if (cleanMetrics?.kind === "metrics") {
    const verdict = cleanMetrics.metrics.find((metric) => metric.label === "Verdict");
    equal(verdict?.value, "in-control", "matching observed data is in control");
  }
}
