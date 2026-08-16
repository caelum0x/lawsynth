export interface Point { readonly x: number; readonly y: number; }
export interface Size { readonly width: number; readonly height: number; }
export interface Rect extends Point, Size {}
export interface LayoutNode { readonly id: string; readonly width: number; readonly height: number; readonly x?: number; readonly y?: number; readonly data?: unknown; }
export interface LayoutEdge { readonly id?: string; readonly source: string; readonly target: string; readonly data?: unknown; }
export interface LayoutGraph { readonly nodes: readonly LayoutNode[]; readonly edges: readonly LayoutEdge[]; }
export interface PositionedNode extends LayoutNode { readonly x: number; readonly y: number; }
export interface LayoutResult { readonly nodes: readonly PositionedNode[]; readonly width: number; readonly height: number; }

export function assertFinite(name: string, value: number): void {
  if (!Number.isFinite(value)) throw new RangeError(`${name} must be finite`);
}
export function validateGraph(graph: LayoutGraph): void {
  const ids = new Set<string>();
  for (const node of graph.nodes) {
    if (!node.id || ids.has(node.id)) throw new Error(`node ids must be unique and non-empty: ${node.id}`);
    ids.add(node.id); assertFinite(`node ${node.id} width`, node.width); assertFinite(`node ${node.id} height`, node.height);
    if (node.width < 0 || node.height < 0) throw new RangeError(`node ${node.id} dimensions must be non-negative`);
    if (node.x !== undefined) assertFinite(`node ${node.id} x`, node.x);
    if (node.y !== undefined) assertFinite(`node ${node.id} y`, node.y);
  }
  const edgeIds = new Set<string>();
  for (const edge of graph.edges) {
    if (!ids.has(edge.source) || !ids.has(edge.target)) throw new Error(`edge ${edge.id ?? `${edge.source}->${edge.target}`} references an unknown node`);
    if (edge.id !== undefined && (!edge.id || edgeIds.has(edge.id))) throw new Error(`edge ids must be unique and non-empty`);
    if (edge.id !== undefined) edgeIds.add(edge.id);
  }
}
export function bounds(nodes: readonly PositionedNode[]): Rect {
  if (nodes.length === 0) return { x: 0, y: 0, width: 0, height: 0 };
  let left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity;
  for (const n of nodes) { left = Math.min(left, n.x); top = Math.min(top, n.y); right = Math.max(right, n.x + n.width); bottom = Math.max(bottom, n.y + n.height); }
  return { x: left, y: top, width: right - left, height: bottom - top };
}
export function translate(nodes: readonly PositionedNode[], dx: number, dy: number): PositionedNode[] {
  assertFinite("dx", dx); assertFinite("dy", dy); return nodes.map((node) => ({ ...node, x: node.x + dx, y: node.y + dy }));
}
export function centeredRect(point: Point, size: Size): Rect { return { x: point.x - size.width / 2, y: point.y - size.height / 2, ...size }; }
