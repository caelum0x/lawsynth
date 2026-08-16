import { decodeViewerBundle, DEFAULT_MAX_BUNDLE_BYTES, type ViewerBundle } from "./bundle.js";
import type { ViewerPanel } from "./layout.js";
import type { ViewerThemeName } from "./theme.js";
import { WorldViewer, type WorldViewerOptions } from "./viewer.js";

export const WORLD_VIEWER_TAG = "lawsynth-world-viewer";

export interface LoadBundleOptions {
  readonly signal?: AbortSignal;
  readonly timeoutMs?: number;
  readonly maxBytes?: number;
  readonly fetch?: typeof globalThis.fetch;
  readonly credentials?: RequestCredentials;
}

export interface EmbeddedViewer {
  readonly viewer: WorldViewer;
  readonly host: HTMLElement;
  destroy(): void;
}

const PANELS = new Set<ViewerPanel>(["overview", "equations", "graph", "trajectory", "parameters", "uncertainty", "regimes", "provenance"]);

function parsePanel(value: string | null): ViewerPanel | undefined {
  return value !== null && PANELS.has(value as ViewerPanel) ? value as ViewerPanel : undefined;
}

function parseTheme(value: string | null): ViewerThemeName | undefined {
  return value === "paper" || value === "midnight" ? value : undefined;
}

function combineAbortSignals(signals: readonly (AbortSignal | undefined)[]): { readonly signal: AbortSignal; readonly cleanup: () => void } {
  const controller = new AbortController();
  const cleanups: (() => void)[] = [];
  for (const signal of signals) {
    if (signal === undefined) continue;
    if (signal.aborted) controller.abort(signal.reason);
    else {
      const listener = (): void => controller.abort(signal.reason);
      signal.addEventListener("abort", listener, { once: true });
      cleanups.push(() => signal.removeEventListener("abort", listener));
    }
  }
  return { signal: controller.signal, cleanup: () => cleanups.forEach((cleanup) => cleanup()) };
}

/** Fetches a JSON viewer envelope with response-size and time limits. */
export async function loadViewerBundle(url: string | URL, options: LoadBundleOptions = {}): Promise<ViewerBundle> {
  const fetcher = options.fetch ?? globalThis.fetch;
  if (fetcher === undefined) throw new Error("loading a remote viewer bundle requires fetch");
  const timeoutMs = options.timeoutMs ?? 20_000;
  const maxBytes = options.maxBytes ?? DEFAULT_MAX_BUNDLE_BYTES;
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) throw new RangeError("timeoutMs must be positive");
  if (!Number.isSafeInteger(maxBytes) || maxBytes <= 0) throw new RangeError("maxBytes must be a positive safe integer");
  const timeoutController = new AbortController();
  const timeout = setTimeout(() => timeoutController.abort(new DOMException("viewer bundle request timed out", "TimeoutError")), timeoutMs);
  const combined = combineAbortSignals([options.signal, timeoutController.signal]);
  try {
    const response = await fetcher(url, { method: "GET", headers: { Accept: "application/vnd.lawsynth.viewer+json, application/json;q=0.9" }, credentials: options.credentials ?? "same-origin", signal: combined.signal });
    if (!response.ok) throw new Error(`viewer bundle request failed with HTTP ${response.status}`);
    const declaredLength = Number(response.headers.get("content-length"));
    if (Number.isFinite(declaredLength) && declaredLength > maxBytes) throw new RangeError(`viewer bundle exceeds ${maxBytes} bytes`);
    if (response.body === null) return decodeViewerBundle(await response.arrayBuffer(), { maxBytes });
    const reader = response.body.getReader();
    const chunks: Uint8Array[] = [];
    let length = 0;
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        if (value === undefined) continue;
        length += value.byteLength;
        if (length > maxBytes) {
          await reader.cancel("viewer bundle size limit exceeded");
          throw new RangeError(`viewer bundle exceeds ${maxBytes} bytes`);
        }
        chunks.push(value);
      }
    } finally { reader.releaseLock(); }
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return decodeViewerBundle(bytes, { maxBytes });
  } finally {
    clearTimeout(timeout);
    combined.cleanup();
  }
}

export function embedWorldViewer(host: HTMLElement, options: WorldViewerOptions = {}): EmbeddedViewer {
  const viewer = new WorldViewer(options).mount(host);
  let destroyed = false;
  return Object.freeze({
    viewer,
    host,
    destroy(): void {
      if (destroyed) return;
      destroyed = true;
      viewer.destroy();
    },
  });
}

export interface DefineViewerElementOptions {
  readonly tagName?: `${string}-${string}`;
  readonly registry?: CustomElementRegistry;
  readonly fetch?: typeof globalThis.fetch;
  readonly maxBytes?: number;
}

/** Registers the declarative `<lawsynth-world-viewer>` integration exactly once. */
export function defineWorldViewerElement(options: DefineViewerElementOptions = {}): CustomElementConstructor {
  const tagName = options.tagName ?? WORLD_VIEWER_TAG;
  const registry = options.registry ?? globalThis.customElements;
  if (registry === undefined) throw new Error("custom elements are unavailable in this environment");
  const existing = registry.get(tagName);
  if (existing !== undefined) return existing;
  const HTMLElementBase = globalThis.HTMLElement;
  if (HTMLElementBase === undefined) throw new Error("HTMLElement is unavailable in this environment");

  class LawSynthWorldViewerElement extends HTMLElementBase {
    static get observedAttributes(): string[] { return ["src", "panel", "theme"]; }
    #viewer: WorldViewer | undefined;
    #loadController: AbortController | undefined;
    #loadSequence = 0;

    connectedCallback(): void {
      if (this.#viewer === undefined) {
        const panel = parsePanel(this.getAttribute("panel"));
        const theme = parseTheme(this.getAttribute("theme"));
        this.#viewer = new WorldViewer({ ...(panel === undefined ? {} : { panel }), ...(theme === undefined ? {} : { theme }) }).mount(this);
      }
      void this.#loadSource();
    }

    disconnectedCallback(): void {
      this.#loadController?.abort(new DOMException("viewer element disconnected", "AbortError"));
    }

    attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null): void {
      if (oldValue === newValue || this.#viewer === undefined) return;
      if (name === "src") void this.#loadSource();
      else if (name === "panel") {
        const panel = parsePanel(newValue);
        if (panel !== undefined) {
          try { this.#viewer.setPanel(panel); } catch { /* panel may become available after src loads */ }
        }
      } else if (name === "theme") {
        const theme = parseTheme(newValue);
        if (theme !== undefined) this.#viewer.setTheme(theme);
      }
    }

    set bundle(value: ViewerBundle) { this.#viewer?.setBundle(value); }
    get viewer(): WorldViewer | undefined { return this.#viewer; }

    async #loadSource(): Promise<void> {
      const source = this.getAttribute("src");
      if (source === null || !this.isConnected || this.#viewer === undefined) return;
      const sequence = ++this.#loadSequence;
      this.#loadController?.abort(new DOMException("viewer source changed", "AbortError"));
      const controller = new AbortController();
      this.#loadController = controller;
      this.setAttribute("aria-busy", "true");
      this.#viewer.setLoading();
      try {
        const bundle = await loadViewerBundle(new URL(source, this.ownerDocument.baseURI), {
          signal: controller.signal,
          ...(options.fetch === undefined ? {} : { fetch: options.fetch }),
          ...(options.maxBytes === undefined ? {} : { maxBytes: options.maxBytes }),
        });
        if (sequence !== this.#loadSequence || controller.signal.aborted) return;
        this.#viewer.setBundle(bundle);
        const requestedPanel = parsePanel(this.getAttribute("panel"));
        if (requestedPanel !== undefined) {
          try { this.#viewer.setPanel(requestedPanel); } catch { /* retain overview when data is absent */ }
        }
        this.dispatchEvent(new CustomEvent("lawsynth-load", { detail: { worldId: bundle.world.id } }));
      } catch (error) {
        if (!controller.signal.aborted) {
          this.#viewer.setError(error);
          this.dispatchEvent(new CustomEvent("lawsynth-error", { detail: { error }, bubbles: true, composed: true }));
        }
      } finally {
        if (sequence === this.#loadSequence) {
          this.removeAttribute("aria-busy");
          if (this.#loadController === controller) this.#loadController = undefined;
        }
      }
    }
  }
  registry.define(tagName, LawSynthWorldViewerElement);
  return LawSynthWorldViewerElement;
}
