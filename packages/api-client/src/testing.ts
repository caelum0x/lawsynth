import type {
  Annotation,
  AnnotationTarget,
  MergeConflict,
  MergeResult,
  ProjectMember,
  ProjectRole,
  RevisionParentRef,
  RevisionRecord,
  ReviewState,
  WorkspaceRow,
} from "./collaboration.js";
import { ApiError } from "./errors.js";
import type {
  JsonObject,
  JsonValue,
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

// --------------------------------------------------------------------------- //
// Collaboration fake transport                                                 //
// --------------------------------------------------------------------------- //

/** A workspace index row to seed into {@link InMemoryCollaborationTransport}. */
export interface SeedRevision {
  readonly contentHash?: string;
  readonly actor?: string;
  readonly reviewState?: ReviewState;
  readonly derivationKind?: RevisionRecord["derivation"]["kind"];
  readonly parents?: readonly RevisionParentRef[];
}

export interface InMemoryCollaborationOptions {
  readonly projectId?: string;
  readonly worldId?: string;
  /** Principal treated as the project owner (the creator). Defaults to `token:owner`. */
  readonly owner?: string;
  /** Additional seeded members (principal -> role). */
  readonly members?: Readonly<Record<string, ProjectRole>>;
  /** How many draft revisions to seed for `worldId` (default 1), or explicit records. */
  readonly revisions?: number | readonly SeedRevision[];
  /**
   * The role the transport treats the caller as, so a single client can exercise
   * the server-side role gate offline. Defaults to `owner` (fully permitted).
   */
  readonly actorRole?: ProjectRole;
  /** The principal the transport attributes writes to. Defaults to `owner`. */
  readonly actor?: string;
}

const REVIEW_TRANSITIONS: Record<ReviewState, readonly ReviewState[]> = {
  draft: ["in_review"],
  in_review: ["approved", "rejected"],
  rejected: ["in_review"],
  approved: [],
};

/**
 * A deterministic, dependency-free {@link Transport} that emulates the P6
 * collaboration surface (members/roles, revision lineage, annotations, the
 * review state machine, and the deterministic merge) so a wired client flow is
 * exercisable end-to-end offline — the collaboration analogue of
 * {@link InMemoryDiscoveryTransport}. Pass an instance straight to
 * `new LawSynthClient(transport)`.
 */
export class InMemoryCollaborationTransport implements Transport {
  readonly #members = new Map<string, Map<string, ProjectRole>>();
  readonly #revisions = new Map<string, RevisionRecord[]>();
  readonly #annotations = new Map<string, Annotation[]>();
  readonly #actorRole: ProjectRole;
  readonly #actor: string;
  #clock = 0;

  constructor(options: InMemoryCollaborationOptions = {}) {
    const owner = options.owner ?? "token:owner";
    this.#actorRole = options.actorRole ?? "owner";
    this.#actor = options.actor ?? owner;
    if (options.projectId !== undefined) {
      const roles = new Map<string, ProjectRole>([[owner, "owner"]]);
      for (const [principal, role] of Object.entries(options.members ?? {})) roles.set(principal, role);
      this.#members.set(options.projectId, roles);
    }
    if (options.worldId !== undefined) {
      const seeds: readonly SeedRevision[] = typeof options.revisions === "number"
        ? Array.from({ length: Math.max(0, options.revisions) }, () => ({}))
        : options.revisions ?? [{}];
      this.#revisions.set(options.worldId, seeds.map((seed, index) => this.#seedRevision(options.worldId!, index + 1, seed)));
    }
  }

  /** The current members of a seeded/mutated project, for assertions. */
  members(projectId: string): readonly ProjectMember[] {
    return this.#memberList(projectId);
  }

  request<T>(request: TransportRequest): Promise<T> {
    const method = request.method ?? "GET";
    const path = request.path.split("?")[0] ?? request.path;
    const parts = path.split("/").filter((part) => part.length > 0).map((part) => decodeURIComponent(part));
    try {
      return Promise.resolve(this.#route(method, parts, request) as T);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  #route(method: string, parts: readonly string[], request: TransportRequest): unknown {
    if (parts[0] !== "v1") throw this.#error(404, "not_found", `no route for ${request.path}`);
    const [, segment, id, action, sub, leaf] = parts;
    if (segment === "projects" && id !== undefined) {
      if (action === "members" && sub === undefined) {
        if (method === "GET") return { items: this.#memberList(id) };
        if (method === "POST") return this.#setMember(id, request.body);
      }
      if (action === "members" && sub !== undefined && method === "DELETE") return this.#removeMember(id, sub);
      if (action === "merge" && method === "POST") return this.#merge(id, request.body);
    }
    if (segment === "worlds" && id !== undefined) {
      if (action === "annotations" && sub === undefined) {
        if (method === "GET") return { items: this.#annotationList(id) };
        if (method === "POST") return this.#addAnnotation(id, request.body);
      }
      if (action === "revisions" && sub === undefined && method === "GET") return this.#listRevisions(id);
      if (action === "revisions" && sub !== undefined && leaf === undefined && method === "GET") return this.#getRevision(id, sub);
      if (action === "revisions" && sub !== undefined && leaf === "review" && method === "POST") return this.#review(id, sub, request.body);
    }
    throw this.#error(404, "not_found", `no route for ${request.path}`);
  }

  // -- members ------------------------------------------------------------- //

  #memberList(projectId: string): readonly ProjectMember[] {
    const roles = this.#members.get(projectId);
    if (roles === undefined) return [];
    return [...roles.entries()].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).map(([principal, role]) => ({ principal, role }));
  }

  #setMember(projectId: string, body: unknown): ProjectMember {
    this.#requireOwner();
    const spec = this.#object(body);
    const principal = typeof spec["principal"] === "string" ? (spec["principal"] as string) : "";
    const role = spec["role"];
    if (!principal) throw this.#error(422, "validation_error", "member principal is required");
    if (role !== "owner" && role !== "editor" && role !== "viewer") throw this.#error(422, "validation_error", "role must be owner, editor, or viewer");
    const roles = this.#members.get(projectId) ?? new Map<string, ProjectRole>();
    roles.set(principal, role);
    this.#members.set(projectId, roles);
    return { principal, role };
  }

  #removeMember(projectId: string, principal: string): undefined {
    this.#requireOwner();
    const roles = this.#members.get(projectId);
    if (roles === undefined || !roles.has(principal)) throw this.#error(404, "not_found", "member not found");
    const owners = [...roles.entries()].filter(([, role]) => role === "owner").map(([who]) => who);
    if (roles.get(principal) === "owner" && owners.length === 1 && owners[0] === principal) {
      throw this.#error(409, "conflict", "cannot remove the last owner of a project");
    }
    roles.delete(principal);
    return undefined;
  }

  // -- revisions & review -------------------------------------------------- //

  #listRevisions(worldId: string): { items: readonly RevisionRecord[]; trusted: boolean } {
    const chain = this.#revisions.get(worldId) ?? [];
    return { items: chain.map((record) => this.#public(record, false)), trusted: chain.some((record) => record.review_state === "approved") };
  }

  #getRevision(worldId: string, raw: string): RevisionRecord {
    return this.#public(this.#revision(worldId, raw), true);
  }

  #review(worldId: string, raw: string, body: unknown): RevisionRecord {
    if (this.#actorRole === "viewer") throw this.#error(403, "forbidden", "reviewing requires editor or owner");
    const record = this.#revision(worldId, raw);
    const spec = this.#object(body);
    const target = spec["state"] ?? spec["to"];
    if (target !== "in_review" && target !== "approved" && target !== "rejected") {
      throw this.#error(422, "validation_error", "review state must be in_review, approved, or rejected");
    }
    if (!REVIEW_TRANSITIONS[record.review_state].includes(target)) {
      throw this.#error(409, "conflict", `cannot transition review from ${record.review_state} to ${target}`);
    }
    if (target === "approved" && this.#actorRole !== "owner") throw this.#error(403, "forbidden", "only an owner may approve a revision");
    const next: RevisionRecord = {
      ...record,
      review_state: target,
      review_history: [...record.review_history, { from: record.review_state, to: target, actor: this.#actor, at: this.#now() }],
    };
    this.#replace(worldId, next);
    return this.#public(next, true);
  }

  // -- annotations --------------------------------------------------------- //

  #annotationList(worldId: string): readonly Annotation[] {
    return [...(this.#annotations.get(worldId) ?? [])];
  }

  #addAnnotation(worldId: string, body: unknown): Annotation {
    if (this.#actorRole === "viewer") throw this.#error(403, "forbidden", "annotating requires editor or owner");
    const spec = this.#object(body);
    const text = typeof spec["text"] === "string" ? (spec["text"] as string).trim() : "";
    if (!text) throw this.#error(422, "validation_error", "annotation text is required");
    const target: AnnotationTarget = spec["target"] === "law" || spec["target"] === "revision" ? spec["target"] : "world";
    const rawRef = spec["ref"];
    let ref: string | number | null = null;
    if (target === "law") {
      if (typeof rawRef !== "string" || rawRef === "") throw this.#error(422, "validation_error", "a law annotation requires a law ref");
      ref = rawRef;
    } else if (target === "revision") {
      if (typeof rawRef !== "number" || !Number.isInteger(rawRef) || rawRef < 1) throw this.#error(422, "validation_error", "a revision annotation requires a positive revision ref");
      ref = rawRef;
    }
    const log = this.#annotations.get(worldId) ?? [];
    const record: Annotation = { world_id: worldId, ordinal: log.length + 1, target, ref, text, actor: this.#actor, created_at: this.#now() };
    log.push(record);
    this.#annotations.set(worldId, log);
    return record;
  }

  // -- merge --------------------------------------------------------------- //

  #merge(projectId: string, body: unknown): MergeResult {
    if (this.#actorRole === "viewer") throw this.#error(403, "forbidden", "merge requires editor or owner");
    void projectId;
    const spec = this.#object(body);
    const left = this.#index(spec["base"], "base");
    const right = this.#index(spec["incoming"], "incoming");
    const merged: WorkspaceRow[] = [];
    const conflicts: MergeConflict[] = [];
    for (const name of [...new Set([...left.keys(), ...right.keys()])].sort()) {
      const a = left.get(name);
      const b = right.get(name);
      if (a !== undefined && b !== undefined) {
        if (a.content_hash === b.content_hash) merged.push(a.revision >= b.revision ? a : b);
        else conflicts.push({ name, base: a, incoming: b });
      } else merged.push((a ?? b) as WorkspaceRow);
    }
    return { merged, conflicts, merged_count: merged.length, conflict_count: conflicts.length };
  }

  #index(rows: unknown, side: string): Map<string, WorkspaceRow> {
    if (!Array.isArray(rows)) throw this.#error(422, "validation_error", `${side} workspace index must be a list of rows`);
    const index = new Map<string, WorkspaceRow>();
    for (const entry of rows as readonly unknown[]) {
      const row = this.#object(entry);
      const name = row["name"];
      const contentHash = row["content_hash"];
      if (typeof name !== "string" || name === "") throw this.#error(422, "validation_error", `${side} workspace rows require a name`);
      if (typeof contentHash !== "string" || contentHash === "") throw this.#error(422, "validation_error", `workspace row '${name}' requires a content_hash`);
      const revision = typeof row["revision"] === "number" ? (row["revision"] as number) : 0;
      index.set(name, { ...(row as Record<string, JsonValue>), name, content_hash: contentHash, revision } as WorkspaceRow);
    }
    return index;
  }

  // -- helpers ------------------------------------------------------------- //

  #seedRevision(worldId: string, number: number, seed: SeedRevision): RevisionRecord {
    const contentHash = seed.contentHash ?? `hash-${worldId}-${number}`;
    return {
      world_id: worldId,
      number,
      content_hash: contentHash,
      derivation: { kind: seed.derivationKind ?? "imported", source_hash: contentHash },
      parents: seed.parents ?? [],
      actor: seed.actor ?? this.#actor,
      created_at: this.#now(),
      review_state: seed.reviewState ?? "draft",
      review_history: [],
    };
  }

  #revision(worldId: string, raw: string): RevisionRecord {
    if (!/^\d+$/u.test(raw)) throw this.#error(422, "validation_error", "revision number must be a positive integer");
    const number = Number(raw);
    const chain = this.#revisions.get(worldId) ?? [];
    const record = chain[number - 1];
    if (number < 1 || record === undefined) throw this.#error(404, "not_found", "revision not found");
    return record;
  }

  #replace(worldId: string, record: RevisionRecord): void {
    const chain = this.#revisions.get(worldId);
    if (chain !== undefined) chain[record.number - 1] = record;
  }

  #public(record: RevisionRecord, withTrust: boolean): RevisionRecord {
    return withTrust ? { ...record, trusted: record.review_state === "approved" } : { ...record };
  }

  #requireOwner(): void {
    if (this.#actorRole !== "owner") throw this.#error(403, "forbidden", "only an owner may manage membership");
  }

  #object(value: unknown): Record<string, JsonValue> {
    if (typeof value !== "object" || value === null) throw this.#error(422, "validation_error", "request body must be an object");
    return value as Record<string, JsonValue>;
  }

  #now(): string {
    this.#clock += 1;
    return new Date(this.#clock * 1000).toISOString();
  }

  #error(status: number, code: string, message: string): ApiError {
    return new ApiError(message, { status, code });
  }
}
