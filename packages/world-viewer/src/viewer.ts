/**
 * Renderer-neutral, serializable models for displaying a LawSynth world.  This
 * package deliberately does not own a DOM, canvas, or framework lifecycle.
 * Browser and server renderers consume the derived models below.
 */
import { buildEquationModel, type EquationModel } from "./equation.js";
import { buildGraphModel, type GraphModel } from "./graph.js";
import { buildParameterModel, type ParameterModel } from "./parameters.js";
import { buildProvenanceModel, type ProvenanceModel } from "./provenance.js";
import { buildRegimeModel, type RegimeViewModel } from "./regime.js";
import { buildTrajectoryModel, type TrajectoryModel, type TrajectorySource } from "./trajectory.js";
import { buildUncertaintyModel, type UncertaintyViewModel } from "./uncertainty.js";

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | readonly JsonValue[] | { readonly [key: string]: JsonValue };

export interface ViewerVariable { readonly id: string; readonly name?: string; readonly role: string; readonly unit?: string; readonly description?: string; }
export interface ViewerParameter { readonly id: string; readonly value: number; readonly unit?: string; readonly bounds?: readonly [number | null, number | null]; readonly fixed?: boolean; readonly description?: string; }
export interface ViewerExpression { readonly kind: string; readonly [key: string]: unknown; }
export interface ViewerLaw { readonly id: string; readonly kind: string; readonly target?: string; readonly expression: ViewerExpression; readonly enabled?: boolean; readonly description?: string; readonly [key: string]: unknown; }
export interface ViewerEdge { readonly id?: string; readonly source: string; readonly target: string; readonly kind: "directed" | "undirected" | "bidirected"; readonly status?: string; readonly lag?: number; readonly strength?: number; }
export interface ViewerWorld {
  readonly formatVersion: string;
  readonly id: string;
  readonly name?: string;
  readonly description?: string;
  readonly time: { readonly kind: "continuous" | "discrete"; readonly symbol?: string; readonly unit?: string; readonly step?: number };
  readonly variables: readonly ViewerVariable[];
  readonly parameters?: readonly ViewerParameter[];
  readonly laws: readonly ViewerLaw[];
  readonly dependencies?: { readonly nodes: readonly string[]; readonly edges: readonly ViewerEdge[] };
  readonly regimes?: unknown;
  readonly events?: readonly unknown[];
  readonly interventions?: readonly unknown[];
  readonly uncertainty?: unknown;
  readonly provenance?: unknown;
  readonly tags?: readonly string[];
  readonly metadata?: Readonly<Record<string, JsonValue>>;
}

export interface ViewerIssue { readonly path: string; readonly message: string; readonly severity: "error" | "warning"; }
export interface ViewerInspection { readonly id: string; readonly label: string; readonly kind: "variable" | "parameter" | "law"; readonly description?: string; readonly references: readonly string[]; }
export interface WorldViewModel {
  readonly world: ViewerWorld;
  readonly title: string;
  readonly subtitle?: string;
  readonly graph: GraphModel;
  readonly equations: readonly EquationModel[];
  readonly parameters: ParameterModel;
  readonly regimes: RegimeViewModel;
  readonly uncertainty: UncertaintyViewModel;
  readonly provenance: ProvenanceModel;
  readonly inspection: readonly ViewerInspection[];
  readonly issues: readonly ViewerIssue[];
  readonly trajectory?: TrajectoryModel;
}

const identifier = /^[A-Za-z_-][A-Za-z0-9_-]*$/u;
const nonEmpty = (value: unknown): value is string => typeof value === "string" && value.trim().length > 0;

/** Validates viewer input without mutating it; errors make a rendering model unsafe. */
export function validateViewerWorld(world: ViewerWorld): readonly ViewerIssue[] {
  const issues: ViewerIssue[] = [];
  if (!nonEmpty(world.id) || !identifier.test(world.id)) issues.push({ path: "/id", severity: "error", message: "world id must be a valid non-empty identifier" });
  if (!nonEmpty(world.formatVersion)) issues.push({ path: "/formatVersion", severity: "error", message: "formatVersion is required" });
  if (!world.time || (world.time.kind !== "continuous" && world.time.kind !== "discrete")) issues.push({ path: "/time/kind", severity: "error", message: "time kind must be continuous or discrete" });
  if (world.time?.step !== undefined && (!Number.isFinite(world.time.step) || world.time.step <= 0)) issues.push({ path: "/time/step", severity: "error", message: "time step must be positive and finite" });
  const variables = new Set<string>();
  for (const [index, variable] of world.variables.entries()) {
    if (!nonEmpty(variable.id) || !identifier.test(variable.id) || variables.has(variable.id)) issues.push({ path: `/variables/${index}/id`, severity: "error", message: "variable ids must be unique valid identifiers" });
    variables.add(variable.id);
    if (!nonEmpty(variable.role)) issues.push({ path: `/variables/${index}/role`, severity: "error", message: "variable role is required" });
  }
  const parameters = new Set<string>();
  for (const [index, parameter] of (world.parameters ?? []).entries()) {
    if (!nonEmpty(parameter.id) || !identifier.test(parameter.id) || parameters.has(parameter.id)) issues.push({ path: `/parameters/${index}/id`, severity: "error", message: "parameter ids must be unique valid identifiers" });
    parameters.add(parameter.id);
    if (!Number.isFinite(parameter.value)) issues.push({ path: `/parameters/${index}/value`, severity: "error", message: "parameter values must be finite" });
    if (parameter.bounds !== undefined && (parameter.bounds[0] !== null && !Number.isFinite(parameter.bounds[0]) || parameter.bounds[1] !== null && !Number.isFinite(parameter.bounds[1]) || parameter.bounds[0] !== null && parameter.bounds[1] !== null && parameter.bounds[0] > parameter.bounds[1])) issues.push({ path: `/parameters/${index}/bounds`, severity: "error", message: "parameter bounds must be finite and ordered" });
  }
  const lawIds = new Set<string>();
  for (const [index, law] of world.laws.entries()) {
    if (!nonEmpty(law.id) || !identifier.test(law.id) || lawIds.has(law.id)) issues.push({ path: `/laws/${index}/id`, severity: "error", message: "law ids must be unique valid identifiers" });
    lawIds.add(law.id);
    if (!nonEmpty(law.kind) || !law.expression || !nonEmpty(law.expression.kind)) issues.push({ path: `/laws/${index}`, severity: "error", message: "laws require kind and expression" });
    if (law.target !== undefined && !variables.has(law.target)) issues.push({ path: `/laws/${index}/target`, severity: "error", message: "law target must reference a variable" });
  }
  if (world.dependencies) {
    const nodes = new Set(world.dependencies.nodes);
    for (const node of nodes) if (!variables.has(node)) issues.push({ path: "/dependencies/nodes", severity: "warning", message: `graph node ${node} is not a declared variable` });
    for (const [index, edge] of world.dependencies.edges.entries()) if (!nodes.has(edge.source) || !nodes.has(edge.target)) issues.push({ path: `/dependencies/edges/${index}`, severity: "error", message: "dependency edge endpoints must appear in graph nodes" });
  }
  return issues;
}

function referencesForLaw(law: ViewerLaw): string[] {
  const references = new Set<string>();
  const visit = (value: unknown): void => {
    if (!value || typeof value !== "object") return;
    const node = value as Record<string, unknown>;
    if (node.kind === "symbol" && typeof node.id === "string") references.add(node.id);
    for (const child of Object.values(node)) if (child && typeof child === "object") Array.isArray(child) ? child.forEach(visit) : visit(child);
  };
  visit(law.expression); return [...references].sort();
}

export function inspectWorld(world: ViewerWorld): readonly ViewerInspection[] {
  return [
    ...world.variables.map((variable) => ({ id: variable.id, label: variable.name ?? variable.id, kind: "variable" as const, ...(variable.description === undefined ? {} : { description: variable.description }), references: [] })),
    ...(world.parameters ?? []).map((parameter) => ({ id: parameter.id, label: parameter.id, kind: "parameter" as const, ...(parameter.description === undefined ? {} : { description: parameter.description }), references: [] })),
    ...world.laws.map((law) => ({ id: law.id, label: law.target ? `${law.target} (${law.kind})` : law.id, kind: "law" as const, ...(law.description === undefined ? {} : { description: law.description }), references: referencesForLaw(law) })),
  ];
}

export function createWorldViewModel(world: ViewerWorld, trajectory?: TrajectorySource): WorldViewModel {
  const issues = validateViewerWorld(world);
  if (issues.some((issue) => issue.severity === "error")) throw new TypeError(`invalid world: ${issues.filter((issue) => issue.severity === "error").map((issue) => issue.message).join("; ")}`);
  return {
    world,
    title: world.name?.trim() || world.id,
    ...(world.description === undefined ? {} : { subtitle: world.description }),
    graph: buildGraphModel(world), equations: world.laws.map(buildEquationModel), parameters: buildParameterModel(world.parameters ?? []),
    regimes: buildRegimeModel(world.regimes), uncertainty: buildUncertaintyModel(world.uncertainty), provenance: buildProvenanceModel(world.provenance),
    inspection: inspectWorld(world), issues,
    ...(trajectory === undefined ? {} : { trajectory: buildTrajectoryModel(trajectory, world.variables) }),
  };
}
