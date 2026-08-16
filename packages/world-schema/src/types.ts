/**
 * Identifier accepted by `lawsynth_core::Identifier`.
 *
 * Rust accepts ASCII letters, digits, `_`, and `-`, rejects an empty value,
 * and rejects a leading digit.  Keeping that exact rule here prevents a
 * Studio document from producing a bundle that the core decoder cannot read.
 */
export type Identifier = string;
export type IsoTimestamp = string;
export type Sha256 = string;

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };

export interface SemanticVersion {
  major: number;
  minor: number;
  patch: number;
}

export type ValueType =
  | "scalar"
  | "vector"
  | "matrix"
  | "boolean"
  | "categorical"
  | "duration"
  | "timestamp";

export type VariableRole =
  | "state"
  | "observed"
  | "control"
  | "exogenous"
  | "parameter"
  | "latent"
  | "derived";

export interface SourceSpan {
  source?: string;
  start: number;
  end: number;
}

/**
 * A canonical unit expression understood by `lawsynth_units::Unit::parse`,
 * such as `m/s` or `kg*m/s^2`.  Unit conversion is intentionally not
 * reimplemented in TypeScript; the Rust core remains authoritative.
 */
export type UnitDefinition = string;

export interface VariableDefinition {
  id: Identifier;
  name?: string;
  role: VariableRole;
  /** The executable Rust core is scalar-only. Omit for its native IR. */
  valueType?: ValueType;
  unit?: UnitDefinition;
  description?: string;
  bounds?: readonly [number | null, number | null];
  categories?: readonly string[];
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface ParameterDefinition {
  id: Identifier;
  value: number;
  unit?: UnitDefinition;
  bounds?: readonly [number | null, number | null];
  fixed?: boolean;
  description?: string;
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface TimeSemantics {
  kind: "continuous" | "discrete";
  symbol?: Identifier;
  unit?: UnitDefinition;
  step?: number;
  timezone?: string;
}

export const IDENTIFIER_PATTERN = /^[A-Za-z_-][A-Za-z0-9_-]*$/u;

export function isIdentifier(value: unknown): value is Identifier {
  return typeof value === "string" && IDENTIFIER_PATTERN.test(value);
}

export function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

export function parseSemanticVersion(value: string): SemanticVersion | undefined {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/u.exec(value);
  if (!match) return undefined;
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
  };
}

export function formatSemanticVersion(version: SemanticVersion): string {
  return `${version.major}.${version.minor}.${version.patch}`;
}
