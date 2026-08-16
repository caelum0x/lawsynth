import { normalizeGraph } from "../src/index.js";

export const dependencyGraph = normalizeGraph({ nodes: [{ id: "x", label: "position" }, { id: "v", label: "velocity" }], edges: [{ source: "x", target: "v", label: "derivative" }] });
