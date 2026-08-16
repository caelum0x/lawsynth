import type { TrajectoryInput } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { createViewerBundle, WorldViewer } from "@lawsynth/world-viewer";
import { WorldEditor } from "./editor.js";
import { userErrorMessage, normalizePlaygroundError } from "./errors.js";
import { ParameterPanel } from "./parameter_panel.js";
import { LocalSimulation, type LocalSimulationSnapshot } from "./simulation.js";
import { applyPlaygroundTheme, type PlaygroundTheme } from "./theme.js";
import { WorldPicker, type WorldChoice } from "./world_picker.js";

export interface PlaygroundSnapshot {
  readonly world?: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly simulation: LocalSimulationSnapshot;
  readonly dirty: boolean;
  readonly error?: Error;
}

export interface SimulationRange {
  readonly start: number;
  readonly end: number;
  readonly step: number;
}

export class PlaygroundController extends EventTarget {
  readonly editor: WorldEditor;
  readonly worlds = new WorldPicker();
  #parameters: ParameterPanel | undefined;
  #trajectory: TrajectoryInput | undefined;
  #error: Error | undefined;
  #parameterListener: (() => void) | undefined;

  constructor(readonly simulation: LocalSimulation, initial?: WorldDefinition) {
    super();
    this.editor = new WorldEditor();
    this.editor.addEventListener("change", () => this.#emit());
    this.worlds.addEventListener("select", (event) => this.load((event as CustomEvent<WorldChoice>).detail.world));
    this.simulation.addEventListener("change", () => this.#emit());
    if (initial !== undefined) this.load(initial);
  }

  get parameters(): ParameterPanel | undefined { return this.#parameters; }

  get snapshot(): PlaygroundSnapshot {
    return Object.freeze({
      ...(this.editor.snapshot.world === undefined ? {} : { world: this.editor.snapshot.world }),
      ...(this.#trajectory === undefined ? {} : { trajectory: this.#trajectory }),
      simulation: this.simulation.snapshot,
      dirty: this.editor.snapshot.dirty,
      ...(this.#error === undefined ? {} : { error: this.#error }),
    });
  }

  load(world: WorldDefinition): void {
    this.#parameterListener?.();
    this.editor.load(world);
    this.#parameters = new ParameterPanel(world);
    const changed = (): void => this.#emit();
    this.#parameters.addEventListener("change", changed);
    this.#parameterListener = () => this.#parameters?.removeEventListener("change", changed);
    this.#trajectory = undefined;
    this.#error = undefined;
    this.#emit();
  }

  async run(range: SimulationRange, initial: Readonly<Record<string, number>>): Promise<TrajectoryInput> {
    const world = this.editor.validate().world;
    if (world === undefined) throw new Error("fix world diagnostics before simulation");
    try {
      const parameters = this.#parameters?.values();
      const trajectory = await this.simulation.run(world, { ...range, initial, ...(parameters === undefined ? {} : { parameters }) });
      this.#trajectory = trajectory;
      this.#error = undefined;
      this.#emit();
      return trajectory;
    } catch (error) {
      this.#error = error instanceof Error ? error : new Error(String(error));
      this.#emit();
      throw this.#error;
    }
  }

  dispose(): void {
    this.simulation.cancel();
    this.editor.dispose();
    this.#parameterListener?.();
    this.#parameterListener = undefined;
  }

  #emit(): void { this.dispatchEvent(new CustomEvent("change", { detail: this.snapshot })); }
}

const PLAYGROUND_CSS = `.lsp{min-height:480px;background:var(--lsp-canvas);color:var(--lsp-ink);font:14px/1.5 Inter,system-ui,sans-serif;border:1px solid var(--lsp-line);display:grid;grid-template-rows:auto 1fr}.lsp *{box-sizing:border-box}.lsp-head{display:flex;align-items:center;padding:13px 18px;background:var(--lsp-surface);border-bottom:1px solid var(--lsp-line);gap:16px}.lsp-head h1{font:650 19px/1 Georgia,serif;margin:0}.lsp-kicker{font:600 10px/1 ui-monospace,monospace;letter-spacing:.1em;text-transform:uppercase;color:var(--lsp-muted)}.lsp-shell{display:grid;grid-template-columns:minmax(320px,46%) minmax(0,1fr);min-height:0}.lsp-editor,.lsp-output{padding:18px;min-width:0}.lsp-editor{border-right:1px solid var(--lsp-line);background:var(--lsp-surface)}.lsp textarea{width:100%;min-height:320px;resize:vertical;padding:14px;border:1px solid var(--lsp-line);background:var(--lsp-canvas);color:var(--lsp-ink);font:13px/1.55 ui-monospace,SFMono-Regular,monospace}.lsp-controls{display:grid;grid-template-columns:repeat(3,1fr) auto;gap:8px;margin-top:12px}.lsp label{font:600 11px/1.2 ui-monospace,monospace;text-transform:uppercase}.lsp input{width:100%;min-height:42px;border:1px solid var(--lsp-line);background:var(--lsp-surface);color:var(--lsp-ink);padding:8px}.lsp button{min-height:44px;border:0;background:var(--lsp-ink);color:var(--lsp-surface);padding:9px 15px;font-weight:650;cursor:pointer}.lsp button:focus-visible,.lsp input:focus-visible,.lsp textarea:focus-visible{outline:3px solid var(--lsp-accent);outline-offset:2px}.lsp-alert{margin:12px 0;padding:10px 12px;border-left:4px solid var(--lsp-danger);background:var(--lsp-canvas)}.lsp-empty{display:grid;place-content:center;min-height:320px;color:var(--lsp-muted);text-align:center}@media(max-width:760px){.lsp-shell{display:block}.lsp-editor{border-right:0;border-bottom:1px solid var(--lsp-line)}.lsp-controls{grid-template-columns:repeat(3,1fr)}.lsp-controls button{grid-column:1/-1}}`;

export interface PlaygroundAppOptions {
  readonly theme?: PlaygroundTheme;
  readonly initialValues?: Readonly<Record<string, number>>;
  readonly range?: SimulationRange;
}

/** Accessible DOM shell around the pure controller. */
export class PlaygroundApp {
  #root: HTMLElement | undefined;
  #style: HTMLStyleElement | undefined;
  #viewer: WorldViewer | undefined;
  readonly #listener = (): void => this.render();

  constructor(readonly controller: PlaygroundController, readonly options: PlaygroundAppOptions = {}) {
    controller.addEventListener("change", this.#listener);
  }

  mount(host: HTMLElement): this {
    if (this.#root !== undefined) throw new Error("PlaygroundApp is already mounted");
    const document = host.ownerDocument;
    this.#style = document.createElement("style");
    this.#style.textContent = PLAYGROUND_CSS;
    this.#root = document.createElement("section");
    this.#root.className = "lsp";
    this.#root.setAttribute("aria-label", "LawSynth Playground");
    applyPlaygroundTheme(this.#root, this.options.theme ?? "paper");
    host.append(this.#style, this.#root);
    this.render();
    return this;
  }

  render(): void {
    if (this.#root === undefined) return;
    const document = this.#root.ownerDocument;
    this.#viewer?.destroy();
    this.#viewer = undefined;
    this.#root.replaceChildren();

    const header = document.createElement("header");
    header.className = "lsp-head";
    const brand = document.createElement("div");
    const kicker = document.createElement("div"); kicker.className = "lsp-kicker"; kicker.textContent = "Local deterministic runtime";
    const title = document.createElement("h1"); title.textContent = "LawSynth Playground";
    brand.append(kicker, title); header.append(brand);

    const shell = document.createElement("div"); shell.className = "lsp-shell";
    shell.append(this.#renderEditor(document), this.#renderOutput(document));
    this.#root.append(header, shell);
  }

  destroy(): void {
    this.controller.removeEventListener("change", this.#listener);
    this.controller.dispose();
    this.#viewer?.destroy();
    this.#root?.remove();
    this.#style?.remove();
    this.#viewer = undefined; this.#root = undefined; this.#style = undefined;
  }

  #renderEditor(document: Document): HTMLElement {
    const panel = document.createElement("section"); panel.className = "lsp-editor";
    const textarea = document.createElement("textarea");
    textarea.setAttribute("aria-label", "World JSON source");
    textarea.spellcheck = false; textarea.value = this.controller.editor.snapshot.text;
    textarea.addEventListener("input", () => this.controller.editor.update(textarea.value));
    panel.append(textarea);
    for (const diagnostic of this.controller.editor.snapshot.diagnostics) {
      const alert = document.createElement("p"); alert.className = "lsp-alert"; alert.setAttribute("role", "alert");
      alert.textContent = `${diagnostic.path ?? diagnostic.line ?? "World"}: ${diagnostic.message}`; panel.append(alert);
    }
    const controls = document.createElement("div"); controls.className = "lsp-controls";
    const defaults = this.options.range ?? { start: 0, end: 10, step: 0.01 };
    const fields = (["start", "end", "step"] as const).map((name) => {
      const wrapper=document.createElement("label"); wrapper.textContent=name; const input=document.createElement("input"); input.type="number"; input.step="any"; input.value=String(defaults[name]); input.dataset.field=name; wrapper.append(input); controls.append(wrapper); return input;
    });
    const run=document.createElement("button"); run.type="button"; run.textContent=this.controller.simulation.snapshot.phase==="running"?"Running…":"Run simulation"; run.disabled=this.controller.simulation.snapshot.phase==="running";
    run.addEventListener("click",()=>{const [start,end,step]=fields.map((field)=>Number(field.value));void this.controller.run({start:start!,end:end!,step:step!},this.options.initialValues??{}).catch(()=>undefined);});
    controls.append(run); panel.append(controls); return panel;
  }

  #renderOutput(document: Document): HTMLElement {
    const output=document.createElement("section"); output.className="lsp-output";
    const snapshot=this.controller.snapshot;
    if(snapshot.error!==undefined){const alert=document.createElement("p");alert.className="lsp-alert";alert.setAttribute("role","alert");alert.textContent=userErrorMessage(normalizePlaygroundError(snapshot.error));output.append(alert);}
    if(snapshot.world===undefined||snapshot.trajectory===undefined){const empty=document.createElement("div");empty.className="lsp-empty";empty.textContent=snapshot.simulation.phase==="running"?"Computing the trajectory locally…":"Run the World to inspect its trajectory.";output.append(empty);return output;}
    const host=document.createElement("div");output.append(host);this.#viewer=new WorldViewer({bundle:createViewerBundle(snapshot.world,snapshot.trajectory),panel:"trajectory",shadow:false}).mount(host);return output;
  }
}
