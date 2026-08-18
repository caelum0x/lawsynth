import { PlaygroundError } from "../src/errors.js";
import { WasmRuntime } from "../src/wasm.js";
import { createWasmLoader, defaultWasmSource, type WasmBindingsFactory } from "../src/wasm_loader.js";
import { decayWorld, equal, ok, rejects, test } from "./testkit.js";

// A fake glue: satisfies LawSynthWasmBindings without a real .wasm, echoing the
// request's states as a two-sample trajectory so the runtime path is exercised.
const fakeFactory: WasmBindingsFactory = () => ({
  version: () => "0.1.0-test",
  simulate: (requestJson: string): string => {
    const request = JSON.parse(requestJson) as { initial: Record<string, number>; start: number; end: number };
    const variables = Object.keys(request.initial);
    return JSON.stringify({
      variables,
      times: [request.start, request.end],
      values: [variables.map((v) => request.initial[v]), variables.map(() => 0)],
    });
  },
  memoryBytes: () => 4096,
});

await test("createWasmLoader adapts an injected factory into a working runtime", async () => {
  const loader = createWasmLoader("about:blank", fakeFactory);
  const runtime = new WasmRuntime({ loader });
  const trajectory = await runtime.simulate({ world: decayWorld, initial: { x: 1 }, start: 0, end: 1, step: 0.5 });
  equal(trajectory.variables[0], "x");
  ok(trajectory.times.length === 2, "two samples returned");
  equal(trajectory.values[0]?.[0], 1);
  equal(runtime.memoryBytes(), 4096);
});

await test("createWasmLoader rejects bindings that lack the required API", async () => {
  // A factory that returns an object missing simulate/version.
  const loader = createWasmLoader("about:blank", (() => ({})) as unknown as WasmBindingsFactory);
  const runtime = new WasmRuntime({ loader });
  await rejects(() => runtime.initialize(), /required API/);
});

await test("createWasmLoader surfaces a factory failure as a capability boundary", async () => {
  const loader = createWasmLoader("about:blank", () => {
    throw new Error("wasm module was not deployed");
  });
  const runtime = new WasmRuntime({ loader });
  await rejects(() => runtime.initialize(), /not deployed/);
});

await test("defaultWasmSource resolves the wasm beside the glue", () => {
  const url = defaultWasmSource("https://lawsynth.dev/playground/app.js");
  equal(url.href, "https://lawsynth.dev/playground/lawsynth_wasm.wasm");
});

await test("PlaygroundError from a missing-API loader carries the wasm-unavailable code", async () => {
  const loader = createWasmLoader("about:blank", (() => ({ version: () => "x" })) as unknown as WasmBindingsFactory);
  const runtime = new WasmRuntime({ loader });
  try {
    await runtime.initialize();
    throw new Error("expected rejection");
  } catch (error) {
    equal((error as PlaygroundError).code, "wasm-unavailable");
  }
});
