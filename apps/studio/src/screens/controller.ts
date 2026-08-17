import type { CandidateSummary, LawSynthClient, RunStatus } from "@lawsynth/api-client";
import type { TrajectoryInput } from "@lawsynth/chart-core";
import { primarySelection, type StateStore } from "@lawsynth/state-store";
import type { WorldDefinition } from "@lawsynth/world-schema";
import {
  defaultDiscoveryConfig,
  discoveryCanvasModel,
  discoveryRunRequest,
  SOLVERS,
  type DiscoveryCanvasConfig,
  type DiscoverySolver,
} from "./discovery-canvas.js";
import { dataLensModel } from "./data-lens.js";
import { equationExplorerModel } from "./equation-explorer.js";
import { exportScreenModel } from "./export-screen.js";
import { fixtureCandidates, fixtureWorld, FIXTURE_INITIAL_STATE } from "./fixtures.js";
import { regimeTimelineModel } from "./regime-timeline.js";
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

export interface ScreensControllerOptions {
  readonly store: StateStore;
  readonly api?: LawSynthClient;
  readonly randomId?: () => string;
  readonly world?: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly candidates?: readonly CandidateSummary[];
  readonly datasets?: readonly { readonly id: string; readonly name: string }[];
  readonly columns?: readonly string[];
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
  readonly #unsubscribe: () => void;

  #screen: ScreenId = "discovery-canvas";
  #world: WorldDefinition;
  #trajectory: TrajectoryInput | undefined;
  #candidates: readonly CandidateSummary[];
  #discovery: DiscoveryCanvasConfig;
  #focusVariable = "";
  #lab: LabState;
  #board: BoardState;
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
    this.#unsubscribe = this.#store.subscribe(() => this.#emit());
  }

  get screen(): ScreenId {
    return this.#screen;
  }
  get world(): WorldDefinition {
    return this.#world;
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
    if (actionId === "board:add") return this.#addScenario();
    if (actionId === "board:remove") return this.#removeSelectedScenario();
    if (actionId === "board:reset") {
      this.#board = { ...this.#board, draft: EMPTY_DRAFT };
      this.#emit();
      return;
    }
    if (actionId === "discovery:run") return this.#runDiscovery();
    if (actionId === "lab:simulate") return this.#runSimulation();
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

  async #runDiscovery(): Promise<void> {
    this.#running = true;
    this.#runStatus = "running";
    this.#runProgress = 0.15;
    this.#emit();
    try {
      if (this.#api !== undefined) {
        const projectId = this.#store.state.workspace.projectId ?? "project-demo";
        const run = await this.#api.runs.create(discoveryRunRequest(this.#discovery, projectId), this.#randomId());
        const page = await this.#api.runs.candidates(run.id, { limit: 100 });
        this.#candidates = Object.freeze([...page.items].sort((a, b) => b.score - a.score));
        this.#runStatus = run.status;
      } else {
        this.#candidates = fixtureCandidates();
        this.#runStatus = "succeeded";
      }
      this.#runProgress = 1;
    } catch (error) {
      this.#runStatus = "failed";
      this.#lastError = error instanceof Error ? error.message : "Discovery run failed.";
    } finally {
      this.#running = false;
      this.#emit();
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
