import type { Identifier, JsonValue } from "./types.js";

export type DependencyKind = "directed" | "undirected" | "bidirected";
export type DependencyStatus = "candidate" | "required" | "forbidden" | "identified";

export interface DependencyEdge {
  id?: Identifier;
  source: Identifier;
  target: Identifier;
  kind: DependencyKind;
  status?: DependencyStatus;
  lag?: number;
  strength?: number;
  stability?: number;
  assumptions?: readonly string[];
  metadata?: Readonly<Record<string, JsonValue>>;
}

export interface DependencyGraph {
  nodes: readonly Identifier[];
  edges: readonly DependencyEdge[];
  equivalenceClass?: readonly DependencyGraph[];
  metadata?: Readonly<Record<string, JsonValue>>;
}

export function incomingEdges(graph: DependencyGraph, node: Identifier): readonly DependencyEdge[] {
  return graph.edges.filter((edge) => edge.target === node);
}

export function outgoingEdges(graph: DependencyGraph, node: Identifier): readonly DependencyEdge[] {
  return graph.edges.filter((edge) => edge.source === node);
}

export function graphHasCycle(graph: DependencyGraph): boolean {
  const directed = graph.edges.filter((edge) => edge.kind === "directed" && edge.status !== "forbidden");
  const adjacency = new Map<Identifier, Identifier[]>();
  for (const node of graph.nodes) adjacency.set(node, []);
  for (const edge of directed) adjacency.get(edge.source)?.push(edge.target);

  const visiting = new Set<Identifier>();
  const visited = new Set<Identifier>();
  const visit = (node: Identifier): boolean => {
    if (visiting.has(node)) return true;
    if (visited.has(node)) return false;
    visiting.add(node);
    for (const target of adjacency.get(node) ?? []) if (visit(target)) return true;
    visiting.delete(node);
    visited.add(node);
    return false;
  };
  return graph.nodes.some(visit);
}
