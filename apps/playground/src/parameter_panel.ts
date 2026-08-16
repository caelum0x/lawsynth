import type { WorldDefinition } from "@lawsynth/world-schema";
import { parametersForWorld, validateParameterOverrides, type ParameterRow } from "@lawsynth/world-viewer";
export interface ParameterPanelSnapshot {
    readonly rows: readonly ParameterRow[];
    readonly overrides: Readonly<Record<string, number>>;
    readonly changed: number;
}
export class ParameterPanel extends EventTarget {
    #world: WorldDefinition;
    #overrides: Readonly<Record<string, number>> = Object.freeze({});
    constructor(world: WorldDefinition) { super(); this.#world = world; }
    get snapshot(): ParameterPanelSnapshot {
      return Object.freeze({ rows: parametersForWorld(this.#world), overrides: this.#overrides, changed: Object.keys(this.#overrides).length });
    }
    set(id: string, value: number): void {
      const next = Object.entries(this.#overrides).filter(([key]) => key !== id).map(([key, current]) => ({ id: key, value: current }));
      next.push({ id, value });
      this.#overrides = validateParameterOverrides(this.#world, next);
      this.#emit();
    }
    reset(id?: string): void { if (id === undefined)
        this.#overrides = Object.freeze({});
    else {
        const { [id]: _removed, ...rest } = this.#overrides;
        this.#overrides = Object.freeze(rest);
    } this.#emit(); }
    replaceWorld(world: WorldDefinition, preserve = false): void {
      this.#world = world;
      this.#overrides = preserve ? validateParameterOverrides(world, Object.entries(this.#overrides).map(([id, value]) => ({ id, value }))) : Object.freeze({});
      this.#emit();
    }
    values(): Readonly<Record<string, number>> {
      return Object.freeze(Object.fromEntries((this.#world.parameters ?? []).map((parameter) => [parameter.id, this.#overrides[parameter.id] ?? parameter.value])));
    }
    #emit(): void { this.dispatchEvent(new CustomEvent("change", { detail: this.snapshot })); }
}
