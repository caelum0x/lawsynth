import { WorldPicker } from "../src/world_picker.js";
import { decayWorld, equal, test, throws } from "./testkit.js";

await test("world picker tracks a selected local world and clears stale selection", () => {
  const picker = new WorldPicker();
  picker.add({ id: "local-decay", name: "Local decay", world: decayWorld, source: "local" });
  equal(picker.select("local-decay").world.id, decayWorld.id);
  equal(picker.selected?.id, "local-decay");
  picker.clear();
  equal(picker.selected, undefined);
  equal(picker.choices.length, 0);
  throws(() => picker.select("local-decay"), /unknown world choice/);
});
