import assert from "node:assert/strict";
import test from "node:test";
import { buildUncertaintyModel } from "../src/index.js";
test("derives actual confidence band samples", () => { const model = buildUncertaintyModel({ entries: [{ level: "trajectory", bands: [{ variable: "x", confidence: 0.95, times: [0, 1], lower: [0, 1], median: [1, 2], upper: [2, 3] }] }, { level: "parameter" }] }); assert.equal(model.bands[0]?.points.length, 2); assert.equal(model.parameterCount, 1); });
