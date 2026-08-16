import { WorkspaceController } from "../src/workspace.js";
import { equal, inMemoryApi, rejects, store } from "./support.js";

export async function workspaceTests(): Promise<void> {
  const state = store(); const controller = new WorkspaceController(inMemoryApi(), state, () => 123);
  const resources = await controller.open("project_1"); equal(resources.loadedAt, 123); equal(state.state.workspace.projectId, "project_1");
  controller.selectWorld("world_1"); equal(state.state.workspace.worldId, "world_1");
  controller.selectRun("run_1"); equal(state.state.workspace.runId, "run_1");
  await rejects(() => controller.open(" "), /required/);
  controller.close(); equal(controller.snapshot.phase, "idle");
}
