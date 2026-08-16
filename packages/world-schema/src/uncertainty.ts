import type { Identifier, JsonValue } from "./types.js";

export type UncertaintyLevel = "data" | "parameter" | "structural" | "trajectory";

export interface IntervalEstimate {
  lower: number;
  upper: number;
  confidence: number;
  method?: string;
}

export interface ParameterUncertainty {
  level: "parameter";
  parameter: Identifier;
  interval?: IntervalEstimate;
  samples?: readonly number[];
  standardError?: number;
}

export interface DataUncertainty {
  level: "data";
  variable: Identifier;
  measurementError?: number;
  missingRate?: number;
  method?: string;
}

export interface StructuralAlternative {
  world: Identifier;
  weight?: number;
  score?: number;
  description?: string;
}

export interface StructuralUncertainty {
  level: "structural";
  alternatives: readonly StructuralAlternative[];
}

export interface TrajectoryBand {
  variable: Identifier;
  times: readonly number[];
  lower: readonly number[];
  median?: readonly number[];
  upper: readonly number[];
  confidence: number;
}

export interface TrajectoryUncertainty {
  level: "trajectory";
  bands: readonly TrajectoryBand[];
}

export type Uncertainty =
  | ParameterUncertainty
  | DataUncertainty
  | StructuralUncertainty
  | TrajectoryUncertainty;

export interface UncertaintyModel {
  entries: readonly Uncertainty[];
  method?: string;
  seed?: number;
  metadata?: Readonly<Record<string, JsonValue>>;
}

export function intervalContains(interval: IntervalEstimate, value: number): boolean {
  return Number.isFinite(value) && interval.lower <= value && value <= interval.upper;
}
