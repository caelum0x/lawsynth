export interface PlaygroundDataset {
  readonly id: string;
  readonly name: string;
  readonly columns: readonly string[];
  readonly rows: readonly (readonly (number | null)[])[];
  readonly source: "example" | "upload";
}

export interface DatasetLimits {
  readonly maximumRows?: number;
  readonly maximumColumns?: number;
  readonly maximumBytes?: number;
}

function parseCsvRecords(text: string): readonly (readonly string[])[] {
  const records: string[][] = [];
  let record: string[] = [];
  let field = "";
  let quoted = false;

  for (let index = 0; index < text.length; index += 1) {
    const character = text[index]!;
    if (character === '"') {
      if (quoted && text[index + 1] === '"') {
        field += '"';
        index += 1;
      } else {
        quoted = !quoted;
      }
    } else if (character === "," && !quoted) {
      record.push(field);
      field = "";
    } else if ((character === "\n" || character === "\r") && !quoted) {
      if (character === "\r" && text[index + 1] === "\n") index += 1;
      record.push(field);
      if (record.some((value) => value.trim())) records.push(record);
      record = [];
      field = "";
    } else {
      field += character;
    }
  }
  if (quoted) throw new SyntaxError("unterminated CSV quote");
  record.push(field);
  if (record.some((value) => value.trim())) records.push(record);
  return records;
}

function datasetId(text: string): string {
  let hash = 2166136261;
  for (const character of text.slice(0, 100_000)) {
    hash ^= character.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  return `upload-${(hash >>> 0).toString(16)}`;
}

export function parseNumericCsv(text: string, name = "Uploaded dataset", limits: DatasetLimits = {}): PlaygroundDataset {
  const maximumRows = limits.maximumRows ?? 100_000;
  const maximumColumns = limits.maximumColumns ?? 256;
  const maximumBytes = limits.maximumBytes ?? 16 * 1024 * 1024;
  if (new TextEncoder().encode(text).byteLength > maximumBytes) throw new RangeError("dataset exceeds browser size limit");

  const records = parseCsvRecords(text.replace(/^\uFEFF/u, ""));
  if (records.length < 2) throw new RangeError("CSV needs a header and at least one row");
  if (records.length - 1 > maximumRows) throw new RangeError(`dataset exceeds ${maximumRows} rows`);

  const columns = records[0]!.map((column) => column.trim());
  if (columns.length > maximumColumns || columns.some((column) => !column) || new Set(columns).size !== columns.length) {
    throw new RangeError("CSV columns must be unique, non-empty, and within the column limit");
  }
  const rows = records.slice(1).map((cells, row) => {
    if (cells.length !== columns.length) throw new RangeError(`row ${row + 2} has ${cells.length} cells; expected ${columns.length}`);
    return Object.freeze(cells.map((cell, column) => {
      if (!cell.trim()) return null;
      const value = Number(cell);
      if (!Number.isFinite(value)) throw new TypeError(`cell ${row + 2}:${column + 1} is not numeric`);
      return value;
    }));
  });
  return Object.freeze({
    id: datasetId(text),
    name: name.trim() || "Uploaded dataset",
    columns: Object.freeze(columns),
    rows: Object.freeze(rows),
    source: "upload",
  });
}

export class DatasetPicker extends EventTarget {
  #items = new Map<string, PlaygroundDataset>();
  #selected: string | undefined;

  add(dataset: PlaygroundDataset): void {
    if (!dataset.id.trim() || !dataset.name.trim() || this.#items.has(dataset.id)) throw new RangeError(`invalid or duplicate dataset ${dataset.id}`);
    this.#items.set(dataset.id, Object.freeze(dataset));
    this.dispatchEvent(new CustomEvent("change", { detail: this.items }));
  }

  remove(id: string): boolean {
    const removed = this.#items.delete(id);
    if (this.#selected === id) this.#selected = undefined;
    if (removed) this.dispatchEvent(new CustomEvent("change", { detail: this.items }));
    return removed;
  }

  select(id: string): void {
    if (!this.#items.has(id)) throw new RangeError(`unknown dataset ${id}`);
    this.#selected = id;
    this.dispatchEvent(new CustomEvent("select", { detail: this.selected }));
  }

  get selected(): PlaygroundDataset | undefined { return this.#selected === undefined ? undefined : this.#items.get(this.#selected); }
  get items(): readonly PlaygroundDataset[] { return Object.freeze([...this.#items.values()].sort((left, right) => left.name.localeCompare(right.name))); }
}
