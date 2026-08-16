import type { WorldDefinition } from "@lawsynth/world-schema";
export interface WorldChoice {
    readonly id: string;
    readonly name: string;
    readonly description?: string;
    readonly world: WorldDefinition;
    readonly source: "example" | "local" | "shared";
}
export class WorldPicker extends EventTarget {
    #choices = new Map<string, WorldChoice>();
    #selected: string | undefined;
    add(choice: WorldChoice): void {
      if (!choice.id.trim() || !choice.name.trim() || this.#choices.has(choice.id)) {
        throw new RangeError(`world choice id is invalid or duplicated: ${choice.id}`);
      }
      this.#choices.set(choice.id, Object.freeze(choice));
      this.#emit();
    }
    remove(id: string): boolean {
      const removed = this.#choices.delete(id);
      if (this.#selected === id) this.#selected = undefined;
      if (removed) this.#emit();
      return removed;
    }
    select(id: string): WorldChoice {
      const choice = this.#choices.get(id);
      if (choice === undefined) throw new RangeError(`unknown world choice: ${id}`);
      this.#selected = id;
      this.dispatchEvent(new CustomEvent("select", { detail: choice }));
      return choice;
    }
    get selected(): WorldChoice | undefined { return this.#selected === undefined ? undefined : this.#choices.get(this.#selected); }
    get choices(): readonly WorldChoice[] { return Object.freeze([...this.#choices.values()].sort((a, b) => a.name.localeCompare(b.name))); }
    clear(): void {
      if (this.#choices.size === 0) return;
      this.#choices.clear();
      this.#selected = undefined;
      this.#emit();
    }
    #emit(): void { this.dispatchEvent(new CustomEvent("change", { detail: this.choices })); }
}
