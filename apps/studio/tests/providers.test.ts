import { ProviderScope } from "../src/providers.js";
import { equal, inMemoryApi, store } from "./support.js";

export async function providersTests(): Promise<void> {
  let factoryCalls = 0; const disposal: string[] = [];
  const scope = new ProviderScope(() => { factoryCalls += 1; return { api: inMemoryApi(), store: store(), persistence: { load: async () => undefined, save: async () => undefined, remove: async () => undefined }, logger: { debug() {}, info() {}, error() {} }, notify() {}, clock: () => 1, randomId: () => "id" }; });
  equal(await scope.get(), await scope.get()); equal(factoryCalls, 1);
  scope.addDisposer(() => { disposal.push("first"); }); scope.addDisposer(() => { disposal.push("second"); });
  await scope.dispose(); equal(disposal.join(","), "second,first");
}
