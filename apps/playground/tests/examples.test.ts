import { ExampleCatalog } from "../src/examples.js";
import { decayWorld, deepEqual, equal, test, throws } from "./testkit.js";

await test("example catalog searches and prioritizes featured executable worlds", () => {
  const catalog = new ExampleCatalog([
    { id: "decay", title: "Decay", summary: "Continuous decay", category: "dynamics", world: decayWorld },
    { id: "featured-decay", title: "Decay explorer", summary: "Decay with an adjustable rate", category: "dynamics", world: decayWorld, featured: true },
  ]);
  deepEqual(catalog.list("dynamics").map((entry) => entry.id), ["featured-decay", "decay"]);
  equal(catalog.search("adjustable")[0]?.id, "featured-decay");
  throws(() => catalog.add({ id: "decay", title: "Duplicate", summary: "Rejected", category: "dynamics", world: decayWorld }), /duplicate/);
});
