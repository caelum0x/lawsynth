import assert from "node:assert/strict";
import test from "node:test";
import { buildTrajectoryModel } from "../src/index.js";
test("preserves all finite monotonic trajectory observations", () => { const model = buildTrajectoryModel({ variables: ["x"], times: [0, 1], values: [[1], [3]] }); assert.deepEqual(model.domain, [0, 1]); assert.equal(model.series[0]?.points[1]?.y, 3); });
test("rejects nonmonotonic observations", () => { assert.throws(() => buildTrajectoryModel({ variables: ["x"], times: [1, 0], values: [[1], [2]] })); });
