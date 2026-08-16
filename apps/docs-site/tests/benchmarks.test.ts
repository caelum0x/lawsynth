import { benchmarkSeries, compareBenchmarks } from "../src/benchmarks.js";
import { deepEqual, equal, test, throws } from "./assertions.js";

const baseline = [{ suite: "discovery", case: "linear", metric: "duration", value: 10, unit: "ms", commit: "base", recordedAt: "2026-01-01T00:00:00Z" }];

test("benchmark comparison identifies matched regressions and ignores zero baselines", () => {
  const comparisons = compareBenchmarks([{ ...baseline[0]!, value: 11, commit: "head", recordedAt: "2026-01-02T00:00:00Z" }, { ...baseline[0]!, case: "cold", value: 1, commit: "head", recordedAt: "2026-01-02T00:00:00Z" }], baseline);
  equal(comparisons.length, 1);
  equal(comparisons[0]!.regression, true);
  equal(comparisons[0]!.change, 0.1);
});

test("benchmark series orders a metric by recording time and validates input", () => {
  const series = benchmarkSeries([{ ...baseline[0]!, recordedAt: "2026-01-02T00:00:00Z" }, { ...baseline[0]!, recordedAt: "2026-01-01T00:00:00Z", value: 8 }]);
  deepEqual(series.get("discovery:linear:duration")!.map((entry) => entry.value), [8, 10]);
  throws(() => compareBenchmarks(baseline, baseline, -1), /non-negative/);
});
