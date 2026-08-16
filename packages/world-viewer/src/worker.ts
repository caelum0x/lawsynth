import type { WorldDefinition } from "@lawsynth/world-schema";
import type { ChartModel } from "@lawsynth/chart-core";
import { graphForWorld, type WorldGraphView } from "./graph.js";
import { equationsForWorld, type EquationView } from "./equation.js";
import { trajectoryPlotGeometry, type PlotGeometry } from "./trajectory.js";

export type ViewerWorkerRequest =
  | { readonly id: number; readonly operation: "graph"; readonly world: WorldDefinition }
  | { readonly id: number; readonly operation: "equations"; readonly world: WorldDefinition }
  | { readonly id: number; readonly operation: "trajectory-geometry"; readonly chart: ChartModel; readonly width: number; readonly height: number };

export type ViewerWorkerValue = WorldGraphView | readonly EquationView[] | PlotGeometry;

export interface ViewerWorkerSuccess {
  readonly id: number;
  readonly ok: true;
  readonly value: ViewerWorkerValue;
}

export interface ViewerWorkerFailure {
  readonly id: number;
  readonly ok: false;
  readonly error: string;
}

export type ViewerWorkerResponse = ViewerWorkerSuccess | ViewerWorkerFailure;

export interface WorkerLike {
  postMessage(message: ViewerWorkerRequest): void;
  addEventListener(type: "message", listener: (event: MessageEvent<ViewerWorkerResponse>) => void): void;
  removeEventListener(type: "message", listener: (event: MessageEvent<ViewerWorkerResponse>) => void): void;
  terminate?(): void;
}

export interface ViewerWorkerClientOptions {
  readonly timeoutMs?: number;
  readonly ownsWorker?: boolean;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Executes one serializable transform. It is also useful as a synchronous fallback. */
export function processViewerWorkerRequest(request: ViewerWorkerRequest): ViewerWorkerResponse {
  try {
    let value: ViewerWorkerValue;
    switch (request.operation) {
      case "graph": value = graphForWorld(request.world); break;
      case "equations": value = equationsForWorld(request.world); break;
      case "trajectory-geometry": value = trajectoryPlotGeometry(request.chart, request.width, request.height); break;
    }
    return { id: request.id, ok: true, value };
  } catch (error) {
    return { id: request.id, ok: false, error: errorMessage(error) };
  }
}

/** Installs the worker-side protocol without assuming a global Worker type at import time. */
export function installViewerWorker(scope: {
  addEventListener(type: "message", listener: (event: MessageEvent<ViewerWorkerRequest>) => void): void;
  postMessage(message: ViewerWorkerResponse): void;
}): () => void {
  const listener = (event: MessageEvent<ViewerWorkerRequest>): void => scope.postMessage(processViewerWorkerRequest(event.data));
  scope.addEventListener("message", listener);
  return () => {
    const removable = scope as typeof scope & { removeEventListener?: typeof scope.addEventListener };
    removable.removeEventListener?.("message", listener);
  };
}

interface Pending {
  readonly resolve: (value: ViewerWorkerValue) => void;
  readonly reject: (error: Error) => void;
  readonly timer: ReturnType<typeof setTimeout>;
}

export class ViewerWorkerClient {
  #nextId = 0;
  #pending = new Map<number, Pending>();
  #disposed = false;
  readonly #timeoutMs: number;
  readonly #ownsWorker: boolean;
  readonly #listener: (event: MessageEvent<ViewerWorkerResponse>) => void;

  constructor(readonly worker: WorkerLike, options: ViewerWorkerClientOptions = {}) {
    this.#timeoutMs = options.timeoutMs ?? 15_000;
    if (!Number.isFinite(this.#timeoutMs) || this.#timeoutMs <= 0) throw new RangeError("worker timeout must be positive");
    this.#ownsWorker = options.ownsWorker === true;
    this.#listener = (event) => this.#receive(event.data);
    worker.addEventListener("message", this.#listener);
  }

  run(request: Omit<ViewerWorkerRequest, "id">): { readonly id: number; readonly promise: Promise<ViewerWorkerValue> } {
    if (this.#disposed) throw new Error("viewer worker client is disposed");
    const id = ++this.#nextId;
    const promise = new Promise<ViewerWorkerValue>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.#pending.delete(id);
        reject(new Error(`viewer worker request ${id} timed out after ${this.#timeoutMs}ms`));
      }, this.#timeoutMs);
      this.#pending.set(id, { resolve, reject, timer });
      this.worker.postMessage({ ...request, id } as ViewerWorkerRequest);
    });
    return { id, promise };
  }

  cancel(id: number): boolean {
    const pending = this.#pending.get(id);
    if (pending === undefined) return false;
    clearTimeout(pending.timer);
    this.#pending.delete(id);
    pending.reject(new DOMException(`viewer worker request ${id} was cancelled`, "AbortError"));
    return true;
  }

  dispose(): void {
    if (this.#disposed) return;
    this.#disposed = true;
    this.worker.removeEventListener("message", this.#listener);
    for (const [id, pending] of this.#pending) {
      clearTimeout(pending.timer);
      pending.reject(new Error(`viewer worker request ${id} was interrupted by disposal`));
    }
    this.#pending.clear();
    if (this.#ownsWorker) this.worker.terminate?.();
  }

  #receive(response: ViewerWorkerResponse): void {
    const pending = this.#pending.get(response.id);
    if (pending === undefined) return;
    clearTimeout(pending.timer);
    this.#pending.delete(response.id);
    if (response.ok) pending.resolve(response.value);
    else pending.reject(new Error(response.error));
  }
}

/** Cooperative local executor for SSR, restrictive CSPs, and small models. */
export class LocalViewerWorker {
  #nextId = 0;
  #cancelled = new Set<number>();

  run(request: Omit<ViewerWorkerRequest, "id">): { readonly id: number; readonly promise: Promise<ViewerWorkerValue> } {
    const id = ++this.#nextId;
    const promise = Promise.resolve().then(() => {
      if (this.#cancelled.delete(id)) throw new DOMException(`viewer task ${id} was cancelled`, "AbortError");
      const response = processViewerWorkerRequest({ ...request, id } as ViewerWorkerRequest);
      if (!response.ok) throw new Error(response.error);
      return response.value;
    });
    return { id, promise };
  }

  cancel(id: number): boolean {
    if (!Number.isInteger(id) || id <= 0 || id > this.#nextId) return false;
    this.#cancelled.add(id);
    return true;
  }
}
