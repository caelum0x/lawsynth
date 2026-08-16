import { StudioApp } from "../src/app.js";
import { ProviderScope } from "../src/providers.js";
import { equal, inMemoryApi, store } from "./support.js";

export async function appTests(): Promise<void> {
  const saved = new Map<string, string>();
  const scope = new ProviderScope(() => ({
    api: inMemoryApi(), store: store(),
    persistence: { load: async (key) => saved.get(key), save: async (key, value) => { saved.set(key, value); }, remove: async (key) => { saved.delete(key); } },
    logger: { debug() {}, info() {}, error() {} }, notify() {}, clock: () => 0, randomId: () => "request_1",
  }));
  const app = new StudioApp({ providers: scope, settingsKey: "studio-test-settings" });
  await app.start(); equal(app.snapshot.phase, "ready");
  app.updateSettings({ theme: "dark", autosaveMs: 250 });
  await app.stop(); equal(app.snapshot.phase, "stopped");
  equal(JSON.parse(saved.get("studio-test-settings")!).theme, "dark");
}
