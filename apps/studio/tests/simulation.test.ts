import { SimulationController, validateSimulationConfiguration } from "../src/simulation.js";
import { equal, inMemoryApi, rejects } from "./support.js";

export async function simulationTests(): Promise<void> {
  const config = validateSimulationConfiguration({ worldId: "world_1", horizon: 10, step: 0.1, method: "rk4", pollIntervalMs: 100 });
  equal(config.pollIntervalMs, 100);
  const result = await new SimulationController(inMemoryApi(), () => "simulation_request", () => 10).run(config);
  equal(result.simulation.status, "succeeded"); equal(result.artifact?.id, "artifact_1"); equal(result.elapsedMs, 0);
  await rejects(() => Promise.resolve(validateSimulationConfiguration({ ...config, step: 11 })), /no larger/);
}
