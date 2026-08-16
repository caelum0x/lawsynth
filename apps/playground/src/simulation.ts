import type { TrajectoryInput } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { WasmRuntime, type WasmSimulationRequest } from "./wasm.js";
export type SimulationPhase = "idle" | "running" | "succeeded" | "failed" | "cancelled";
export interface LocalSimulationSnapshot {
    readonly phase: SimulationPhase;
    readonly trajectory?: TrajectoryInput;
    readonly error?: Error;
    readonly elapsedMs?: number;
}
export class LocalSimulation extends EventTarget {
    #snapshot: LocalSimulationSnapshot = { phase: "idle" };
    #abort: AbortController | undefined;
    constructor(readonly runtime: WasmRuntime, readonly clock: () => number = () => performance.now()) { super(); }
    get snapshot(): LocalSimulationSnapshot { return this.#snapshot; }
    async run(world: WorldDefinition, request: Omit<WasmSimulationRequest, "world">): Promise<TrajectoryInput> {
      if (this.#abort !== undefined) throw new Error("simulation already running");
      const controller = new AbortController();
      this.#abort = controller;
      const started = this.clock();
      this.#commit({ phase: "running" });
      try {
        const trajectory = await this.runtime.simulate({ ...request, world }, controller.signal);
        this.#commit({ phase: "succeeded", trajectory, elapsedMs: this.clock() - started });
        return trajectory;
    }
    catch (error) {
        const failure = error instanceof Error ? error : new Error(String(error));
        this.#commit({ phase: controller.signal.aborted ? "cancelled" : "failed", error: failure, elapsedMs: this.clock() - started });
        throw failure;
    }
    finally {
        if (this.#abort === controller)
            this.#abort = undefined;
      }
    }
    cancel(): void { this.#abort?.abort(new DOMException("simulation cancelled", "AbortError")); }
    #commit(snapshot: LocalSimulationSnapshot): void { this.#snapshot = Object.freeze(snapshot); this.dispatchEvent(new CustomEvent("change", { detail: this.#snapshot })); }
}
