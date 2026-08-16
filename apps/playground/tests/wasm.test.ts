import { PlaygroundError } from "../src/errors.js";
import { WasmRuntime } from "../src/wasm.js";
import { decayWorld, equal, rejects, test } from "./testkit.js";

await test("WASM runtime keeps unavailable bindings as an explicit capability boundary", async () => {
  const runtime = new WasmRuntime({ loader: () => { throw new Error("generated glue was not deployed"); } });
  await rejects(() => runtime.initialize(), /generated glue was not deployed/);
  await rejects(() => runtime.initialize(), /generated glue was not deployed/);
  equal(runtime.memoryBytes(), undefined);
});

await test("WASM runtime enforces sample limits without fabricating a simulation result", async () => {
  const runtime = new WasmRuntime({ loader: () => { throw new Error("must not load"); }, maximumSamples: 4 });
  try {
    await runtime.simulate({ world: decayWorld, initial: { x: 1 }, start: 0, end: 1, step: 0.1 });
    throw new Error("expected sample limit rejection");
  } catch (error) {
    equal((error as PlaygroundError).code, "limit-exceeded");
  }
});
