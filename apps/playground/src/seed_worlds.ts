import type { WorldDefinition } from "@lawsynth/world-schema";
import { ExampleCatalog, type PlaygroundExample } from "./examples.js";
import { createLocalBindings } from "./integrator.js";
import { PlaygroundController } from "./playground.js";
import { LocalSimulation } from "./simulation.js";
import { WasmRuntime } from "./wasm.js";
import type { WorldChoice } from "./world_picker.js";

/**
 * Seed worlds are authored in the minimal Rust-native IR shape that
 * `validateWorld` accepts (closed key-sets: no law `id`, no world-level
 * metadata, expression nodes carry only their structural keys). The public
 * `WorldDefinition` type is richer than that persisted shape, so — exactly like
 * the editor fixtures — we assert through `unknown` at this single boundary.
 */
function seedWorld(world: unknown): WorldDefinition {
  return world as WorldDefinition;
}

/** Damped harmonic oscillator: x'' + 2ζω x' + ω² x = 0 as a first-order system. */
export const dampedOscillatorWorld: WorldDefinition = seedWorld({
  formatVersion: "0.1.0",
  id: "damped-oscillator",
  time: { kind: "continuous", symbol: "t", unit: "s" },
  variables: [
    { id: "x", role: "state", unit: "m" },
    { id: "v", role: "state", unit: "m/s" },
  ],
  parameters: [
    { id: "omega", value: 2, unit: "1/s" },
    { id: "zeta", value: 0.15, unit: "1" },
  ],
  laws: [
    { kind: "continuous", target: "x", expression: { kind: "symbol", id: "v" } },
    {
      kind: "continuous",
      target: "v",
      expression: {
        kind: "binary",
        operator: "sub",
        left: {
          kind: "unary",
          operator: "neg",
          operand: {
            kind: "binary",
            operator: "mul",
            left: { kind: "binary", operator: "pow", left: { kind: "symbol", id: "omega" }, right: { kind: "constant", value: 2 } },
            right: { kind: "symbol", id: "x" },
          },
        },
        right: {
          kind: "binary",
          operator: "mul",
          left: {
            kind: "binary",
            operator: "mul",
            left: { kind: "constant", value: 2 },
            right: { kind: "symbol", id: "zeta" },
          },
          right: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "omega" }, right: { kind: "symbol", id: "v" } },
        },
      },
    },
  ],
});

/** Lotka–Volterra predator–prey system. */
export const lotkaVolterraWorld: WorldDefinition = seedWorld({
  formatVersion: "0.1.0",
  id: "lotka-volterra",
  time: { kind: "continuous", symbol: "t", unit: "s" },
  variables: [
    { id: "prey", role: "state", unit: "1" },
    { id: "pred", role: "state", unit: "1" },
  ],
  parameters: [
    { id: "alpha", value: 1.1, unit: "1/s" },
    { id: "beta", value: 0.4, unit: "1/s" },
    { id: "delta", value: 0.1, unit: "1/s" },
    { id: "gamma", value: 0.4, unit: "1/s" },
  ],
  laws: [
    {
      kind: "continuous",
      target: "prey",
      expression: {
        kind: "binary",
        operator: "sub",
        left: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "alpha" }, right: { kind: "symbol", id: "prey" } },
        right: {
          kind: "binary",
          operator: "mul",
          left: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "beta" }, right: { kind: "symbol", id: "prey" } },
          right: { kind: "symbol", id: "pred" },
        },
      },
    },
    {
      kind: "continuous",
      target: "pred",
      expression: {
        kind: "binary",
        operator: "sub",
        left: {
          kind: "binary",
          operator: "mul",
          left: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "delta" }, right: { kind: "symbol", id: "prey" } },
          right: { kind: "symbol", id: "pred" },
        },
        right: { kind: "binary", operator: "mul", left: { kind: "symbol", id: "gamma" }, right: { kind: "symbol", id: "pred" } },
      },
    },
  ],
});

/** Suggested initial state for each seed world, keyed by world id. */
export const SEED_INITIAL_STATE: Readonly<Record<string, Readonly<Record<string, number>>>> = Object.freeze({
  "damped-oscillator": Object.freeze({ x: 1, v: 0 }),
  "lotka-volterra": Object.freeze({ prey: 10, pred: 5 }),
});

/** The seed worlds as playground catalog entries (title, summary, category). */
export const SEED_EXAMPLES: readonly PlaygroundExample[] = Object.freeze([
  {
    id: "damped-oscillator",
    title: "Damped oscillator",
    summary: "A mass on a spring losing energy to friction — tune ω and ζ to move between under-, critical, and over-damped regimes.",
    category: "dynamics",
    world: dampedOscillatorWorld,
    featured: true,
  },
  {
    id: "lotka-volterra",
    title: "Lotka–Volterra",
    summary: "Classic predator–prey cycles: prey grow, predators feed, both oscillate out of phase.",
    category: "ecology",
    world: lotkaVolterraWorld,
    featured: true,
  },
]);

/** Seed worlds as `WorldPicker` choices, ready to register on a controller. */
export function seedWorldChoices(): readonly WorldChoice[] {
  return SEED_EXAMPLES.map((example) => Object.freeze({
    id: example.id,
    name: example.title,
    description: example.summary,
    world: example.world,
    source: "example" as const,
  }));
}

/** A catalog preloaded with the seed worlds. */
export function seedExampleCatalog(): ExampleCatalog {
  return new ExampleCatalog(SEED_EXAMPLES);
}

/** A `LocalSimulation` driven by the deterministic Euler integrator — no WASM artifact required. */
export function createLocalSimulation(clock?: () => number): LocalSimulation {
  const runtime = new WasmRuntime({ loader: () => createLocalBindings() });
  return clock === undefined ? new LocalSimulation(runtime) : new LocalSimulation(runtime, clock);
}

/**
 * Builds a ready-to-mount controller: a local Euler-backed simulation, the seed
 * worlds registered in the picker, and the damped oscillator loaded as the
 * starting world. This is the smallest "front door" a host page needs.
 */
export function createSeededPlayground(clock?: () => number): PlaygroundController {
  const controller = new PlaygroundController(createLocalSimulation(clock), dampedOscillatorWorld);
  for (const choice of seedWorldChoices()) controller.worlds.add(choice);
  return controller;
}
