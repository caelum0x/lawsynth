export interface HeatmapCell { readonly x: number; readonly y: number; readonly value: number; }
export interface HeatmapGrid { readonly x: readonly number[]; readonly y: readonly number[]; readonly values: readonly (readonly number[])[]; }

/** Validates a dense grid and flattens it into renderer-independent cells. */
export function heatmapCells(grid: HeatmapGrid): HeatmapCell[] {
  if (grid.x.length === 0 || grid.y.length === 0) throw new RangeError("heatmap axes must be non-empty");
  if (grid.values.length !== grid.y.length) throw new RangeError("heatmap row count must match y axis");
  const cells: HeatmapCell[] = [];
  grid.y.forEach((y, row) => {
    if (!Number.isFinite(y)) throw new RangeError("heatmap y coordinates must be finite");
    const values = grid.values[row]; if (values === undefined || values.length !== grid.x.length) throw new RangeError("heatmap rows must match x axis");
    grid.x.forEach((x, column) => { const value = values[column]!; if (!Number.isFinite(x) || !Number.isFinite(value)) throw new RangeError("heatmap values must be finite"); cells.push({ x, y, value }); });
  });
  return cells;
}
