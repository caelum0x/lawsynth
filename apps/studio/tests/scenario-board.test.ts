import { BASELINE_SCENARIO_ID, defaultScenarios, scenarioBoardModel } from "../src/screens/index.js";
import { equal, world } from "./support.js";

export async function scenarioBoardTests(): Promise<void> {
  const scenarios = defaultScenarios(world);
  // The support world exposes one adjustable parameter (`growth`), so the board
  // seeds a high/low pair of illustrative scenarios.
  equal(scenarios.length, 2);

  const model = scenarioBoardModel({
    world,
    initialState: { prey: 1 },
    scenarios,
    draft: { name: "", overrides: {}, activeInterventionIds: [] },
    horizon: 5,
    step: 1,
    selectedScenarioId: scenarios[0]!.id,
  });
  equal(model.id, "scenario-board");

  // Multi-series overlay: one labeled series (with legend) per scenario plus baseline.
  const chart = model.sections.find((section) => section.kind === "chart");
  equal(chart?.kind, "chart");
  if (chart?.kind === "chart") {
    equal(chart.legend?.length, scenarios.length + 1);
    equal(chart.chart.series.length, scenarios.length + 1);
  }

  // Comparison table: a baseline row plus one row per scenario, baseline first.
  const table = model.sections.find((section) => section.kind === "table" && section.id === "scenarios");
  equal(table?.kind, "table");
  if (table?.kind === "table") {
    equal(table.rows.length, scenarios.length + 1);
    equal(table.rows[0]?.id, BASELINE_SCENARIO_ID);
    // The selected scenario is highlighted, and it has diverged from baseline.
    const selected = table.rows.find((row) => row.selected === true);
    equal(selected?.id, scenarios[0]!.id);
    equal(Number(selected?.cells[selected.cells.length - 1]) > 0, true);
  }
}
