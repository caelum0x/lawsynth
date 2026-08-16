import type { WorldDefinition } from "@lawsynth/world-schema";
import { PlaygroundError } from "./errors.js";

export interface SavedPlayground {
  readonly version: 1;
  readonly id: string;
  readonly name: string;
  readonly updatedAt: string;
  readonly world: WorldDefinition;
  readonly parameters: Readonly<Record<string, number>>;
}

export interface SavedPlaygroundSummary { readonly id: string; readonly name: string; readonly updatedAt: string; }
export interface KeyValueStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
  key(index: number): string | null;
  readonly length: number;
}

function validateSaved(value: SavedPlayground, expectedId?: string): SavedPlayground {
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$/u.test(value.id) || (expectedId !== undefined && value.id !== expectedId)) throw new TypeError("saved playground id is invalid");
  if (!value.name.trim() || !Number.isFinite(Date.parse(value.updatedAt))) throw new TypeError("saved playground metadata is invalid");
  if (value.version !== 1 || typeof value.world !== "object" || value.world === null) throw new TypeError("saved playground version or world is invalid");
  if (Object.values(value.parameters).some((parameter) => !Number.isFinite(parameter))) throw new TypeError("saved parameters must be finite");
  return value;
}

export class PlaygroundStorage {
  constructor(
    readonly storage: KeyValueStorage,
    readonly prefix = "lawsynth:playground:",
    readonly maximumBytes = 4 * 1024 * 1024,
  ) {
    if (!prefix || /[\r\n\0]/u.test(prefix)) throw new RangeError("storage prefix is invalid");
    if (!Number.isSafeInteger(maximumBytes) || maximumBytes < 1024) throw new RangeError("maximumBytes must be at least 1 KiB");
  }

  save(value: SavedPlayground): void {
    validateSaved(value);
    const serialized = JSON.stringify(value);
    if (new TextEncoder().encode(serialized).byteLength > this.maximumBytes) throw new PlaygroundError("limit-exceeded", "saved playground is too large");
    try { this.storage.setItem(this.prefix + value.id, serialized); }
    catch (error) { throw new PlaygroundError("storage-failed", "browser storage rejected the save", error); }
  }

  load(id: string): SavedPlayground | undefined {
    let raw: string | null;
    try { raw = this.storage.getItem(this.prefix + id); }
    catch (error) { throw new PlaygroundError("storage-failed", "browser storage could not be read", error); }
    if (raw === null) return undefined;
    try { return validateSaved(JSON.parse(raw) as SavedPlayground, id); }
    catch (error) { throw new PlaygroundError("storage-failed", "saved playground is corrupt", error); }
  }

  remove(id: string): void {
    try { this.storage.removeItem(this.prefix + id); }
    catch (error) { throw new PlaygroundError("storage-failed", "saved playground could not be removed", error); }
  }

  list(): readonly SavedPlaygroundSummary[] {
    const result: SavedPlaygroundSummary[] = [];
    for (let index = 0; index < this.storage.length; index += 1) {
      const key = this.storage.key(index);
      if (!key?.startsWith(this.prefix)) continue;
      const value = this.load(key.slice(this.prefix.length));
      if (value !== undefined) result.push({ id: value.id, name: value.name, updatedAt: value.updatedAt });
    }
    return Object.freeze(result.sort((left, right) => right.updatedAt.localeCompare(left.updatedAt)));
  }
}
