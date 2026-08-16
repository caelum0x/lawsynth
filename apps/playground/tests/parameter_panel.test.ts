import { ParameterPanel } from "../src/parameter_panel.js";
import { decayWorld, deepEqual, equal, test, throws } from "./testkit.js";

await test("parameter panel accepts declared parameters and returns effective values", () => {
  const panel = new ParameterPanel(decayWorld);
  panel.set("rate", 0.4);
  deepEqual(panel.snapshot.overrides, { rate: 0.4 });
  deepEqual(panel.values(), { rate: 0.4 });
  throws(() => panel.set("unknown", 0), /unknown/);
  panel.reset("rate");
  equal(panel.snapshot.changed, 0);
  deepEqual(panel.values(), { rate: 0.25 });
});
