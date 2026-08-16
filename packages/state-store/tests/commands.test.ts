import assert from "node:assert/strict";
import test from "node:test";
import { commandEvent } from "../src/commands.js";

test("commands map one-to-one to auditable event drafts", () => {
  assert.deepEqual(commandEvent({ kind: "selection.toggle", id: "var:x" }), { type: "selection.toggled", id: "var:x" });
  assert.deepEqual(commandEvent({ kind: "workspace.clear" }), { type: "workspace.cleared" });
});
