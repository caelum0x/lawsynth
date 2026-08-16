import { PlaygroundController } from "../src/playground.js";
import { LocalSimulation } from "../src/simulation.js";
import { WasmRuntime, type WasmSimulationRequest } from "../src/wasm.js";
import { installPlaygroundWorker, type WorkerRequest, type WorkerResponse } from "../src/worker.js";
import { decayWorld, deepEqual, equal, rejects, test } from "./testkit.js";

await test("controller preserves a validated world while reporting a missing local runtime", async () => {
  const simulation = new LocalSimulation(new WasmRuntime({ loader: () => { throw new Error("browser WASM artifact unavailable"); } }));
  const controller = new PlaygroundController(simulation, decayWorld);
  await rejects(() => controller.run({ start: 0, end: 1, step: 0.1 }, { x: 1 }), /artifact unavailable/);
  equal(controller.snapshot.world?.id, decayWorld.id);
  equal(controller.snapshot.simulation.phase, "failed");
  equal(controller.snapshot.error?.message, "browser WASM artifact unavailable");
  controller.dispose();
});

await test("worker protocol acknowledges cancellation and never reports a fabricated trajectory", async () => {
  let listener: ((event: MessageEvent<WorkerRequest>) => void) | undefined;
  const responses: WorkerResponse[] = [];
  const scope = {
    addEventListener(_type: "message", callback: (event: MessageEvent<WorkerRequest>) => void): void { listener = callback; },
    postMessage(response: WorkerResponse): void { responses.push(response); },
  };
  // This deliberately unavailable host only rejects on cancellation. A browser deployment
  // must supply a real WASM-backed host before the protocol can return a trajectory.
  const host = {
    simulate(_request: WasmSimulationRequest, signal: AbortSignal): Promise<never> {
      return new Promise((_, reject) => signal.addEventListener("abort", () => reject(signal.reason), { once: true }));
    },
  };
  installPlaygroundWorker(scope, host);
  const request = { id: 4, type: "simulate" as const, request: { world: decayWorld, initial: { x: 1 }, start: 0, end: 1, step: 0.1 } };
  listener?.({ data: request } as unknown as MessageEvent<WorkerRequest>);
  listener?.({ data: request } as unknown as MessageEvent<WorkerRequest>);
  listener?.({ data: { id: 5, type: "cancel", targetId: 4 } } as unknown as MessageEvent<WorkerRequest>);
  await new Promise<void>((resolve) => setTimeout(resolve, 0));
  equal(responses.some((response) => response.id === 4 && !response.ok && response.error === "duplicate worker request id"), true);
  equal(responses.some((response) => response.id === 5 && response.ok), true);
  deepEqual(responses.filter((response) => response.id === 4 && response.ok), []);
});
