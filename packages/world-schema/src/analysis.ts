/**
 * Typed models for the LawSynth engine's analysis `--json` reports.
 *
 * These interfaces mirror, key-for-key, the machine-readable output emitted by
 * the Rust `lawsynth` CLI:
 *
 * - `lawsynth stability ... --json`  ->  {@link StabilityReport}
 *   (see `crates/lawsynth-cli/src/stability.rs::render_json`)
 * - `lawsynth control ... --json`    ->  {@link ControlledModel}
 *   (see `crates/lawsynth-cli/src/control.rs::render_json`)
 * - `lawsynth domains run NAME --json`  ->  {@link DomainRunReport}
 *   (see `crates/lawsynth-cli/src/domains.rs::render_run_json`)
 * - `lawsynth bifurcation ... --json`  ->  {@link BifurcationReport}
 *   (see `crates/lawsynth-cli/src/bifurcation.rs::render_json`)
 * - `lawsynth sensitivity ... --json`  ->  {@link SensitivityReport}
 *   (see `crates/lawsynth-cli/src/sensitivity.rs::render_json`)
 * - `lawsynth estimate ... --json`     ->  {@link EstimateReport}
 *   (see `crates/lawsynth-cli/src/estimate.rs::render_json`)
 * - `lawsynth reduce ... --json`       ->  {@link ReductionReport}
 *   (see `crates/lawsynth-cli/src/reduce.rs::render_json`)
 *
 * The parsers below are pure: they validate `unknown` engine JSON and narrow it
 * into the typed model, or throw. They never touch the network — the engine runs
 * as a CLI and there is no HTTP analysis endpoint to fetch these from.
 *
 * Note: `lawsynth domains` (list) and `lawsynth domains show NAME` are
 * text-only in the engine and emit no JSON; only `domains run --json` is modelled
 * here ({@link DomainRunReport}).
 */

import { isFiniteNumber, type Identifier } from "./types.js";
import { SchemaValidationError, type ValidationIssue, type ValidationResult } from "./validators.js";

/**
 * Linear-stability verdict for a fixed point.
 *
 * The strings are exactly the labels the engine writes into the JSON
 * `classification` field (`classification_label` in
 * `crates/lawsynth-cli/src/stability.rs`), which decorate the raw
 * `lawsynth_stability::Classification` enum with human-readable qualifiers.
 * The `Center` and `Marginal` verdicts carry an "(... inconclusive)" suffix in
 * the JSON — they are NOT bare `"center"` / `"marginal"`.
 */
export const CLASSIFICATIONS = [
  "stable node",
  "stable spiral",
  "unstable node",
  "unstable spiral",
  "saddle",
  "center (marginal, inconclusive)",
  "marginal (inconclusive)",
] as const;

export type Classification = (typeof CLASSIFICATIONS)[number];

const CLASSIFICATION_SET: ReadonlySet<string> = new Set(CLASSIFICATIONS);

// --- stability -------------------------------------------------------------

/** One Jacobian eigenvalue `re + im i`. Mirrors `{ "re", "im" }`. */
export interface Eigenvalue {
  readonly re: number;
  readonly im: number;
}

/** A located fixed point of the autonomous vector field. */
export interface FixedPoint {
  readonly coordinates: readonly number[];
  readonly classification: Classification;
  /** True for non-hyperbolic (center/marginal) points where linearization cannot decide. */
  readonly inconclusive: boolean;
  readonly eigenvalues: readonly Eigenvalue[];
}

/** `lawsynth stability ... --json`. */
export interface StabilityReport {
  readonly world: string;
  readonly states: readonly Identifier[];
  readonly seeds_total: number;
  readonly seeds_converged: number;
  readonly fixed_points: readonly FixedPoint[];
}

// --- controlled discovery (SINDYc) -----------------------------------------

/** One active library term `coefficient * term` of a fitted equation. */
export interface ControlTerm {
  readonly term: string;
  readonly coefficient: number;
}

/** The fitted right-hand side for one state: `d/dt state = Σ coefficient·term`. */
export interface ControlEquation {
  readonly state: Identifier;
  readonly residual_sum_squares: number;
  readonly terms: readonly ControlTerm[];
}

/** In-sample rollout score for one state. */
export interface ControlPerStateScore {
  readonly state: Identifier;
  readonly r_squared: number;
  readonly rmse: number;
}

/**
 * In-sample validation block (present only when the CLI ran with `--validate`;
 * otherwise the model's `validation` field is `null`).
 */
export interface ControlValidation {
  /** Always `true` — the engine scores against the same data it fitted. */
  readonly in_sample: boolean;
  readonly per_state: readonly ControlPerStateScore[];
  readonly aggregate_r_squared: number;
  readonly aggregate_rmse: number;
}

/** `lawsynth control ... --json`. */
export interface ControlledModel {
  readonly source: string;
  readonly states: readonly Identifier[];
  readonly controls: readonly Identifier[];
  readonly equations: readonly ControlEquation[];
  readonly validation: ControlValidation | null;
}

// --- domain round-trip recovery --------------------------------------------

/** Per-state recovery of a preset's reference right-hand side. */
export interface DomainRecovery {
  readonly state: Identifier;
  readonly rhs_rmse: number;
  readonly discovered_terms: number;
  readonly reference_terms: number;
}

/** `lawsynth domains run NAME --json`. */
export interface DomainRunReport {
  readonly preset: string;
  readonly recovered: boolean;
  readonly tolerance: number;
  /** Discovered laws rendered as readable polynomials (e.g. `d/dt x = -1*x`). */
  readonly laws: readonly string[];
  readonly recovery: readonly DomainRecovery[];
}

// --- bifurcation continuation ----------------------------------------------

/**
 * The stable JSON tokens for a detected bifurcation's kind.
 *
 * These are exactly the strings emitted by `kind_token` in
 * `crates/lawsynth-cli/src/bifurcation.rs` (confirmed at
 * `bifurcation.rs::kind_token`): a real eigenvalue through zero is reported
 * generically as a `"fold"` (saddle-node / transcritical / pitchfork), a complex
 * pair crossing the imaginary axis as a `"hopf"`. They are NOT the longer
 * human-readable `kind_label` strings used by the text renderer.
 */
export const BIFURCATION_KINDS = ["fold", "hopf"] as const;

export type BifurcationKind = (typeof BIFURCATION_KINDS)[number];

const BIFURCATION_KIND_SET: ReadonlySet<string> = new Set(BIFURCATION_KINDS);

/** The swept parameter interval `[min, max]`. Mirrors `{ "min", "max" }`. */
export interface BifurcationRange {
  readonly min: number;
  readonly max: number;
}

/** One detected bifurcation along a continued branch. */
export interface Bifurcation {
  readonly parameter_value: number;
  readonly kind: BifurcationKind;
  /** Index of the branch the bifurcation sits on (a `usize` count). */
  readonly branch_id: number;
  readonly fixed_point: readonly number[];
  /** The Jacobian eigenvalue that crosses the imaginary axis here. */
  readonly eigenvalue: Eigenvalue;
}

/** `lawsynth bifurcation ... --json`. */
export interface BifurcationReport {
  readonly world: string;
  readonly states: readonly Identifier[];
  readonly parameter: Identifier;
  readonly range: BifurcationRange;
  readonly steps: number;
  readonly branch_count: number;
  readonly bifurcations: readonly Bifurcation[];
}

// --- forward sensitivity ---------------------------------------------------

/** One final-time trajectory sensitivity `∂ state / ∂ parameter`. */
export interface SensitivityEntry {
  readonly state: Identifier;
  readonly parameter: Identifier;
  readonly value: number;
}

/** `lawsynth sensitivity ... --json`. */
export interface SensitivityReport {
  readonly world: string;
  readonly states: readonly Identifier[];
  readonly parameters: readonly Identifier[];
  readonly final_time: number;
  /** The `dx_i/dtheta_j` matrix at `final_time`, flattened row-major over (state, parameter). */
  readonly sensitivities: readonly SensitivityEntry[];
}

// --- state estimator design ------------------------------------------------

/**
 * The stable JSON tokens for the estimator design method.
 *
 * Exactly the strings written by `render_json` in
 * `crates/lawsynth-cli/src/estimate.rs` (confirmed at the `match observer.method`
 * there): Ackermann pole placement is `"pole_placement"`, the steady-state Kalman
 * filter is `"kalman"`.
 */
export const OBSERVER_METHODS = ["pole_placement", "kalman"] as const;

export type ObserverMethod = (typeof OBSERVER_METHODS)[number];

const OBSERVER_METHOD_SET: ReadonlySet<string> = new Set(OBSERVER_METHODS);

/** A dense row-major matrix, mirroring the engine's `matrix_json` (`[[...], ...]`). */
export type Matrix = readonly (readonly number[])[];

/** `lawsynth estimate ... --json`. */
export interface EstimateReport {
  readonly world: string;
  readonly states: readonly Identifier[];
  /** The located fixed point the field was linearized at. */
  readonly fixed_point: readonly number[];
  readonly fixed_points_found: number;
  readonly measured: readonly Identifier[];
  readonly method: ObserverMethod;
  /** Observer gain `L`. */
  readonly gain: Matrix;
  /** Eigenvalues of the error dynamics `A - L C`. */
  readonly error_poles: readonly Eigenvalue[];
  readonly convergent: boolean;
  /** Steady-state error covariance `P` (present only for `--kalman`; otherwise `null`). */
  readonly covariance: Matrix | null;
}

// --- balanced-truncation model reduction -----------------------------------

/** The reduced linear system `(A, B, C)`. */
export interface ReducedSystem {
  readonly a: Matrix;
  readonly b: Matrix;
  readonly c: Matrix;
}

/** `lawsynth reduce ... --json`. */
export interface ReductionReport {
  readonly world: string;
  readonly states: readonly Identifier[];
  readonly fixed_point: readonly number[];
  /** Measured states selected for `C`; `null` when `C = I` (no `--measure`). */
  readonly measured: readonly Identifier[] | null;
  readonly hankel_singular_values: readonly number[];
  readonly order: number;
  readonly error_bound: number;
  readonly reduced: ReducedSystem;
}

// --- validation helpers ----------------------------------------------------

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function issue(issues: ValidationIssue[], path: string, code: ValidationIssue["code"], message: string): void {
  issues.push({ path, code, message });
}

function result<T>(value: unknown, issues: ValidationIssue[]): ValidationResult<T> {
  return issues.length === 0 ? { ok: true, value: value as T, issues: [] } : { ok: false, issues };
}

/** Reads a finite number at `path`, recording an issue if absent or non-finite. */
function num(value: unknown, path: string, issues: ValidationIssue[]): number {
  if (!isFiniteNumber(value)) {
    issue(issues, path, "value", "must be a finite number");
    return Number.NaN;
  }
  return value;
}

/** Reads a `usize`-style count: a non-negative safe integer. */
function count(value: unknown, path: string, issues: ValidationIssue[]): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    issue(issues, path, "value", "must be a non-negative integer");
    return Number.NaN;
  }
  return value;
}

function str(value: unknown, path: string, issues: ValidationIssue[]): string {
  if (typeof value !== "string") {
    issue(issues, path, "type", "must be a string");
    return "";
  }
  return value;
}

function bool(value: unknown, path: string, issues: ValidationIssue[]): boolean {
  if (typeof value !== "boolean") {
    issue(issues, path, "type", "must be a boolean");
    return false;
  }
  return value;
}

/** Reads an array of strings (e.g. `states`, `controls`, `laws`). */
function stringArray(value: unknown, path: string, issues: ValidationIssue[]): string[] {
  if (!Array.isArray(value)) {
    issue(issues, path, "type", "must be an array");
    return [];
  }
  return value.map((item, index) => str(item, `${path}/${index}`, issues));
}

/** Reads an array of finite numbers (e.g. `coordinates`). */
function numberArray(value: unknown, path: string, issues: ValidationIssue[]): number[] {
  if (!Array.isArray(value)) {
    issue(issues, path, "type", "must be an array");
    return [];
  }
  return value.map((item, index) => num(item, `${path}/${index}`, issues));
}

/** Reads a dense row-major matrix (`number[][]`), mirroring the engine's `matrix_json`. */
function readMatrix(value: unknown, path: string, issues: ValidationIssue[]): number[][] {
  if (!Array.isArray(value)) {
    issue(issues, path, "type", "matrix must be an array of rows");
    return [];
  }
  return value.map((row, index) => numberArray(row, `${path}/${index}`, issues));
}

function readEigenvalue(value: unknown, path: string, issues: ValidationIssue[]): Eigenvalue {
  if (!record(value)) {
    issue(issues, path, "type", "eigenvalue must be an object");
    return { re: Number.NaN, im: Number.NaN };
  }
  return { re: num(value.re, `${path}/re`, issues), im: num(value.im, `${path}/im`, issues) };
}

function readClassification(value: unknown, path: string, issues: ValidationIssue[]): Classification {
  if (typeof value !== "string" || !CLASSIFICATION_SET.has(value)) {
    issue(issues, path, "value", "classification is not one of the engine's known verdicts");
    return "marginal (inconclusive)";
  }
  return value as Classification;
}

function readFixedPoint(value: unknown, path: string, issues: ValidationIssue[]): FixedPoint {
  if (!record(value)) {
    issue(issues, path, "type", "fixed point must be an object");
    return { coordinates: [], classification: "marginal (inconclusive)", inconclusive: true, eigenvalues: [] };
  }
  const eigenvalues = Array.isArray(value.eigenvalues)
    ? value.eigenvalues.map((item, index) => readEigenvalue(item, `${path}/eigenvalues/${index}`, issues))
    : (issue(issues, `${path}/eigenvalues`, "type", "must be an array"), [] as Eigenvalue[]);
  return {
    coordinates: numberArray(value.coordinates, `${path}/coordinates`, issues),
    classification: readClassification(value.classification, `${path}/classification`, issues),
    inconclusive: bool(value.inconclusive, `${path}/inconclusive`, issues),
    eigenvalues,
  };
}

// --- stability parser ------------------------------------------------------

/** Validates a `lawsynth stability --json` report without throwing. */
export function validateStabilityReport(input: unknown): ValidationResult<StabilityReport> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "stability report must be an object");
    return result(input, issues);
  }
  const fixedPoints = Array.isArray(input.fixed_points)
    ? input.fixed_points.map((item, index) => readFixedPoint(item, `/fixed_points/${index}`, issues))
    : (issue(issues, "/fixed_points", "type", "must be an array"), [] as FixedPoint[]);
  const report: StabilityReport = {
    world: str(input.world, "/world", issues),
    states: stringArray(input.states, "/states", issues),
    seeds_total: count(input.seeds_total, "/seeds_total", issues),
    seeds_converged: count(input.seeds_converged, "/seeds_converged", issues),
    fixed_points: fixedPoints,
  };
  return issues.length === 0 ? { ok: true, value: report, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth stability --json` report, throwing {@link SchemaValidationError} on any issue. */
export function parseStabilityReport(input: unknown): StabilityReport {
  const checked = validateStabilityReport(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

// --- control parser --------------------------------------------------------

function readControlTerm(value: unknown, path: string, issues: ValidationIssue[]): ControlTerm {
  if (!record(value)) {
    issue(issues, path, "type", "term must be an object");
    return { term: "", coefficient: Number.NaN };
  }
  return { term: str(value.term, `${path}/term`, issues), coefficient: num(value.coefficient, `${path}/coefficient`, issues) };
}

function readControlEquation(value: unknown, path: string, issues: ValidationIssue[]): ControlEquation {
  if (!record(value)) {
    issue(issues, path, "type", "equation must be an object");
    return { state: "", residual_sum_squares: Number.NaN, terms: [] };
  }
  const terms = Array.isArray(value.terms)
    ? value.terms.map((item, index) => readControlTerm(item, `${path}/terms/${index}`, issues))
    : (issue(issues, `${path}/terms`, "type", "must be an array"), [] as ControlTerm[]);
  return {
    state: str(value.state, `${path}/state`, issues),
    residual_sum_squares: num(value.residual_sum_squares, `${path}/residual_sum_squares`, issues),
    terms,
  };
}

function readPerStateScore(value: unknown, path: string, issues: ValidationIssue[]): ControlPerStateScore {
  if (!record(value)) {
    issue(issues, path, "type", "per-state score must be an object");
    return { state: "", r_squared: Number.NaN, rmse: Number.NaN };
  }
  return {
    state: str(value.state, `${path}/state`, issues),
    r_squared: num(value.r_squared, `${path}/r_squared`, issues),
    rmse: num(value.rmse, `${path}/rmse`, issues),
  };
}

function readValidation(value: unknown, path: string, issues: ValidationIssue[]): ControlValidation | null {
  if (value === null) return null;
  if (!record(value)) {
    issue(issues, path, "type", "validation must be an object or null");
    return null;
  }
  const perState = Array.isArray(value.per_state)
    ? value.per_state.map((item, index) => readPerStateScore(item, `${path}/per_state/${index}`, issues))
    : (issue(issues, `${path}/per_state`, "type", "must be an array"), [] as ControlPerStateScore[]);
  return {
    in_sample: bool(value.in_sample, `${path}/in_sample`, issues),
    per_state: perState,
    aggregate_r_squared: num(value.aggregate_r_squared, `${path}/aggregate_r_squared`, issues),
    aggregate_rmse: num(value.aggregate_rmse, `${path}/aggregate_rmse`, issues),
  };
}

/** Validates a `lawsynth control --json` model without throwing. */
export function validateControlledModel(input: unknown): ValidationResult<ControlledModel> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "controlled model must be an object");
    return result(input, issues);
  }
  const equations = Array.isArray(input.equations)
    ? input.equations.map((item, index) => readControlEquation(item, `/equations/${index}`, issues))
    : (issue(issues, "/equations", "type", "must be an array"), [] as ControlEquation[]);
  if (!("validation" in input)) issue(issues, "/validation", "required", "missing 'validation' (use null when --validate was not passed)");
  const model: ControlledModel = {
    source: str(input.source, "/source", issues),
    states: stringArray(input.states, "/states", issues),
    controls: stringArray(input.controls, "/controls", issues),
    equations,
    validation: readValidation(input.validation, "/validation", issues),
  };
  return issues.length === 0 ? { ok: true, value: model, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth control --json` model, throwing {@link SchemaValidationError} on any issue. */
export function parseControlledModel(input: unknown): ControlledModel {
  const checked = validateControlledModel(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

// --- domains run parser ----------------------------------------------------

function readDomainRecovery(value: unknown, path: string, issues: ValidationIssue[]): DomainRecovery {
  if (!record(value)) {
    issue(issues, path, "type", "recovery entry must be an object");
    return { state: "", rhs_rmse: Number.NaN, discovered_terms: Number.NaN, reference_terms: Number.NaN };
  }
  return {
    state: str(value.state, `${path}/state`, issues),
    rhs_rmse: num(value.rhs_rmse, `${path}/rhs_rmse`, issues),
    discovered_terms: count(value.discovered_terms, `${path}/discovered_terms`, issues),
    reference_terms: count(value.reference_terms, `${path}/reference_terms`, issues),
  };
}

/** Validates a `lawsynth domains run NAME --json` report without throwing. */
export function validateDomainRun(input: unknown): ValidationResult<DomainRunReport> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "domain run report must be an object");
    return result(input, issues);
  }
  const recovery = Array.isArray(input.recovery)
    ? input.recovery.map((item, index) => readDomainRecovery(item, `/recovery/${index}`, issues))
    : (issue(issues, "/recovery", "type", "must be an array"), [] as DomainRecovery[]);
  const report: DomainRunReport = {
    preset: str(input.preset, "/preset", issues),
    recovered: bool(input.recovered, "/recovered", issues),
    tolerance: num(input.tolerance, "/tolerance", issues),
    laws: stringArray(input.laws, "/laws", issues),
    recovery,
  };
  return issues.length === 0 ? { ok: true, value: report, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth domains run NAME --json` report, throwing {@link SchemaValidationError} on any issue. */
export function parseDomainRun(input: unknown): DomainRunReport {
  const checked = validateDomainRun(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

// --- bifurcation parser ----------------------------------------------------

function readBifurcationKind(value: unknown, path: string, issues: ValidationIssue[]): BifurcationKind {
  if (typeof value !== "string" || !BIFURCATION_KIND_SET.has(value)) {
    issue(issues, path, "value", "kind must be one of the engine's tokens ('fold' | 'hopf')");
    return "fold";
  }
  return value as BifurcationKind;
}

function readRange(value: unknown, path: string, issues: ValidationIssue[]): BifurcationRange {
  if (!record(value)) {
    issue(issues, path, "type", "range must be an object");
    return { min: Number.NaN, max: Number.NaN };
  }
  return { min: num(value.min, `${path}/min`, issues), max: num(value.max, `${path}/max`, issues) };
}

function readBifurcation(value: unknown, path: string, issues: ValidationIssue[]): Bifurcation {
  if (!record(value)) {
    issue(issues, path, "type", "bifurcation must be an object");
    return { parameter_value: Number.NaN, kind: "fold", branch_id: Number.NaN, fixed_point: [], eigenvalue: { re: Number.NaN, im: Number.NaN } };
  }
  return {
    parameter_value: num(value.parameter_value, `${path}/parameter_value`, issues),
    kind: readBifurcationKind(value.kind, `${path}/kind`, issues),
    branch_id: count(value.branch_id, `${path}/branch_id`, issues),
    fixed_point: numberArray(value.fixed_point, `${path}/fixed_point`, issues),
    eigenvalue: readEigenvalue(value.eigenvalue, `${path}/eigenvalue`, issues),
  };
}

/** Validates a `lawsynth bifurcation --json` report without throwing. */
export function validateBifurcationReport(input: unknown): ValidationResult<BifurcationReport> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "bifurcation report must be an object");
    return result(input, issues);
  }
  const bifurcations = Array.isArray(input.bifurcations)
    ? input.bifurcations.map((item, index) => readBifurcation(item, `/bifurcations/${index}`, issues))
    : (issue(issues, "/bifurcations", "type", "must be an array"), [] as Bifurcation[]);
  const report: BifurcationReport = {
    world: str(input.world, "/world", issues),
    states: stringArray(input.states, "/states", issues),
    parameter: str(input.parameter, "/parameter", issues),
    range: readRange(input.range, "/range", issues),
    steps: count(input.steps, "/steps", issues),
    branch_count: count(input.branch_count, "/branch_count", issues),
    bifurcations,
  };
  return issues.length === 0 ? { ok: true, value: report, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth bifurcation --json` report, throwing {@link SchemaValidationError} on any issue. */
export function parseBifurcationReport(input: unknown): BifurcationReport {
  const checked = validateBifurcationReport(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

// --- sensitivity parser ----------------------------------------------------

function readSensitivityEntry(value: unknown, path: string, issues: ValidationIssue[]): SensitivityEntry {
  if (!record(value)) {
    issue(issues, path, "type", "sensitivity entry must be an object");
    return { state: "", parameter: "", value: Number.NaN };
  }
  return {
    state: str(value.state, `${path}/state`, issues),
    parameter: str(value.parameter, `${path}/parameter`, issues),
    value: num(value.value, `${path}/value`, issues),
  };
}

/** Validates a `lawsynth sensitivity --json` report without throwing. */
export function validateSensitivityReport(input: unknown): ValidationResult<SensitivityReport> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "sensitivity report must be an object");
    return result(input, issues);
  }
  const sensitivities = Array.isArray(input.sensitivities)
    ? input.sensitivities.map((item, index) => readSensitivityEntry(item, `/sensitivities/${index}`, issues))
    : (issue(issues, "/sensitivities", "type", "must be an array"), [] as SensitivityEntry[]);
  const report: SensitivityReport = {
    world: str(input.world, "/world", issues),
    states: stringArray(input.states, "/states", issues),
    parameters: stringArray(input.parameters, "/parameters", issues),
    final_time: num(input.final_time, "/final_time", issues),
    sensitivities,
  };
  return issues.length === 0 ? { ok: true, value: report, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth sensitivity --json` report, throwing {@link SchemaValidationError} on any issue. */
export function parseSensitivityReport(input: unknown): SensitivityReport {
  const checked = validateSensitivityReport(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

// --- estimate parser -------------------------------------------------------

function readObserverMethod(value: unknown, path: string, issues: ValidationIssue[]): ObserverMethod {
  if (typeof value !== "string" || !OBSERVER_METHOD_SET.has(value)) {
    issue(issues, path, "value", "method must be one of the engine's tokens ('pole_placement' | 'kalman')");
    return "pole_placement";
  }
  return value as ObserverMethod;
}

function readEigenvalueArray(value: unknown, path: string, issues: ValidationIssue[]): Eigenvalue[] {
  if (!Array.isArray(value)) {
    issue(issues, path, "type", "must be an array");
    return [];
  }
  return value.map((item, index) => readEigenvalue(item, `${path}/${index}`, issues));
}

function readCovariance(value: unknown, path: string, issues: ValidationIssue[]): number[][] | null {
  if (value === null) return null;
  return readMatrix(value, path, issues);
}

/** Validates a `lawsynth estimate --json` report without throwing. */
export function validateEstimateReport(input: unknown): ValidationResult<EstimateReport> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "estimate report must be an object");
    return result(input, issues);
  }
  if (!("covariance" in input)) issue(issues, "/covariance", "required", "missing 'covariance' (use null when not --kalman)");
  const report: EstimateReport = {
    world: str(input.world, "/world", issues),
    states: stringArray(input.states, "/states", issues),
    fixed_point: numberArray(input.fixed_point, "/fixed_point", issues),
    fixed_points_found: count(input.fixed_points_found, "/fixed_points_found", issues),
    measured: stringArray(input.measured, "/measured", issues),
    method: readObserverMethod(input.method, "/method", issues),
    gain: readMatrix(input.gain, "/gain", issues),
    error_poles: readEigenvalueArray(input.error_poles, "/error_poles", issues),
    convergent: bool(input.convergent, "/convergent", issues),
    covariance: readCovariance(input.covariance, "/covariance", issues),
  };
  return issues.length === 0 ? { ok: true, value: report, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth estimate --json` report, throwing {@link SchemaValidationError} on any issue. */
export function parseEstimateReport(input: unknown): EstimateReport {
  const checked = validateEstimateReport(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

// --- reduce parser ---------------------------------------------------------

function readReducedSystem(value: unknown, path: string, issues: ValidationIssue[]): ReducedSystem {
  if (!record(value)) {
    issue(issues, path, "type", "reduced system must be an object");
    return { a: [], b: [], c: [] };
  }
  return {
    a: readMatrix(value.a, `${path}/a`, issues),
    b: readMatrix(value.b, `${path}/b`, issues),
    c: readMatrix(value.c, `${path}/c`, issues),
  };
}

function readMeasuredOrNull(value: unknown, path: string, issues: ValidationIssue[]): string[] | null {
  if (value === null) return null;
  return stringArray(value, path, issues);
}

/** Validates a `lawsynth reduce --json` report without throwing. */
export function validateReductionReport(input: unknown): ValidationResult<ReductionReport> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "reduction report must be an object");
    return result(input, issues);
  }
  if (!("measured" in input)) issue(issues, "/measured", "required", "missing 'measured' (use null when C = I)");
  if (!("reduced" in input)) issue(issues, "/reduced", "required", "missing 'reduced' system block");
  const report: ReductionReport = {
    world: str(input.world, "/world", issues),
    states: stringArray(input.states, "/states", issues),
    fixed_point: numberArray(input.fixed_point, "/fixed_point", issues),
    measured: readMeasuredOrNull(input.measured, "/measured", issues),
    hankel_singular_values: numberArray(input.hankel_singular_values, "/hankel_singular_values", issues),
    order: count(input.order, "/order", issues),
    error_bound: num(input.error_bound, "/error_bound", issues),
    reduced: readReducedSystem(input.reduced, "/reduced", issues),
  };
  return issues.length === 0 ? { ok: true, value: report, issues: [] } : { ok: false, issues };
}

/** Parses a `lawsynth reduce --json` report, throwing {@link SchemaValidationError} on any issue. */
export function parseReductionReport(input: unknown): ReductionReport {
  const checked = validateReductionReport(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}
