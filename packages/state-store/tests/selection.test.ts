import assert from "node:assert/strict";
import test from "node:test";
import { select, setHovered, toggleSelection } from "../src/selection.js";

test("selection is unique, ordered, and has a primary selected value", () => {
  const start = select(["x", "y", "x"], "y");
  assert.deepEqual(start.ids, ["x", "y"]);
  const removed = toggleSelection(start, "y");
  assert.equal(removed.primaryId, "x");
  assert.equal(setHovered(removed, "x").hoveredId, "x");
});
