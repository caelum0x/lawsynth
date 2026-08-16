export interface BenchmarkResult {
  readonly suite: string; readonly case: string; readonly metric: string; readonly value: number;
  readonly unit: string; readonly commit: string; readonly recordedAt: string;
}
export interface BenchmarkComparison { readonly current: BenchmarkResult; readonly baseline: BenchmarkResult; readonly change: number; readonly regression: boolean; }

function benchmarkKey(result: BenchmarkResult): string { return `${result.suite}:${result.case}:${result.metric}`; }
function validateResult(result: BenchmarkResult): void {
  if (![result.suite, result.case, result.metric, result.unit, result.commit].every((value) => value.trim())) throw new RangeError("benchmark identity fields are required");
  if (!Number.isFinite(result.value) || !Number.isFinite(Date.parse(result.recordedAt))) throw new RangeError("benchmark value and timestamp must be valid");
}

export function compareBenchmarks(current: readonly BenchmarkResult[], baseline: readonly BenchmarkResult[], threshold = 0.05): readonly BenchmarkComparison[] {
  if (!Number.isFinite(threshold) || threshold < 0) throw new RangeError("benchmark threshold must be non-negative");
  baseline.forEach(validateResult);
  const lookup = new Map(baseline.map((result) => [benchmarkKey(result), result]));
  return Object.freeze(current.flatMap((result): BenchmarkComparison[] => {
    validateResult(result);
    const prior = lookup.get(benchmarkKey(result));
    if (prior === undefined || prior.value === 0) return [];
    const change = (result.value - prior.value) / Math.abs(prior.value);
    return [{ current: result, baseline: prior, change, regression: change > threshold }];
  }));
}

export function benchmarkSeries(results: readonly BenchmarkResult[]): ReadonlyMap<string, readonly BenchmarkResult[]> {
  const mutable = new Map<string, BenchmarkResult[]>();
  for (const result of results) {
    validateResult(result);
    const key = benchmarkKey(result);
    mutable.set(key, [...(mutable.get(key) ?? []), result]);
  }
  return new Map([...mutable].map(([key, values]) => [key, Object.freeze(values.sort((a, b) => a.recordedAt.localeCompare(b.recordedAt)))]));
}
