export function equal<T>(actual: T, expected: T, message = "values differ"): void {
  if (!Object.is(actual, expected)) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
}

export function deepEqual(actual: unknown, expected: unknown, message = "values differ"): void {
  const a = JSON.stringify(actual); const b = JSON.stringify(expected);
  if (a !== b) throw new Error(`${message}: expected ${b}, got ${a}`);
}

export function throws(fn: () => unknown, message = "expected function to throw"): void {
  try { fn(); } catch { return; }
  throw new Error(message);
}
