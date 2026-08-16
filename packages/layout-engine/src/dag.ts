import { type LayoutGraph, type PositionedNode, validateGraph } from "./layout.js";

export interface DagLayers { readonly layers: readonly (readonly string[])[]; readonly rankById: ReadonlyMap<string, number>; }
export function topologicalLayers(graph: LayoutGraph): DagLayers {
  validateGraph(graph);
  const outgoing = new Map(graph.nodes.map((node) => [node.id, [] as string[]]));
  const indegree = new Map(graph.nodes.map((node) => [node.id, 0]));
  for (const edge of graph.edges) { outgoing.get(edge.source)?.push(edge.target); indegree.set(edge.target, (indegree.get(edge.target) ?? 0) + 1); }
  for (const list of outgoing.values()) list.sort();
  const available = graph.nodes.map((n) => n.id).filter((id) => indegree.get(id) === 0).sort();
  const ranks = new Map<string, number>();
  for (const id of available) ranks.set(id, 0);
  let visited = 0;
  while (available.length) {
    const id = available.shift()!; visited++;
    for (const target of outgoing.get(id) ?? []) { ranks.set(target, Math.max(ranks.get(target) ?? 0, (ranks.get(id) ?? 0) + 1)); const next = (indegree.get(target) ?? 1) - 1; indegree.set(target, next); if (next === 0) { available.push(target); available.sort(); } }
  }
  if (visited !== graph.nodes.length) throw new Error("layered layout requires a directed acyclic graph");
  const layers: string[][] = [];
  for (const node of [...graph.nodes].sort((a, b) => a.id.localeCompare(b.id))) { const rank = ranks.get(node.id)!; (layers[rank] ??= []).push(node.id); }
  return { layers, rankById: ranks };
}
export interface LayeredOptions { readonly rankGap?: number; readonly nodeGap?: number; readonly direction?: "TB" | "LR"; readonly margin?: number; }
export function layeredLayout(graph: LayoutGraph, options: LayeredOptions = {}): PositionedNode[] {
  const rankGap = options.rankGap ?? 80, nodeGap = options.nodeGap ?? 32, margin = options.margin ?? 0;
  if (rankGap < 0 || nodeGap < 0 || margin < 0) throw new RangeError("layout spacing must be non-negative");
  const { layers } = topologicalLayers(graph), nodeById = new Map(graph.nodes.map((n) => [n.id, n]));
  const output: PositionedNode[] = []; let rankCursor = margin;
  for (const layer of layers) {
    const layerNodes = layer.map((id) => nodeById.get(id)!);
    const cross = layerNodes.reduce((sum, n) => sum + (options.direction === "LR" ? n.height : n.width), 0) + Math.max(0, layerNodes.length - 1) * nodeGap;
    let crossCursor = margin;
    for (const n of layerNodes) { const along = options.direction === "LR" ? n.width : n.height; const across = options.direction === "LR" ? n.height : n.width; output.push({ ...n, x: options.direction === "LR" ? rankCursor : crossCursor, y: options.direction === "LR" ? crossCursor : rankCursor }); crossCursor += across + nodeGap; }
    const maxAlong = Math.max(...layerNodes.map((n) => options.direction === "LR" ? n.width : n.height)); rankCursor += maxAlong + rankGap;
    void cross;
  }
  return output;
}
