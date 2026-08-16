import type { DatasetDescriptor } from "@lawsynth/api-client";

export type DatasetValue = number | string | boolean | null;
export type InferredColumnType = "number" | "boolean" | "timestamp" | "string" | "empty";

export interface DatasetPreview {
  readonly columns: readonly string[];
  readonly rows: readonly (readonly DatasetValue[])[];
  readonly totalRows?: number;
  readonly truncated: boolean;
}

export interface ColumnProfile {
  readonly name: string;
  readonly type: InferredColumnType;
  readonly present: number;
  readonly missing: number;
  readonly unique: number;
  readonly minimum?: number;
  readonly maximum?: number;
}

export interface DatasetViewModel {
  readonly descriptor: DatasetDescriptor;
  readonly preview?: DatasetPreview;
  readonly profiles: readonly ColumnProfile[];
  readonly usableForDiscovery: boolean;
  readonly issues: readonly string[];
}

function typeOf(value: DatasetValue): InferredColumnType {
  if (value === null || value === "") return "empty";
  if (typeof value === "number") return "number";
  if (typeof value === "boolean") return "boolean";
  if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}T/u.test(value) && Number.isFinite(Date.parse(value))) return "timestamp";
  return "string";
}

function mergedType(types: ReadonlySet<InferredColumnType>): InferredColumnType {
  const nonempty = [...types].filter((type) => type !== "empty");
  return nonempty.length === 0 ? "empty" : nonempty.every((type) => type === nonempty[0]) ? nonempty[0]! : "string";
}

export function validatePreview(preview: DatasetPreview, maxRows = 10_000): DatasetPreview {
  if (preview.columns.length === 0) throw new RangeError("dataset preview needs columns");
  if (preview.rows.length > maxRows) throw new RangeError(`dataset preview exceeds ${maxRows} rows`);
  const names = new Set<string>();
  const columns = preview.columns.map((name, index) => {
    const value = name.trim();
    if (!value || names.has(value)) throw new RangeError(`column ${index} must have a unique non-empty name`);
    names.add(value); return value;
  });
  const rows = preview.rows.map((row, index) => {
    if (row.length !== columns.length) throw new RangeError(`preview row ${index} has ${row.length} cells; expected ${columns.length}`);
    return Object.freeze(row.map((value, column) => {
      if (typeof value === "number" && !Number.isFinite(value)) throw new RangeError(`cell ${index}:${column} must be finite`);
      if (value !== null && !["number", "string", "boolean"].includes(typeof value)) throw new TypeError(`cell ${index}:${column} has an unsupported type`);
      return value;
    }));
  });
  return Object.freeze({ columns: Object.freeze(columns), rows: Object.freeze(rows), ...(preview.totalRows === undefined ? {} : { totalRows: preview.totalRows }), truncated: preview.truncated });
}

export function profilePreview(preview: DatasetPreview): readonly ColumnProfile[] {
  const valid = validatePreview(preview);
  return Object.freeze(valid.columns.map((name, column) => {
    const values = valid.rows.map((row) => row[column]!);
    const presentValues = values.filter((value) => value !== null && value !== "");
    const type = mergedType(new Set(values.map(typeOf)));
    const numeric = type === "number" ? presentValues as number[] : [];
    return Object.freeze({
      name, type, present: presentValues.length, missing: values.length - presentValues.length,
      unique: new Set(presentValues.map((value) => `${typeof value}:${String(value)}`)).size,
      ...(numeric.length === 0 ? {} : { minimum: Math.min(...numeric), maximum: Math.max(...numeric) }),
    });
  }));
}

export function datasetViewModel(descriptor: DatasetDescriptor, preview?: DatasetPreview): DatasetViewModel {
  const normalized = preview === undefined ? undefined : validatePreview(preview);
  const profiles = normalized === undefined ? [] : profilePreview(normalized);
  const issues: string[] = [];
  if (descriptor.schema.length === 0) issues.push("Dataset schema is empty.");
  if (profiles.filter((profile) => profile.type === "number").length < 2) issues.push("Discovery requires at least two numeric columns.");
  if (profiles.some((profile) => profile.present === 0)) issues.push("One or more preview columns contain no values.");
  return Object.freeze({ descriptor, ...(normalized === undefined ? {} : { preview: normalized }), profiles: Object.freeze(profiles), usableForDiscovery: issues.length === 0, issues: Object.freeze(issues) });
}
