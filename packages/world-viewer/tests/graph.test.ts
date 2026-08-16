import assert from "node:assert/strict";
import test from "node:test";
import { adjacentNodes, buildGraphModel, layoutGraphModel, type ViewerWorld } from "../src/index.js";
const world: ViewerWorld = { formatVersion: "0.1.0", id: "g", time: { kind: "continuous" }, variables: ["a", "b", "c"].map((id) => ({ id, role: "state" })), laws: ["a", "b", "c"].map((target) => ({ id: `d${target}`, kind: "continuous", target, expression: { kind: "constant", value: 0 } })), dependencies: { nodes: ["a", "b", "c"], edges: [{ source: "a", target: "b", kind: "directed" }, { source: "b", target: "c", kind: "directed" }] } };
test("derives graph adjacency and a deterministic fallback layout", () => { const graph = buildGraphModel(world); assert.deepEqual(adjacentNodes(graph, "b"), ["a", "c"]); assert.equal(layoutGraphModel(graph).nodes.length, 3); });
