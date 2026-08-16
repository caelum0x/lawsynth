import assert from "node:assert/strict";
import test from "node:test";
import { buildRegimeModel } from "../src/index.js";
test("normalizes chronological regime lanes", () => { const model = buildRegimeModel({ regimes: [{ id: "a" }], intervals: [{ regime: "a", start: 2, end: 3 }, { regime: "a", start: 0, end: 1 }] }); assert.deepEqual(model.lanes[0]?.intervals.map((entry) => entry.start), [0, 2]); });
