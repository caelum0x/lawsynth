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
