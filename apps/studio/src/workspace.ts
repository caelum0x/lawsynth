import type { Project, RunSummary, WorldRevision } from "@lawsynth/api-client";
import type { StateStore } from "@lawsynth/state-store";
import type { LawSynthClient } from "@lawsynth/api-client";

export interface WorkspaceResources {
  readonly project: Project;
  readonly worlds: readonly WorldRevision[];
  readonly runs: readonly RunSummary[];
  readonly loadedAt: number;
}

export type WorkspacePhase = "idle" | "loading" | "ready" | "error";
export interface WorkspaceSnapshot {
  readonly phase: WorkspacePhase;
  readonly projectId?: string;
  readonly resources?: WorkspaceResources;
  readonly error?: Error;
}

function errorValue(error: unknown): Error { return error instanceof Error ? error : new Error(String(error)); }

export class WorkspaceController extends EventTarget {
  #snapshot: WorkspaceSnapshot = Object.freeze({ phase: "idle" });
  #abort: AbortController | undefined;
  #sequence = 0;

  constructor(readonly api: LawSynthClient, readonly store: StateStore, readonly clock: () => number = Date.now) { super(); }
  get snapshot(): WorkspaceSnapshot { return this.#snapshot; }

  async open(projectId: string): Promise<WorkspaceResources> {
    if (!projectId.trim()) throw new RangeError("project id is required");
    const sequence = ++this.#sequence;
    this.#abort?.abort(new DOMException("workspace changed", "AbortError"));
    const controller = new AbortController();
    this.#abort = controller;
    this.#commit({ phase: "loading", projectId });
    try {
      const [project, worldsPage, runsPage] = await Promise.all([
        this.api.projects.get(projectId, controller.signal),
        this.api.worlds.list(projectId, { limit: 100, signal: controller.signal }),
        this.api.runs.list(projectId, { limit: 100, signal: controller.signal }),
      ]);
      if (controller.signal.aborted || sequence !== this.#sequence) throw new DOMException("stale workspace load", "AbortError");
      const resources = Object.freeze({ project, worlds: Object.freeze([...worldsPage.items]), runs: Object.freeze([...runsPage.items]), loadedAt: this.clock() });
      this.store.dispatch({ kind: "workspace.update", patch: { projectId, route: `/projects/${encodeURIComponent(projectId)}` } });
      this.#commit({ phase: "ready", projectId, resources });
      return resources;
    } catch (error) {
      if (controller.signal.aborted) throw error;
      const failure = errorValue(error);
      this.#commit({ phase: "error", projectId, error: failure });
      throw failure;
    } finally { if (this.#abort === controller) this.#abort = undefined; }
  }

  selectWorld(worldId: string): void {
    const projectId = this.#snapshot.projectId;
    if (projectId === undefined || this.#snapshot.phase !== "ready") throw new Error("open a workspace before selecting a world");
    if (!this.#snapshot.resources?.worlds.some((world) => world.id === worldId || world.world_id === worldId)) throw new RangeError(`world ${worldId} is not in the active workspace`);
    this.store.dispatch({ kind: "workspace.update", patch: { worldId, route: `/projects/${encodeURIComponent(projectId)}/worlds/${encodeURIComponent(worldId)}` } });
  }

  selectRun(runId: string): void {
    const projectId = this.#snapshot.projectId;
    if (projectId === undefined || this.#snapshot.phase !== "ready") throw new Error("open a workspace before selecting a run");
    if (!this.#snapshot.resources?.runs.some((run) => run.id === runId)) throw new RangeError(`run ${runId} is not in the active workspace`);
    this.store.dispatch({ kind: "workspace.update", patch: { runId, route: `/projects/${encodeURIComponent(projectId)}/discovery/${encodeURIComponent(runId)}` } });
  }

  close(): void {
    this.#sequence += 1;
    this.#abort?.abort(new DOMException("workspace closed", "AbortError"));
    this.#abort = undefined;
    this.store.dispatch({ kind: "workspace.clear" });
    this.#commit({ phase: "idle" });
  }

  #commit(snapshot: WorkspaceSnapshot): void {
    this.#snapshot = Object.freeze(snapshot);
    this.dispatchEvent(new CustomEvent("change", { detail: this.#snapshot }));
  }
}
