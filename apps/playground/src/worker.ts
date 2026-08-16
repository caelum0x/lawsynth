import type { WasmSimulationRequest } from "./wasm.js";
import type { TrajectoryInput } from "@lawsynth/chart-core";
export type WorkerRequest = {
    readonly id: number;
    readonly type: "simulate";
    readonly request: WasmSimulationRequest;
} | {
    readonly id: number;
    readonly type: "cancel";
    readonly targetId: number;
};
export type WorkerResponse = {
    readonly id: number;
    readonly ok: true;
    readonly trajectory?: TrajectoryInput;
} | {
    readonly id: number;
    readonly ok: false;
    readonly error: string;
};
export interface WorkerSimulationHost {
    simulate(request: WasmSimulationRequest, signal: AbortSignal): Promise<TrajectoryInput>;
}
export function installPlaygroundWorker(scope: {
    addEventListener(type: "message", listener: (event: MessageEvent<WorkerRequest>) => void): void;
    postMessage(value: WorkerResponse): void;
}, host: WorkerSimulationHost): void {
  const active = new Map<number, AbortController>();
  scope.addEventListener("message", (event) => {
    const message = event.data;
    if (message.type === "cancel") {
      active.get(message.targetId)?.abort();
      active.delete(message.targetId);
      scope.postMessage({ id: message.id, ok: true });
      return;
    }
    if (active.has(message.id)) {
      scope.postMessage({ id: message.id, ok: false, error: "duplicate worker request id" });
      return;
    }
    const controller = new AbortController();
    active.set(message.id, controller);
    void host.simulate(message.request, controller.signal)
      .then((trajectory) => scope.postMessage({ id: message.id, ok: true, trajectory }))
      .catch((error) => scope.postMessage({ id: message.id, ok: false, error: error instanceof Error ? error.message : String(error) }))
      .finally(() => active.delete(message.id));
  });
}
