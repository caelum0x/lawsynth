import type { ChartModel } from "./chart.js";

function csvCell(value: string | number): string { const text = String(value); return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text; }

/** Exports source samples, not rendered pixels, to a standards-compatible CSV string. */
export function chartToCsv(chart: ChartModel): string {
  const rows = ["series_id,series_label,x,y"];
  for (const series of chart.series) for (const point of series.points) rows.push([series.id, series.label, point.x, point.y].map(csvCell).join(","));
  return rows.join("\n") + "\n";
}

/** Stable JSON snapshot for audit trails and renderer hand-off. */
export function chartToJson(chart: ChartModel): string { return JSON.stringify(chart); }
