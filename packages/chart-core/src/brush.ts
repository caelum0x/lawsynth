import type { Domain } from "./scales.js";

export interface BrushSelection { readonly x: Domain; readonly y?: Domain; }

export function normalizeBrush(selection: BrushSelection): BrushSelection {
  const normalize = (domain: Domain): Domain => {
    if (!Number.isFinite(domain.min) || !Number.isFinite(domain.max)) throw new RangeError("brush coordinates must be finite");
    return domain.min <= domain.max ? { ...domain } : { min: domain.max, max: domain.min };
  };
  return { x: normalize(selection.x), ...(selection.y === undefined ? {} : { y: normalize(selection.y) }) };
}

export function pointInBrush(point: { readonly x: number; readonly y: number }, selection: BrushSelection): boolean {
  const brush = normalizeBrush(selection);
  return point.x >= brush.x.min && point.x <= brush.x.max && (brush.y === undefined || (point.y >= brush.y.min && point.y <= brush.y.max));
}
