/** An async coordinator suitable for browser workers or server queues; it owns no DOM APIs. */
export class LayoutCoordinator<I, O> {
  #sequence = 0; #cancelled = new Set<number>();
  async run(input: I, operation: (input: I, signal: { readonly cancelled: () => boolean }) => O | Promise<O>): Promise<{ readonly id: number; readonly value: O }> { const id=++this.#sequence; const value=await operation(input,{cancelled:()=>this.#cancelled.has(id)}); if (this.#cancelled.delete(id)) throw new Error(`layout request ${id} was cancelled`); return {id,value}; }
  cancel(id: number): boolean { if (!Number.isInteger(id) || id < 1 || id > this.#sequence) return false; this.#cancelled.add(id); return true; }
}
