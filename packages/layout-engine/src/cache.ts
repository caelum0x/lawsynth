export interface CacheStats { readonly hits: number; readonly misses: number; readonly evictions: number; readonly size: number; }
export class LayoutCache<K, V> {
  readonly #values = new Map<K, V>(); #hits=0; #misses=0; #evictions=0;
  constructor(readonly capacity = 128) { if (!Number.isInteger(capacity) || capacity < 1) throw new RangeError("cache capacity must be a positive integer"); }
  get(key: K): V | undefined { const value=this.#values.get(key); if (value === undefined) { this.#misses++; return undefined; } this.#hits++; this.#values.delete(key); this.#values.set(key,value); return value; }
  set(key: K, value: V): this { if (this.#values.has(key)) this.#values.delete(key); this.#values.set(key,value); if (this.#values.size > this.capacity) { this.#values.delete(this.#values.keys().next().value as K); this.#evictions++; } return this; }
  getOrCompute(key: K, compute: () => V): V { const existing=this.get(key); if (existing !== undefined) return existing; const value=compute(); this.set(key,value); return value; }
  clear(): void { this.#values.clear(); }
  get stats(): CacheStats { return { hits:this.#hits, misses:this.#misses, evictions:this.#evictions, size:this.#values.size }; }
}
