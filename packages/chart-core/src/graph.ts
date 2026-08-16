export interface GraphNode { readonly id: string; readonly label?: string; readonly x?: number; readonly y?: number; }
export interface GraphEdge { readonly source: string; readonly target: string; readonly label?: string; readonly weight?: number; }
export interface Graph { readonly nodes: readonly GraphNode[]; readonly edges: readonly GraphEdge[]; }

/** Validate a directed graph; layout and rendering remain caller responsibilities. */
export function normalizeGraph(graph: Graph): Graph {
  const ids = new Set<string>();
  const nodes = graph.nodes.map((node) => {
    if (!node.id.trim() || ids.has(node.id)) throw new RangeError("graph node ids must be unique and non-empty");
    if ((node.x !== undefined && !Number.isFinite(node.x)) || (node.y !== undefined && !Number.isFinite(node.y))) throw new RangeError("node coordinates must be finite");
    ids.add(node.id); return { ...node };
  });
  const edges = graph.edges.map((edge) => { if (!ids.has(edge.source) || !ids.has(edge.target)) throw new RangeError("edge endpoints must exist"); if (edge.weight !== undefined && !Number.isFinite(edge.weight)) throw new RangeError("edge weights must be finite"); return { ...edge }; });
  return { nodes, edges };
}
