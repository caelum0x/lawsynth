import type { ArtifactDescriptor, LawSynthClient, SimulationSummary } from "@lawsynth/api-client";

export interface SimulationConfiguration {
  readonly worldId: string;
  readonly horizon: number;
  readonly step: number;
  readonly method: "rk4";
  readonly pollIntervalMs?: number;
  readonly timeoutMs?: number;
}

export interface SimulationResult {
  readonly simulation: SimulationSummary;
  readonly artifact?: ArtifactDescriptor;
  readonly elapsedMs: number;
}

export function validateSimulationConfiguration(input: SimulationConfiguration): SimulationConfiguration {
  if (!input.worldId.trim()) throw new RangeError("world id is required");
  if (!Number.isFinite(input.horizon) || input.horizon <= 0) throw new RangeError("simulation horizon must be positive");
  if (!Number.isFinite(input.step) || input.step <= 0 || input.step > input.horizon) throw new RangeError("simulation step must be positive and no larger than the horizon");
  const samples = Math.ceil(input.horizon / input.step) + 1;
  if (samples > 10_000_000) throw new RangeError("simulation request exceeds ten million output samples");
  const pollIntervalMs = input.pollIntervalMs ?? 1_000;
  const timeoutMs = input.timeoutMs ?? 30 * 60_000;
  if (!Number.isSafeInteger(pollIntervalMs) || pollIntervalMs < 100 || pollIntervalMs > 30_000) throw new RangeError("poll interval must be in 100..30000ms");
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < pollIntervalMs || timeoutMs > 24 * 60 * 60_000) throw new RangeError("simulation timeout is invalid");
  return Object.freeze({ ...input, pollIntervalMs, timeoutMs });
}

function wait(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const aborted = (): void => { clearTimeout(timer); reject(signal.reason); };
    const timer = setTimeout(() => { signal.removeEventListener("abort", aborted); resolve(); }, ms);
    signal.addEventListener("abort", aborted, { once: true });
  });
}

export class SimulationController extends EventTarget {
  #controller: AbortController | undefined;
  #active: SimulationSummary | undefined;
  constructor(readonly api: LawSynthClient, readonly id: () => string, readonly clock: () => number = Date.now) { super(); }
  get active(): SimulationSummary | undefined { return this.#active; }

  async run(input: SimulationConfiguration): Promise<SimulationResult> {
    if (this.#controller !== undefined) throw new Error("a simulation is already active");
    const config = validateSimulationConfiguration(input);
    const controller = new AbortController(); this.#controller = controller;
    const started = this.clock();
    try {
      this.#active = await this.api.worlds.simulate(config.worldId, { horizon: config.horizon, step: config.step, method: config.method }, this.id(), controller.signal);
      this.#emit();
      while (this.#active.status === "queued" || this.#active.status === "running") {
        if (this.clock() - started >= config.timeoutMs!) throw new DOMException("simulation timed out", "TimeoutError");
        await wait(config.pollIntervalMs!, controller.signal);
        this.#active = await this.api.simulations.get(this.#active.id, controller.signal);
        this.#emit();
      }
      if (this.#active.status === "failed") throw new Error(`simulation ${this.#active.id} failed`);
      const artifact = this.#active.status === "succeeded" ? await this.api.simulations.artifact(this.#active.id, controller.signal) : undefined;
      return Object.freeze({ simulation: this.#active, ...(artifact === undefined ? {} : { artifact }), elapsedMs: this.clock() - started });
    } finally { if (this.#controller === controller) this.#controller = undefined; }
  }

  async cancel(): Promise<SimulationSummary | undefined> {
    const active = this.#active;
    this.#controller?.abort(new DOMException("simulation cancelled locally", "AbortError"));
    this.#controller = undefined;
    if (active === undefined || active.status === "failed" || active.status === "succeeded" || active.status === "cancelled") return active;
    this.#active = await this.api.simulations.cancel(active.id);
    this.#emit();
    return this.#active;
  }

  #emit(): void { this.dispatchEvent(new CustomEvent("change", { detail: this.#active })); }
}
