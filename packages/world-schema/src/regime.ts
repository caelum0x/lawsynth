import type { Expression } from "./expression.js";
import type { Identifier, JsonValue } from "./types.js";

export interface RegimeDefinition {
  id: Identifier;
  name?: string;
  description?: string;
  guard?: Expression;
  lawIds?: readonly Identifier[];
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface RegimeInterval {
  regime: Identifier;
  start: number;
  end: number;
  confidence?: number;
}

export interface RegimeTransition {
  from: Identifier;
  to: Identifier;
  probability?: number;
  guard?: Expression;
  event?: Identifier;
}

export interface RegimeModel {
  regimes: readonly RegimeDefinition[];
  intervals?: readonly RegimeInterval[];
  transitions?: readonly RegimeTransition[];
  initial?: Identifier;
}

export function activeRegimeAt(model: RegimeModel, time: number): Identifier | undefined {
  if (!Number.isFinite(time)) return undefined;
  return model.intervals?.find((interval) => interval.start <= time && time < interval.end)?.regime;
}

export function transitionProbability(model: RegimeModel, from: Identifier, to: Identifier): number | undefined {
  return model.transitions?.find((transition) => transition.from === from && transition.to === to)?.probability;
}
