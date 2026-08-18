// @ts-check
/**
 * Dependency-free ES-module glue over the `lawsynth-wasm-bindings` C-ABI.
 *
 * It instantiates the `.wasm` produced by `scripts/build-wasm.sh` and wraps the
 * raw linear-memory protocol (see the crate's `ffi` module / README) into the
 * ergonomic `LawSynthWasmBindings` contract the playground's `wasm.ts` expects:
 *
 *   - `version(): string`
 *   - `simulate(requestJson: string): string`      // returns TrajectoryInput JSON
 *   - `validateWorld(worldJson: string): string`
 *   - `memoryBytes(): number`
 *
 * Plus convenience wrappers: `derivative`, `evalExpression`, `bundleEncode`
 * (returns Uint8Array), and `bundleDecode`.
 *
 * ## Packing protocol (must match the Rust `ffi` module)
 *   result buffer = [u32 LE payloadLen N][u8 status (0 ok, 1 error)][N payload bytes]
 *   On error the payload is JSON `{ "code", "message" }`.
 *
 * The module has zero runtime dependencies and imports nothing.
 */

const HEADER = 5; // 4-byte length prefix + 1-byte status.
const STATUS_OK = 0;

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: false });

/**
 * @typedef {object} LawSynthWasmBindings
 * @property {() => string} version
 * @property {(requestJson: string) => string} simulate
 * @property {(worldJson: string) => string} validateWorld
 * @property {() => number} memoryBytes
 * @property {(requestJson: string) => string} derivative
 * @property {(requestJson: string) => string} evalExpression
 * @property {(requestJson: string) => Uint8Array} bundleEncode
 * @property {(bundle: Uint8Array) => string} bundleDecode
 */

/**
 * An error carrying the stable machine-readable code from the WASM boundary.
 */
export class LawSynthWasmError extends Error {
  /** @param {string} code @param {string} message */
  constructor(code, message) {
    super(message);
    this.name = "LawSynthWasmError";
    /** @type {string} */
    this.code = code;
  }
}

/**
 * Resolve any accepted source into a `WebAssembly.Instance`.
 * @param {WebAssembly.Instance | WebAssembly.Module | Response | Promise<Response> | ArrayBuffer | ArrayBufferView | string} source
 * @returns {Promise<WebAssembly.Instance>}
 */
async function resolveInstance(source) {
  if (source instanceof WebAssembly.Instance) return source;
  if (source instanceof WebAssembly.Module) return new WebAssembly.Instance(source, {});
  if (typeof source === "string") {
    // A URL to the .wasm asset.
    const response = fetch(source);
    return (await WebAssembly.instantiateStreaming(response, {})).instance;
  }
  if (typeof Response !== "undefined" && (source instanceof Response || source instanceof Promise)) {
    return (await WebAssembly.instantiateStreaming(source, {})).instance;
  }
  if (source instanceof ArrayBuffer || ArrayBuffer.isView(source)) {
    return (await WebAssembly.instantiate(source, {})).instance;
  }
  throw new TypeError("unsupported wasm source for LawSynth bindings");
}

/**
 * Instantiate the LawSynth WASM module and return the ergonomic bindings.
 * @param {WebAssembly.Instance | WebAssembly.Module | Response | Promise<Response> | ArrayBuffer | ArrayBufferView | string} source
 * @returns {Promise<LawSynthWasmBindings>}
 */
export async function createBindings(source) {
  const instance = await resolveInstance(source);
  const exports = /** @type {Record<string, any>} */ (instance.exports);

  const required = [
    "memory",
    "ls_alloc",
    "ls_free",
    "ls_version",
    "ls_simulate",
    "ls_validate_world",
    "ls_derivative",
    "ls_eval_expression",
    "ls_bundle_encode",
    "ls_bundle_decode",
  ];
  for (const name of required) {
    if (exports[name] === undefined) {
      throw new LawSynthWasmError("wasm-unavailable", `wasm export '${name}' is missing`);
    }
  }

  /** @returns {ArrayBuffer} */
  const buffer = () => exports.memory.buffer;

  /**
   * Copy bytes into a freshly allocated region of linear memory.
   * @param {Uint8Array} bytes
   * @returns {number} pointer to the written region
   */
  const writeBytes = (bytes) => {
    const ptr = exports.ls_alloc(bytes.length);
    // View created AFTER alloc: allocation may have grown (and detached) memory.
    new Uint8Array(buffer(), ptr, bytes.length).set(bytes);
    return ptr;
  };

  /**
   * Read `{ status, payload }` from a packed result buffer and free it.
   * @param {number} resultPtr
   * @returns {{ status: number, payload: Uint8Array }}
   */
  const readResult = (resultPtr) => {
    // Fresh view: the entry point may have grown memory to allocate the result.
    const view = new DataView(buffer());
    const n = view.getUint32(resultPtr, true);
    const status = view.getUint8(resultPtr + 4);
    // `.slice()` copies out of linear memory before we free it.
    const payload = new Uint8Array(buffer(), resultPtr + HEADER, n).slice();
    exports.ls_free(resultPtr, HEADER + n);
    return { status, payload };
  };

  /**
   * Invoke a `(ptr, len) -> resultPtr` entry point with byte input.
   * @param {(ptr: number, len: number) => number} entry
   * @param {Uint8Array} input
   * @returns {{ status: number, payload: Uint8Array }}
   */
  const callBytes = (entry, input) => {
    const ptr = writeBytes(input);
    const resultPtr = entry(ptr, input.length);
    exports.ls_free(ptr, input.length);
    return readResult(resultPtr);
  };

  /**
   * Text-in / text-out helper. Throws `LawSynthWasmError` on an error status.
   * @param {(ptr: number, len: number) => number} entry
   * @param {string} request
   * @returns {string}
   */
  const callText = (entry, request) => {
    const { status, payload } = callBytes(entry, encoder.encode(request));
    const text = decoder.decode(payload);
    if (status !== STATUS_OK) {
      /** @type {{code?: string, message?: string}} */
      let parsed = {};
      try {
        parsed = JSON.parse(text);
      } catch {
        parsed = { code: "SIMULATION_FAILED", message: text };
      }
      throw new LawSynthWasmError(parsed.code ?? "SIMULATION_FAILED", parsed.message ?? "wasm error");
    }
    return text;
  };

  return {
    version() {
      const resultPtr = exports.ls_version();
      const { status, payload } = readResult(resultPtr);
      if (status !== STATUS_OK) throw new LawSynthWasmError("wasm-unavailable", "version failed");
      return decoder.decode(payload);
    },
    simulate(requestJson) {
      return callText(exports.ls_simulate, requestJson);
    },
    validateWorld(worldJson) {
      return callText(exports.ls_validate_world, worldJson);
    },
    derivative(requestJson) {
      return callText(exports.ls_derivative, requestJson);
    },
    evalExpression(requestJson) {
      return callText(exports.ls_eval_expression, requestJson);
    },
    bundleEncode(requestJson) {
      const { status, payload } = callBytes(exports.ls_bundle_encode, encoder.encode(requestJson));
      if (status !== STATUS_OK) {
        const parsed = JSON.parse(decoder.decode(payload));
        throw new LawSynthWasmError(parsed.code ?? "INVALID_BUNDLE", parsed.message ?? "encode failed");
      }
      return payload;
    },
    bundleDecode(bundle) {
      const { status, payload } = callBytes(exports.ls_bundle_decode, bundle);
      const text = decoder.decode(payload);
      if (status !== STATUS_OK) {
        const parsed = JSON.parse(text);
        throw new LawSynthWasmError(parsed.code ?? "INVALID_BUNDLE", parsed.message ?? "decode failed");
      }
      return text;
    },
    memoryBytes() {
      return exports.memory.buffer.byteLength;
    },
  };
}

export default createBindings;
