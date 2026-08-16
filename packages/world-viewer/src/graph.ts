import type { DependencyEdge, DependencyGraph, Law, WorldDefinition } from "@lawsynth/world-schema";
import { layoutDag, type LayoutEdge, type LayoutGraph, type PositionedNode } from "@lawsynth/layout-engine";
import { expressionSymbols } from "./equation.js";

export interface WorldGraphNode extends PositionedNode {
  readonly label: string;
  readonly role: string;
  readonly unit?: string;
}

export interface WorldGraphEdge {
  readonly id: string;
  readonly source: string;
  readonly target: string;
  readonly kind: DependencyEdge["kind"];
  readonly status: NonNullable<DependencyEdge["status"]>;
  readonly strength?: number;
}

export interface WorldGraphView {
  readonly nodes: readonly WorldGraphNode[];
  readonly edges: readonly WorldGraphEdge[];
  readonly width: number;
  readonly height: number;
  readonly inferred: boolean;
}

function inferredDependencyGraph(world: WorldDefinition): DependencyGraph {
  const nodeIds = new Set(world.variables.map((variable) => variable.id));
  const edges: DependencyEdge[] = [];
  for (const law of world.laws) {
    if (!("target" in law)) continue;
    for (const source of expressionSymbols(law.expression)) {
      if (source === law.target || !nodeIds.has(source)) continue;
      edges.push({ id: `${law.id}:${source}:${law.target}`, source, target: law.target, kind: "directed", status: "identified" });
    }
  }
  return { nodes: [...nodeIds], edges };
}

function normalizedEdges(graph: DependencyGraph): readonly WorldGraphEdge[] {
  const seen = new Set<string>();
  return Object.freeze(graph.edges.filter((edge) => edge.status !== "forbidden").map((edge, index) => {
    let id = edge.id ?? `${edge.source}:${edge.target}:${edge.kind}:${index}`;
    while (seen.has(id)) id = `${id}:${index}`;
    seen.add(id);
    return Object.freeze({
      id,
      source: edge.source,
      target: edge.target,
      kind: edge.kind,
      status: edge.status ?? "candidate",
      ...(edge.strength === undefined ? {} : { strength: edge.strength }),
    });
  }));
}

/** Builds a stable left-to-right graph suitable for SVG or canvas renderers. */
export function graphForWorld(world: WorldDefinition): WorldGraphView {
  const inferred = world.dependencies === undefined;
  const graph = world.dependencies ?? inferredDependencyGraph(world);
  const variables = new Map(world.variables.map((variable) => [variable.id, variable]));
  const nodes = graph.nodes.map((id) => ({ id, width: 168, height: 58 }));
  const edges = normalizedEdges(graph);
  const layoutGraph: LayoutGraph = {
    nodes,
    edges: edges.filter((edge) => edge.kind === "directed").map((edge): LayoutEdge => ({ id: edge.id, source: edge.source, target: edge.target })),
  };
  let positions: readonly PositionedNode[];
  try {
    positions = layoutDag(layoutGraph, { direction: "LR", rankGap: 96, nodeGap: 28, margin: 12 }).nodes;
  } catch {
    positions = nodes.map((node, index) => ({ ...node, x: (index % 4) * 220, y: Math.floor(index / 4) * 100 }));
  }
  const viewNodes = positions.map((node): WorldGraphNode => {
    const variable = variables.get(node.id);
    return Object.freeze({
      ...node,
      label: variable?.name ?? node.id,
      role: variable?.role ?? "unknown",
      ...(variable?.unit === undefined ? {} : { unit: variable.unit }),
    });
  });
  const width = Math.max(1, ...viewNodes.map((node) => node.x + node.width)) + 24;
  const height = Math.max(1, ...viewNodes.map((node) => node.y + node.height)) + 24;
  return Object.freeze({ nodes: Object.freeze(viewNodes), edges, width, height, inferred });
}

export function lawsByTarget(laws: readonly Law[]): ReadonlyMap<string, readonly Law[]> {
  const grouped = new Map<string, Law[]>();
  for (const law of laws) {
    const target = "target" in law ? law.target : law.id;
    grouped.set(target, [...(grouped.get(target) ?? []), law]);
  }
  return grouped;
}
