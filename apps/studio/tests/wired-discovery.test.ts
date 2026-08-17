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
