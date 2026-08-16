import type { Expression } from "./expression.js";
import type { Identifier, JsonValue } from "./types.js";

export type InterventionKind =
  | "set"
  | "shift"
  | "scale"
  | "clamp"
  | "replace-law"
  | "remove-edge";

interface InterventionBase {
  id: Identifier;
  kind: InterventionKind;
  time?: number;
  end?: number;
  description?: string;
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface ValueIntervention extends InterventionBase {
  kind: "set" | "shift" | "scale";
  target: Identifier;
  value: number;
}

export interface ClampIntervention extends InterventionBase {
  kind: "clamp";
  target: Identifier;
  minimum?: number;
  maximum?: number;
}

export interface ReplaceLawIntervention extends InterventionBase {
  kind: "replace-law";
  law: Identifier;
  expression: Expression;
}

export interface RemoveEdgeIntervention extends InterventionBase {
  kind: "remove-edge";
  source: Identifier;
  target: Identifier;
  lag?: number;
}

export type Intervention =
  | ValueIntervention
  | ClampIntervention
  | ReplaceLawIntervention
  | RemoveEdgeIntervention;

export function interventionIsActive(intervention: Intervention, time: number): boolean {
  const start = intervention.time ?? Number.NEGATIVE_INFINITY;
  const end = intervention.end ?? Number.POSITIVE_INFINITY;
  return Number.isFinite(time) && start <= time && time < end;
}
