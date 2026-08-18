import { PlaygroundError } from "./errors.js";
import type { LawSynthWasmBindings } from "./wasm.js";

/**
 * A wasm source the generated glue can instantiate: a URL (string or {@link URL}),
 * raw bytes, a `fetch` `Response`, or an already-built module/instance.
 */
export type WasmSource =
  | string
  | URL
  | ArrayBuffer
  | ArrayBufferView
  | Response
  | Promise<Response>
  | WebAssembly.Module
  | WebAssembly.Instance;

/**
 * Turns a wasm {@link WasmSource} into the playground's {@link LawSynthWasmBindings}.
 *
 * The generated glue's `createBindings` (from
 * `crates/lawsynth-wasm-bindings/web/lawsynth_wasm.mjs`) satisfies this exactly.
 * It is *injected* rather than imported so this module carries no build-time
 * dependency on the compiled `.wasm` (which is produced by
 * `scripts/build-wasm.sh` on a networked machine) and can be exercised in tests
 * with a fake factory.
 */
export type WasmBindingsFactory = (
  source: WasmSource,
) => LawSynthWasmBindings | Promise<LawSynthWasmBindings>;

/**
 * Build a {@link import("./wasm.js").WasmRuntime} loader that instantiates the
 * engine from `source` via `factory`, validating that the produced bindings
 * expose the required API.
 *
 * Production wiring, once the `.wasm` and glue are deployed beside the app:
 *
 * ```ts
 * import { createBindings } from "./lawsynth_wasm.mjs";
 * const loader = createWasmLoader(defaultWasmSource(), createBindings);
 * const runtime = new WasmRuntime({ loader });
 * ```
 */
export function createWasmLoader(
  source: WasmSource,
  factory: WasmBindingsFactory,
): () => Promise<LawSynthWasmBindings> {
  return async () => {
    const bindings = await factory(source);
    if (typeof bindings.simulate !== "function" || typeof bindings.version !== "function") {
      throw new PlaygroundError(
        "wasm-unavailable",
        "generated bindings do not expose the required API",
      );
    }
    return bindings;
  };
}

/**
 * The conventional URL of the built `.wasm`, deployed beside the glue module.
 * Resolve against a base (defaults to this module's own URL).
 */
export function defaultWasmSource(base: string | URL = import.meta.url): URL {
  return new URL("lawsynth_wasm.wasm", base);
}
