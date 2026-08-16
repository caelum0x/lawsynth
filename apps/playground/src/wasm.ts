import type { TrajectoryInput } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { PlaygroundError, normalizePlaygroundError } from "./errors.js";
export interface WasmSimulationRequest {
    readonly world: WorldDefinition;
    readonly initial: Readonly<Record<string, number>>;
    readonly parameters?: Readonly<Record<string, number>>;
    readonly start: number;
    readonly end: number;
    readonly step: number;
}
/** Contract implemented by the generated JS glue around `lawsynth-wasm`. */
export interface LawSynthWasmBindings {
    readonly version: () => string;
    readonly simulate: (requestJson: string) => string | Promise<string>;
    readonly validateWorld?: (worldJson: string) => string | Promise<string>;
    readonly memoryBytes?: () => number;
}
export interface WasmRuntimeOptions {
    readonly loader: () => LawSynthWasmBindings | Promise<LawSynthWasmBindings>;
    readonly maximumSamples?: number;
    readonly maximumRequestBytes?: number;
}
export class WasmRuntime {
    #bindings: LawSynthWasmBindings | undefined;
    #pending: Promise<LawSynthWasmBindings> | undefined;
    readonly #maximumSamples: number;
    readonly #maximumRequestBytes: number;
    constructor(readonly options: WasmRuntimeOptions) {
        this.#maximumSamples = options.maximumSamples ?? 1000000;
        this.#maximumRequestBytes = options.maximumRequestBytes ?? 8 * 1024 * 1024;
        if (!Number.isSafeInteger(this.#maximumSamples) || this.#maximumSamples < 2)
            throw new RangeError("maximumSamples must be at least two");
        if (!Number.isSafeInteger(this.#maximumRequestBytes) || this.#maximumRequestBytes < 1024)
            throw new RangeError("maximumRequestBytes must be at least 1 KiB");
    }
    async initialize(): Promise<LawSynthWasmBindings> {
        if (this.#bindings !== undefined)
            return this.#bindings;
        this.#pending ??= Promise.resolve(this.options.loader()).then((bindings) => {
            if (typeof bindings.simulate !== "function" || typeof bindings.version !== "function")
                throw new PlaygroundError("wasm-unavailable", "generated bindings do not expose the required API");
            this.#bindings = bindings;
            return bindings;
        }).catch((error) => { throw normalizePlaygroundError(error, "wasm-unavailable"); }).finally(() => { this.#pending = undefined; });
        return this.#pending;
    }
    async simulate(request: WasmSimulationRequest, signal?: AbortSignal): Promise<TrajectoryInput> {
        signal?.throwIfAborted();
        if (![request.start, request.end, request.step].every(Number.isFinite) || request.end <= request.start || request.step <= 0)
            throw new PlaygroundError("simulation-failed", "time range must be finite, increasing, and use a positive step");
        const samples = Math.ceil((request.end - request.start) / request.step) + 1;
        if (samples > this.#maximumSamples)
            throw new PlaygroundError("limit-exceeded", `${samples} samples requested; maximum is ${this.#maximumSamples}`);
        const serialized = JSON.stringify(request);
        if (new TextEncoder().encode(serialized).byteLength > this.#maximumRequestBytes)
            throw new PlaygroundError("limit-exceeded", "serialized simulation request is too large");
        const bindings = await this.initialize();
        signal?.throwIfAborted();
        try {
            const result = await bindings.simulate(serialized);
            signal?.throwIfAborted();
            const value = JSON.parse(result) as Partial<TrajectoryInput>;
            if (!Array.isArray(value.variables) || !Array.isArray(value.times) || !Array.isArray(value.values))
                throw new TypeError("WASM returned a malformed trajectory");
            return value as TrajectoryInput;
        }
        catch (error) {
            throw normalizePlaygroundError(error);
        }
    }
    memoryBytes(): number | undefined { const value = this.#bindings?.memoryBytes?.(); return value === undefined || !Number.isFinite(value) ? undefined : value; }
}
