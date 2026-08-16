import { graphHasCycle, incomingEdges, outgoingEdges } from "../src/graph.js";
import { equal } from "./test-support.js";
export function runGraphTests(): void { const graph = { nodes: ["x", "y"], edges: [{ source: "x", target: "y", kind: "directed" as const }] }; equal(graphHasCycle(graph), false); equal(outgoingEdges(graph, "x").length, 1); equal(incomingEdges(graph, "y").length, 1); equal(graphHasCycle({ ...graph, edges: [...graph.edges, { source: "y", target: "x", kind: "directed" as const }] }), true); }
