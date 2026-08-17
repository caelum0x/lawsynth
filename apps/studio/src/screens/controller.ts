import type {
  CandidateSummary,
  ForecastRequest,
  LawSynthClient,
  RunStatus,
  RunSummary,
  WorldComparison,
  WorldExplanation,
  WorldForecast,
} from "@lawsynth/api-client";
import type { TrajectoryInput } from "@lawsynth/chart-core";
import { primarySelection, type StateStore } from "@lawsynth/state-store";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { worldFromRecord } from "../world-adapter.js";
import {
  defaultDiscoveryConfig,
  discoveryCanvasModel,
  discoverySubmitRequest,
  SOLVERS,
  type DiscoveryCanvasConfig,
  type DiscoverySolver,
} from "./discovery-canvas.js";
import { dataLensModel } from "./data-lens.js";
import { dataPrepModel, defaultDataPrepConfig, prepareDataset, type DataPrepConfig } from "./data-prep.js";
import { equationExplorerModel } from "./equation-explorer.js";
import { exportScreenModel } from "./export-screen.js";
import { fixtureCandidates, fixtureWorld, FIXTURE_INITIAL_STATE } from "./fixtures.js";
import { defaultMonitorConfig, monitorModel, MONITOR_SOURCES, type MonitorConfig, type MonitorSource } from "./monitor.js";
import { regimeTimelineModel } from "./regime-timeline.js";
import { forwardEuler } from "./simulate.js";
import {
  BASELINE_SCENARIO_ID,
  defaultScenarios,
  scenarioBoardModel,
  type ScenarioDefinition,
  type ScenarioDraft,
} from "./scenario-board.js";
import { structureMapModel } from "./structure-map.js";
import type { Notice, ScreenId, ScreenModel, ScreenSection } from "./types.js";
import { uncertaintyLensModel } from "./uncertainty-lens.js";
import { worldLabModel } from "./world-lab.js";

/** Bounded polling of a live discovery run, injectable so tests avoid wall-clock waits. */
export interface DiscoveryPollOptions {
  readonly waitMs?: number;
  readonly maxAttempts?: number;
  readonly sleep?: (ms: number) => Promise<void>;
}

interface ResolvedPoll {
  readonly waitMs: number;
  readonly maxAttempts: number;
  readonly sleep: (ms: number) => Promise<void>;
}

const TERMINAL: ReadonlySet<RunStatus> = new Set<RunStatus>(["succeeded", "failed", "cancelled"]);

export interface ScreensControllerOptions {
  readonly store: StateStore;
  readonly api?: LawSynthClient;
  readonly randomId?: () => string;
  readonly world?: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly candidates?: readonly CandidateSummary[];
  readonly datasets?: readonly { readonly id: string; readonly name: string }[];
  readonly columns?: readonly string[];
  readonly poll?: DiscoveryPollOptions;
}

interface LabState {
  readonly overrides: Readonly<Record<string, number>>;
  readonly activeInterventionIds: readonly string[];
  readonly horizon: number;
  readonly step: number;
}

interface BoardState {
  readonly scenarios: readonly ScenarioDefinition[];
  readonly draft: ScenarioDraft;
  readonly focusVariable: string;
  readonly horizon: number;
  readonly step: number;
}

const EMPTY_DRAFT: ScenarioDraft = Object.freeze({ name: "", overrides: Object.freeze({}), activeInterventionIds: Object.freeze([]) });

/** Maps a table/section id to the selection namespace it drives. */
const SECTION_KIND: Readonly<Record<string, string>> = {
  candidates: "cand",
  "eq-list": "law",
  bands: "bandvar",
  timeline: "regime",
  "structure-graph": "var",
  "structure-laws": "law",
  scenarios: "scenario",
  "mon-anomalies": "anomaly",
};

function num(raw: string, fallback: number): number {
  const value = Number(raw);
  return Number.isFinite(value) ? value : fallback;
}

/**
 * Reactive hub for the Studio screens. Selection is stored in the shared
 * `state-store` (namespaced so screens never collide), while screen-specific
 * configuration (discovery run, lab overrides) lives here as immutable state.
 * Any store change or config edit emits a `change` event so the host re-renders.
 */
export class ScreensController extends EventTarget {
  readonly #store: StateStore;
  readonly #api: LawSynthClient | undefined;
  readonly #randomId: () => string;
  readonly #columns: readonly string[];
  readonly #datasets: readonly { readonly id: string; readonly name: string }[];
  readonly #poll: ResolvedPoll;
  readonly #unsubscribe: () => void;
  #worldId: string | undefined;
  #explanation: WorldExplanation | undefined;
  #forecast: WorldForecast | undefined;
  #comparison: WorldComparison | undefined;
  #report: string | undefined;

  #screen: ScreenId = "discovery-canvas";
  #world: WorldDefinition;
  #trajectory: TrajectoryInput | undefined;
  #candidates: readonly CandidateSummary[];
  #discovery: DiscoveryCanvasConfig;
  #focusVariable = "";
  #lab: LabState;
  #board: BoardState;
  #prep: DataPrepConfig;
  #prepApplied = false;
  #monitor: MonitorConfig;
  #runStatus: RunStatus | undefined;
  #runProgress = 0;
  #running = false;
  #simStatus: RunStatus | undefined;
  #simRunning = false;
  #lastError: string | undefined;

  constructor(options: ScreensControllerOptions) {
    super();
    this.#store = options.store;
    this.#api = options.api;
    this.#randomId = options.randomId ?? (() => `screen-${Math.random().toString(36).slice(2)}`);
    this.#world = options.world ?? fixtureWorld();
    this.#trajectory = options.trajectory;
    this.#candidates = options.candidates ?? [];
    this.#datasets = options.datasets ?? [{ id: "dataset-demo", name: "Demo dataset" }];
    this.#columns = options.columns ?? [this.#world.time.symbol ?? "t", ...this.#world.variables.map((variable) => variable.id)];
    this.#poll = {
      waitMs: options.poll?.waitMs ?? 250,
      maxAttempts: options.poll?.maxAttempts ?? 40,
      sleep: options.poll?.sleep ?? ((ms) => new Promise((resolve) => setTimeout(resolve, ms))),
    };
    this.#discovery = defaultDiscoveryConfig(this.#datasets[0]?.id ?? "", this.#columns);
    this.#lab = {
      overrides: {},
      activeInterventionIds: (this.#world.interventions ?? []).map((intervention) => intervention.id),
      horizon: 12,
      step: 0.1,
    };
    this.#board = {
      scenarios: defaultScenarios(this.#world),
      draft: EMPTY_DRAFT,
      focusVariable: "",
      horizon: 12,
      step: 0.1,
    };
    this.#prep = defaultDataPrepConfig();
    this.#monitor = defaultMonitorConfig();
    this.#unsubscribe = this.#store.subscribe(() => this.#emit());
  }

  get screen(): ScreenId {
    return this.#screen;
  }
  get world(): WorldDefinition {
    return this.#world;
  }
  get runStatus(): RunStatus | undefined {
    return this.#runStatus;
  }
  get worldId(): string | undefined {
    return this.#worldId;
  }
  get explanation(): WorldExplanation | undefined {
    return this.#explanation;
  }
  get forecast(): WorldForecast | undefined {
    return this.#forecast;
  }
  get comparison(): WorldComparison | undefined {
    return this.#comparison;
  }
  get report(): string | undefined {
    return this.#report;
  }
  get lastError(): string | undefined {
    return this.#lastError;
  }

  setScreen(screen: ScreenId): void {
    if (screen === this.#screen) return;
    this.#screen = screen;
    this.#emit();
  }

  setWorld(world: WorldDefinition, trajectory?: TrajectoryInput): void {
    this.#world = world;
    this.#trajectory = trajectory;
    this.#lab = { ...this.#lab, overrides: {}, activeInterventionIds: (world.interventions ?? []).map((intervention) => intervention.id) };
    this.#board = { ...this.#board, scenarios: defaultScenarios(world), draft: EMPTY_DRAFT, focusVariable: "" };
    this.#prepApplied = false;
    this.#emit();
  }

  dispose(): void {
    this.#unsubscribe();
  }

  /** Builds the render description for the active screen from current state + shared selection. */
  model(): ScreenModel {
    const model = this.#buildModel();
    if (this.#lastError === undefined) return model;
    const notice: ScreenSection = { kind: "notices", id: "controller-error", notices: [{ tone: "error", message: this.#lastError } satisfies Notice] };
    return { ...model, sections: [notice, ...model.sections] };
  }

  #buildModel(): ScreenModel {
    switch (this.#screen) {
      case "data-lens":
        return dataLensModel({
          world: this.#world,
          initialState: this.#trajectoryInitial(),
          ...(this.#trajectory === undefined ? {} : { trajectory: this.#trajectory }),
        });
      case "data-prep":
        return dataPrepModel({
          world: this.#world,
          initialState: this.#trajectoryInitial(),
          config: this.#prep,
          applied: this.#prepApplied,
          ...(this.#trajectory === undefined ? {} : { trajectory: this.#trajectory }),
        });
      case "monitor":
        return monitorModel({
          world: this.#world,
          initialState: this.#trajectoryInitial(),
          config: this.#monitor,
          ...(this.#trajectory === undefined ? {} : { observed: this.#trajectory }),
          ...(this.#selected("anomaly") === undefined ? {} : { selectedAnomalyId: this.#selected("anomaly")! }),
        });
      case "discovery-canvas":
        return discoveryCanvasModel({
          config: this.#discovery,
          columns: this.#columns,
          datasets: this.#datasets,
          candidates: this.#candidates,
          running: this.#running,
          ...(this.#selected("cand") === undefined ? {} : { selectedCandidateId: this.#selected("cand")! }),
          ...(this.#runStatus === undefined ? {} : { runStatus: this.#runStatus }),
          progress: this.#runProgress,
        });
      case "equation-explorer": {
        // Focus follows a Structure Map node selection (shared store) or this
        // screen's own dropdown, so selecting a variable there focuses it here.
        const focus = this.#selected("var") ?? (this.#focusVariable === "" ? undefined : this.#focusVariable);
        return equationExplorerModel({
          world: this.#world,
          ...(this.#selected("law") === undefined ? {} : { selectedLawId: this.#selected("law")! }),
          ...(focus === undefined || focus === "" ? {} : { focusVariableId: focus }),
        });
      }
      case "structure-map":
        return structureMapModel({
          world: this.#world,
          ...(this.#selected("var") === undefined ? {} : { selectedVariableId: this.#selected("var")! }),
        });
      case "export-screen":
        return exportScreenModel({ world: this.#world });
      case "regime-timeline":
        return regimeTimelineModel({
          world: this.#world,
          ...(this.#selected("regime") === undefined ? {} : { selectedRegime: this.#selected("regime")! }),
        });
      case "uncertainty-lens":
        return uncertaintyLensModel({
          world: this.#world,
          ...(this.#trajectory === undefined ? {} : { trajectory: this.#trajectory }),
          ...(this.#selected("bandvar") === undefined ? {} : { selectedVariable: this.#selected("bandvar")! }),
        });
      case "world-lab":
        return worldLabModel({
          world: this.#world,
          initialState: this.#trajectoryInitial(),
          overrides: this.#lab.overrides,
          activeInterventionIds: this.#lab.activeInterventionIds,
          horizon: this.#lab.horizon,
          step: this.#lab.step,
          running: this.#simRunning,
          ...(this.#simStatus === undefined ? {} : { simulationStatus: this.#simStatus }),
        });
      case "scenario-board":
        return scenarioBoardModel({
          world: this.#world,
          initialState: this.#trajectoryInitial(),
          scenarios: this.#board.scenarios,
          draft: this.#board.draft,
          horizon: this.#board.horizon,
          step: this.#board.step,
          ...(this.#board.focusVariable === "" ? {} : { focusVariableId: this.#board.focusVariable }),
          ...(this.#selected("scenario") === undefined ? {} : { selectedScenarioId: this.#selected("scenario")! }),
        });
    }
  }

  onSelect(sectionId: string, rowId: string): void {
    const kind = SECTION_KIND[sectionId];
    if (kind === undefined) return;
    this.#selectEntity(kind, rowId);
  }

  onControl(fieldId: string, raw: string): void {
    if (fieldId.startsWith("cfg:")) this.#updateDiscovery(fieldId.slice(4), raw);
    else if (fieldId.startsWith("lab:")) this.#updateLab(fieldId.slice(4), raw);
    else if (fieldId.startsWith("board:")) this.#updateBoard(fieldId.slice("board:".length), raw);
    else if (fieldId.startsWith("prep:")) this.#updatePrep(fieldId.slice("prep:".length), raw);
    else if (fieldId.startsWith("mon:")) this.#updateMonitor(fieldId.slice("mon:".length), raw);
    else if (fieldId === "eq:law") this.#selectEntity("law", raw);
    else if (fieldId === "eq:variable") {
      this.#focusVariable = raw;
      this.#emit();
    } else if (fieldId === "unc:variable") this.#selectEntity("bandvar", raw);
    else if (fieldId === "structure:variable") this.#selectEntity("var", raw);
  }

  async onAction(actionId: string): Promise<void> {
    this.#lastError = undefined;
    if (actionId === "discovery:reset") {
      this.#discovery = defaultDiscoveryConfig(this.#datasets[0]?.id ?? "", this.#columns);
      this.#emit();
      return;
    }
    if (actionId === "lab:reset") {
      this.#lab = { ...this.#lab, overrides: {} };
      this.#emit();
      return;
    }
    if (actionId === "prep:apply") { this.#applyPrep(); return; }
    if (actionId === "prep:reset") {
      this.#prep = defaultDataPrepConfig();
      this.#prepApplied = false;
      this.#emit();
      return;
    }
    if (actionId === "board:add") return this.#addScenario();
    if (actionId === "board:remove") return this.#removeSelectedScenario();
    if (actionId === "board:reset") {
      this.#board = { ...this.#board, draft: EMPTY_DRAFT };
      this.#emit();
      return;
    }
    if (actionId === "discovery:run") return this.#runDiscovery();
    if (actionId === "lab:simulate") return this.#runSimulation();
    if (actionId === "world:explain") { await this.explainWorld(); return; }
    if (actionId === "world:report") { await this.reportWorld(); return; }
    if (actionId === "world:forecast") {
      await this.forecastWorld({ horizon: this.#lab.horizon, step: this.#lab.step, initial: this.#trajectoryInitial() });
      return;
    }
  }

  #trajectoryInitial(): Readonly<Record<string, number>> {
    if (this.#trajectory !== undefined && this.#trajectory.times.length > 0) {
      const first = this.#trajectory.values[0] ?? [];
      const state: Record<string, number> = {};
      this.#trajectory.variables.forEach((variable, index) => (state[variable] = first[index] ?? 0));
      return state;
    }
    return FIXTURE_INITIAL_STATE;
  }

  #updateDiscovery(key: string, raw: string): void {
    const config = this.#discovery;
    if (key.startsWith("input:")) {
      const column = key.slice("input:".length);
      const include = raw === "true";
      const set = new Set(config.stateColumns);
      if (include) set.add(column);
      else set.delete(column);
      this.#discovery = { ...config, stateColumns: Object.freeze([...set]) };
    } else if (key === "dataset") this.#discovery = { ...config, datasetId: raw };
    else if (key === "target") this.#discovery = { ...config, target: raw, stateColumns: Object.freeze(config.stateColumns.filter((column) => column !== raw)) };
    else if (key === "threshold") this.#discovery = { ...config, threshold: num(raw, config.threshold) };
    else if (key === "degree") this.#discovery = { ...config, degree: Math.round(num(raw, config.degree)) };
    else if (key === "seed") this.#discovery = { ...config, seed: Math.round(num(raw, config.seed)) };
    else if (key === "solver" && (SOLVERS as readonly string[]).includes(raw)) this.#discovery = { ...config, solver: raw as DiscoverySolver };
    else if (key === "pareto") this.#discovery = { ...config, pareto: raw === "true" };
    else if (key === "regimes") this.#discovery = { ...config, regimes: raw === "true" };
    else if (key === "refine") this.#discovery = { ...config, refine: raw === "true" };
    else if (key === "causal") this.#discovery = { ...config, causal: raw === "true" };
    this.#emit();
  }

  #updateLab(key: string, raw: string): void {
    if (key === "horizon") this.#lab = { ...this.#lab, horizon: num(raw, this.#lab.horizon) };
    else if (key === "step") this.#lab = { ...this.#lab, step: num(raw, this.#lab.step) };
    else if (key.startsWith("param:")) {
      const id = key.slice("param:".length);
      this.#lab = { ...this.#lab, overrides: { ...this.#lab.overrides, [id]: num(raw, this.#lab.overrides[id] ?? 0) } };
    } else if (key.startsWith("int:")) {
      const id = key.slice("int:".length);
      const active = raw === "true";
      const set = new Set(this.#lab.activeInterventionIds);
      if (active) set.add(id);
      else set.delete(id);
      this.#lab = { ...this.#lab, activeInterventionIds: Object.freeze([...set]) };
    }
    this.#emit();
  }

  #updateBoard(key: string, raw: string): void {
    const board = this.#board;
    if (key === "focus") this.#board = { ...board, focusVariable: raw };
    else if (key === "horizon") this.#board = { ...board, horizon: num(raw, board.horizon) };
    else if (key === "step") this.#board = { ...board, step: num(raw, board.step) };
    else if (key === "name") this.#board = { ...board, draft: { ...board.draft, name: raw } };
    else if (key.startsWith("param:")) {
      const id = key.slice("param:".length);
      this.#board = { ...board, draft: { ...board.draft, overrides: { ...board.draft.overrides, [id]: num(raw, board.draft.overrides[id] ?? 0) } } };
    } else if (key.startsWith("int:")) {
      const id = key.slice("int:".length);
      const set = new Set(board.draft.activeInterventionIds);
      if (raw === "true") set.add(id);
      else set.delete(id);
      this.#board = { ...board, draft: { ...board.draft, activeInterventionIds: Object.freeze([...set]) } };
    }
    this.#emit();
  }

  #updatePrep(key: string, raw: string): void {
    const prep = this.#prep;
    if (key === "smooth") this.#prep = { ...prep, smoothingWindow: Math.round(num(raw, prep.smoothingWindow)) };
    else if (key === "dt") this.#prep = { ...prep, resampleDt: Math.max(0, num(raw, prep.resampleDt)) };
    else if (key === "trim") this.#prep = { ...prep, trim: Math.max(0, Math.round(num(raw, prep.trim))) };
    else if (key === "detrend") this.#prep = { ...prep, detrend: raw === "true" };
    this.#emit();
  }

  #updateMonitor(key: string, raw: string): void {
    const monitor = this.#monitor;
    if (key === "threshold") this.#monitor = { ...monitor, threshold: num(raw, monitor.threshold) };
    else if (key === "step") this.#monitor = { ...monitor, step: num(raw, monitor.step) };
    else if (key === "source" && (MONITOR_SOURCES as readonly string[]).includes(raw)) this.#monitor = { ...monitor, source: raw as MonitorSource };
    this.#emit();
  }

  /**
   * Promotes the prepared dataset to the shared working dataset. The raw input
   * is the current working trajectory (or a local integration when none exists);
   * the prepared result becomes `#trajectory`, so the Data Lens, Monitor, and any
   * trajectory-driven screen immediately operate on the cleaned series — closing
   * the prep → discover loop.
   */
  #applyPrep(): void {
    const raw = this.#trajectory ?? (() => {
      try {
        return forwardEuler(this.#world, { horizon: 12, step: 0.1, initialState: this.#trajectoryInitial() });
      } catch {
        return undefined;
      }
    })();
    if (raw === undefined || raw.times.length === 0) return;
    const prepared = prepareDataset(raw, this.#prep).prepared;
    if (prepared.times.length === 0) return;
    this.#trajectory = prepared;
    this.#prepApplied = true;
    this.#emit();
  }

  #addScenario(): void {
    const draft = this.#board.draft;
    const index = this.#board.scenarios.length + 1;
    const name = draft.name.trim() === "" ? `Scenario ${index}` : draft.name.trim();
    const scenario: ScenarioDefinition = {
      id: this.#randomId(),
      name,
      overrides: Object.freeze({ ...draft.overrides }),
      activeInterventionIds: Object.freeze([...draft.activeInterventionIds]),
    };
    this.#board = { ...this.#board, scenarios: Object.freeze([...this.#board.scenarios, scenario]), draft: EMPTY_DRAFT };
    this.#selectEntity("scenario", scenario.id);
  }

  #removeSelectedScenario(): void {
    const selected = this.#selected("scenario");
    if (selected === undefined || selected === BASELINE_SCENARIO_ID) return;
    this.#board = { ...this.#board, scenarios: Object.freeze(this.#board.scenarios.filter((scenario) => scenario.id !== selected)) };
    this.#store.dispatch({ kind: "selection.set", ids: [] });
  }

  /**
   * Drives a LIVE discovery run against the API client when one is present:
   * submit -> bounded poll to a terminal status -> load the discovered world and
   * hand it to every screen. Falls back to the offline fixture candidates only
   * when no client is configured.
   */
  async #runDiscovery(): Promise<void> {
    this.#running = true;
    this.#runStatus = "queued";
    this.#runProgress = 0.1;
    this.#emit();
    try {
      if (this.#api === undefined) {
        this.#candidates = fixtureCandidates();
        this.#runStatus = "succeeded";
        this.#runProgress = 1;
        return;
      }
      const projectId = this.#store.state.workspace.projectId ?? "project-demo";
      const submitted = await this.#api.runs.submitDiscovery(discoverySubmitRequest(this.#discovery, projectId), this.#randomId());
      this.#runStatus = submitted.status;
      this.#runProgress = 0.25;
      this.#emit();
      const settled = await this.#pollRun(submitted.id);
      this.#runStatus = settled.status;
      if (settled.status === "succeeded") {
        this.#runProgress = 0.85;
        this.#emit();
        const runWorld = await this.#api.runs.getWorld(settled.id);
        this.#worldId = runWorld.world_id;
        this.#candidates = Object.freeze([]);
        this.setWorld(worldFromRecord(runWorld.world)); // hands the live world to every screen
        this.#runProgress = 1;
      } else if (settled.status === "failed" || settled.status === "cancelled") {
        this.#runProgress = 1;
        this.#lastError = this.#failureReason(settled) ?? `Discovery run ${settled.status}.`;
      } else {
        this.#runProgress = 1;
        this.#lastError = "Discovery run did not complete within the polling budget.";
      }
    } catch (error) {
      this.#runStatus = "failed";
      this.#lastError = error instanceof Error ? error.message : "Discovery run failed.";
    } finally {
      this.#running = false;
      this.#emit();
    }
  }

  /** Poll `runs.get` until a terminal status or the attempt budget is exhausted. */
  async #pollRun(runId: string): Promise<RunSummary> {
    let last: RunSummary | undefined;
    for (let attempt = 0; attempt < this.#poll.maxAttempts; attempt += 1) {
      const run = await this.#api!.runs.get(runId);
      last = run;
      this.#runStatus = run.status;
      this.#runProgress = Math.min(0.8, 0.25 + ((attempt + 1) / this.#poll.maxAttempts) * 0.5);
      this.#emit();
      if (TERMINAL.has(run.status)) return run;
      await this.#poll.sleep(this.#poll.waitMs);
    }
    if (last === undefined) throw new Error("Discovery run could not be polled.");
    return last;
  }

  #failureReason(run: RunSummary): string | undefined {
    const metadata = run.metadata;
    if (metadata === undefined) return undefined;
    const error = (metadata as Record<string, unknown>)["error"];
    return typeof error === "string" ? error : undefined;
  }

  #currentWorldId(): string | undefined {
    return this.#worldId ?? this.#store.state.workspace.worldId ?? this.#world.id;
  }

  /** Load a plain-language explanation of the current world (`worlds.explain`). */
  async explainWorld(): Promise<WorldExplanation | undefined> {
    return this.#withWorld("explain", async (api, worldId) => {
      this.#explanation = await api.worlds.explain(worldId);
      return this.#explanation;
    });
  }

  /** Fetch the self-contained HTML report of the current world (`worlds.report`). */
  async reportWorld(): Promise<string | undefined> {
    return this.#withWorld("report", async (api, worldId) => {
      this.#report = await api.worlds.report(worldId);
      return this.#report;
    });
  }

  /** Forecast the current world forward (`worlds.forecast`). */
  async forecastWorld(request: ForecastRequest): Promise<WorldForecast | undefined> {
    return this.#withWorld("forecast", async (api, worldId) => {
      this.#forecast = await api.worlds.forecast(worldId, request, this.#randomId());
      return this.#forecast;
    });
  }

  /** Diff the current world against another (`worlds.compare`). */
  async compareWorlds(otherWorldId: string): Promise<WorldComparison | undefined> {
    return this.#withWorld("compare", async (api, worldId) => {
      this.#comparison = await api.worlds.compare(worldId, otherWorldId, this.#randomId());
      return this.#comparison;
    });
  }

  async #withWorld<T>(action: string, run: (api: LawSynthClient, worldId: string) => Promise<T>): Promise<T | undefined> {
    this.#lastError = undefined;
    const api = this.#api;
    const worldId = this.#currentWorldId();
    if (api === undefined || worldId === undefined) {
      this.#lastError = `No live service configured for ${action}.`;
      this.#emit();
      return undefined;
    }
    try {
      const result = await run(api, worldId);
      this.#emit();
      return result;
    } catch (error) {
      this.#lastError = error instanceof Error ? error.message : `World ${action} failed.`;
      this.#emit();
      return undefined;
    }
  }

  async #runSimulation(): Promise<void> {
    this.#simRunning = true;
    this.#simStatus = "running";
    this.#emit();
    try {
      if (this.#api !== undefined) {
        const worldId = this.#store.state.workspace.worldId ?? this.#world.id;
        const summary = await this.#api.worlds.simulate(worldId, { horizon: this.#lab.horizon, step: this.#lab.step, method: "rk4" }, this.#randomId());
        this.#simStatus = summary.status;
      } else {
        this.#simStatus = "succeeded";
      }
    } catch (error) {
      this.#simStatus = "failed";
      this.#lastError = error instanceof Error ? error.message : "Simulation failed.";
    } finally {
      this.#simRunning = false;
      this.#emit();
    }
  }

  #selectEntity(kind: string, id: string): void {
    this.#store.dispatch({ kind: "selection.set", ids: [`${kind}:${id}`], primaryId: `${kind}:${id}` });
  }

  #selected(kind: string): string | undefined {
    const primary = primarySelection(this.#store.state);
    if (primary === undefined) return undefined;
    const prefix = `${kind}:`;
    return primary.startsWith(prefix) ? primary.slice(prefix.length) : undefined;
  }

  #emit(): void {
    this.dispatchEvent(new CustomEvent("change", { detail: { screen: this.#screen } }));
  }
}
