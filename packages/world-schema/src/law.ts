import type { Expression } from "./expression.js";
import type { Identifier, JsonValue } from "./types.js";

export type LawKind =
  | "continuous"
  | "discrete"
  | "algebraic"
  | "observation"
  | "stochastic"
  | "event"
  | "regime"
  | "constraint";

interface LawBase {
  id: Identifier;
  kind: LawKind;
  expression: Expression;
  description?: string;
  enabled?: boolean;
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface ContinuousLaw extends LawBase {
  kind: "continuous";
  target: Identifier;
}

export interface DiscreteLaw extends LawBase {
  kind: "discrete";
  target: Identifier;
  lag?: number;
}

export interface AlgebraicLaw extends LawBase {
  kind: "algebraic" | "constraint";
}

export interface ObservationLaw extends LawBase {
  kind: "observation";
  target: Identifier;
  noise?: Identifier;
}

export interface StochasticLaw extends LawBase {
  kind: "stochastic";
  target: Identifier;
  diffusion: Expression;
  noise: Identifier;
}

export interface EventLaw extends LawBase {
  kind: "event";
  event: Identifier;
}

export interface RegimeLaw extends LawBase {
  kind: "regime";
  target: Identifier;
  regime: Identifier;
}

export type Law =
  | ContinuousLaw
  | DiscreteLaw
  | AlgebraicLaw
  | ObservationLaw
  | StochasticLaw
  | EventLaw
  | RegimeLaw;

export function lawTarget(law: Law): Identifier | undefined {
  return "target" in law ? law.target : undefined;
}
