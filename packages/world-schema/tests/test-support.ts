export function equal<T>(actual: T, expected: T, message = "values differ"): void { if (!Object.is(actual, expected)) throw new Error(`${message}: expected ${String(expected)}, received ${String(actual)}`); }
export function ok(value: unknown, message = "expected truthy value"): asserts value { if (!value) throw new Error(message); }
export function throws(action: () => unknown, message = "expected function to throw"): void { try { action(); } catch { return; } throw new Error(message); }
