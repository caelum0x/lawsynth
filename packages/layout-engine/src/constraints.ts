import type { PositionedNode, Rect } from "./layout.js";
export type Constraint = { readonly kind: "pin"; readonly id: string; readonly x: number; readonly y: number } | { readonly kind: "align"; readonly ids: readonly string[]; readonly axis: "x" | "y" } | { readonly kind: "contain"; readonly bounds: Rect } | { readonly kind: "minimumGap"; readonly first: string; readonly second: string; readonly axis: "x" | "y"; readonly gap: number };
export function applyConstraints(nodes: readonly PositionedNode[], constraints: readonly Constraint[]): PositionedNode[] {
  const output = nodes.map((node) => ({ ...node })); const byId = new Map(output.map((node) => [node.id,node]));
  for (const constraint of constraints) {
    if (constraint.kind === "pin") { const node=byId.get(constraint.id); if (!node) throw new Error(`unknown constrained node ${constraint.id}`); node.x=constraint.x; node.y=constraint.y; }
    else if (constraint.kind === "align") { const members=constraint.ids.map((id)=>byId.get(id) ?? (()=>{throw new Error(`unknown constrained node ${id}`)})()); if (members.length) { const value=members.reduce((sum,n)=>sum+n[constraint.axis],0)/members.length; for (const node of members) node[constraint.axis]=value; } }
    else if (constraint.kind === "contain") for (const node of output) { node.x=Math.max(constraint.bounds.x,Math.min(node.x,constraint.bounds.x+constraint.bounds.width-node.width)); node.y=Math.max(constraint.bounds.y,Math.min(node.y,constraint.bounds.y+constraint.bounds.height-node.height)); }
    else { const first=byId.get(constraint.first), second=byId.get(constraint.second); if (!first || !second) throw new Error("minimumGap references an unknown node"); if (constraint.gap < 0) throw new RangeError("minimum gap must be non-negative"); const extent=constraint.axis === "x" ? first.width : first.height; const required=first[constraint.axis]+extent+constraint.gap; if (second[constraint.axis] < required) second[constraint.axis]=required; }
  }
  return output;
}
