import { componentNode, type ComponentNode } from "./tokens.js";

export interface TableColumn { readonly id: string; readonly label: string; readonly numeric?: boolean; }
export interface TableRow { readonly id: string; readonly cells: Readonly<Record<string, string | number>>; }
export interface TableProps { readonly caption: string; readonly columns: readonly TableColumn[]; readonly rows: readonly TableRow[]; }

export function createTable(props: TableProps): ComponentNode {
  if (!props.caption.trim() || props.columns.length === 0) throw new RangeError("table requires a caption and columns");
  const columnIds = new Set<string>();
  for (const column of props.columns) {
    if (!column.id.trim() || !column.label.trim() || columnIds.has(column.id)) throw new RangeError("table columns must have unique ids and labels");
    columnIds.add(column.id);
  }
  const rowIds = new Set<string>();
  for (const row of props.rows) {
    if (!row.id.trim() || rowIds.has(row.id)) throw new RangeError("table rows require unique ids");
    rowIds.add(row.id);
    const keys = Object.keys(row.cells);
    if (keys.length !== columnIds.size || keys.some((key) => !columnIds.has(key))) throw new RangeError(`row ${row.id} does not match table columns`);
  }
  return componentNode("table", {}, { children: [
    componentNode("caption", {}, { text: props.caption }),
    componentNode("thead", {}, { children: [componentNode("tr", {}, { children: props.columns.map((column) => componentNode("th", { scope: "col", ...(column.numeric ? { "data-align": "end" } : {}) }, { text: column.label })) })] }),
    componentNode("tbody", {}, { children: props.rows.map((row) => componentNode("tr", { id: row.id }, { children: props.columns.map((column) => componentNode("td", { ...(column.numeric ? { "data-align": "end" } : {}) }, { text: String(row.cells[column.id]) })) })) }),
  ] });
}
