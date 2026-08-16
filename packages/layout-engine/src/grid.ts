import { bounds, type LayoutNode, type LayoutResult, type PositionedNode } from "./layout.js";
export interface GridOptions { readonly columns?: number; readonly gap?: number; readonly rowGap?: number; readonly columnGap?: number; readonly margin?: number; readonly cellWidth?: number; readonly cellHeight?: number; }
export function gridLayout(nodes: readonly LayoutNode[], options: GridOptions = {}): LayoutResult {
  const columns = options.columns ?? Math.max(1, Math.ceil(Math.sqrt(nodes.length))); const gap = options.gap ?? 16, rowGap = options.rowGap ?? gap, columnGap = options.columnGap ?? gap, margin = options.margin ?? 0;
  if (!Number.isInteger(columns) || columns < 1 || rowGap < 0 || columnGap < 0 || margin < 0) throw new RangeError("invalid grid options");
  const cellWidth = options.cellWidth ?? Math.max(0, ...nodes.map((node) => node.width)); const cellHeight = options.cellHeight ?? Math.max(0, ...nodes.map((node) => node.height));
  if (cellWidth < 0 || cellHeight < 0) throw new RangeError("grid cell dimensions must be non-negative");
  const positioned: PositionedNode[] = nodes.map((node, index) => ({ ...node, x: margin + (index % columns) * (cellWidth + columnGap) + (cellWidth - node.width) / 2, y: margin + Math.floor(index / columns) * (cellHeight + rowGap) + (cellHeight - node.height) / 2 }));
  const box = bounds(positioned); return { nodes: positioned, width: box.width + margin * 2, height: box.height + margin * 2 };
}
