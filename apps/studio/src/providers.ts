import type { LawSynthClient } from "@lawsynth/api-client";
import type { PersistenceAdapter, StateStore } from "@lawsynth/state-store";

export interface StudioLogger {
  debug(message: string, context?: Readonly<Record<string, unknown>>): void;
  info(message: string, context?: Readonly<Record<string, unknown>>): void;
  error(message: string, context?: Readonly<Record<string, unknown>>): void;
}

export interface StudioNotification {
  readonly id: string;
  readonly tone: "info" | "success" | "warning" | "error";
  readonly title: string;
  readonly detail?: string;
  readonly action?: { readonly label: string; readonly command: string };
}

export interface StudioProviders {
  readonly api: LawSynthClient;
  readonly store: StateStore;
  readonly persistence: PersistenceAdapter;
  readonly logger: StudioLogger;
  readonly notify: (notification: StudioNotification) => void;
  readonly clock: () => number;
  readonly randomId: () => string;
}

export type ProviderFactory = () => StudioProviders | Promise<StudioProviders>;

export class ProviderScope {
  #providers: StudioProviders | undefined;
  #pending: Promise<StudioProviders> | undefined;
  #disposed = false;
  readonly #disposers: (() => void | Promise<void>)[] = [];

  constructor(readonly factory: ProviderFactory) {}

  async get(): Promise<StudioProviders> {
    if (this.#disposed) throw new Error("Studio provider scope is disposed");
    if (this.#providers !== undefined) return this.#providers;
    this.#pending ??= Promise.resolve(this.factory()).then((providers) => {
      if (this.#disposed) throw new Error("Studio provider scope was disposed during initialization");
      this.#providers = providers;
      return providers;
    }).finally(() => { this.#pending = undefined; });
    return this.#pending;
  }

  addDisposer(disposer: () => void | Promise<void>): () => void {
    if (this.#disposed) throw new Error("cannot register a disposer after disposal");
    this.#disposers.push(disposer);
    return () => { const index = this.#disposers.indexOf(disposer); if (index >= 0) this.#disposers.splice(index, 1); };
  }

  async dispose(): Promise<void> {
    if (this.#disposed) return;
    this.#disposed = true;
    const errors: unknown[] = [];
    for (const disposer of [...this.#disposers].reverse()) try { await disposer(); } catch (error) { errors.push(error); }
    this.#disposers.length = 0;
    this.#providers = undefined;
    if (errors.length > 0) throw new AggregateError(errors, "one or more Studio providers failed to dispose");
  }
}
