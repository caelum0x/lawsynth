import { bounds, type LayoutGraph, type LayoutResult } from "./layout.js";
import { layeredLayout, type LayeredOptions } from "./dag.js";

export function layoutDag(graph: LayoutGraph, options: LayeredOptions = {}): LayoutResult {
  const nodes = layeredLayout(graph, options); const box = bounds(nodes); return { nodes, width: box.width, height: box.height };
}
export function edgeCrossings(graph: LayoutGraph, positions: ReadonlyMap<string, { x: number; y: number }>): number {
  const segments = graph.edges.map((edge) => [positions.get(edge.source), positions.get(edge.target)] as const).filter((pair): pair is readonly [{ x: number; y: number }, { x: number; y: number }] => pair[0] !== undefined && pair[1] !== undefined);
  let count = 0; for (let i = 0; i < segments.length; i++) for (let j = i + 1; j < segments.length; j++) if (intersects(segments[i]!, segments[j]!)) count++; return count;
}
function intersects(a: readonly [{ x: number; y: number }, { x: number; y: number }], b: readonly [{ x: number; y: number }, { x: number; y: number }]): boolean {
  const cross = (p: {x:number;y:number}, q:{x:number;y:number}, r:{x:number;y:number}) => (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x);
  const a1 = cross(a[0], a[1], b[0]), a2 = cross(a[0], a[1], b[1]), b1 = cross(b[0], b[1], a[0]), b2 = cross(b[0], b[1], a[1]);
  return a1 * a2 < 0 && b1 * b2 < 0;
}
