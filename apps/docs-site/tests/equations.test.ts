import type { Law, WorldDefinition } from "@lawsynth/world-schema";
import { equationReference, renderEquation, worldEquationReferences } from "../src/equations.js";
import { contains, deepEqual, equal, test } from "./assertions.js";

const growthLaw: Law = {
  id: "population-growth",
  kind: "continuous",
  target: "population",
  expression: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "rate" }, right: { kind: "symbol", id: "population" } },
  description: "Exponential population growth.",
};

test("equation references preserve executable law semantics in accessible HTML", () => {
  const reference = equationReference(growthLaw, "τ");
  equal(reference.plainText, "dpopulation/dτ = rate × population");
  contains(reference.html, 'aria-label="dpopulation/dτ = rate × population"');
  contains(renderEquation({ kind: "binary", operator: "add", left: { kind: "symbol", id: "x" }, right: { kind: "constant", value: 1 } }), "x + 1");
});

test("world equation references use the world's declared time symbol", () => {
  const world: WorldDefinition = { formatVersion: "0.1.0", id: "growth", time: { kind: "continuous", symbol: "time" }, variables: [{ id: "population", role: "state" }], laws: [growthLaw] };
  deepEqual(worldEquationReferences(world).map((entry) => [entry.id, entry.plainText]), [["population-growth", "dpopulation/dtime = rate × population"]]);
});
