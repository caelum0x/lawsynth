import type { WorldDefinition } from "@lawsynth/world-schema";

export const decayWorld: WorldDefinition = Object.freeze({
  formatVersion: "0.1.0",
  id: "playground-decay",
  time: { kind: "continuous", unit: "s" },
  variables: [{ id: "x", role: "state", unit: "1" }],
  parameters: [{ id: "rate", value: 0.25, unit: "1/s" }],
  laws: [{
    kind: "continuous", target: "x",
    expression: {
      kind: "binary", operator: "mul",
      left: { kind: "unary", operator: "neg", operand: { kind: "symbol", id: "rate" } },
      right: { kind: "symbol", id: "x" },
    },
  }],
// The current Rust codec deliberately omits a law id; the public editor type
// still requires one. This is the narrow boundary the editor actually parses.
} as unknown as WorldDefinition);

export async function test(name: string, operation: () => void | Promise<void>): Promise<void> {
  try {
    await operation();
  } catch (error) {
    throw new Error(`${name}: ${error instanceof Error ? error.message : String(error)}`, { cause: error });
  }
}

export function equal<T>(actual: T, expected: T, message = "values differ"): void {
  if (!Object.is(actual, expected)) throw new Error(`${message}: expected ${String(expected)}, received ${String(actual)}`);
}

export function deepEqual(actual: unknown, expected: unknown, message = "values differ"): void {
  const left = JSON.stringify(actual);
  const right = JSON.stringify(expected);
  if (left !== right) throw new Error(`${message}: expected ${right}, received ${left}`);
}

export function ok(value: unknown, message = "expected a truthy value"): asserts value {
  if (!value) throw new Error(message);
}

export function throws(operation: () => unknown, expression?: RegExp): void {
  try {
    operation();
  } catch (error) {
    if (expression !== undefined && !expression.test(error instanceof Error ? error.message : String(error))) {
      throw new Error(`error did not match ${expression}`);
    }
    return;
  }
  throw new Error("expected operation to throw");
}

export async function rejects(operation: () => Promise<unknown>, expression?: RegExp): Promise<void> {
  try {
    await operation();
  } catch (error) {
    if (expression !== undefined && !expression.test(error instanceof Error ? error.message : String(error))) {
      throw new Error(`error did not match ${expression}`);
    }
    return;
  }
  throw new Error("expected operation to reject");
}
