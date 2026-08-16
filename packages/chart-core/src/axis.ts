import { linearTicks, type Domain } from "./scales.js";

export interface AxisTick { readonly value: number; readonly label: string; }
export interface AxisSpec {
  readonly domain: Domain;
  readonly label: string;
  readonly tickCount?: number;
  readonly formatter?: (value: number) => string;
}

export interface Annotation {
  readonly id: string;
  readonly kind: "point" | "line" | "band";
  readonly x?: number;
  readonly y?: number;
  readonly x1?: number;
  readonly x2?: number;
  readonly y1?: number;
  readonly y2?: number;
  readonly label: string;
  readonly severity?: "info" | "warning" | "error";
}

export function defaultTickFormatter(value: number): string {
  if (!Number.isFinite(value)) throw new RangeError("tick value must be finite");
  const magnitude = Math.abs(value);
  if ((magnitude !== 0 && magnitude >= 1e5) || (magnitude !== 0 && magnitude < 1e-3)) return value.toExponential(2);
  return Number(value.toPrecision(8)).toString();
}

export function makeAxisTicks(axis: AxisSpec): AxisTick[] {
  if (!axis.label.trim()) throw new TypeError("axis label must be non-empty");
  const formatter = axis.formatter ?? defaultTickFormatter;
  return linearTicks(axis.domain, axis.tickCount ?? 6).map((value) => ({ value, label: formatter(value) }));
}

/** Validates annotation coordinates before a renderer consumes them. */
export function normalizeAnnotation(annotation: Annotation): Annotation {
  if (!annotation.id.trim() || !annotation.label.trim()) throw new TypeError("annotation id and label must be non-empty");
  const coordinates = [annotation.x, annotation.y, annotation.x1, annotation.x2, annotation.y1, annotation.y2];
  for (const coordinate of coordinates) if (coordinate !== undefined && !Number.isFinite(coordinate)) throw new RangeError("annotation coordinates must be finite");
  if (annotation.kind === "point" && (annotation.x === undefined || annotation.y === undefined)) throw new RangeError("point annotations require x and y");
  if (annotation.kind === "line" && annotation.x === undefined && annotation.y === undefined) throw new RangeError("line annotations require x or y");
  if (annotation.kind === "band" && annotation.x1 === undefined && annotation.y1 === undefined) throw new RangeError("band annotations require a lower bound");
  if (annotation.kind === "band" && annotation.x2 === undefined && annotation.y2 === undefined) throw new RangeError("band annotations require an upper bound");
  return { ...annotation };
}
