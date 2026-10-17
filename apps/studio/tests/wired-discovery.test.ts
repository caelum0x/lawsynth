import { InMemoryDiscoveryTransport, LawSynthClient } from "@lawsynth/api-client";
import { equationView } from "@lawsynth/world-viewer";
import { ScreensController } from "../src/screens/controller.js";
import { equal, store } from "./support.js";

/**
 * Proves the LIVE-wired Studio run flow end to end offline: a discovery submit
 * drives the real api-client through submit -> poll -> world, the discovered
 * world is handed to the screens, and the equation explorer renders its laws.
 */
export async function wiredDiscoveryTests(): Promise<void> {
  const transport = new InMemoryDiscoveryTransport({
    pollsUntilSucceeded: 1,
    world: {
      id: "world_discovered",
      name: "Discovered oscillator",
      states: ["x", "v"],
      equations: { x: "v", v: "-4*x - 0.5*v" },
    },
  });
  const api = new LawSynthClient(transport);
  const controller = new ScreensController({
    store: store(),
    api,
    randomId: () => "idem",
    poll: { sleep: async () => {}, waitMs: 0, maxAttempts: 10 },
  });

  await controller.onAction("discovery:run");

  // Submit -> poll -> world loaded.
  equal(controller.runStatus, "succeeded");
  equal(controller.worldId, "world_discovered");
  equal(controller.world.id, "world_discovered");
  equal(controller.world.laws.length, 2);

  // The discovered laws parsed into the schema AST (dv/dt = -4*x - 0.5*v).
  const dv = controller.world.laws.find((law) => "target" in law && law.target === "v");
  if (dv === undefined) throw new Error("expected a law for dv/dt");
  const dvView = equationView(dv, "t");
  equal(dvView.text.startsWith("dv/dt ="), true);
  equal([...dvView.symbols].sort().join(","), "v,x");

  // The equation explorer screen renders the world's laws.
  controller.setScreen("equation-explorer");
  const model = controller.model();
  const equations = model.sections.find((section) => section.kind === "equations");
  if (equations === undefined || equations.kind !== "equations") throw new Error("equation explorer has no equations section");
  equal(equations.equations.length, 2);
  equal(equations.equations.some((block) => block.text.startsWith("dv/dt =")), true);

  controller.dispose();
}

/**
 * Proves the World Lab "Request live forecast" button is actually wired: the
 * `world:forecast` action id a renderer dispatches from that button's click
 * reaches the live api-client (`worlds.forecast`), and the result is reflected
 * back in the World Lab model rather than being silently dropped.
 */
export async function worldLabForecastTests(): Promise<void> {
  const transport = new InMemoryDiscoveryTransport({
    world: { id: "oscillator-demo", name: "Damped oscillator (demo)", states: ["x", "v"], equations: { x: "v", v: "-4*x - 0.5*v" } },
  });
  const api = new LawSynthClient(transport);
  const controller = new ScreensController({ store: store(), api, randomId: () => "idem" });
  controller.setScreen("world-lab");

  // Before the action runs, the button is present and the live-forecast metric
  // honestly reports nothing has been requested yet.
  const before = controller.model();
  const beforeActions = before.sections.find((section) => section.kind === "actions");
  if (beforeActions === undefined || beforeActions.kind !== "actions") throw new Error("world lab has no actions section");
  const button = beforeActions.buttons.find((candidate) => candidate.id === "world:forecast");
  if (button === undefined) throw new Error("world lab is missing the world:forecast button");
  const beforeMetrics = before.sections.find((section) => section.kind === "metrics");
  if (beforeMetrics === undefined || beforeMetrics.kind !== "metrics") throw new Error("world lab has no metrics section");
  const beforeMetric = beforeMetrics.metrics.find((metric) => metric.label === "Live forecast (API)");
  if (beforeMetric === undefined) throw new Error("world lab is missing the live-forecast metric");
  equal(beforeMetric.value, "Not requested");

  // Clicking the button (onAction("world:forecast")) must reach the live client.
  await controller.onAction("world:forecast");
  equal(controller.lastError, undefined);
  if (controller.forecast === undefined) throw new Error("world:forecast did not populate a forecast");

  const after = controller.model();
  const afterMetrics = after.sections.find((section) => section.kind === "metrics");
  if (afterMetrics === undefined || afterMetrics.kind !== "metrics") throw new Error("world lab has no metrics section");
  const afterMetric = afterMetrics.metrics.find((metric) => metric.label === "Live forecast (API)");
  if (afterMetric === undefined) throw new Error("world lab is missing the live-forecast metric");
  equal(afterMetric.value !== "Not requested", true);
  equal(afterMetric.value.includes(`sample(s), horizon ${controller.forecast.horizon}`), true);

  controller.dispose();
}
