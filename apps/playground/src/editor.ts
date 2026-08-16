import { validateWorld, type ValidationIssue, type WorldDefinition } from "@lawsynth/world-schema";
export interface EditorDiagnostic {
    readonly line?: number;
    readonly column?: number;
    readonly path?: string;
    readonly severity: "error" | "warning";
    readonly message: string;
}
export interface EditorSnapshot {
    readonly text: string;
    readonly revision: number;
    readonly dirty: boolean;
    readonly world?: WorldDefinition;
    readonly diagnostics: readonly EditorDiagnostic[];
}
function jsonLocation(text: string, error: SyntaxError): Pick<EditorDiagnostic, "line" | "column"> {
    const match = /position\s+(\d+)/iu.exec(error.message);
    const offset = match?.[1] === undefined ? 0 : Number(match[1]);
    const before = text.slice(0, offset).split("\n");
    return { line: before.length, column: (before.at(-1)?.length ?? 0) + 1 };
}
export function parseWorldText(text: string, maximumBytes = 4 * 1024 * 1024): {
    readonly world?: WorldDefinition;
    readonly diagnostics: readonly EditorDiagnostic[];
} {
    if (new TextEncoder().encode(text).byteLength > maximumBytes) {
      return { diagnostics: [{ severity: "error", message: `World source exceeds ${maximumBytes} bytes` }] };
    }
    let parsed: unknown;
    try {
        parsed = JSON.parse(text) as unknown;
    }
    catch (error) {
        const syntax = error as SyntaxError;
        return { diagnostics: [{ severity: "error", message: syntax.message, ...jsonLocation(text, syntax) }] };
    }
    const result = validateWorld(parsed);
    if (!result.ok) {
      return { diagnostics: result.issues.map((issue: ValidationIssue) => ({
        path: issue.path,
        severity: issue.code === "unsupported" ? "warning" : "error",
        message: issue.message,
      })) };
    }
    return { world: result.value, diagnostics: [] };
}
export class WorldEditor extends EventTarget {
    #snapshot: EditorSnapshot;
    #timer: ReturnType<typeof setTimeout> | undefined;
    constructor(initial = "{\n  \"formatVersion\": \"0.1.0\"\n}\n", readonly debounceMs = 250) {
        super();
        if (!Number.isFinite(debounceMs) || debounceMs < 0)
            throw new RangeError("debounceMs must be non-negative");
        this.#snapshot = Object.freeze({ text: initial, revision: 0, dirty: false, diagnostics: [] });
    }
    get snapshot(): EditorSnapshot { return this.#snapshot; }
    update(text: string): void {
      this.#snapshot = Object.freeze({ text, revision: this.#snapshot.revision + 1, dirty: true, diagnostics: this.#snapshot.diagnostics });
      this.#schedule();
      this.#emit();
    }
    validate(): EditorSnapshot {
      if (this.#timer !== undefined) {
        clearTimeout(this.#timer);
        this.#timer = undefined;
      }
      const result = parseWorldText(this.#snapshot.text);
      this.#snapshot = Object.freeze({ ...this.#snapshot, ...result });
      this.#emit();
      return this.#snapshot;
    }
    load(world: WorldDefinition): void {
      const text = JSON.stringify(world, null, 2) + "\n";
      this.#snapshot = Object.freeze({ text, revision: this.#snapshot.revision + 1, dirty: false, world, diagnostics: [] });
      this.#emit();
    }
    markSaved(): void {
      if (!this.#snapshot.dirty) return;
      this.#snapshot = Object.freeze({ ...this.#snapshot, dirty: false });
      this.#emit();
    }
    dispose(): void {
      if (this.#timer !== undefined) clearTimeout(this.#timer);
      this.#timer = undefined;
    }
    #schedule(): void {
      if (this.#timer !== undefined) clearTimeout(this.#timer);
      this.#timer = setTimeout(() => this.validate(), this.debounceMs);
    }
    #emit(): void { this.dispatchEvent(new CustomEvent("change", { detail: this.#snapshot })); }
}
