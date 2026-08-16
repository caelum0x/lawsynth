import type { DependencyGraph } from "./graph.js";
import type { EventDefinition } from "./event.js";
import type { Intervention } from "./intervention.js";
import type { Law } from "./law.js";
import type { Provenance } from "./provenance.js";
import type { RegimeModel } from "./regime.js";
import type { Identifier, JsonValue, ParameterDefinition, TimeSemantics, VariableDefinition } from "./types.js";
import type { UncertaintyModel } from "./uncertainty.js";

/** Version carried by the Rust bundle manifest. */
export const CURRENT_WORLD_VERSION = "0.1.0";

export interface WorldDefinition {
  /** Metadata version for a document intended for the current Rust core. */
  formatVersion: string;
  id: Identifier;
  name?: string;
  description?: string;
  time: TimeSemantics;
  variables: readonly VariableDefinition[];
  parameters?: readonly ParameterDefinition[];
  laws: readonly Law[];
  dependencies?: DependencyGraph;
  regimes?: RegimeModel;
  events?: readonly EventDefinition[];
  interventions?: readonly Intervention[];
  uncertainty?: UncertaintyModel;
  provenance?: Provenance;
  tags?: readonly string[];
  metadata?: Readonly<Record<string, JsonValue>>;
}

export function variableById(world: WorldDefinition, id: Identifier): VariableDefinition | undefined {
  return world.variables.find((variable) => variable.id === id);
}

export function parameterById(world: WorldDefinition, id: Identifier): ParameterDefinition | undefined {
  return world.parameters?.find((parameter) => parameter.id === id);
}

export function stateVariables(world: WorldDefinition): readonly VariableDefinition[] {
  return world.variables.filter((variable) => variable.role === "state");
}

export function cloneWorld(world: WorldDefinition): WorldDefinition {
  if (typeof structuredClone === "function") return structuredClone(world);
  return JSON.parse(JSON.stringify(world)) as WorldDefinition;
}
