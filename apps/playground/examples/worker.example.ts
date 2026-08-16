import { installPlaygroundWorker, type WorkerSimulationHost } from "../src/worker.js";

export interface PlaygroundWorkerScope {
  addEventListener(type: "message", listener: (event: MessageEvent) => void): void;
  postMessage(value: unknown): void;
}

/**
 * Connect the browser worker protocol to a real simulation host.
 * The host must be backed by generated `lawsynth-wasm` bindings; this adapter does not emulate one.
 */
export function connectPlaygroundWorker(scope: PlaygroundWorkerScope, host: WorkerSimulationHost): void {
  installPlaygroundWorker(scope, host);
}
