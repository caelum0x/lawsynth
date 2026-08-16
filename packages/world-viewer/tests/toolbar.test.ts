import assert from "node:assert/strict";
import test from "node:test";
import { createToolbar, reduceToolbar } from "../src/index.js";
test("exposes only supported view actions", () => { const state = { hasTrajectory: false, showEquations: true, showProvenance: false }; assert.equal(createToolbar(state).find((action) => action.id === "export-csv")?.enabled, false); assert.equal(reduceToolbar(state, "toggle-equations").showEquations, false); });
