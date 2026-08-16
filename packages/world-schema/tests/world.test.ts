import { parameterById, stateVariables, variableById } from "../src/world.js";
import { equal } from "./test-support.js";
export function runWorldTests(): void { const world = { formatVersion: "0.1.0", id: "w", time: { kind: "continuous" as const }, variables: [{ id: "x", role: "state" as const }], parameters: [{ id: "k", value: 1 }], laws: [{ id: "dx", kind: "continuous" as const, target: "x", expression: { kind: "constant" as const, value: 0 } }] }; equal(variableById(world, "x")?.role, "state"); equal(parameterById(world, "k")?.value, 1); equal(stateVariables(world).length, 1); }
