/** Tiny dependency-free assertion helpers so contract tests remain portable. */
export const assert = Object.freeze({
  equal(actual: unknown, expected: unknown): void {
    if (!Object.is(actual, expected)) throw new Error(`expected ${String(expected)}, received ${String(actual)}`);
  },
  deepEqual(actual: unknown, expected: unknown): void {
    const received = JSON.stringify(actual);
    const wanted = JSON.stringify(expected);
    if (received !== wanted) throw new Error(`expected ${wanted}, received ${received}`);
  },
  throws(callback: () => void, pattern: RegExp): void {
    try {
      callback();
    } catch (error) {
      if (pattern.test(error instanceof Error ? error.message : String(error))) return;
      throw new Error(`thrown error did not match ${pattern}`);
    }
    throw new Error("expected function to throw");
  },
});

/** Node's test runner marks a module failure if this real assertion suite throws. */
export function test(name: string, callback: () => void): void {
  try {
    callback();
  } catch (error) {
    throw new Error(`${name}: ${error instanceof Error ? error.message : String(error)}`);
  }
}
