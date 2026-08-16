import { applyConstraints, forceLayout } from "../src/index.js";

const graph = { nodes: [{ id: "source", width: 80, height: 32 }, { id: "target", width: 80, height: 32 }], edges: [{ source: "source", target: "target" }] };
const unconstrained = forceLayout(graph, { seed: 7, iterations: 100 });
export const constrained = applyConstraints(unconstrained.nodes, [{ kind: "pin", id: "source", x: 0, y: 0 }, { kind: "minimumGap", first: "source", second: "target", axis: "x", gap: 64 }]);
