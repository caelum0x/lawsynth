import { normalizeAnnotation, type Annotation, type AxisSpec } from "./axis.js";
import { extent, padDomain, type Domain } from "./scales.js";
import { normalizeSeries, type Series } from "./series.js";

export interface ChartModel {
  readonly title: string;
  readonly series: readonly Series[];
  readonly xAxis: AxisSpec;
  readonly yAxis: AxisSpec;
  readonly annotations: readonly Annotation[];
}

export interface ChartModelInput {
  readonly title: string;
  readonly series: readonly Series[];
  readonly xLabel?: string;
  readonly yLabel?: string;
  readonly annotations?: readonly Annotation[];
  readonly xDomain?: Domain;
  readonly yDomain?: Domain;
}

function domainFor(series: readonly Series[], coordinate: "x" | "y"): Domain {
  const values = series.flatMap((line) => line.points.map((point) => point[coordinate]));
  if (values.length === 0) return { min: 0, max: 1 };
  return padDomain(extent(values));
}

/** Builds a renderer-neutral chart model with independent copies of user data. */
export function createChartModel(input: ChartModelInput): ChartModel {
  if (!input.title.trim()) throw new TypeError("chart title must be non-empty");
  const ids = new Set<string>();
  const series = input.series.map((line) => {
    const normalized = normalizeSeries(line);
    if (ids.has(normalized.id)) throw new RangeError(`duplicate series id: ${normalized.id}`);
    ids.add(normalized.id);
    return normalized;
  });
  const xDomain = input.xDomain ?? domainFor(series, "x");
  const yDomain = input.yDomain ?? domainFor(series, "y");
  return {
    title: input.title,
    series,
    xAxis: { domain: xDomain, label: input.xLabel ?? "time" },
    yAxis: { domain: yDomain, label: input.yLabel ?? "value" },
    annotations: (input.annotations ?? []).map(normalizeAnnotation),
  };
}
