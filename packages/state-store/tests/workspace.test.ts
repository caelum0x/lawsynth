import assert from "node:assert/strict";
import test from "node:test";
import { EMPTY_WORKSPACE, updateWorkspace } from "../src/workspace.js";

test("workspace requires project context for worlds and runs", () => {
  assert.throws(() => updateWorkspace(EMPTY_WORKSPACE, { worldId: "world:1" }), /active project/);
  const workspace = updateWorkspace(EMPTY_WORKSPACE, { projectId: "project:1", worldId: "world:1", route: "/projects/1/worlds/1" });
  assert.equal(workspace.worldId, "world:1");
  assert.throws(() => updateWorkspace(workspace, { route: "relative" }), /absolute safe path/);
});
