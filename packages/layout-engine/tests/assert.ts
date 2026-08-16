export function equal<T>(actual: T, expected: T, message = "values differ"): void { if (!Object.is(actual, expected)) throw new Error(`${message}: expected ${String(expected)}, received ${String(actual)}`); }
export function ok(value: unknown, message = "assertion failed"): asserts value { if (!value) throw new Error(message); }
export function throws(fn: () => unknown, pattern?: RegExp): void { try { fn(); } catch (error) { if (!pattern || pattern.test(String(error))) return; throw error; } throw new Error("expected function to throw"); }
