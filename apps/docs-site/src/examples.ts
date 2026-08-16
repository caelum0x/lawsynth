export interface DocumentedExample {
  readonly id: string; readonly title: string; readonly description: string;
  readonly language: "python" | "rust" | "typescript" | "bash"; readonly source: string;
  readonly sourcePath?: string; readonly runnable: boolean; readonly capabilities: readonly string[];
}
export class ExampleRegistry {
  #examples = new Map<string, DocumentedExample>();
  add(example: DocumentedExample): void {
    if (!/^[a-z0-9][a-z0-9-]{1,100}$/u.test(example.id) || this.#examples.has(example.id) || !example.source.trim()) {
      throw new RangeError(`invalid or duplicate documented example: ${example.id}`);
    }
    if (!example.title.trim() || !example.description.trim()) throw new RangeError("example title and description are required");
    this.#examples.set(example.id, Object.freeze({ ...example, capabilities: Object.freeze([...new Set(example.capabilities)].sort()) }));
  }
  get(id: string): DocumentedExample | undefined { return this.#examples.get(id); }
  list(capability?: string): readonly DocumentedExample[] {
    return Object.freeze([...this.#examples.values()].filter((example) => capability === undefined || example.capabilities.includes(capability)).sort((a, b) => a.title.localeCompare(b.title)));
  }
}
