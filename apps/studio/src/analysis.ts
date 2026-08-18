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
