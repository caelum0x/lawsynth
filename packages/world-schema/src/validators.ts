import { collectSymbols, type Expression } from "./expression.js";
import { CURRENT_BUNDLE_VERSION, RUST_WORLD_ENCODING as BUNDLE_ENCODING, type BundleCatalog, type WorldManifest } from "./manifest.js";
import { IDENTIFIER_PATTERN, isFiniteNumber, isIdentifier, type Identifier, type UnitDefinition } from "./types.js";
import { CURRENT_WORLD_VERSION, type WorldDefinition } from "./world.js";

export interface ValidationIssue {
  readonly path: string;
  readonly code: "type" | "required" | "value" | "unknown" | "duplicate" | "reference" | "unsupported" | "limit" | "unit";
  readonly message: string;
}

export type ValidationResult<T> =
  | { readonly ok: true; readonly value: T; readonly issues: readonly [] }
  | { readonly ok: false; readonly issues: readonly ValidationIssue[] };

export class SchemaValidationError extends Error {
  readonly issues: readonly ValidationIssue[];
  constructor(issues: readonly ValidationIssue[]) {
    super(issues.map((issue) => `${issue.path || "/"}: ${issue.message}`).join("; "));
    this.name = "SchemaValidationError";
    this.issues = issues;
  }
}

/** The JSON source shape that maps one-for-one to current `lawsynth-world` IR. */
export interface RustWorldSource {
  readonly formatVersion: typeof CURRENT_WORLD_VERSION;
  readonly id: Identifier;
  readonly time: { readonly kind: "continuous" | "discrete"; readonly symbol?: Identifier; readonly unit?: UnitDefinition; readonly step?: number };
  readonly variables: readonly CoreVariable[];
  readonly parameters?: readonly CoreParameter[];
  readonly laws: readonly CoreLaw[];
}

export interface CoreVariable {
  readonly id: Identifier;
  readonly role: "state" | "control" | "exogenous" | "observed" | "latent" | "derived";
  readonly unit?: UnitDefinition;
}

export interface CoreParameter { readonly id: Identifier; readonly value: number; readonly unit?: UnitDefinition; }

export type CoreExpression =
  | { readonly kind: "constant"; readonly value: number }
  | { readonly kind: "symbol"; readonly id: Identifier }
  | { readonly kind: "unary"; readonly operator: "neg" | "exp" | "log" | "sin" | "cos"; readonly operand: CoreExpression }
  | { readonly kind: "binary"; readonly operator: "add" | "sub" | "mul" | "div" | "pow"; readonly left: CoreExpression; readonly right: CoreExpression };

export interface CoreLaw { readonly kind: "continuous" | "discrete"; readonly target: Identifier; readonly expression: CoreExpression; }

const WORLD_KEYS = new Set(["formatVersion", "id", "time", "variables", "parameters", "laws"]);
const ROLES = new Set(["state", "control", "exogenous", "observed", "latent", "derived"]);
const UNARY = new Set(["neg", "exp", "log", "sin", "cos"]);
const BINARY = new Set(["add", "sub", "mul", "div", "pow"]);

/** Validate the exact three-field JSON manifest accepted by `lawsynth-bundle`. */
export function validateManifest(input: unknown): ValidationResult<WorldManifest> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) issue(issues, "", "type", "manifest must be an object");
  else {
    closed(input, new Set(["format", "format_version", "world_encoding"]), "", issues, "current Rust manifests reject unknown fields");
    literal(input, "format", "lawsynth-world", issues);
    literal(input, "format_version", CURRENT_BUNDLE_VERSION, issues);
    literal(input, "world_encoding", BUNDLE_ENCODING, issues);
  }
  return result(input, issues);
}

export const validateBundleManifest = validateManifest;

/**
 * Validates only features the current Rust world encoder can persist. It
 * deliberately rejects events, regimes, stochastic laws, rich graph data,
 * delay/call expressions, and metadata rather than pretending they execute.
 */
export function validateRustWorldSource(input: unknown): ValidationResult<RustWorldSource> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) {
    issue(issues, "", "type", "world source must be an object");
    return result(input, issues);
  }
  closed(input, WORLD_KEYS, "", issues, "field is not encoded by the current Rust world format");
  literal(input, "formatVersion", CURRENT_WORLD_VERSION, issues);
  id(input.id, "/id", issues);
  const time = readTime(input.time, "/time", issues);
  const variables = readVariables(input.variables, "/variables", issues);
  const parameters = readParameters(input.parameters, "/parameters", issues);
  const laws = readLaws(input.laws, "/laws", issues);

  if (time && laws) for (const [index, law] of laws.entries()) {
    if (law.kind !== time.kind) issue(issues, `/laws/${index}/kind`, "value", `${time.kind} worlds require ${time.kind} laws`);
  }
  if (variables && parameters) validateNamespaces(variables, parameters, issues);
  if (variables && laws) validateLaws(variables, parameters ?? [], laws, time?.kind, issues);
  return result(input, issues);
}

/** Alias for callers that use WorldDefinition as their document type. */
export function validateWorld(input: unknown): ValidationResult<WorldDefinition> {
  return validateRustWorldSource(input) as unknown as ValidationResult<WorldDefinition>;
}
export const validateWorldDefinition = validateWorld;

export function assertWorld(input: unknown): WorldDefinition {
  const checked = validateWorld(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}
export function assertRustWorldSource(input: unknown): RustWorldSource {
  const checked = validateRustWorldSource(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}
export function assertManifest(input: unknown): WorldManifest {
  const checked = validateManifest(input);
  if (!checked.ok) throw new SchemaValidationError(checked.issues);
  return checked.value;
}

/** Core identifier rule copied from `lawsynth_core::Identifier`. */
export function isRustIdentifier(value: unknown): value is Identifier { return isIdentifier(value) && IDENTIFIER_PATTERN.test(value); }

/** Catalog validation is intentionally separate: catalog metadata is not a bundle manifest. */
export function validateBundleCatalog(input: unknown): ValidationResult<BundleCatalog> {
  const issues: ValidationIssue[] = [];
  if (!record(input)) issue(issues, "", "type", "catalog must be an object");
  else {
    id(input.worldId, "/worldId", issues);
    if (typeof input.createdAt !== "string" || !Number.isFinite(Date.parse(input.createdAt))) issue(issues, "/createdAt", "value", "createdAt must be an ISO timestamp");
    if (!safePath(input.root)) issue(issues, "/root", "value", "root must be a safe relative path");
    if (!Array.isArray(input.entries) || input.entries.length === 0) issue(issues, "/entries", "required", "catalog needs at least one entry");
    else validateCatalogEntries(input.entries, input.root, issues);
  }
  return result(input, issues);
}

function readTime(input: unknown, path: string, issues: ValidationIssue[]): RustWorldSource["time"] | undefined {
  if (!record(input)) { issue(issues, path, "type", "time must be an object"); return undefined; }
  closed(input, new Set(["kind", "symbol", "unit", "step"]), path, issues, "time field is not encoded by Rust");
  if (input.kind !== "continuous" && input.kind !== "discrete") { issue(issues, `${path}/kind`, "value", "kind must be continuous or discrete"); return undefined; }
  if (input.symbol !== undefined) id(input.symbol, `${path}/symbol`, issues);
  if (input.unit !== undefined) unit(input.unit, `${path}/unit`, issues);
  if (input.step !== undefined && (!isFiniteNumber(input.step) || input.step <= 0)) issue(issues, `${path}/step`, "value", "step must be a positive finite number");
  if (input.kind === "continuous" && input.step !== undefined) issue(issues, `${path}/step`, "unsupported", "continuous worlds have no discrete step in Rust IR");
  return input as RustWorldSource["time"];
}

function readVariables(input: unknown, path: string, issues: ValidationIssue[]): CoreVariable[] | undefined {
  if (!Array.isArray(input) || input.length === 0) { issue(issues, path, "required", "variables must be a non-empty array"); return undefined; }
  return input.map((candidate, index) => {
    const itemPath = `${path}/${index}`;
    if (!record(candidate)) { issue(issues, itemPath, "type", "variable must be an object"); return candidate as CoreVariable; }
    closed(candidate, new Set(["id", "role", "unit"]), itemPath, issues, "variable field is not encoded by Rust");
    id(candidate.id, `${itemPath}/id`, issues);
    if (!ROLES.has(candidate.role as string)) issue(issues, `${itemPath}/role`, "unsupported", "role is not implemented by lawsynth-world");
    if (candidate.unit !== undefined) unit(candidate.unit, `${itemPath}/unit`, issues);
    return candidate as unknown as CoreVariable;
  });
}

function readParameters(input: unknown, path: string, issues: ValidationIssue[]): CoreParameter[] | undefined {
  if (input === undefined) return [];
  if (!Array.isArray(input)) { issue(issues, path, "type", "parameters must be an array"); return undefined; }
  return input.map((candidate, index) => {
    const itemPath = `${path}/${index}`;
    if (!record(candidate)) { issue(issues, itemPath, "type", "parameter must be an object"); return candidate as CoreParameter; }
    closed(candidate, new Set(["id", "value", "unit"]), itemPath, issues, "parameter field is not encoded by Rust");
    id(candidate.id, `${itemPath}/id`, issues);
    if (!isFiniteNumber(candidate.value)) issue(issues, `${itemPath}/value`, "value", "parameter value must be finite");
    if (candidate.unit !== undefined) unit(candidate.unit, `${itemPath}/unit`, issues);
    return candidate as unknown as CoreParameter;
  });
}

function readLaws(input: unknown, path: string, issues: ValidationIssue[]): CoreLaw[] | undefined {
  if (!Array.isArray(input)) { issue(issues, path, "type", "laws must be an array"); return undefined; }
  return input.map((candidate, index) => {
    const itemPath = `${path}/${index}`;
    if (!record(candidate)) { issue(issues, itemPath, "type", "law must be an object"); return candidate as CoreLaw; }
    closed(candidate, new Set(["kind", "target", "expression"]), itemPath, issues, "law feature is not encoded by Rust");
    if (candidate.kind !== "continuous" && candidate.kind !== "discrete") issue(issues, `${itemPath}/kind`, "unsupported", "only continuous and discrete laws are implemented");
    id(candidate.target, `${itemPath}/target`, issues);
    expression(candidate.expression, `${itemPath}/expression`, issues, 0, new Set());
    return candidate as unknown as CoreLaw;
  });
}

function expression(input: unknown, path: string, issues: ValidationIssue[], depth: number, active: Set<object>): void {
  if (depth >= 128) { issue(issues, path, "limit", "Rust bundle encoding limits expression depth to 127"); return; }
  if (!record(input)) { issue(issues, path, "type", "expression must be an object"); return; }
  if (active.has(input)) { issue(issues, path, "limit", "expression must be acyclic"); return; }
  active.add(input);
  switch (input.kind) {
    case "constant": closed(input, new Set(["kind", "value"]), path, issues, "expression metadata is not encoded"); if (!isFiniteNumber(input.value)) issue(issues, `${path}/value`, "value", "constant must be finite"); break;
    case "symbol": closed(input, new Set(["kind", "id"]), path, issues, "expression metadata is not encoded"); id(input.id, `${path}/id`, issues); break;
    case "unary": closed(input, new Set(["kind", "operator", "operand"]), path, issues, "expression metadata is not encoded"); if (!UNARY.has(input.operator as string)) issue(issues, `${path}/operator`, "unsupported", "unary operator is not implemented"); expression(input.operand, `${path}/operand`, issues, depth + 1, active); break;
    case "binary": closed(input, new Set(["kind", "operator", "left", "right"]), path, issues, "expression metadata is not encoded"); if (!BINARY.has(input.operator as string)) issue(issues, `${path}/operator`, "unsupported", "binary operator is not implemented"); expression(input.left, `${path}/left`, issues, depth + 1, active); expression(input.right, `${path}/right`, issues, depth + 1, active); break;
    default: issue(issues, `${path}/kind`, "unsupported", `${String(input.kind)} expressions are not implemented`);
  }
  active.delete(input);
}

function validateNamespaces(variables: readonly CoreVariable[], parameters: readonly CoreParameter[], issues: ValidationIssue[]): void {
  const names = new Set<string>();
  for (const [index, variable] of variables.entries()) if (typeof variable.id === "string" && !names.add(variable.id)) issue(issues, `/variables/${index}/id`, "duplicate", "variable identifier is duplicated");
  for (const [index, parameter] of parameters.entries()) if (typeof parameter.id === "string" && !names.add(parameter.id)) issue(issues, `/parameters/${index}/id`, "duplicate", "parameter identifier conflicts with a declared symbol");
}

function validateLaws(variables: readonly CoreVariable[], parameters: readonly CoreParameter[], laws: readonly CoreLaw[], time: "continuous" | "discrete" | undefined, issues: ValidationIssue[]): void {
  const variableById = new Map(variables.map((value) => [value.id, value]));
  const declared = new Set([...variables.map((value) => value.id), ...parameters.map((value) => value.id)]);
  const targets = new Set<string>();
  for (const [index, law] of laws.entries()) {
    const path = `/laws/${index}`;
    const target = variableById.get(law.target);
    if (!target || target.role !== "state") issue(issues, `${path}/target`, "reference", "law target must be a declared state variable");
    if (!targets.add(law.target)) issue(issues, `${path}/target`, "duplicate", "a state variable may have only one law");
    for (const symbol of collectSymbols(law.expression as Expression)) if (!declared.has(symbol)) issue(issues, `${path}/expression`, "reference", `undeclared expression symbol '${symbol}'`);
    if (target?.unit !== undefined) validateLawUnit(law, target.unit, variables, parameters, path, time, issues);
  }
  for (const [index, variable] of variables.entries()) if (variable.role === "state" && !targets.has(variable.id)) issue(issues, `/variables/${index}/id`, "required", "every state variable requires one law");
}

type Dimension = readonly [number, number, number, number, number, number, number];
const DIMENSIONLESS: Dimension = [0, 0, 0, 0, 0, 0, 0];
const TIME: Dimension = [0, 0, 1, 0, 0, 0, 0];
const NAMED_UNITS: Readonly<Record<string, Dimension>> = { "1": DIMENSIONLESS, m: [1, 0, 0, 0, 0, 0, 0], km: [1, 0, 0, 0, 0, 0, 0], s: TIME, min: TIME, kg: [0, 1, 0, 0, 0, 0, 0], g: [0, 1, 0, 0, 0, 0, 0] };

/** Exact named-unit grammar currently accepted by `lawsynth_units::Unit::parse`. */
export function parseRustUnit(input: unknown): Dimension | undefined {
  if (typeof input !== "string" || input.length === 0) return undefined;
  const tokens = input.split(/([*/])/u); let result: Dimension = DIMENSIONLESS; let divide = false;
  for (const token of tokens) {
    if (token === "*") { divide = false; continue; }
    if (token === "/") { divide = true; continue; }
    if (token === "") return undefined;
    const match = /^([^^]+)(?:\^(-?\d+))?$/u.exec(token);
    const name = match?.[1];
    if (!match || name === undefined || !Object.hasOwn(NAMED_UNITS, name)) return undefined;
    const exponent = match[2] === undefined ? 1 : Number(match[2]);
    if (!Number.isInteger(exponent) || exponent < -128 || exponent > 127) return undefined;
    const powered = scale(NAMED_UNITS[name]!, exponent); if (!powered) return undefined;
    const next = combine(result, powered, divide ? -1 : 1); if (!next) return undefined; result = next;
  }
  return result;
}

function validateLawUnit(law: CoreLaw, targetUnit: string, variables: readonly CoreVariable[], parameters: readonly CoreParameter[], path: string, time: "continuous" | "discrete" | undefined, issues: ValidationIssue[]): void {
  const target = parseRustUnit(targetUnit); if (!target) return;
  const units = new Map<string, Dimension>();
  for (const item of [...variables, ...parameters]) if (item.unit !== undefined) { const parsed = parseRustUnit(item.unit); if (parsed) units.set(item.id, parsed); }
  const actual = dimension(law.expression, units, `${path}/expression`, issues); if (!actual) return;
  const expected = law.kind === "continuous" || time === "continuous" ? combine(target, TIME, -1) : target;
  if (expected && !same(actual, expected)) issue(issues, `${path}/expression`, "unit", "law dimensions do not match the target state transition");
}
function dimension(expression: CoreExpression, units: ReadonlyMap<string, Dimension>, path: string, issues: ValidationIssue[]): Dimension | undefined {
  if (expression.kind === "constant") return DIMENSIONLESS;
  if (expression.kind === "symbol") {
    const known = units.get(expression.id);
    if (!known) issue(issues, path, "unit", `no unit is declared for symbol '${expression.id}'`);
    return known;
  }
  if (expression.kind === "unary") { const value = dimension(expression.operand, units, `${path}/operand`, issues); if (!value) return undefined; if (expression.operator === "neg") return value; if (!same(value, DIMENSIONLESS)) issue(issues, path, "unit", `${expression.operator} requires a dimensionless operand`); return DIMENSIONLESS; }
  const left = dimension(expression.left, units, `${path}/left`, issues); const right = dimension(expression.right, units, `${path}/right`, issues); if (!left || !right) return undefined;
  if (expression.operator === "add" || expression.operator === "sub") { if (!same(left, right)) issue(issues, path, "unit", "addition/subtraction requires equal dimensions"); return left; }
  if (expression.operator === "mul") return combine(left, right, 1);
  if (expression.operator === "div") return combine(left, right, -1);
  if (same(left, DIMENSIONLESS)) return DIMENSIONLESS;
  if (expression.right.kind === "constant" && Number.isInteger(expression.right.value) && expression.right.value >= -128 && expression.right.value <= 127) return scale(left, expression.right.value);
  issue(issues, path, "unit", "a dimensional power requires an integer constant exponent"); return undefined;
}
function unit(value: unknown, path: string, issues: ValidationIssue[]): void { if (!parseRustUnit(value)) issue(issues, path, "value", "unit is not accepted by lawsynth_units"); }
function combine(left: Dimension, right: Dimension, sign: 1 | -1): Dimension | undefined { const value = left.map((item, index) => item + sign * right[index]!); return value.every((item) => item >= -128 && item <= 127) ? value as unknown as Dimension : undefined; }
function scale(value: Dimension, exponent: number): Dimension | undefined { const scaled = value.map((item) => item * exponent); return scaled.every((item) => item >= -128 && item <= 127) ? scaled as unknown as Dimension : undefined; }
function same(left: Dimension, right: Dimension): boolean { return left.every((item, index) => item === right[index]); }

function validateCatalogEntries(entries: readonly unknown[], root: unknown, issues: ValidationIssue[]): void { const paths = new Set<string>(); for (const [index, entry] of entries.entries()) { const path = `/entries/${index}`; if (!record(entry)) { issue(issues, path, "type", "entry must be an object"); continue; } if (!safePath(entry.path)) issue(issues, `${path}/path`, "value", "entry path must be relative and traversal-free"); else if (!paths.add(entry.path)) issue(issues, `${path}/path`, "duplicate", "entry path is duplicated"); if (typeof entry.mediaType !== "string" || !/^[\w.+-]+\/[\w.+-]+$/u.test(entry.mediaType)) issue(issues, `${path}/mediaType`, "value", "mediaType must be a MIME type"); if (typeof entry.sha256 !== "string" || !/^[0-9a-f]{64}$/u.test(entry.sha256)) issue(issues, `${path}/sha256`, "value", "sha256 must be lowercase hexadecimal"); if (!Number.isSafeInteger(entry.bytes) || (entry.bytes as number) < 0) issue(issues, `${path}/bytes`, "value", "bytes must be a non-negative safe integer"); } if (typeof root === "string" && !paths.has(root)) issue(issues, "/root", "reference", "root must name an entry"); }
function safePath(value: unknown): value is string { return typeof value === "string" && value.length > 0 && !value.startsWith("/") && !value.includes("\\") && !value.includes("\0") && value.split("/").every((item) => item !== "" && item !== "." && item !== ".."); }
function closed(object: Record<string, unknown>, allowed: ReadonlySet<string>, path: string, issues: ValidationIssue[], message: string): void { for (const key of Object.keys(object)) if (!allowed.has(key)) issue(issues, `${path}/${key.replace(/~/gu, "~0").replace(/\//gu, "~1")}`, "unknown", message); }
function literal(object: Record<string, unknown>, key: string, expected: string, issues: ValidationIssue[]): void { if (!(key in object)) issue(issues, `/${key}`, "required", `missing '${key}'`); else if (object[key] !== expected) issue(issues, `/${key}`, "value", `must equal '${expected}'`); }
function id(value: unknown, path: string, issues: ValidationIssue[]): void { if (!isRustIdentifier(value)) issue(issues, path, "value", "identifier must be ASCII alphanumeric, '_' or '-', and cannot start with a digit"); }
function issue(issues: ValidationIssue[], path: string, code: ValidationIssue["code"], message: string): void { issues.push({ path, code, message }); }
function result<T>(value: unknown, issues: ValidationIssue[]): ValidationResult<T> { return issues.length === 0 ? { ok: true, value: value as T, issues: [] } : { ok: false, issues }; }
function record(value: unknown): value is Record<string, unknown> { return typeof value === "object" && value !== null && !Array.isArray(value); }
