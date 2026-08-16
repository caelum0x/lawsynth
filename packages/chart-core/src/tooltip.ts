import { nearestPoint, type Series } from "./series.js";

export interface TooltipValue { readonly seriesId: string; readonly label: string; readonly x: number; readonly y: number; readonly unit?: string; }

/** Returns nearest observations; it intentionally does not interpolate data. */
export function tooltipAtX(series: readonly Series[], x: number): TooltipValue[] {
  if (!Number.isFinite(x)) throw new RangeError("tooltip x must be finite");
  return series.flatMap((line) => {
    const point = nearestPoint(line, x);
    return point === undefined ? [] : [{ seriesId: line.id, label: line.label, x: point.x, y: point.y, ...(line.unit === undefined ? {} : { unit: line.unit }) }];
  });
}
