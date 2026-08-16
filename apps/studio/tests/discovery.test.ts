import { DiscoveryController, validateDiscoveryConfiguration } from "../src/discovery.js";
import { equal, inMemoryApi, rejects } from "./support.js";

export async function discoveryTests(): Promise<void> {
  const config = validateDiscoveryConfiguration({ datasetId: "dataset_1", target: "prey", inputs: ["predator", "time"], library: "polynomial", maximumComplexity: 4, validationFraction: 0.2, seed: 7 });
  equal(Object.isFrozen(config.inputs), true);
  const controller = new DiscoveryController(inMemoryApi(), () => "request_1");
  await controller.start("project_1", config);
  await new Promise((resolve) => setTimeout(resolve, 0));
  equal(controller.progress?.run.status, "succeeded");
  equal(controller.progress?.candidates[0]?.id, "candidate_1");
  await rejects(() => Promise.resolve(validateDiscoveryConfiguration({ ...config, inputs: ["prey"] })), /target/);
}
