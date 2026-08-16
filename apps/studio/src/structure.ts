import type { DependencyEdge, DependencyGraph, DependencyStatus, WorldDefinition } from "@lawsynth/world-schema";
import { graphForWorld, type WorldGraphView } from "@lawsynth/world-viewer";

export interface StructureFilter {
  readonly statuses?: readonly DependencyStatus[];
  readonly minimumStrength?: number;
  readonly includeUndirected?: boolean;
  readonly query?: string;
}

export interface StructureView {
  readonly graph: WorldGraphView;
  readonly visibleNodeIds: ReadonlySet<string>;
  readonly visibleEdgeIds: ReadonlySet<string>;
  readonly hiddenEdges: number;
}

export function filterStructure(world: WorldDefinition, filter: StructureFilter = {}): StructureView {
  if (filter.minimumStrength !== undefined && (!Number.isFinite(filter.minimumStrength) || filter.minimumStrength < 0)) throw new RangeError("minimum strength must be non-negative");
  const graph = graphForWorld(world);
  const statuses = filter.statuses === undefined ? undefined : new Set(filter.statuses);
  const query = filter.query?.trim().toLocaleLowerCase();
  const matchingNodes = new Set(graph.nodes.filter((node) => !query || node.id.toLocaleLowerCase().includes(query) || node.label.toLocaleLowerCase().includes(query)).map((node) => node.id));
  const visibleEdgeIds = new Set(graph.edges.filter((edge) => {
    if (statuses !== undefined && !statuses.has(edge.status)) return false;
    if (filter.includeUndirected === false && edge.kind !== "directed") return false;
    if (filter.minimumStrength !== undefined && (edge.strength === undefined || Math.abs(edge.strength) < filter.minimumStrength)) return false;
    return !query || matchingNodes.has(edge.source) || matchingNodes.has(edge.target);
  }).map((edge) => edge.id));
  const visibleNodeIds = new Set(matchingNodes);
  for (const edge of graph.edges) if (visibleEdgeIds.has(edge.id)) { visibleNodeIds.add(edge.source); visibleNodeIds.add(edge.target); }
  return Object.freeze({ graph, visibleNodeIds, visibleEdgeIds, hiddenEdges: graph.edges.length - visibleEdgeIds.size });
}

export function updateDependencyStatus(graph: DependencyGraph, edgeId: string, status: DependencyStatus): DependencyGraph {
  let found = false;
  const edges = graph.edges.map((edge, index): DependencyEdge => {
    const id = edge.id ?? `${edge.source}:${edge.target}:${index}`;
    if (id !== edgeId) return edge;
    found = true; return { ...edge, status };
  });
  if (!found) throw new RangeError(`unknown dependency edge: ${edgeId}`);
  return { ...graph, edges };
}
