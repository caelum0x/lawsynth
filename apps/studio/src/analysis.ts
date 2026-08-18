/**
 * Studio view models for the LawSynth engine's analysis reports.
 *
 * The engine emits these reports from its `lawsynth ... --json` CLI mode (there
 * is no HTTP analysis endpoint). `@lawsynth/api-client` re-exports the typed
 * models and pure `parse*` validators from `@lawsynth/world-schema`; this module
 * feeds raw engine JSON (or an already-parsed report) through those parsers and
 * maps the checked result into the flat, presentation-ready shapes the Studio UI
 * layer consumes — mirroring how `equations.ts` / `structure.ts` shape their
 * domain data for the screen renderers.
 *
 * Every function here is pure and deterministic. Honest engine outcomes are
 * surfaced honestly: a non-hyperbolic (marginal / center) fixed point is shown
 * as `inconclusive` rather than a definitive stable/unstable verdict; an empty
 * fixed-point set reports "none found in the searched region"; a `null` control
 * validation block is shown as "not validated" (never fabricated as a pass).
 */

import {
  parseBifurcationReport,
  parseControlledModel,
  parseDomainRun,
  parseEstimateReport,
  parseReductionReport,
  parseSensitivityReport,
  parseStabilityReport,
  type Bifurcation,
  type BifurcationKind,
  type BifurcationReport,
  type Classification,
  type ControlEquation,
  type ControlledModel,
  type ControlValidation,
  type DomainRecovery,
  type DomainRunReport,
  type Eigenvalue,
  type EstimateReport,
  type FixedPoint,
  type Matrix,
  type ObserverMethod,
  type ReductionReport,
  type SensitivityReport,
  type StabilityReport,
} from "@lawsynth/api-client";
import { SchemaValidationError } from "@lawsynth/world-schema";

/** The seven analysis reports the engine can emit, in a stable presentation order. */
export const ANALYSIS_REPORTS = [
  "stability",
  "control",
  "domains",
  "bifurcation",
  "sensitivity",
  "estimate",
  "reduce",
] as const;

export type AnalysisReport = (typeof ANALYSIS_REPORTS)[number];

const ANALYSIS_REPORT_SET: ReadonlySet<string> = new Set(ANALYSIS_REPORTS);

/** Narrows an arbitrary string to a known {@link AnalysisReport} id (used by routing). */
export function isAnalysisReport(value: unknown): value is AnalysisReport {
  return typeof value === "string" && ANALYSIS_REPORT_SET.has(value);
}

const ANALYSIS_REPORT_LABELS: Readonly<Record<AnalysisReport, string>> = Object.freeze({
  stability: "Stability",
  control: "Controlled discovery",
  domains: "Domain recovery",
  bifurcation: "Bifurcation",
  sensitivity: "Sensitivity",
  estimate: "Estimator design",
  reduce: "Model reduction",
});

const ANALYSIS_REPORT_SUMMARIES: Readonly<Record<AnalysisReport, string>> = Object.freeze({
  stability: "Fixed points, linear classification, and Jacobian eigenvalues.",
  control: "SINDYc equations fitted with control inputs and in-sample scores.",
  domains: "Round-trip recovery of a benchmark preset's reference laws.",
  bifurcation: "Parameter continuation with detected fold and Hopf crossings.",
  sensitivity: "Final-time trajectory sensitivities dxi/dtheta_j.",
  estimate: "Observer gain design (pole placement or Kalman) and error poles.",
  reduce: "Balanced-truncation reduced system with a Hankel error bound.",
});

/** Human label for an analysis report id (drives Studio navigation cards). */
export function analysisReportLabel(report: AnalysisReport): string {
  return ANALYSIS_REPORT_LABELS[report];
}

/** One-line description of what an analysis report surfaces. */
export function analysisReportSummary(report: AnalysisReport): string {
  return ANALYSIS_REPORT_SUMMARIES[report];
}

// --- presentation primitives -----------------------------------------------

/** Semantic tone, matching the app's `lss-tone-*` classes. */
export type AnalysisTone = "success" | "warning" | "error" | "info";

/** Coarse stability verdict grouping for a linear classification. */
export type Stability = "stable" | "unstable" | "inconclusive";

/**
 * Formats a scalar for display: integers stay exact, everything else is rounded
 * to six significant digits and re-parsed so trailing zeros disappear. Pure and
 * deterministic; non-finite values (which the parsers reject upstream) round-trip
 * through `String` for defensive display only.
 */
export function formatScalar(value: number): string {
  if (!Number.isFinite(value)) return String(value);
  if (value === 0) return "0";
  if (Number.isInteger(value) && Math.abs(value) < 1e15) return String(value);
  return String(Number(value.toPrecision(6)));
}

/** Formats an eigenvalue `{ re, im }` as `a`, `a + b i`, or `a - b i`. */
export function formatEigenvalue(value: Eigenvalue): string {
  const re = formatScalar(value.re);
  if (value.im === 0) return re;
  const sign = value.im < 0 ? "-" : "+";
  return `${re} ${sign} ${formatScalar(Math.abs(value.im))} i`;
}

/** Formats a coordinate vector as `(a, b, ...)`. */
export function formatCoordinates(coordinates: readonly number[]): string {
  return `(${coordinates.map(formatScalar).join(", ")})`;
}

function titleCase(value: string): string {
  return value.length === 0 ? value : value[0]!.toUpperCase() + value.slice(1);
}

/** A displayable eigenvalue with its `a ± b i` rendering and a complex flag. */
export interface EigenvalueView {
  readonly value: Eigenvalue;
  readonly display: string;
  readonly complex: boolean;
}

function eigenvalueView(value: Eigenvalue): EigenvalueView {
  return Object.freeze({ value, display: formatEigenvalue(value), complex: value.im !== 0 });
}

function eigenvalueViews(values: readonly Eigenvalue[]): readonly EigenvalueView[] {
  return Object.freeze(values.map(eigenvalueView));
}

/** A classification decorated with a label, semantic tone, and coarse verdict. */
export interface ClassificationBadge {
  readonly classification: Classification;
  readonly label: string;
  readonly tone: AnalysisTone;
  readonly stability: Stability;
}

/**
 * Maps a linear classification to a display badge. Marginal / center verdicts —
 * where linearization cannot decide — are `inconclusive` with a `warning` tone,
 * never reported as a definitive stable or unstable outcome.
 */
export function classificationBadge(classification: Classification): ClassificationBadge {
  switch (classification) {
    case "stable node":
    case "stable spiral":
      return Object.freeze({ classification, label: titleCase(classification), tone: "success", stability: "stable" });
    case "unstable node":
    case "unstable spiral":
    case "saddle":
      return Object.freeze({ classification, label: titleCase(classification), tone: "error", stability: "unstable" });
    case "center (marginal, inconclusive)":
    case "marginal (inconclusive)":
      return Object.freeze({ classification, label: titleCase(classification), tone: "warning", stability: "inconclusive" });
  }
}

// --- stability --------------------------------------------------------------

/** One fixed point rendered as a table row for the stability view. */
export interface FixedPointRow {
  readonly index: number;
  readonly coordinates: readonly number[];
  readonly coordinatesDisplay: string;
  readonly classification: Classification;
  readonly label: string;
  readonly tone: AnalysisTone;
  readonly stability: Stability;
  /** True for non-hyperbolic points where linearization cannot decide. */
  readonly inconclusive: boolean;
  readonly eigenvalues: readonly EigenvalueView[];
}

/** App-level model of a `lawsynth stability --json` report. */
export interface StabilityView {
  readonly world: string;
  readonly states: readonly string[];
  readonly seedsTotal: number;
  readonly seedsConverged: number;
  readonly convergenceRatio: number;
  readonly rows: readonly FixedPointRow[];
  readonly stableCount: number;
  readonly unstableCount: number;
  readonly inconclusiveCount: number;
  /** True when no fixed point was located in the searched region. */
  readonly empty: boolean;
  readonly summary: string;
}

function fixedPointRow(point: FixedPoint, index: number): FixedPointRow {
  const badge = classificationBadge(point.classification);
  return Object.freeze({
    index,
    coordinates: Object.freeze([...point.coordinates]),
    coordinatesDisplay: formatCoordinates(point.coordinates),
    classification: point.classification,
    label: badge.label,
    tone: badge.tone,
    stability: badge.stability,
    inconclusive: point.inconclusive,
    eigenvalues: eigenvalueViews(point.eigenvalues),
  });
}

function buildStabilityView(report: StabilityReport): StabilityView {
  const rows = report.fixed_points.map(fixedPointRow);
  const stableCount = rows.filter((row) => row.stability === "stable").length;
  const unstableCount = rows.filter((row) => row.stability === "unstable").length;
  const inconclusiveCount = rows.filter((row) => row.stability === "inconclusive").length;
  const empty = rows.length === 0;
  const seeds = `${report.seeds_converged}/${report.seeds_total} seeds converged`;
  const plural = rows.length === 1 ? "" : "s";
  const summary = empty
    ? "No fixed points found in the searched region."
    : `${rows.length} fixed point${plural} — ${stableCount} stable, ${unstableCount} unstable, ${inconclusiveCount} inconclusive (${seeds}).`;
  return Object.freeze({
    world: report.world,
    states: Object.freeze([...report.states]),
    seedsTotal: report.seeds_total,
    seedsConverged: report.seeds_converged,
    convergenceRatio: report.seeds_total === 0 ? 0 : report.seeds_converged / report.seeds_total,
    rows: Object.freeze(rows),
    stableCount,
    unstableCount,
    inconclusiveCount,
    empty,
    summary,
  });
}

/** Parses raw `stability --json` (or an already-parsed report) into a {@link StabilityView}. */
export function stabilityView(input: unknown): StabilityView {
  return buildStabilityView(parseStabilityReport(input));
}

// --- controlled discovery ---------------------------------------------------

/** One fitted right-hand side rendered as a readable polynomial. */
export interface ControlEquationRow {
  readonly state: string;
  readonly residualSumSquares: number;
  readonly residualDisplay: string;
  readonly expression: string;
  readonly terms: readonly { readonly term: string; readonly coefficient: number; readonly display: string }[];
}

/** Per-state in-sample score row. */
export interface ControlScoreRow {
  readonly state: string;
  readonly rSquared: number;
  readonly rmse: number;
}

/** In-sample validation summary (present only when the CLI ran with `--validate`). */
export interface ControlValidationView {
  readonly inSample: boolean;
  readonly aggregateRSquared: number;
  readonly aggregateRmse: number;
  readonly perState: readonly ControlScoreRow[];
}

/** App-level model of a `lawsynth control --json` report. */
export interface ControlView {
  readonly source: string;
  readonly states: readonly string[];
  readonly controls: readonly string[];
  readonly equations: readonly ControlEquationRow[];
  /** False when the engine emitted a `null` validation block. */
  readonly validated: boolean;
  /** `null` (never a fabricated pass) when the model was not validated. */
  readonly validation: ControlValidationView | null;
  readonly validationStatus: string;
}

function controlEquationRow(equation: ControlEquation): ControlEquationRow {
  const terms = equation.terms.map((term) =>
    Object.freeze({ term: term.term, coefficient: term.coefficient, display: `${formatScalar(term.coefficient)} * ${term.term}` }),
  );
  const rhs = terms.length === 0 ? "0" : terms.map((term) => term.display).join(" + ");
  return Object.freeze({
    state: equation.state,
    residualSumSquares: equation.residual_sum_squares,
    residualDisplay: formatScalar(equation.residual_sum_squares),
    expression: `d/dt ${equation.state} = ${rhs}`,
    terms: Object.freeze(terms),
  });
}

function controlValidationView(validation: ControlValidation): ControlValidationView {
  return Object.freeze({
    inSample: validation.in_sample,
    aggregateRSquared: validation.aggregate_r_squared,
    aggregateRmse: validation.aggregate_rmse,
    perState: Object.freeze(
      validation.per_state.map((score) => Object.freeze({ state: score.state, rSquared: score.r_squared, rmse: score.rmse })),
    ),
  });
}

function buildControlView(model: ControlledModel): ControlView {
  const validated = model.validation !== null;
  const validation = model.validation === null ? null : controlValidationView(model.validation);
  const validationStatus = validation === null
    ? "Not validated"
    : `In-sample validated — R² ${formatScalar(validation.aggregateRSquared)}, RMSE ${formatScalar(validation.aggregateRmse)}`;
  return Object.freeze({
    source: model.source,
    states: Object.freeze([...model.states]),
    controls: Object.freeze([...model.controls]),
    equations: Object.freeze(model.equations.map(controlEquationRow)),
    validated,
    validation,
    validationStatus,
  });
}

/** Parses raw `control --json` (or an already-parsed model) into a {@link ControlView}. */
export function controlView(input: unknown): ControlView {
  return buildControlView(parseControlledModel(input));
}

// --- domain recovery --------------------------------------------------------

/** Per-state recovery row for a domain preset run. */
export interface DomainRecoveryRow {
  readonly state: string;
  readonly rhsRmse: number;
  readonly rhsRmseDisplay: string;
  readonly discoveredTerms: number;
  readonly referenceTerms: number;
  readonly termsMatch: boolean;
  readonly withinTolerance: boolean;
}

/** App-level model of a `lawsynth domains run NAME --json` report. */
export interface DomainRunView {
  readonly preset: string;
  readonly recovered: boolean;
  readonly recoveredLabel: string;
  readonly tone: AnalysisTone;
  readonly tolerance: number;
  readonly laws: readonly string[];
  readonly recovery: readonly DomainRecoveryRow[];
  readonly worstRmse: number;
}

/** A compact preset entry aggregated from one domain run (for a preset list). */
export interface DomainPresetSummary {
  readonly preset: string;
  readonly recovered: boolean;
  readonly recoveredLabel: string;
  readonly tone: AnalysisTone;
  readonly worstRmse: number;
}

function domainRecoveryRow(recovery: DomainRecovery, tolerance: number): DomainRecoveryRow {
  return Object.freeze({
    state: recovery.state,
    rhsRmse: recovery.rhs_rmse,
    rhsRmseDisplay: formatScalar(recovery.rhs_rmse),
    discoveredTerms: recovery.discovered_terms,
    referenceTerms: recovery.reference_terms,
    termsMatch: recovery.discovered_terms === recovery.reference_terms,
    withinTolerance: recovery.rhs_rmse <= tolerance,
  });
}

function buildDomainRunView(report: DomainRunReport): DomainRunView {
  const recovery = report.recovery.map((entry) => domainRecoveryRow(entry, report.tolerance));
  const worstRmse = recovery.reduce((worst, row) => Math.max(worst, row.rhsRmse), 0);
  return Object.freeze({
    preset: report.preset,
    recovered: report.recovered,
    recoveredLabel: report.recovered ? "Recovered" : "Not recovered",
    tone: report.recovered ? "success" : "warning",
    tolerance: report.tolerance,
    laws: Object.freeze([...report.laws]),
    recovery: Object.freeze(recovery),
    worstRmse,
  });
}

/** Parses raw `domains run --json` (or an already-parsed report) into a {@link DomainRunView}. */
export function domainRunView(input: unknown): DomainRunView {
  return buildDomainRunView(parseDomainRun(input));
}

/** Aggregates one or more domain run views into a sorted preset list. */
export function domainPresetList(runs: readonly DomainRunView[]): readonly DomainPresetSummary[] {
  const summaries = runs.map((run) =>
    Object.freeze({ preset: run.preset, recovered: run.recovered, recoveredLabel: run.recoveredLabel, tone: run.tone, worstRmse: run.worstRmse }),
  );
  return Object.freeze([...summaries].sort((left, right) => left.preset.localeCompare(right.preset)));
}

// --- bifurcation ------------------------------------------------------------

const BIFURCATION_KIND_LABELS: Readonly<Record<BifurcationKind, string>> = Object.freeze({
  fold: "Fold (saddle-node / transcritical / pitchfork)",
  hopf: "Hopf",
});

/** Human label for a bifurcation kind token. */
export function bifurcationKindLabel(kind: BifurcationKind): string {
  return BIFURCATION_KIND_LABELS[kind];
}

/** One detected bifurcation rendered as a table row. */
export interface BifurcationRow {
  readonly parameterValue: number;
  readonly parameterDisplay: string;
  readonly kind: BifurcationKind;
  readonly kindLabel: string;
  readonly branchId: number;
  readonly fixedPoint: readonly number[];
  readonly fixedPointDisplay: string;
  readonly eigenvalue: EigenvalueView;
}

/** App-level model of a `lawsynth bifurcation --json` report. */
export interface BifurcationView {
  readonly world: string;
  readonly states: readonly string[];
  readonly parameter: string;
  readonly range: { readonly min: number; readonly max: number };
  readonly rangeDisplay: string;
  readonly steps: number;
  readonly branchCount: number;
  readonly rows: readonly BifurcationRow[];
  readonly empty: boolean;
  readonly summary: string;
}

function bifurcationRow(bifurcation: Bifurcation): BifurcationRow {
  return Object.freeze({
    parameterValue: bifurcation.parameter_value,
    parameterDisplay: formatScalar(bifurcation.parameter_value),
    kind: bifurcation.kind,
    kindLabel: bifurcationKindLabel(bifurcation.kind),
    branchId: bifurcation.branch_id,
    fixedPoint: Object.freeze([...bifurcation.fixed_point]),
    fixedPointDisplay: formatCoordinates(bifurcation.fixed_point),
    eigenvalue: eigenvalueView(bifurcation.eigenvalue),
  });
}

function buildBifurcationView(report: BifurcationReport): BifurcationView {
  const rows = report.bifurcations.map(bifurcationRow);
  const empty = rows.length === 0;
  const plural = rows.length === 1 ? "" : "s";
  const branchPlural = report.branch_count === 1 ? "" : "es";
  const summary = empty
    ? "No bifurcations detected across the swept range."
    : `${rows.length} bifurcation${plural} across ${report.branch_count} branch${branchPlural}.`;
  return Object.freeze({
    world: report.world,
    states: Object.freeze([...report.states]),
    parameter: report.parameter,
    range: Object.freeze({ min: report.range.min, max: report.range.max }),
    rangeDisplay: `[${formatScalar(report.range.min)}, ${formatScalar(report.range.max)}]`,
    steps: report.steps,
    branchCount: report.branch_count,
    rows: Object.freeze(rows),
    empty,
    summary,
  });
}

/** Parses raw `bifurcation --json` (or an already-parsed report) into a {@link BifurcationView}. */
export function bifurcationView(input: unknown): BifurcationView {
  return buildBifurcationView(parseBifurcationReport(input));
}

// --- sensitivity ------------------------------------------------------------

/** One `dx/dtheta` cell in the sensitivity grid. */
export interface SensitivityCell {
  readonly parameter: string;
  readonly value: number;
  readonly display: string;
}

/** One state's row of parameter sensitivities. */
export interface SensitivityRow {
  readonly state: string;
  readonly cells: readonly SensitivityCell[];
}

/** App-level model of a `lawsynth sensitivity --json` report. */
export interface SensitivityView {
  readonly world: string;
  readonly states: readonly string[];
  readonly parameters: readonly string[];
  readonly finalTime: number;
  readonly rows: readonly SensitivityRow[];
  /** The largest-magnitude sensitivity, or `null` when the matrix is empty. */
  readonly peak: { readonly state: string; readonly parameter: string; readonly value: number } | null;
  readonly empty: boolean;
}

function buildSensitivityView(report: SensitivityReport): SensitivityView {
  const lookup = new Map<string, number>();
  for (const entry of report.sensitivities) lookup.set(`${entry.state} ${entry.parameter}`, entry.value);
  const rows = report.states.map((state) =>
    Object.freeze({
      state,
      cells: Object.freeze(
        report.parameters.map((parameter) => {
          const value = lookup.get(`${state} ${parameter}`) ?? 0;
          return Object.freeze({ parameter, value, display: formatScalar(value) });
        }),
      ),
    }),
  );
  let peak: { state: string; parameter: string; value: number } | null = null;
  for (const entry of report.sensitivities) {
    if (peak === null || Math.abs(entry.value) > Math.abs(peak.value)) peak = { state: entry.state, parameter: entry.parameter, value: entry.value };
  }
  return Object.freeze({
    world: report.world,
    states: Object.freeze([...report.states]),
    parameters: Object.freeze([...report.parameters]),
    finalTime: report.final_time,
    rows: Object.freeze(rows),
    peak: peak === null ? null : Object.freeze(peak),
    empty: report.sensitivities.length === 0,
  });
}

/** Parses raw `sensitivity --json` (or an already-parsed report) into a {@link SensitivityView}. */
export function sensitivityView(input: unknown): SensitivityView {
  return buildSensitivityView(parseSensitivityReport(input));
}

// --- estimator design -------------------------------------------------------

const OBSERVER_METHOD_LABELS: Readonly<Record<ObserverMethod, string>> = Object.freeze({
  pole_placement: "Pole placement (Ackermann)",
  kalman: "Kalman filter (steady-state)",
});

/** Human label for an observer design method token. */
export function observerMethodLabel(method: ObserverMethod): string {
  return OBSERVER_METHOD_LABELS[method];
}

/** App-level model of a `lawsynth estimate --json` report. */
export interface EstimateView {
  readonly world: string;
  readonly states: readonly string[];
  readonly fixedPoint: readonly number[];
  readonly fixedPointDisplay: string;
  readonly fixedPointsFound: number;
  readonly measured: readonly string[];
  readonly method: ObserverMethod;
  readonly methodLabel: string;
  readonly gain: Matrix;
  readonly errorPoles: readonly EigenvalueView[];
  readonly convergent: boolean;
  readonly convergentLabel: string;
  readonly convergentTone: AnalysisTone;
  /** True when the Kalman branch supplied a steady-state error covariance. */
  readonly hasCovariance: boolean;
  /** `null` (not `--kalman`) when no covariance was produced. */
  readonly covariance: Matrix | null;
}

function buildEstimateView(report: EstimateReport): EstimateView {
  return Object.freeze({
    world: report.world,
    states: Object.freeze([...report.states]),
    fixedPoint: Object.freeze([...report.fixed_point]),
    fixedPointDisplay: formatCoordinates(report.fixed_point),
    fixedPointsFound: report.fixed_points_found,
    measured: Object.freeze([...report.measured]),
    method: report.method,
    methodLabel: observerMethodLabel(report.method),
    gain: report.gain,
    errorPoles: eigenvalueViews(report.error_poles),
    convergent: report.convergent,
    convergentLabel: report.convergent ? "Convergent" : "Not convergent",
    convergentTone: report.convergent ? "success" : "error",
    hasCovariance: report.covariance !== null,
    covariance: report.covariance,
  });
}

/** Parses raw `estimate --json` (or an already-parsed report) into an {@link EstimateView}. */
export function estimateView(input: unknown): EstimateView {
  return buildEstimateView(parseEstimateReport(input));
}

// --- model reduction --------------------------------------------------------

/** App-level model of a `lawsynth reduce --json` report. */
export interface ReductionView {
  readonly world: string;
  readonly states: readonly string[];
  readonly fixedPoint: readonly number[];
  readonly fixedPointDisplay: string;
  /** `null` when `C = I` (no `--measure`); a state list otherwise. */
  readonly measured: readonly string[] | null;
  readonly measuredLabel: string;
  readonly hankelSingularValues: readonly number[];
  readonly order: number;
  readonly retained: number;
  readonly discarded: number;
  readonly errorBound: number;
  readonly errorBoundDisplay: string;
  readonly reduced: { readonly a: Matrix; readonly b: Matrix; readonly c: Matrix };
}

function buildReductionView(report: ReductionReport): ReductionView {
  const retained = report.order;
  const discarded = Math.max(0, report.hankel_singular_values.length - report.order);
  return Object.freeze({
    world: report.world,
    states: Object.freeze([...report.states]),
    fixedPoint: Object.freeze([...report.fixed_point]),
    fixedPointDisplay: formatCoordinates(report.fixed_point),
    measured: report.measured === null ? null : Object.freeze([...report.measured]),
    measuredLabel: report.measured === null ? "C = I (all states measured)" : report.measured.join(", "),
    hankelSingularValues: Object.freeze([...report.hankel_singular_values]),
    order: report.order,
    retained,
    discarded,
    errorBound: report.error_bound,
    errorBoundDisplay: formatScalar(report.error_bound),
    reduced: Object.freeze({ a: report.reduced.a, b: report.reduced.b, c: report.reduced.c }),
  });
}

/** Parses raw `reduce --json` (or an already-parsed report) into a {@link ReductionView}. */
export function reductionView(input: unknown): ReductionView {
  return buildReductionView(parseReductionReport(input));
}

// --- bundled samples --------------------------------------------------------

/**
 * Deterministic `lawsynth <report> --json` samples, VERBATIM engine output shapes
 * (mirroring the fixtures in packages/world-schema/tests/analysis.test.ts). These
 * make the Analysis screen demoable with no engine or backend: "Load sample" fills
 * the paste box with the matching constant below. Every string is a fixed literal —
 * nothing here reads the clock, the network, or randomness.
 */
const ANALYSIS_SAMPLES: Readonly<Record<AnalysisReport, string>> = Object.freeze({
  stability: `{
  "world": "decay2d.lsworld",
  "states": ["x", "y"],
  "seeds_total": 25,
  "seeds_converged": 25,
  "fixed_points": [
    {
      "coordinates": [0.0, 0.0],
      "classification": "stable node",
      "inconclusive": false,
      "eigenvalues": [{"re": -2.0416666610439438, "im": 0.0}, {"re": -1.0102040817416067, "im": 0.0}]
    }
  ]
}`,
  control: `{
  "source": "forced1d.csv",
  "states": ["x"],
  "controls": ["u"],
  "equations": [
    {
      "state": "x",
      "residual_sum_squares": 0.00195699903604763716,
      "terms": [{"term": "u", "coefficient": 0.999393977864784677}, {"term": "x", "coefficient": -0.495801160602859781}]
    }
  ],
  "validation": {
    "in_sample": true,
    "per_state": [{"state": "x", "r_squared": 0.999990552004691668, "rmse": 0.00196976022511570177}],
    "aggregate_r_squared": 0.999990552004691668,
    "aggregate_rmse": 0.00196976022511570177
  }
}`,
  domains: `{
  "preset": "damped-oscillator",
  "recovered": true,
  "tolerance": 0.001,
  "laws": [
    "dv/dt = -0.999987 * x + -0.499985 * v",
    "dx/dt = 0.999983 * v"
  ],
  "recovery": [
    {"state": "x", "rhs_rmse": 0.00000372573809999326259, "discovered_terms": 1, "reference_terms": 1},
    {"state": "v", "rhs_rmse": 0.00000336418375435850972, "discovered_terms": 2, "reference_terms": 2}
  ]
}`,
  bifurcation: `{
  "world": "van-der-pol.lsworld",
  "states": ["x", "y"],
  "parameter": "mu",
  "range": {"min": -1.0, "max": 1.0},
  "steps": 21,
  "branch_count": 1,
  "bifurcations": [
    {
      "parameter_value": -0.000000002,
      "kind": "hopf",
      "branch_id": 0,
      "fixed_point": [0.0, 0.0],
      "eigenvalue": {"re": -0.000000001, "im": 1.0}
    }
  ]
}`,
  sensitivity: `{
  "world": "lotka-volterra.lsworld",
  "states": ["x", "y"],
  "parameters": ["alpha", "beta"],
  "final_time": 0.5,
  "sensitivities": [
    {"state": "x", "parameter": "alpha", "value": 0.718024197761129912},
    {"state": "x", "parameter": "beta", "value": -0.668825445802069152},
    {"state": "y", "parameter": "alpha", "value": 0.0138592584652729028},
    {"state": "y", "parameter": "beta", "value": -0.0131930603898465209}
  ]
}`,
  estimate: `{
  "world": "pendulum.lsworld",
  "states": ["omega", "theta"],
  "fixed_point": [0.0, 0.0],
  "fixed_points_found": 1,
  "measured": ["theta"],
  "method": "pole_placement",
  "gain": [[-0.1875], [4.75]],
  "error_poles": [{"re": -3.0, "im": 0.0}, {"re": -2.0, "im": 0.0}],
  "convergent": true,
  "covariance": null
}`,
  reduce: `{
  "world": "pendulum.lsworld",
  "states": ["omega", "theta"],
  "fixed_point": [0.0, 0.0],
  "measured": null,
  "hankel_singular_values": [5.57534663944548292, 5.17456615089844707],
  "order": 1,
  "error_bound": 10.3491323017968941,
  "reduced": {
    "a": [[-0.12033972395446968]],
    "b": [[0.439580802799826253, -1.07174627076214346]],
    "c": [[1.04223883190820343], [-0.505578449249294404]]
  }
}`,
});

/** The bundled, deterministic `--json` sample for a report (drives the "Load sample" action). */
export function analysisSample(report: AnalysisReport): string {
  return ANALYSIS_SAMPLES[report];
}

// --- DOM rendering ----------------------------------------------------------
//
// Pure, deterministic element builders: given a `Document` they turn a parsed
// report into a DOM subtree, or an honest empty / parse-error notice. No state,
// no clock, no network — every branch is a function of its inputs, so the same
// input always yields the same tree. app.ts owns the surrounding paste controls
// and simply mounts `renderAnalysisReport(...)` on submit.

function node<K extends keyof HTMLElementTagNameMap>(
  document: Document,
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const value = document.createElement(tag);
  if (className !== undefined) value.className = className;
  if (text !== undefined) value.textContent = text;
  return value;
}

function toneClass(tone: AnalysisTone): string {
  return `lss-tone-${tone}`;
}

function badge(document: Document, label: string, tone: AnalysisTone): HTMLElement {
  return node(document, "span", `lss-badge ${toneClass(tone)}`, label);
}

function notice(document: Document, text: string, tone: AnalysisTone): HTMLElement {
  const element = node(document, "p", `lss-scr-notice ${toneClass(tone)}`, text);
  element.setAttribute("role", tone === "error" ? "alert" : "status");
  return element;
}

function metric(document: Document, label: string, value: string): HTMLElement {
  const cell = node(document, "div", "lss-scr-metric");
  cell.append(node(document, "span", "lss-scr-metric-label", label), node(document, "span", "lss-scr-metric-value", value));
  return cell;
}

function metrics(document: Document, entries: readonly (readonly [string, string])[]): HTMLElement {
  const grid = node(document, "div", "lss-scr-metrics");
  for (const [label, value] of entries) grid.append(metric(document, label, value));
  return grid;
}

function heading(document: Document, text: string): HTMLElement {
  return node(document, "h2", "lss-scr-heading", text);
}

function tableRow(document: Document, cells: readonly (string | HTMLElement)[], tag: "td" | "th" = "td"): HTMLElement {
  const row = node(document, "tr");
  for (const cell of cells) {
    const container = node(document, tag);
    if (typeof cell === "string") container.textContent = cell;
    else container.append(cell);
    row.append(container);
  }
  return row;
}

function renderStabilityDetail(document: Document, view: StabilityView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["World", view.world],
      ["States", view.states.join(", ")],
      ["Seeds converged", `${view.seedsConverged}/${view.seedsTotal}`],
      ["Fixed points", String(view.rows.length)],
    ]),
  );
  section.append(notice(document, view.summary, view.empty ? "info" : "success"));
  if (view.empty) {
    section.append(node(document, "p", "lss-scr-empty", "No fixed points found in the searched region."));
    return section;
  }
  const table = node(document, "table", "lss-scr-table");
  const head = node(document, "thead");
  head.append(tableRow(document, ["#", "Coordinates", "Classification", "Verdict", "Eigenvalues"], "th"));
  table.append(head);
  const body = node(document, "tbody");
  for (const row of view.rows) {
    const verdict = row.inconclusive
      ? badge(document, "Inconclusive (linearization cannot decide)", "warning")
      : badge(document, titleCase(row.stability), row.tone);
    const eigen = node(document, "span", undefined, row.eigenvalues.map((value) => value.display).join(",  "));
    body.append(
      tableRow(document, [
        String(row.index + 1),
        row.coordinatesDisplay,
        badge(document, row.label, row.tone),
        verdict,
        eigen,
      ]),
    );
  }
  table.append(body);
  section.append(table);
  return section;
}

function renderControlDetail(document: Document, view: ControlView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["Source", view.source],
      ["States", view.states.join(", ")],
      ["Controls", view.controls.join(", ")],
    ]),
  );
  section.append(notice(document, view.validationStatus, view.validated ? "success" : "warning"));
  const list = node(document, "div", "lss-scr-equations");
  for (const equation of view.equations) {
    const item = node(document, "div", "lss-scr-equation");
    item.append(node(document, "span", "lss-scr-equation-text", equation.expression));
    item.append(node(document, "span", "lss-scr-field-help", `residual SS ${equation.residualDisplay}`));
    list.append(item);
  }
  section.append(list);
  return section;
}

function renderDomainDetail(document: Document, view: DomainRunView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(metrics(document, [["Preset", view.preset], ["Tolerance", formatScalar(view.tolerance)], ["Worst RHS RMSE", formatScalar(view.worstRmse)]]));
  section.append(notice(document, view.recoveredLabel, view.tone));
  const table = node(document, "table", "lss-scr-table");
  const head = node(document, "thead");
  head.append(tableRow(document, ["State", "RHS RMSE", "Terms (found/ref)", "Terms match", "Within tolerance"], "th"));
  table.append(head);
  const body = node(document, "tbody");
  for (const row of view.recovery) {
    body.append(
      tableRow(document, [
        row.state,
        row.rhsRmseDisplay,
        `${row.discoveredTerms}/${row.referenceTerms}`,
        badge(document, row.termsMatch ? "Match" : "Differ", row.termsMatch ? "success" : "warning"),
        badge(document, row.withinTolerance ? "Yes" : "No", row.withinTolerance ? "success" : "warning"),
      ]),
    );
  }
  table.append(body);
  section.append(table);
  return section;
}

function renderBifurcationDetail(document: Document, view: BifurcationView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["World", view.world],
      ["Parameter", view.parameter],
      ["Range", view.rangeDisplay],
      ["Branches", String(view.branchCount)],
    ]),
  );
  section.append(notice(document, view.summary, view.empty ? "info" : "success"));
  if (view.empty) {
    section.append(node(document, "p", "lss-scr-empty", "No bifurcations detected across the swept range."));
    return section;
  }
  const table = node(document, "table", "lss-scr-table");
  const head = node(document, "thead");
  head.append(tableRow(document, [view.parameter, "Kind", "Branch", "Fixed point", "Eigenvalue"], "th"));
  table.append(head);
  const body = node(document, "tbody");
  for (const row of view.rows) {
    body.append(
      tableRow(document, [row.parameterDisplay, badge(document, row.kindLabel, "info"), String(row.branchId), row.fixedPointDisplay, row.eigenvalue.display]),
    );
  }
  table.append(body);
  section.append(table);
  return section;
}

function renderSensitivityDetail(document: Document, view: SensitivityView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["World", view.world],
      ["Final time", formatScalar(view.finalTime)],
      ["Peak |dx/dθ|", view.peak === null ? "—" : `${view.peak.state}/${view.peak.parameter} = ${formatScalar(view.peak.value)}`],
    ]),
  );
  if (view.empty) {
    section.append(node(document, "p", "lss-scr-empty", "No sensitivities were computed."));
    return section;
  }
  const table = node(document, "table", "lss-scr-table");
  const head = node(document, "thead");
  head.append(tableRow(document, ["dxi/dθj", ...view.parameters], "th"));
  table.append(head);
  const body = node(document, "tbody");
  for (const row of view.rows) {
    body.append(tableRow(document, [row.state, ...row.cells.map((cell) => cell.display)]));
  }
  table.append(body);
  section.append(table);
  return section;
}

function renderMatrix(document: Document, label: string, matrix: Matrix): HTMLElement {
  const wrapper = node(document, "div", "lss-scr-section");
  wrapper.append(heading(document, label));
  const table = node(document, "table", "lss-scr-table");
  const body = node(document, "tbody");
  for (const line of matrix) body.append(tableRow(document, line.map((value) => formatScalar(value))));
  table.append(body);
  wrapper.append(table);
  return wrapper;
}

function renderEstimateDetail(document: Document, view: EstimateView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["World", view.world],
      ["Method", view.methodLabel],
      ["Fixed point", view.fixedPointDisplay],
      ["Measured", view.measured.join(", ")],
    ]),
  );
  section.append(notice(document, view.convergentLabel, view.convergentTone));
  section.append(heading(document, "Error poles"));
  section.append(node(document, "p", undefined, view.errorPoles.map((value) => value.display).join(",  ")));
  section.append(renderMatrix(document, "Observer gain L", view.gain));
  if (view.hasCovariance && view.covariance !== null) {
    section.append(renderMatrix(document, "Steady-state error covariance", view.covariance));
  } else {
    section.append(node(document, "p", "lss-scr-empty", "No steady-state covariance (pole placement, not --kalman)."));
  }
  return section;
}

function renderReductionDetail(document: Document, view: ReductionView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["World", view.world],
      ["Measured (C)", view.measuredLabel],
      ["Order (retained/total)", `${view.retained}/${view.hankelSingularValues.length}`],
      ["Discarded", String(view.discarded)],
      ["Hankel error bound", view.errorBoundDisplay],
    ]),
  );
  section.append(heading(document, "Hankel singular values"));
  section.append(node(document, "p", undefined, view.hankelSingularValues.map(formatScalar).join(",  ")));
  section.append(renderMatrix(document, "Reduced A", view.reduced.a));
  section.append(renderMatrix(document, "Reduced B", view.reduced.b));
  section.append(renderMatrix(document, "Reduced C", view.reduced.c));
  return section;
}

function renderReportDetail(document: Document, report: AnalysisReport, data: unknown): HTMLElement {
  switch (report) {
    case "stability":
      return renderStabilityDetail(document, stabilityView(data));
    case "control":
      return renderControlDetail(document, controlView(data));
    case "domains":
      return renderDomainDetail(document, domainRunView(data));
    case "bifurcation":
      return renderBifurcationDetail(document, bifurcationView(data));
    case "sensitivity":
      return renderSensitivityDetail(document, sensitivityView(data));
    case "estimate":
      return renderEstimateDetail(document, estimateView(data));
    case "reduce":
      return renderReductionDetail(document, reductionView(data));
  }
}

/**
 * Renders the result region for one analysis report from raw engine `--json`.
 *
 * Pure and deterministic. Honest outcomes are surfaced, never crashed on:
 * `null`/empty input shows a paste prompt; invalid JSON and schema failures are
 * shown as an error notice carrying the {@link SchemaValidationError} message;
 * an empty result set renders as a normal "none found" state, not an error.
 */
export function renderAnalysisReport(document: Document, report: AnalysisReport, rawJson: string | null): HTMLElement {
  const container = node(document, "div", "lss-analysis-result");
  const trimmed = rawJson === null ? "" : rawJson.trim();
  if (trimmed.length === 0) {
    container.append(
      node(
        document,
        "p",
        "lss-scr-empty",
        `Paste \`lawsynth ${report} … --json\` output above, or press "Load sample" to preview a bundled report.`,
      ),
    );
    return container;
  }
  let data: unknown;
  try {
    data = JSON.parse(trimmed);
  } catch (error) {
    container.append(notice(document, `That text is not valid JSON — ${error instanceof Error ? error.message : String(error)}`, "error"));
    return container;
  }
  try {
    container.append(renderReportDetail(document, report, data));
  } catch (error) {
    const message = error instanceof SchemaValidationError
      ? `Schema validation failed — ${error.message}`
      : error instanceof Error
        ? error.message
        : String(error);
    container.append(notice(document, message, "error"));
  }
  return container;
}
