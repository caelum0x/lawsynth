import { PlaygroundError } from "../src/errors.js";
import { LocalSimulation } from "../src/simulation.js";
import { WasmRuntime } from "../src/wasm.js";
import { decayWorld, equal, rejects, test } from "./testkit.js";

const unavailableRuntime = (): WasmRuntime => new WasmRuntime({
  loader: () => Promise.reject(new Error("lawsynth-wasm bindings are not installed")),
});

await test("simulation fails explicitly when the generated WASM bindings are unavailable", async () => {
  const simulation = new LocalSimulation(unavailableRuntime(), () => 10);
  await rejects(() => simulation.run(decayWorld, { initial: { x: 1 }, start: 0, end: 1, step: 0.1 }), /bindings are not installed/);
  equal(simulation.snapshot.phase, "failed");
  equal((simulation.snapshot.error as PlaygroundError | undefined)?.code, "wasm-unavailable");
});

await test("simulation rejects invalid ranges before attempting to load WASM", async () => {
  const runtime = unavailableRuntime();
  await rejects(() => runtime.simulate({ world: decayWorld, initial: { x: 1 }, start: 1, end: 1, step: 0.1 }), /time range/);
});
