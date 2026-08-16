export type ShortcutScope = "global" | "workspace" | "editor" | "dialog";

export interface ShortcutBinding {
  readonly id: string;
  readonly keys: string;
  readonly label: string;
  readonly scope: ShortcutScope;
  readonly allowInInput?: boolean;
  readonly run: () => void | Promise<void>;
}

function normalizePart(part: string): string {
  const value = part.trim().toLowerCase();
  return ({ command: "meta", cmd: "meta", control: "ctrl", option: "alt", escape: "esc", " ": "space" } as Record<string, string>)[value] ?? value;
}

export function normalizeShortcut(keys: string): string {
  const parts = keys.split("+").map(normalizePart).filter(Boolean);
  const key = parts.at(-1);
  if (key === undefined) throw new RangeError("shortcut cannot be empty");
  const modifiers = new Set(parts.slice(0, -1));
  const invalid = [...modifiers].filter((part) => !["meta", "ctrl", "alt", "shift"].includes(part));
  if (invalid.length > 0) throw new RangeError(`unknown shortcut modifiers: ${invalid.join(", ")}`);
  return [...["meta", "ctrl", "alt", "shift"].filter((part) => modifiers.has(part)), key].join("+");
}

export function shortcutFromEvent(event: KeyboardEvent): string {
  const key = normalizePart(event.key.length === 1 ? event.key : event.key);
  return [event.metaKey ? "meta" : "", event.ctrlKey ? "ctrl" : "", event.altKey ? "alt" : "", event.shiftKey ? "shift" : "", key]
    .filter(Boolean).join("+");
}

function isEditingTarget(target: EventTarget | null): boolean {
  if (target === null || typeof target !== "object" || !("closest" in target) || typeof target.closest !== "function") return false;
  const element = target as Element;
  return element.matches("input,textarea,select,[contenteditable=true]") || element.closest("[contenteditable=true]") !== null;
}

export class ShortcutRegistry {
  #bindings = new Map<string, ShortcutBinding>();
  #scope: ShortcutScope = "global";
  #attached: Document | undefined;
  readonly #listener = (event: KeyboardEvent): void => { void this.handle(event); };

  setScope(scope: ShortcutScope): void { this.#scope = scope; }

  register(binding: ShortcutBinding): () => void {
    if (!binding.id.trim() || !binding.label.trim()) throw new RangeError("shortcut id and label are required");
    const normalized = normalizeShortcut(binding.keys);
    const key = `${binding.scope}:${normalized}`;
    if (this.#bindings.has(key)) throw new RangeError(`shortcut already registered: ${key}`);
    this.#bindings.set(key, Object.freeze({ ...binding, keys: normalized }));
    return () => { this.#bindings.delete(key); };
  }

  async handle(event: KeyboardEvent): Promise<boolean> {
    if (event.defaultPrevented || event.repeat || event.isComposing) return false;
    const chord = shortcutFromEvent(event);
    const binding = this.#bindings.get(`${this.#scope}:${chord}`) ?? this.#bindings.get(`global:${chord}`);
    if (binding === undefined || (isEditingTarget(event.target) && binding.allowInInput !== true)) return false;
    event.preventDefault();
    await binding.run();
    return true;
  }

  attach(document: Document): void {
    if (this.#attached === document) return;
    if (this.#attached !== undefined) throw new Error("shortcut registry is already attached");
    this.#attached = document;
    document.addEventListener("keydown", this.#listener);
  }

  detach(): void {
    this.#attached?.removeEventListener("keydown", this.#listener);
    this.#attached = undefined;
  }

  list(scope?: ShortcutScope): readonly Omit<ShortcutBinding, "run">[] {
    return [...this.#bindings.values()].filter((binding) => scope === undefined || binding.scope === scope)
      .map(({ run: _run, ...binding }) => binding).sort((left, right) => left.label.localeCompare(right.label));
  }
}
