import type { CandidateSummary, CreateRunRequest, RunStatus } from "@lawsynth/api-client";
import { compareCandidates } from "../equations.js";
import type { ControlField, Metric, Notice, NoticeTone, ScreenModel, ScreenSection, TableRow } from "./types.js";

export type DiscoverySolver = "sindy" | "symbolic" | "ensemble";

export interface DiscoveryCanvasConfig {
  readonly datasetId: string;
  readonly target: string;
  readonly stateColumns: readonly string[];
  readonly threshold: number;
  readonly degree: number;
  readonly solver: DiscoverySolver;
  readonly seed: number;
  readonly pareto: boolean;
  readonly regimes: boolean;
  readonly refine: boolean;
  readonly causal: boolean;
}

export const SOLVERS: readonly DiscoverySolver[] = Object.freeze(["sindy", "symbolic", "ensemble"]);

export function defaultDiscoveryConfig(datasetId: string, columns: readonly string[]): DiscoveryCanvasConfig {
  const target = columns[0] ?? "";
  return Object.freeze({
    datasetId,
    target,
    stateColumns: Object.freeze(columns.filter((column) => column !== target)),
    threshold: 0.1,
    degree: 3,
    solver: "sindy",
    seed: 7,
    pareto: true,
    regimes: false,
    refine: true,
    causal: false,
  });
}

export function validateDiscoveryConfig(config: DiscoveryCanvasConfig): readonly string[] {
  const issues: string[] = [];
  if (!config.datasetId.trim()) issues.push("Select a dataset to discover from.");
  if (!config.target.trim()) issues.push("Choose a target column to model.");
  if (config.stateColumns.includes(config.target)) issues.push("The target cannot also be a state column.");
  if (config.stateColumns.length === 0) issues.push("Select at least one state column.");
  if (!(config.threshold >= 0 && config.threshold <= 1)) issues.push("Sparsity threshold must be in [0, 1].");
  if (!(Number.isInteger(config.degree) && config.degree >= 1 && config.degree <= 6)) issues.push("Polynomial degree must be an integer in 1..6.");
  return Object.freeze(issues);
}

/** Maps the canvas configuration onto the service's generic run-create contract. */
export function discoveryRunRequest(config: DiscoveryCanvasConfig, projectId: string): CreateRunRequest {
  return {
    name: `Discover ${config.target || "target"}`,
    status: "queued",
    dataset_id: config.datasetId,
    metadata: {
      project_id: projectId,
      target: config.target,
      state_columns: [...config.stateColumns],
      sparsity_threshold: config.threshold,
      polynomial_degree: config.degree,
      solver: config.solver,
      seed: config.seed,
      toggles: { pareto: config.pareto, regimes: config.regimes, refine: config.refine, causal: config.causal },
    },
  };
}

export interface DiscoveryCanvasInput {
  readonly config: DiscoveryCanvasConfig;
  readonly columns: readonly string[];
  readonly datasets: readonly { readonly id: string; readonly name: string }[];
  readonly candidates: readonly CandidateSummary[];
  readonly selectedCandidateId?: string;
  readonly runStatus?: RunStatus;
  readonly progress?: number;
  readonly running: boolean;
}

function toggle(id: string, label: string, value: boolean, help: string): ControlField {
  return { id, label, kind: "toggle", value, help };
}

export function discoveryCanvasModel(input: DiscoveryCanvasInput): ScreenModel {
  const { config } = input;
  const issues = validateDiscoveryConfig(config);
  const compared = compareCandidates(input.candidates, input.selectedCandidateId);

  const controls: readonly ControlField[] = [
    {
      id: "cfg:dataset",
      label: "Dataset",
      kind: "select",
      value: config.datasetId,
      options: input.datasets.map((dataset) => ({ value: dataset.id, label: dataset.name })),
      help: "Source observations for the discovery run.",
    },
    {
      id: "cfg:target",
      label: "Target column",
      kind: "select",
      value: config.target,
      options: input.columns.map((column) => ({ value: column, label: column })),
    },
    ...input.columns
      .filter((column) => column !== config.target)
      .map((column): ControlField => toggle(`cfg:input:${column}`, `State: ${column}`, config.stateColumns.includes(column), "Include as a model state.")),
    { id: "cfg:threshold", label: "Sparsity threshold", kind: "range", value: config.threshold, min: 0, max: 1, step: 0.01 },
    { id: "cfg:degree", label: "Polynomial degree", kind: "number", value: config.degree, min: 1, max: 6, step: 1 },
    {
      id: "cfg:solver",
      label: "Solver",
      kind: "select",
      value: config.solver,
      options: SOLVERS.map((solver) => ({ value: solver, label: solver })),
    },
    { id: "cfg:seed", label: "Seed", kind: "number", value: config.seed, min: 0, step: 1 },
    toggle("cfg:pareto", "Pareto frontier", config.pareto, "Keep the accuracy/complexity trade-off set."),
    toggle("cfg:regimes", "Regime search", config.regimes, "Segment the series into distinct regimes."),
    toggle("cfg:refine", "Joint refinement", config.refine, "Refit parameters jointly after selection."),
    toggle("cfg:causal", "Causal hypotheses", config.causal, "Propose directed dependency edges."),
  ];

  const statusTint = statusTone(input.runStatus);
  const metrics: readonly Metric[] = [
    { label: "Status", value: input.runStatus ?? "idle", ...(statusTint === undefined ? {} : { tone: statusTint }) },
    { label: "Progress", value: `${Math.round((input.progress ?? 0) * 100)}%` },
    { label: "Candidates", value: String(compared.length) },
    { label: "Best score", value: compared[0] ? compared[0].candidate.score.toFixed(3) : "—" },
  ];

  const rows: readonly TableRow[] = compared.map((entry) => ({
    id: entry.candidate.id,
    selected: entry.selected,
    emphasis: entry.rank === 1,
    cells: [String(entry.rank), entry.candidate.equation ?? entry.candidate.id, entry.candidate.score.toFixed(3), entry.scoreDelta === 0 ? "best" : `-${entry.scoreDelta.toFixed(3)}`],
  }));

  const notices: readonly Notice[] = issues.map((message) => ({ tone: "warning", message }));

  const sections: ScreenSection[] = [];
  if (notices.length > 0) sections.push({ kind: "notices", id: "issues", notices });
  sections.push({ kind: "metrics", id: "run-metrics", title: "Run", metrics });
  sections.push({ kind: "controls", id: "config", title: "Configuration", fields: controls });
  sections.push({
    kind: "actions",
    id: "run-actions",
    buttons: [
      { id: "discovery:run", label: input.running ? "Running…" : "Run discovery", tone: "success", disabled: input.running || issues.length > 0 },
      { id: "discovery:reset", label: "Reset config" },
    ],
  });
  sections.push({
    kind: "table",
    id: "candidates",
    title: "Candidate laws",
    columns: [
      { key: "rank", label: "#", align: "end" },
      { key: "equation", label: "Equation" },
      { key: "score", label: "Score", align: "end" },
      { key: "delta", label: "Δ", align: "end" },
    ],
    rows,
    empty: "No candidates yet. Configure the run and press Run discovery.",
  });

  return { id: "discovery-canvas", title: "Discovery Canvas", subtitle: "Configure a run and inspect the candidate set", sections };
}

function statusTone(status: RunStatus | undefined): NoticeTone | undefined {
  switch (status) {
    case "succeeded":
      return "success";
    case "failed":
      return "error";
    case "running":
    case "queued":
      return "info";
    default:
      return undefined;
  }
}
