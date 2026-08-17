import { ApiError } from "./errors.js";
import type {
  JsonObject,
  RunStatus,
  RunSummary,
  RunWorld,
  WorldComparison,
  WorldExplanation,
  WorldForecast,
  WorldRecord,
} from "./generated.js";
import type { Transport, TransportRequest } from "./transport.js";

/**
 * A minimal declarative world the fake transport hands back when a run
 * succeeds. `equations` map each state to a plain arithmetic expression, the
 * same shape the real service returns for `GET /v1/runs/{id}/world`.
 */
export interface FakeWorldSpec {
  readonly id: string;
  readonly name: string;
  readonly states: readonly string[];
  readonly controls?: readonly string[];
  readonly parameters?: Readonly<Record<string, number>>;
  readonly equations: Readonly<Record<string, string>>;
}

export interface InMemoryDiscoveryOptions {
  /** The world produced when a run succeeds. Defaults to a damped oscillator. */
  readonly world?: FakeWorldSpec;
  /**
   * How many `running` polls precede `succeeded`. `0` means the first
   * `GET /v1/runs/{id}` already reports `succeeded`; the default `1` reports
   * `running` once and then `succeeded`, exercising a real polling loop.
   */
  readonly pollsUntilSucceeded?: number;
  readonly organizationId?: string;
  /** When set, a run whose submit carries this dataset id fails at execution. */
  readonly failWith?: string;
}

const DEFAULT_WORLD: FakeWorldSpec = Object.freeze({
  id: "world-oscillator",
  name: "Discovered oscillator",
  states: Object.freeze(["x", "v"]),
  controls: Object.freeze([]),
  parameters: Object.freeze({}),
  equations: Object.freeze({ x: "v", v: "-4*x - 0.5*v" }),
});

interface RunEntry {
  summary: RunSummary;
  reads: number;
  worldId: string | null;
  failure: string | null;
}

/**
 * A deterministic, dependency-free {@link Transport} that emulates the discovery
 * run lifecycle (submit -> queued -> running -> succeeded -> world) plus the
 * world product actions, so a wired client flow can be exercised end-to-end
 * offline. Pass an instance straight to `new LawSynthClient(transport)`.
 */
export class InMemoryDiscoveryTransport implements Transport {
  readonly #world: FakeWorldSpec;
  readonly #pollsUntilSucceeded: number;
  readonly #org: string;
  readonly #failWith: string | undefined;
  readonly #runs = new Map<string, RunEntry>();
  #counter = 0;

  constructor(options: InMemoryDiscoveryOptions = {}) {
    this.#world = options.world ?? DEFAULT_WORLD;
    this.#pollsUntilSucceeded = Math.max(0, Math.trunc(options.pollsUntilSucceeded ?? 1));
    this.#org = options.organizationId ?? "org-test";
    this.#failWith = options.failWith;
  }

  /** The runs this transport has observed, newest submissions last. */
  get runs(): readonly RunSummary[] {
    return [...this.#runs.values()].map((entry) => entry.summary);
  }

  request<T>(request: TransportRequest): Promise<T> {
    const method = request.method ?? "GET";
    const path = request.path.split("?")[0] ?? request.path;
    const parts = path.split("/").filter((part) => part.length > 0);
    // parts[0] === "v1"
    try {
      return Promise.resolve(this.#route(method, parts, request) as T);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  #route(method: string, parts: readonly string[], request: TransportRequest): unknown {
    if (parts[0] !== "v1") throw this.#notFound(request.path);
    const [, segment, id, action] = parts;
    if (segment === "runs" && method === "POST" && id === undefined) return this.#submit(request);
    if (segment === "runs" && method === "GET" && id !== undefined && action === undefined) return this.#getRun(id);
    if (segment === "runs" && method === "GET" && id !== undefined && action === "world") return this.#getWorld(id);
    if (segment === "worlds" && method === "GET" && id !== undefined && action === "explain") return this.#explain(id);
    if (segment === "worlds" && method === "POST" && id !== undefined && action === "forecast") return this.#forecast(id, request.body);
    if (segment === "worlds" && method === "GET" && id !== undefined && action === "report") return this.#report(id);
    if (segment === "worlds" && method === "POST" && id === "compare") return this.#compare(request.body);
    throw this.#notFound(request.path);
  }

  #submit(request: TransportRequest): RunSummary {
    const body = (typeof request.body === "object" && request.body !== null ? request.body : {}) as Record<string, unknown>;
    const datasetId = typeof body["dataset_id"] === "string" ? (body["dataset_id"] as string) : null;
    this.#counter += 1;
    const runId = `run-${this.#counter}`;
    const name = typeof body["name"] === "string" ? (body["name"] as string) : `discovery-${this.#counter}`;
    const summary: RunSummary = {
      id: runId,
      organization_id: this.#org,
      name,
      status: "queued",
      created_at: new Date(0).toISOString(),
      deleted_at: null,
      metadata: { kind: "discovery", phase: "queued" },
    };
    const failure = this.#failWith !== undefined && datasetId === this.#failWith ? "native discovery failed" : null;
    this.#runs.set(runId, { summary, reads: 0, worldId: null, failure });
    return summary;
  }

  #getRun(runId: string): RunSummary {
    const entry = this.#run(runId);
    entry.reads += 1;
    const status = this.#statusFor(entry);
    if (status === "succeeded" && entry.worldId === null) entry.worldId = this.#world.id;
    const metadata: Record<string, unknown> = { kind: "discovery", phase: status };
    if (status === "failed" && entry.failure !== null) metadata["error"] = entry.failure;
    if (status === "succeeded") metadata["summary"] = { world_id: entry.worldId, laws: Object.keys(this.#world.equations).length };
    const summary: RunSummary = {
      ...entry.summary,
      status,
      ...(entry.worldId !== null ? { world_id: entry.worldId } : {}),
      metadata: metadata as unknown as JsonObject,
    };
    entry.summary = summary;
    return summary;
  }

  #statusFor(entry: RunEntry): RunStatus {
    // reads === 1 is the first poll after submit; keep it `running` for
    // `pollsUntilSucceeded` reads, then settle to the terminal state.
    if (entry.reads <= this.#pollsUntilSucceeded) return "running";
    return entry.failure !== null ? "failed" : "succeeded";
  }

  #getWorld(runId: string): RunWorld {
    const entry = this.#run(runId);
    const status = entry.summary.status;
    if (entry.worldId === null || status !== "succeeded") {
      throw new ApiError(`run has not produced a world yet (status=${status})`, { status: 409, code: "conflict" });
    }
    return {
      run_id: runId,
      world_id: entry.worldId,
      world: this.#worldRecord(),
      links: {
        self: `/v1/worlds/${entry.worldId}`,
        explain: `/v1/worlds/${entry.worldId}/explain`,
        report: `/v1/worlds/${entry.worldId}/report`,
      },
    };
  }

  #worldRecord(): WorldRecord {
    return {
      id: this.#world.id,
      organization_id: this.#org,
      name: this.#world.name,
      created_at: new Date(0).toISOString(),
      deleted_at: null,
      states: [...this.#world.states],
      controls: [...(this.#world.controls ?? [])],
      parameters: { ...(this.#world.parameters ?? {}) },
      equations: { ...this.#world.equations },
    };
  }

  #explain(worldId: string): WorldExplanation {
    const targets = Object.keys(this.#world.equations).sort();
    return {
      id: worldId,
      name: this.#world.name,
      variables: [...this.#world.states],
      controls: [...(this.#world.controls ?? [])],
      parameters: { ...(this.#world.parameters ?? {}) },
      laws: targets.map((target) => ({
        target,
        expression: this.#world.equations[target] ?? "",
        readable: `d${target}/dt = ${this.#world.equations[target] ?? ""}`,
        terms: [],
        dominant_term: null,
      })),
      dependencies: Object.fromEntries(targets.map((target) => [target, []])),
      complexity: {
        laws: targets.length,
        parameters: Object.keys(this.#world.parameters ?? {}).length,
        controls: (this.#world.controls ?? []).length,
        total_terms: 0,
        terms_per_law: Object.fromEntries(targets.map((target) => [target, 0])),
      },
      assumptions: ["Deterministic and offline — identical inputs reproduce this world exactly."],
    };
  }

  #forecast(worldId: string, body: unknown): WorldForecast {
    const spec = (typeof body === "object" && body !== null ? body : {}) as Record<string, unknown>;
    const horizon = typeof spec["horizon"] === "number" ? (spec["horizon"] as number) : 1;
    const step = typeof spec["step"] === "number" ? (spec["step"] as number) : horizon;
    const start = typeof spec["start"] === "number" ? (spec["start"] as number) : 0;
    const initial = (typeof spec["initial"] === "object" && spec["initial"] !== null ? spec["initial"] : {}) as Record<string, number>;
    const times: number[] = [];
    for (let t = start; t <= horizon + 1e-9; t += step) times.push(Number(t.toFixed(6)));
    const values: Record<string, number[]> = {};
    for (const name of this.#world.states) values[name] = times.map(() => initial[name] ?? 0);
    return {
      id: worldId,
      name: this.#world.name,
      start,
      horizon,
      step,
      interventions: [],
      trajectory: { time: times, values },
    };
  }

  #report(worldId: string): string {
    return `<!doctype html><title>LawSynth World — ${this.#world.name}</title><h1>${worldId}</h1>`;
  }

  #compare(body: unknown): WorldComparison {
    const spec = (typeof body === "object" && body !== null ? body : {}) as Record<string, unknown>;
    const left = typeof spec["left"] === "string" ? (spec["left"] as string) : null;
    const right = typeof spec["right"] === "string" ? (spec["right"] as string) : null;
    const empty = { added: [], removed: [], common: [] };
    return {
      left: { id: left, name: this.#world.name },
      right: { id: right, name: this.#world.name },
      variables: { ...empty, common: [...this.#world.states] },
      controls: empty,
      parameters: { added: {}, removed: {}, changed: {}, unchanged: [] },
      laws: { added: [], removed: [], changed: [], unchanged: Object.keys(this.#world.equations).sort() },
      complexity_delta: { laws: 0, parameters: 0, controls: 0, total_terms: 0 },
    };
  }

  #run(runId: string): RunEntry {
    const entry = this.#runs.get(runId);
    if (entry === undefined) throw new ApiError(`unknown run: ${runId}`, { status: 404, code: "not_found" });
    return entry;
  }

  #notFound(path: string): ApiError {
    return new ApiError(`the fake transport has no route for ${path}`, { status: 404, code: "not_found" });
  }
}
