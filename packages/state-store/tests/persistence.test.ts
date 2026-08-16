import assert from "node:assert/strict";
import test from "node:test";
import { deserializeState, loadState, saveState, serializeState, type PersistenceAdapter } from "../src/persistence.js";
import { DEFAULT_STUDIO_STATE, StateStore } from "../src/store.js";

class MemoryPersistence implements PersistenceAdapter {
  #values = new Map<string, string>();
  async load(key: string): Promise<string | undefined> { return this.#values.get(key); }
  async save(key: string, value: string): Promise<void> { this.#values.set(key, value); }
  async remove(key: string): Promise<void> { this.#values.delete(key); }
}

test("state persistence round-trips validated state through an explicit adapter", async () => {
  const store = new StateStore();
  store.dispatch({ kind: "workspace.update", patch: { projectId: "project:1" } });
  const adapter = new MemoryPersistence();
  await saveState(adapter, "studio", store.state);
  assert.equal((await loadState(adapter, "studio"))?.workspace.projectId, "project:1");
  assert.deepEqual(deserializeState(serializeState(DEFAULT_STUDIO_STATE)), DEFAULT_STUDIO_STATE);
  assert.throws(() => deserializeState("{}"), /unsupported shape/);
});
