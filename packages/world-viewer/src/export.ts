import type { WorldViewModel } from "./viewer.js";
function cell(value: string | number): string { const text = String(value); return /[",\n]/u.test(text) ? `"${text.replaceAll('"', '""')}"` : text; }
export function exportViewerJson(model: WorldViewModel): string { return JSON.stringify(model, null, 2) + "\n"; }
/** Exports raw trajectory points, not screen coordinates or raster output. */
export function exportTrajectoryCsv(model: WorldViewModel): string { if (!model.trajectory) throw new Error("viewer model has no trajectory"); const rows = ["variable,time,value"]; for (const series of model.trajectory.series) for (const point of series.points) rows.push([series.id, point.x, point.y].map(cell).join(",")); return rows.join("\n") + "\n"; }
