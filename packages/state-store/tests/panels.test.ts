import assert from "node:assert/strict";
import test from "node:test";
import { DEFAULT_PANELS, setPanel } from "../src/panels.js";

test("panel mutations preserve the model and validate dimensions", () => {
  const panels = setPanel(DEFAULT_PANELS, "inspector", { open: false, size: 480 });
  assert.equal(panels.inspector.open, false);
  assert.equal(DEFAULT_PANELS.inspector.open, true);
  assert.throws(() => setPanel(panels, "inspector", { size: 10 }), /160/);
});
