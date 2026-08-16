import type {
  IntervalEstimate,
  ParameterUncertainty,
  TrajectoryBand,
  Uncertainty,
  UncertaintyModel,
  WorldDefinition,
} from "@lawsynth/world-schema";

export interface UncertaintySummary {
  readonly counts: Readonly<Record<Uncertainty["level"], number>>;
  readonly method?: string;
  readonly seed?: number;
  readonly entries: readonly Uncertainty[];
}

function assertProbability(value: number, label: string): void {
  if (!Number.isFinite(value) || value <= 0 || value > 1) throw new RangeError(`${label} must be in (0, 1]`);
}

export function validateInterval(interval: IntervalEstimate, label = "interval"): void {
  if (!Number.isFinite(interval.lower) || !Number.isFinite(interval.upper) || interval.lower > interval.upper) {
    throw new RangeError(`${label} bounds must be finite and ordered`);
  }
  assertProbability(interval.confidence, `${label} confidence`);
}

export function validateTrajectoryBand(band: TrajectoryBand): void {
  assertProbability(band.confidence, `band ${band.variable} confidence`);
  const length = band.times.length;
  if (length === 0 || band.lower.length !== length || band.upper.length !== length || (band.median !== undefined && band.median.length !== length)) {
    throw new RangeError(`band ${band.variable} arrays must be non-empty and aligned`);
  }
  let prior = Number.NEGATIVE_INFINITY;
  for (let index = 0; index < length; index += 1) {
    const time = band.times[index]!;
    const lower = band.lower[index]!;
    const upper = band.upper[index]!;
    const median = band.median?.[index];
    if (![time, lower, upper, ...(median === undefined ? [] : [median])].every(Number.isFinite)) throw new RangeError(`band ${band.variable} contains non-finite data`);
    if (time < prior) throw new RangeError(`band ${band.variable} times must be monotonic`);
    if (lower > upper || (median !== undefined && (median < lower || median > upper))) throw new RangeError(`band ${band.variable} bounds are inconsistent at ${index}`);
    prior = time;
  }
}

export function uncertaintySummary(model: UncertaintyModel | undefined): UncertaintySummary {
  const counts: Record<Uncertainty["level"], number> = { data: 0, parameter: 0, structural: 0, trajectory: 0 };
  if (model === undefined) return Object.freeze({ counts: Object.freeze(counts), entries: Object.freeze([]) });
  for (const entry of model.entries) {
    counts[entry.level] += 1;
    if (entry.level === "parameter") {
      if (entry.interval !== undefined) validateInterval(entry.interval, `parameter ${entry.parameter}`);
      if (entry.standardError !== undefined && (!Number.isFinite(entry.standardError) || entry.standardError < 0)) throw new RangeError(`parameter ${entry.parameter} standard error must be non-negative`);
      if (entry.samples?.some((sample) => !Number.isFinite(sample))) throw new RangeError(`parameter ${entry.parameter} samples must be finite`);
    } else if (entry.level === "trajectory") {
      entry.bands.forEach(validateTrajectoryBand);
    }
  }
  return Object.freeze({
    counts: Object.freeze(counts),
    ...(model.method === undefined ? {} : { method: model.method }),
    ...(model.seed === undefined ? {} : { seed: model.seed }),
    entries: Object.freeze([...model.entries]),
  });
}

export function parameterUncertaintyFor(world: WorldDefinition, parameter: string): ParameterUncertainty | undefined {
  return world.uncertainty?.entries.find((entry): entry is ParameterUncertainty => entry.level === "parameter" && entry.parameter === parameter);
}

export function bandPolygonPoints(band: TrajectoryBand): readonly { readonly time: number; readonly value: number }[] {
  validateTrajectoryBand(band);
  const upper = band.times.map((time, index) => ({ time, value: band.upper[index]! }));
  const lower = [...band.times].reverse().map((time, reverseIndex) => ({ time, value: band.lower[band.lower.length - reverseIndex - 1]! }));
  return Object.freeze([...upper, ...lower]);
}
