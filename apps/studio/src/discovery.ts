import type { CandidateSummary, LawSynthClient, RunSummary } from "@lawsynth/api-client";

export interface DiscoveryConfiguration {
  readonly datasetId: string;
  readonly target: string;
  readonly inputs: readonly string[];
  readonly library: "polynomial" | "trigonometric" | "rational" | "mixed";
  readonly maximumComplexity: number;
  readonly validationFraction: number;
  readonly seed: number;
}

export interface DiscoveryProgress {
  readonly run: RunSummary;
  readonly progress: number;
  readonly message?: string;
  readonly candidates: readonly CandidateSummary[];
}

export function validateDiscoveryConfiguration(config: DiscoveryConfiguration): DiscoveryConfiguration {
  if (!config.datasetId.trim() || !config.target.trim()) throw new RangeError("dataset and target are required");
  if (config.inputs.length === 0 || new Set(config.inputs).size !== config.inputs.length) throw new RangeError("discovery inputs must be non-empty and unique");
  if (config.inputs.includes(config.target)) throw new RangeError("target cannot also be an input");
  if (!Number.isSafeInteger(config.maximumComplexity) || config.maximumComplexity < 1 || config.maximumComplexity > 100) throw new RangeError("maximumComplexity must be in 1..100");
  if (!Number.isFinite(config.validationFraction) || config.validationFraction <= 0 || config.validationFraction >= 0.5) throw new RangeError("validationFraction must be in (0, 0.5)");
  if (!Number.isSafeInteger(config.seed) || config.seed < 0) throw new RangeError("seed must be a non-negative integer");
  return Object.freeze({ ...config, inputs: Object.freeze([...config.inputs]) });
}

export class DiscoveryController extends EventTarget {
  #abort: AbortController | undefined;
  #progress: DiscoveryProgress | undefined;
  constructor(readonly api: LawSynthClient, readonly id: () => string) { super(); }
  get progress(): DiscoveryProgress | undefined { return this.#progress; }

  async start(projectId: string, configuration: DiscoveryConfiguration): Promise<RunSummary> {
    if (this.#abort !== undefined) throw new Error("a discovery run is already active");
    const config = validateDiscoveryConfiguration(configuration);
    const controller = new AbortController();
    this.#abort = controller;
    try {
      const run = await this.api.runs.create({
        name: `Discover ${config.target}`,
        status: "queued",
        dataset_id: config.datasetId,
        metadata: { project_id: projectId, target: config.target, inputs: config.inputs, library: config.library, maximum_complexity: config.maximumComplexity, validation_fraction: config.validationFraction, seed: config.seed },
      }, this.id(), controller.signal);
      this.#commit({ run, progress: 0, candidates: [] });
      void this.#observe(run.id, controller);
      return run;
    } catch (error) { if (this.#abort === controller) this.#abort = undefined; throw error; }
  }

  async cancel(): Promise<void> {
    const progress = this.#progress;
    const active = progress?.run;
    if (active === undefined || progress === undefined || this.#abort === undefined) return;
    this.#abort.abort(new DOMException("discovery cancelled locally", "AbortError"));
    const cancelled = await this.api.runs.cancel(active.id);
    this.#commit({ ...progress, run: cancelled, message: "Cancelled" });
    this.#abort = undefined;
  }

  async #observe(runId: string, controller: AbortController): Promise<void> {
    let sequence = 0;
    try {
      for await (const event of this.api.events(runId, { signal: controller.signal })) {
        sequence += 1;
        const payload = typeof event.payload === "object" && event.payload !== null ? event.payload as Record<string, unknown> : {};
        const value = typeof payload.progress === "number" && Number.isFinite(payload.progress) ? Math.max(0, Math.min(1, payload.progress)) : this.#progress?.progress ?? 0;
        const message = typeof payload.message === "string" ? payload.message.slice(0, 500) : undefined;
        const run = this.#progress?.run;
        if (run !== undefined) this.#commit({ ...this.#progress!, progress: value, ...(message === undefined ? {} : { message }) });
        if (event.topic.endsWith("succeeded") || event.topic.endsWith("failed") || event.topic.endsWith("cancelled")) break;
        if (sequence > 100_000) throw new Error("discovery event stream exceeded safety limit");
      }
      const [run, candidates] = await Promise.all([this.api.runs.get(runId, controller.signal), this.api.runs.candidates(runId, { limit: 250, signal: controller.signal })]);
      this.#commit({ run, progress: run.status === "succeeded" ? 1 : this.#progress?.progress ?? 0, candidates: Object.freeze([...candidates.items].sort((left, right) => right.score - left.score)) });
    } catch (error) {
      if (!controller.signal.aborted) this.dispatchEvent(new CustomEvent("error", { detail: { error } }));
    } finally { if (this.#abort === controller) this.#abort = undefined; }
  }

  #commit(progress: DiscoveryProgress): void {
    this.#progress = Object.freeze(progress);
    this.dispatchEvent(new CustomEvent("change", { detail: this.#progress }));
  }
}
