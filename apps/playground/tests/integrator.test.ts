import { forwardEuler } from "../src/integrator.js";
import { dampedOscillatorWorld, lotkaVolterraWorld, SEED_INITIAL_STATE } from "../src/seed_worlds.js";
import { deepEqual, equal, ok, test, throws } from "./testkit.js";

await test("forwardEuler decays exponentially for a first-order world", () => {
  const trajectory = forwardEuler(dampedOscillatorWorld, {
    start: 0, end: 4, step: 0.01, initial: SEED_INITIAL_STATE["damped-oscillator"]!,
  });
  deepEqual([...trajectory.variables], ["x", "v"]);
  equal(trajectory.times[0], 0);
  equal(trajectory.values[0]![0], 1);
  // With ζ=0.15, ω=2 the amplitude must decay below its initial displacement.
  const finalX = trajectory.values.at(-1)![0]!;
  ok(Math.abs(finalX) < 1, "oscillator amplitude should decay");
});

await test("forwardEuler produces bounded predator-prey cycles", () => {
  const trajectory = forwardEuler(lotkaVolterraWorld, {
    start: 0, end: 10, step: 0.005, initial: SEED_INITIAL_STATE["lotka-volterra"]!,
  });
  ok(trajectory.values.every((row) => row.every((value) => Number.isFinite(value) && value > 0)), "populations stay finite and positive");
});

await test("forwardEuler rejects a world without continuous laws", () => {
  const empty = { formatVersion: "0.1.0", id: "empty", time: { kind: "continuous" }, variables: [{ id: "x", role: "state" }], laws: [] } as never;
  throws(() => forwardEuler(empty, { start: 0, end: 1, step: 0.1, initial: {} }), /no integrable continuous laws/);
});
