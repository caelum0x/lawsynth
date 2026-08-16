import { bounds, type LayoutGraph, type LayoutResult, type PositionedNode, validateGraph } from "./layout.js";

export interface ForceOptions { readonly iterations?: number; readonly repulsion?: number; readonly springLength?: number; readonly springStrength?: number; readonly damping?: number; readonly seed?: number; readonly padding?: number; }
function hash(value: string, seed: number): number { let h = (2166136261 ^ seed) >>> 0; for (let i = 0; i < value.length; i++) h = Math.imul(h ^ value.charCodeAt(i), 16777619); return h >>> 0; }
export function forceLayout(graph: LayoutGraph, options: ForceOptions = {}): LayoutResult {
  validateGraph(graph); const iterations = options.iterations ?? 300, repulsion = options.repulsion ?? 6000, springLength = options.springLength ?? 120, strength = options.springStrength ?? 0.08, damping = options.damping ?? 0.82, seed = options.seed ?? 0, padding = options.padding ?? 0;
  if (!Number.isInteger(iterations) || iterations < 0 || repulsion < 0 || springLength <= 0 || strength < 0 || damping < 0 || damping > 1 || padding < 0) throw new RangeError("invalid force layout options");
  const state = [...graph.nodes].sort((a,b) => a.id.localeCompare(b.id)).map((node) => { const h = hash(node.id, seed); const angle = (h % 360) * Math.PI / 180, radius = 40 + ((h >>> 9) % 90); return { ...node, x: node.x ?? Math.cos(angle) * radius, y: node.y ?? Math.sin(angle) * radius, vx: 0, vy: 0 }; });
  const byId = new Map(state.map((node) => [node.id, node]));
  for (let step = 0; step < iterations; step++) {
    const forces = state.map(() => ({ x: 0, y: 0 }));
    for (let i = 0; i < state.length; i++) for (let j = i + 1; j < state.length; j++) { const a = state[i]!, b = state[j]!; let dx = b.x - a.x, dy = b.y - a.y; const d2 = dx * dx + dy * dy || 0.01, d = Math.sqrt(d2), f = repulsion / d2; dx /= d; dy /= d; forces[i]!.x -= dx * f; forces[i]!.y -= dy * f; forces[j]!.x += dx * f; forces[j]!.y += dy * f; }
    for (const edge of graph.edges) { const a = byId.get(edge.source)!, b = byId.get(edge.target)!; const i = state.indexOf(a), j = state.indexOf(b); let dx = b.x - a.x, dy = b.y - a.y; const d = Math.hypot(dx, dy) || 0.01, f = (d - springLength) * strength; dx /= d; dy /= d; forces[i]!.x += dx * f; forces[i]!.y += dy * f; forces[j]!.x -= dx * f; forces[j]!.y -= dy * f; }
    const cooling = 1 - step / Math.max(1, iterations); for (let i = 0; i < state.length; i++) { const node = state[i]!, f = forces[i]!; node.vx = (node.vx + f.x * 0.01) * damping; node.vy = (node.vy + f.y * 0.01) * damping; node.x += node.vx * cooling; node.y += node.vy * cooling; }
  }
  const nodes: PositionedNode[] = state.map(({ vx, vy, ...node }) => node); const box = bounds(nodes); const shifted = nodes.map((node) => ({ ...node, x: node.x - box.x + padding, y: node.y - box.y + padding })); const final = bounds(shifted); return { nodes: shifted, width: final.width + padding * 2, height: final.height + padding * 2 };
}
