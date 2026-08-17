import { createScale, extent, padDomain, type Domain } from "@lawsynth/chart-core";
import type { PlotGeometry, SvgPath } from "@lawsynth/world-viewer";

export interface PlotPoint {
  readonly x: number;
  readonly y: number;
}

export interface PlotLine {
  readonly id: string;
  readonly label: string;
  readonly color?: string;
  readonly points: readonly PlotPoint[];
}

export interface PlotScales {
  readonly geometry: PlotGeometry;
  readonly xDomain: Domain;
  readonly yDomain: Domain;
  readonly toX: (value: number) => number;
  readonly toY: (value: number) => number;
}

const DEFAULT_PADDING = 32;

function pathFrom(points: readonly PlotPoint[], toX: (v: number) => number, toY: (v: number) => number): string {
  if (points.length === 0) return "";
  return points
    .map((point, index) => `${index === 0 ? "M" : "L"}${toX(point.x).toFixed(2)},${toY(point.y).toFixed(2)}`)
    .join(" ");
}

/**
 * Builds a renderer-neutral plot geometry from `chart-core` scale primitives.
 * Every line shares one x/y mapping so overlays (e.g. an uncertainty band) stay
 * pixel-aligned with the lines drawn on top of it.
 */
export function buildPlot(lines: readonly PlotLine[], width: number, height: number, padding = DEFAULT_PADDING): PlotScales {
  if (!(width > 2 * padding) || !(height > 2 * padding)) throw new RangeError("plot area is too small for the padding");
  const xs = lines.flatMap((line) => line.points.map((point) => point.x));
  const ys = lines.flatMap((line) => line.points.map((point) => point.y));
  const xDomain = padDomain(extent(xs.length === 0 ? [0, 1] : xs));
  const yDomain = padDomain(extent(ys.length === 0 ? [0, 1] : ys), 0.08);
  const toX = createScale(xDomain, { min: padding, max: width - padding });
  // `createScale` requires an ascending pixel range; SVG y grows downward, so we
  // build an ascending scale and mirror it to place larger values near the top.
  const yScale = createScale(yDomain, { min: padding, max: height - padding });
  const toY = (value: number): number => height - yScale(value);
  const paths: SvgPath[] = lines.map((line) => ({
    id: line.id,
    label: line.label,
    ...(line.color === undefined ? {} : { color: line.color }),
    d: pathFrom(line.points, toX, toY),
  }));
  const geometry: PlotGeometry = { paths, xDomain, yDomain, width, height };
  return { geometry, xDomain, yDomain, toX, toY };
}

/** Closed polygon tracing the upper edge forward and the lower edge backward. */
export function bandPolygon(
  upper: readonly PlotPoint[],
  lower: readonly PlotPoint[],
  toX: (v: number) => number,
  toY: (v: number) => number,
): string {
  if (upper.length === 0 || lower.length === 0) return "";
  const forward = upper.map((point) => `${toX(point.x).toFixed(2)},${toY(point.y).toFixed(2)}`);
  const backward = [...lower].reverse().map((point) => `${toX(point.x).toFixed(2)},${toY(point.y).toFixed(2)}`);
  return `M${forward.join(" L")} L${backward.join(" L")} Z`;
}

export function linePath(points: readonly PlotPoint[], toX: (v: number) => number, toY: (v: number) => number): string {
  return pathFrom(points, toX, toY);
}
