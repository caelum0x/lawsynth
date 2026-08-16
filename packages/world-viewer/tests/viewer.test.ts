import assert from "node:assert/strict";
import test from "node:test";
import { createWorldViewModel, validateViewerWorld, type ViewerWorld } from "../src/index.js";
const world: ViewerWorld = { formatVersion: "0.1.0", id: "view", name: "Viewer", time: { kind: "continuous" }, variables: [{ id: "x", role: "state" }], parameters: [{ id: "rate", value: 1 }], laws: [{ id: "dx", kind: "continuous", target: "x", expression: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "rate" }, right: { kind: "symbol", id: "x" } } }] };
test("builds a validated renderer-neutral world model", () => { const model = createWorldViewModel(world, { variables: ["x"], times: [0, 1], values: [[1], [2]] }); assert.equal(model.title, "Viewer"); assert.equal(model.equations[0]?.text, "(rate × x)"); assert.equal(model.inspection.length, 3); assert.equal(model.trajectory?.series[0]?.points.length, 2); });
test("rejects invalid target references", () => { assert.ok(validateViewerWorld({ ...world, laws: [{ ...world.laws[0]!, target: "missing" }] }).some((issue) => issue.severity === "error")); });
