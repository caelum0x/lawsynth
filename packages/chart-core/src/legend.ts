import { categoricalColor } from "./palette.js";
import type { Series } from "./series.js";

export interface LegendEntry { readonly id: string; readonly label: string; readonly color: string; readonly visible: boolean; }

export function buildLegend(series: readonly Series[], hidden: ReadonlySet<string> = new Set()): LegendEntry[] {
  return series.map((line) => ({ id: line.id, label: line.label, color: line.color ?? categoricalColor(line.id), visible: !hidden.has(line.id) }));
}

export function toggleLegendSeries(hidden: ReadonlySet<string>, id: string): Set<string> {
  const next = new Set(hidden); if (next.has(id)) next.delete(id); else next.add(id); return next;
}
