/** Dependency-free assertions keep docs compilation tests runnable in Node. */
export function equal<T>(actual: T, expected: T, message = "values differ"): void {
  if (!Object.is(actual, expected)) {
    throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
  }
}

export function deepEqual(actual: unknown, expected: unknown, message = "values differ"): void {
  const received = JSON.stringify(actual);
  const wanted = JSON.stringify(expected);
  if (received !== wanted) throw new Error(`${message}: expected ${wanted}, got ${received}`);
}

export function contains(value: string, expected: string, message = "text is missing expected value"): void {
  if (!value.includes(expected)) throw new Error(`${message}: ${expected}`);
}

export function throws(callback: () => unknown, pattern: RegExp): void {
  try {
    callback();
  } catch (error) {
    if (pattern.test(error instanceof Error ? error.message : String(error))) return;
    throw new Error(`thrown error did not match ${pattern}`);
  }
  throw new Error("expected callback to throw");
}

/** A thrown assertion makes Node's test runner fail the containing module. */
export function test(name: string, callback: () => void): void {
  try {
    callback();
  } catch (error) {
    throw new Error(`${name}: ${error instanceof Error ? error.message : String(error)}`);
  }
}
