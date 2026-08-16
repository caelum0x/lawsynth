import type {
  ArtifactDescriptor, CandidateSummary, LawSynthClient, Project, RunSummary,
  SimulationSummary, WorldRevision,
} from "@lawsynth/api-client";
import { StateStore } from "@lawsynth/state-store";
import type { WorldDefinition } from "@lawsynth/world-schema";

export function equal<T>(actual: T, expected: T, message?: string): void {
  if (!Object.is(actual, expected)) throw new Error(message ?? `expected ${String(expected)}, received ${String(actual)}`);
}

export function deepEqual(actual: unknown, expected: unknown, message?: string): void {
  const left = JSON.stringify(actual);
  const right = JSON.stringify(expected);
  if (left !== right) throw new Error(message ?? `expected ${right}, received ${left}`);
}

export function rejects(action: () => unknown | Promise<unknown>, pattern: RegExp): Promise<void> {
  return Promise.resolve().then(action).then(
    () => { throw new Error(`expected rejection matching ${pattern}`); },
    (error: unknown) => { if (!pattern.test(error instanceof Error ? error.message : String(error))) throw error; },
  );
}

export const world: WorldDefinition = {
  formatVersion: "0.1.0",
  id: "predator_prey",
  name: "Predator prey",
  time: { kind: "continuous", symbol: "t", unit: "day" },
  variables: [{ id: "prey", role: "state" }, { id: "predator", role: "state" }],
  parameters: [{ id: "growth", value: 0.2, bounds: [0, 1] }],
  laws: [
    { id: "prey_growth", kind: "continuous", target: "prey", expression: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "growth" }, right: { kind: "symbol", id: "prey" } } },
    { id: "predator_decay", kind: "continuous", target: "predator", enabled: false, expression: { kind: "unary", operator: "neg", operand: { kind: "symbol", id: "predator" } } },
  ],
  dependencies: { nodes: ["prey", "predator"], edges: [{ id: "prey_to_predator", source: "prey", target: "predator", kind: "directed", status: "candidate", strength: 0.8 }, { id: "predator_peer", source: "predator", target: "prey", kind: "undirected", status: "identified", strength: 0.2 }] },
  regimes: { regimes: [{ id: "baseline", lawIds: ["prey_growth"] }], intervals: [{ regime: "baseline", start: 0, end: 10 }] },
  uncertainty: { entries: [{ level: "parameter", parameter: "growth", standardError: 0.02 }, { level: "trajectory", bands: [{ variable: "prey", times: [0, 1], lower: [1, 2], upper: [2, 3], confidence: 0.95 }] }] },
  provenance: { createdAt: "2026-01-01T00:00:00.000Z", seed: 7, worldHash: "a".repeat(64), algorithms: [{ name: "sindy", version: "1.0.0", deterministic: true }] },
};

const project: Project = { id: "project_1", organization_id: "org", name: "Ecology", created_at: "2026-01-01T00:00:00Z", deleted_at: null };
const run: RunSummary = { id: "run_1", organization_id: "org", name: "Discover prey", status: "queued", created_at: "2026-01-01T00:00:00Z", deleted_at: null };
const revision: WorldRevision = { id: "world_1", world_id: "world_1", organization_id: "org", name: "Ecology world", equations: ["dprey/dt = growth * prey"], created_at: "2026-01-01T00:00:00Z", deleted_at: null };
const simulation: SimulationSummary = { id: "simulation_1", organization_id: "org", name: "Simulation", world_id: "world_1", status: "queued", created_at: "2026-01-01T00:00:00Z", deleted_at: null };
const artifact: ArtifactDescriptor = { id: "artifact_1", project_id: "project_1", run_id: null, media_type: "json", byte_len: 42, sha256: "b".repeat(64) };

/**
 * Deterministic in-memory implementation of the Studio API contract.
 * It owns run/simulation state transitions rather than returning a fabricated
 * success response, so controller tests exercise observation and completion.
 */
export function inMemoryApi(): LawSynthClient {
  let activeRun: RunSummary = run;
  let activeSimulation: SimulationSummary = simulation;
  const candidates: readonly CandidateSummary[] = [{ id: "candidate_2", run_id: run.id, score: 0.3 }, { id: "candidate_1", run_id: run.id, score: 0.9 }];
  const api = {
    projects: { get: async () => project },
    worlds: {
      list: async () => ({ items: [revision], next: null }),
      simulate: async (worldId: string) => {
        if (worldId !== revision.id) throw new RangeError(`unknown world: ${worldId}`);
        activeSimulation = { ...simulation, status: "running" };
        return activeSimulation;
      },
    },
    runs: {
      list: async () => ({ items: [activeRun], next: null }),
      create: async (request: { readonly name: string }) => { activeRun = { ...run, name: request.name, status: "queued" }; return activeRun; },
      get: async () => activeRun,
      cancel: async () => { activeRun = { ...activeRun, status: "cancelled" }; return activeRun; },
      candidates: async () => ({ items: activeRun.status === "succeeded" ? candidates : [], next: null }),
    },
    simulations: {
      get: async () => { activeSimulation = { ...activeSimulation, status: "succeeded", artifact_id: artifact.id }; return activeSimulation; },
      cancel: async () => { activeSimulation = { ...activeSimulation, status: "cancelled" }; return activeSimulation; },
      artifact: async () => { if (activeSimulation.status !== "succeeded") throw new Error("simulation has no artifact"); return artifact; },
    },
    async *events() {
      yield { event_id: "event_1", organization_id: "org", topic: "runs.progress", occurred_at: "2026-01-01T00:00:00Z", payload: { progress: 0.5, message: "searching" } };
      activeRun = { ...activeRun, status: "succeeded" };
      yield { event_id: "event_2", organization_id: "org", topic: "runs.succeeded", occurred_at: "2026-01-01T00:00:01Z", payload: { progress: 1 } };
    },
  };
  return api as unknown as LawSynthClient;
}

export function store(): StateStore { return new StateStore({ clock: () => 0, eventId: () => "event" }); }
