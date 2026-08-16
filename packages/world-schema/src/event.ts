import type { Expression } from "./expression.js";
import type { Identifier, JsonValue } from "./types.js";

export type EventDirection = "any" | "rising" | "falling";

export interface EventAssignment {
  target: Identifier;
  value: Expression;
}

export interface EventDefinition {
  id: Identifier;
  name?: string;
  condition: Expression;
  direction?: EventDirection;
  terminal?: boolean;
  priority?: number;
  assignments?: readonly EventAssignment[];
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface EventOccurrence {
  event: Identifier;
  time: number;
  direction?: EventDirection;
  values?: Readonly<Record<Identifier, number>>;
}

export function crossesZero(previous: number, current: number, direction: EventDirection = "any"): boolean {
  if (!Number.isFinite(previous) || !Number.isFinite(current)) return false;
  if (direction === "rising") return previous < 0 && current >= 0;
  if (direction === "falling") return previous > 0 && current <= 0;
  return (previous < 0 && current >= 0) || (previous > 0 && current <= 0);
}
