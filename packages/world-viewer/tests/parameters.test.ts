import assert from "node:assert/strict";
import test from "node:test";
import { buildParameterModel } from "../src/index.js";
test("sorts and audits parameter bounds", () => { const model = buildParameterModel([{ id: "b", value: 4, bounds: [0, 3] }, { id: "a", value: 2, fixed: true }]); assert.deepEqual(model.rows.map((row) => row.id), ["a", "b"]); assert.equal(model.rows[1]?.inBounds, false); assert.equal(model.fixedCount, 1); });
