import type { WorldDefinition } from "@lawsynth/world-schema";
import type { PlaygroundDataset } from "./dataset_picker.js";
export interface PlaygroundExample {
    readonly id: string;
    readonly title: string;
    readonly summary: string;
    readonly category: "dynamics" | "ecology" | "epidemiology" | "operations";
    readonly world: WorldDefinition;
    readonly dataset?: PlaygroundDataset;
    readonly featured?: boolean;
}
export class ExampleCatalog {
  #entries = new Map<string, PlaygroundExample>();

  constructor(entries: readonly PlaygroundExample[] = []) {
    entries.forEach((entry) => this.add(entry));
  }

  add(entry: PlaygroundExample): void {
    if (!/^[a-z0-9][a-z0-9-]{1,63}$/u.test(entry.id) || this.#entries.has(entry.id)) {
      throw new RangeError(`invalid or duplicate example id: ${entry.id}`);
    }
    if (!entry.title.trim() || !entry.summary.trim()) throw new RangeError("example title and summary are required");
    this.#entries.set(entry.id, Object.freeze(entry));
  }

  get(id: string): PlaygroundExample | undefined { return this.#entries.get(id); }

  list(category?: PlaygroundExample["category"]): readonly PlaygroundExample[] {
    return Object.freeze([...this.#entries.values()]
      .filter((entry) => category === undefined || entry.category === category)
      .sort((a, b) => Number(Boolean(b.featured)) - Number(Boolean(a.featured)) || a.title.localeCompare(b.title)));
  }

  search(query: string): readonly PlaygroundExample[] {
    const normalized = query.trim().toLocaleLowerCase();
    if (!normalized) return this.list();
    return Object.freeze(this.list().filter((entry) =>
      entry.title.toLocaleLowerCase().includes(normalized) || entry.summary.toLocaleLowerCase().includes(normalized)));
  }
}
