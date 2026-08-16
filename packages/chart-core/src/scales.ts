export interface Domain { readonly min: number; readonly max: number; }
export type ScaleKind = "linear" | "log";

function checkDomain(domain: Domain): void {
  if (!Number.isFinite(domain.min) || !Number.isFinite(domain.max) || domain.min > domain.max) throw new RangeError("invalid domain");
}

export function extent(values: readonly number[]): Domain {
  if (values.length === 0) throw new RangeError("cannot calculate extent of an empty array");
  let min = Infinity; let max = -Infinity;
  for (const value of values) { if (!Number.isFinite(value)) throw new RangeError("extent requires finite values"); min = Math.min(min, value); max = Math.max(max, value); }
  return { min, max };
}

/** Pads a degenerate domain so a constant trace has visible scale. */
export function padDomain(domain: Domain, fraction = 0.05): Domain {
  checkDomain(domain);
  if (!Number.isFinite(fraction) || fraction < 0) throw new RangeError("fraction must be non-negative");
  const span = domain.max - domain.min;
  const padding = span === 0 ? Math.max(Math.abs(domain.min) * fraction, 1) : span * fraction;
  return { min: domain.min - padding, max: domain.max + padding };
}

export function createScale(domain: Domain, range: Domain, kind: ScaleKind = "linear"): (value: number) => number {
  checkDomain(domain); checkDomain(range);
  if (domain.min === domain.max) throw new RangeError("scale domain must have non-zero span");
  if (kind === "log" && domain.min <= 0) throw new RangeError("log domains must be strictly positive");
  const sourceMin = kind === "log" ? Math.log(domain.min) : domain.min;
  const sourceMax = kind === "log" ? Math.log(domain.max) : domain.max;
  return (value: number) => {
    if (!Number.isFinite(value) || (kind === "log" && value <= 0)) throw new RangeError("value is outside scale domain type");
    const source = kind === "log" ? Math.log(value) : value;
    return range.min + ((source - sourceMin) / (sourceMax - sourceMin)) * (range.max - range.min);
  };
}

export function invertScale(domain: Domain, range: Domain, kind: ScaleKind = "linear"): (pixel: number) => number {
  const forward = createScale(range, domain, kind === "linear" ? "linear" : "linear");
  if (kind === "linear") return forward;
  checkDomain(domain); checkDomain(range);
  if (domain.min <= 0) throw new RangeError("log domains must be strictly positive");
  const logMin = Math.log(domain.min); const logMax = Math.log(domain.max);
  return (pixel: number) => Math.exp(logMin + ((pixel - range.min) / (range.max - range.min)) * (logMax - logMin));
}

export function linearTicks(domain: Domain, count = 6): number[] {
  checkDomain(domain);
  if (!Number.isInteger(count) || count < 2) throw new RangeError("tick count must be at least two");
  if (domain.min === domain.max) return [domain.min];
  const raw = (domain.max - domain.min) / (count - 1);
  const power = 10 ** Math.floor(Math.log10(raw));
  const multiplier = [1, 2, 2.5, 5, 10].find((candidate) => raw <= candidate * power) ?? 10;
  const step = multiplier * power;
  const start = Math.ceil(domain.min / step) * step;
  const ticks: number[] = [];
  for (let value = start; value <= domain.max + step * 1e-10; value += step) ticks.push(Number(value.toPrecision(14)));
  return ticks;
}
