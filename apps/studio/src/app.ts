import { DiscoveryController } from "./discovery.js";
import { ProviderScope, type StudioProviders } from "./providers.js";
import { StudioRouter, parseRoute, routePath, type StudioRoute } from "./routes.js";
import { ScreensController, renderScreenModel, SCREEN_REGISTRY, type ScreenActions } from "./screens/index.js";
import { DEFAULT_SETTINGS, mergeSettings, parseSettings, type StudioSettings } from "./settings.js";
import { ShortcutRegistry } from "./shortcuts.js";
import { SimulationController } from "./simulation.js";
import { WorkspaceController } from "./workspace.js";

export type StudioPhase = "created" | "starting" | "ready" | "failed" | "stopped";

export interface StudioAppOptions {
  readonly providers: ProviderScope;
  readonly route?: StudioRoute;
  readonly settingsKey?: string;
  readonly document?: Document;
}

export interface StudioAppSnapshot {
  readonly phase: StudioPhase;
  readonly route: StudioRoute;
  readonly settings: StudioSettings;
  readonly error?: Error;
}

function errorValue(error: unknown): Error { return error instanceof Error ? error : new Error(String(error)); }
function node<K extends keyof HTMLElementTagNameMap>(document: Document, tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const value = document.createElement(tag); if (className !== undefined) value.className = className; if (text !== undefined) value.textContent = text; return value;
}

const STUDIO_CSS = `.lss{--ink:#18201d;--paper:#f3f0e8;--surface:#fffdf7;--line:#c8c6ba;--accent:#b54b2a;min-height:100%;color:var(--ink);background:var(--paper);font:14px/1.5 Inter,system-ui,sans-serif;display:grid;grid-template-rows:auto 1fr}.lss *{box-sizing:border-box}.lss-header{min-height:64px;display:flex;align-items:center;padding:12px 20px;border-bottom:1px solid var(--line);background:var(--surface);gap:18px}.lss-wordmark{font:700 19px/1 Georgia,serif}.lss-context{font:600 11px/1 ui-monospace,monospace;text-transform:uppercase;letter-spacing:.08em;color:#59635e}.lss-layout{display:grid;grid-template-columns:230px minmax(0,1fr);min-height:0}.lss-nav{padding:18px;border-right:1px solid var(--line);background:var(--surface)}.lss-nav button{display:block;width:100%;min-height:44px;padding:8px 10px;border:0;background:transparent;text-align:left;color:inherit;font-weight:600;cursor:pointer}.lss-nav button[aria-current=page]{background:var(--ink);color:var(--surface)}.lss-nav button:focus-visible{outline:3px solid var(--accent);outline-offset:2px}.lss-main{padding:28px;overflow:auto}.lss-main h1{margin:0 0 8px;font:650 30px/1.1 Georgia,serif}.lss-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:14px;margin-top:22px}.lss-card{padding:16px;border:1px solid var(--line);background:var(--surface)}.lss-card strong{display:block;font:600 12px/1.2 ui-monospace,monospace;text-transform:uppercase;letter-spacing:.06em;margin-bottom:8px}.lss-status{padding:12px 14px;border-left:4px solid var(--accent);background:var(--surface)}@media(max-width:680px){.lss-layout{display:block}.lss-nav{display:flex;overflow:auto;border-right:0;border-bottom:1px solid var(--line)}.lss-nav button{width:auto;white-space:nowrap}.lss-main{padding:18px}}
.lss-nav-label{margin:16px 0 4px;font:600 10px/1.2 ui-monospace,monospace;text-transform:uppercase;letter-spacing:.08em;color:#8a9089}
.lss-scr-header h1{margin:0 0 4px;font:650 26px/1.1 Georgia,serif}.lss-scr-subtitle{margin:0 0 18px;color:#59635e}
.lss-scr-section{margin:0 0 22px}.lss-scr-heading{margin:0 0 10px;font:600 12px/1.2 ui-monospace,monospace;text-transform:uppercase;letter-spacing:.06em;color:#59635e}
.lss-scr-metrics{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:10px}.lss-scr-metric{padding:12px;border:1px solid var(--line);background:var(--surface);display:flex;flex-direction:column;gap:4px}.lss-scr-metric-label{font:600 10px/1 ui-monospace,monospace;text-transform:uppercase;letter-spacing:.05em;color:#8a9089}.lss-scr-metric-value{font:600 17px/1.1 Georgia,serif}
.lss-scr-controls{display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:12px}.lss-scr-field{display:flex;flex-direction:column;gap:4px;font-size:12px}.lss-scr-field input,.lss-scr-field select{min-height:34px;padding:4px 8px;border:1px solid var(--line);background:var(--surface);font:inherit}.lss-scr-field input[type=checkbox]{min-height:0;width:18px;height:18px}.lss-scr-field-label{font-weight:600}.lss-scr-field-help{color:#8a9089;font-size:11px}
.lss-scr-actions{display:flex;gap:10px;flex-wrap:wrap}.lss-scr-btn{min-height:38px;padding:8px 16px;border:1px solid var(--line);background:var(--surface);font:600 13px/1 inherit;cursor:pointer}.lss-scr-btn:disabled{opacity:.5;cursor:not-allowed}.lss-scr-btn.lss-tone-success{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.lss-scr-table{width:100%;border-collapse:collapse;font-size:13px}.lss-scr-table th,.lss-scr-table td{padding:7px 10px;text-align:left;border-bottom:1px solid var(--line)}.lss-scr-table th{font:600 10px/1 ui-monospace,monospace;text-transform:uppercase;color:#8a9089}.lss-scr-table .lss-end{text-align:right}.lss-scr-table tr.lss-selected{background:#efe7d8}.lss-scr-table tr.lss-emphasis td{font-weight:600}
.lss-scr-chart,.lss-scr-timeline{width:100%;height:auto;border:1px solid var(--line);background:var(--surface)}
.lss-scr-equations{display:flex;flex-direction:column;gap:10px}.lss-scr-equation{border:1px solid var(--line);background:var(--surface);padding:10px 12px}.lss-scr-equation.lss-selected{border-color:var(--accent);border-left-width:4px}.lss-scr-equation.lss-disabled{opacity:.55}.lss-scr-equation-head{display:block;width:100%;text-align:left;border:0;background:transparent;font:600 12px/1.2 ui-monospace,monospace;color:#59635e;cursor:pointer;padding:0 0 6px}.lss-scr-equation-text{display:block;font:15px/1.4 ui-monospace,monospace}.lss-scr-terms{margin:10px 0 0;padding-left:0;list-style:none;display:flex;flex-direction:column;gap:6px}.lss-scr-terms li{display:flex;gap:8px;align-items:baseline;font-size:13px}.lss-scr-term-sign{font-weight:700;color:var(--accent)}.lss-scr-term-symbols{color:#8a9089;font-size:11px}
.lss-scr-notice{padding:10px 12px;border-left:4px solid var(--accent);background:var(--surface);margin:0 0 8px}.lss-tone-warning{border-color:#c58a1e}.lss-tone-error{border-color:#b32d2d}.lss-tone-success{border-color:#2f7d43}.lss-tone-info{border-color:#3768a6}.lss-scr-empty{color:#8a9089;font-style:italic}`;

export class StudioApp extends EventTarget {
  readonly router: StudioRouter;
  readonly shortcuts = new ShortcutRegistry();
  #phase: StudioPhase = "created";
  #settings = DEFAULT_SETTINGS;
  #error: Error | undefined;
  #providers: StudioProviders | undefined;
  #workspace: WorkspaceController | undefined;
  #discovery: DiscoveryController | undefined;
  #simulation: SimulationController | undefined;
  #screens: ScreensController | undefined;
  #host: HTMLElement | undefined;
  #root: HTMLElement | undefined;
  #style: HTMLStyleElement | undefined;
  #unsubscribe: (() => void) | undefined;
  #saveTimer: ReturnType<typeof setTimeout> | undefined;
  readonly #document: Document | undefined;
  readonly #settingsKey: string;
  readonly #popstate = (): void => {
    const location = this.#document?.defaultView?.location;
    if (location === undefined) return;
    try { this.router.navigate(parseRoute(location.href), true); }
    catch (error) { this.#error = errorValue(error); this.#commit("failed"); }
  };

  constructor(readonly options: StudioAppOptions) {
    super();
    this.router = new StudioRouter(options.route ?? { name: "home" });
    this.#document = options.document ?? globalThis.document;
    this.#settingsKey = options.settingsKey ?? "lawsynth:studio:settings:v1";
    this.router.addEventListener("navigate", (event) => this.#onNavigate(event as CustomEvent<{ route: StudioRoute; path: string; replace: boolean }>));
  }

  get snapshot(): StudioAppSnapshot { return Object.freeze({ phase: this.#phase, route: this.router.current, settings: this.#settings, ...(this.#error === undefined ? {} : { error: this.#error }) }); }
  get workspace(): WorkspaceController { if (this.#workspace === undefined) throw new Error("Studio has not started"); return this.#workspace; }
  get discovery(): DiscoveryController { if (this.#discovery === undefined) throw new Error("Studio has not started"); return this.#discovery; }
  get simulation(): SimulationController { if (this.#simulation === undefined) throw new Error("Studio has not started"); return this.#simulation; }
  get screens(): ScreensController { if (this.#screens === undefined) throw new Error("Studio has not started"); return this.#screens; }

  async start(): Promise<void> {
    if (this.#phase === "starting" || this.#phase === "ready") return;
    if (this.#phase === "stopped") throw new Error("a stopped StudioApp cannot be restarted");
    this.#commit("starting");
    try {
      const providers = await this.options.providers.get();
      this.#providers = providers;
      const serialized = await providers.persistence.load(this.#settingsKey);
      if (serialized !== undefined) this.#settings = parseSettings(serialized);
      this.#workspace = new WorkspaceController(providers.api, providers.store, providers.clock);
      this.#discovery = new DiscoveryController(providers.api, providers.randomId);
      this.#simulation = new SimulationController(providers.api, providers.randomId, providers.clock);
      this.#screens = new ScreensController({ store: providers.store, api: providers.api, randomId: providers.randomId });
      this.#workspace.addEventListener("change", () => this.render());
      this.#discovery.addEventListener("change", () => this.render());
      this.#simulation.addEventListener("change", () => this.render());
      this.#screens.addEventListener("change", () => this.render());
      this.#unsubscribe = providers.store.subscribe(() => this.#scheduleSave());
      this.#registerShortcuts();
      if (this.#document !== undefined) {
        this.shortcuts.attach(this.#document);
        this.#document.defaultView?.addEventListener("popstate", this.#popstate);
      }
      this.shortcuts.setScope(this.router.current.name === "home" || this.router.current.name === "settings" ? "global" : "workspace");
      await this.#activateRoute(this.router.current);
      this.#commit("ready");
    } catch (error) {
      this.#error = errorValue(error);
      this.#commit("failed");
      throw this.#error;
    }
  }

  mount(host: HTMLElement): this {
    if (this.#host !== undefined) throw new Error("StudioApp is already mounted");
    this.#host = host;
    const document = host.ownerDocument;
    const style = node(document, "style"); style.textContent = STUDIO_CSS; this.#style = style;
    this.#root = node(document, "section", "lss"); this.#root.setAttribute("aria-label", "LawSynth Studio");
    host.append(style, this.#root);
    this.render();
    return this;
  }

  updateSettings(patch: Partial<StudioSettings>): void {
    this.#settings = mergeSettings(this.#settings, patch);
    this.#scheduleSave(); this.render(); this.#emit();
  }

  navigate(route: StudioRoute, replace = false): void { this.router.navigate(route, replace); }

  render(): void {
    if (this.#root === undefined) return;
    const document = this.#root.ownerDocument;
    this.#root.replaceChildren();
    const header = node(document, "header", "lss-header");
    header.append(node(document, "div", "lss-wordmark", "LawSynth"), node(document, "div", "lss-context", "Scientific model studio"));
    const layout = node(document, "div", "lss-layout");
    const nav = node(document, "nav", "lss-nav"); nav.setAttribute("aria-label", "Studio navigation");
    const routes: readonly [string, StudioRoute][] = [["Home", { name: "home" }], ["Workspace", this.#workspace?.snapshot.projectId === undefined ? { name: "home" } : { name: "project", projectId: this.#workspace.snapshot.projectId }], ["Settings", { name: "settings" }]];
    for (const [label, route] of routes) {
      const button = node(document, "button", undefined, label); button.type = "button";
      if (route.name === this.router.current.name) button.setAttribute("aria-current", "page");
      button.addEventListener("click", () => this.navigate(route)); nav.append(button);
    }
    const current = this.router.current;
    const projectId = this.#workspace?.snapshot.projectId ?? "demo";
    nav.append(node(document, "div", "lss-nav-label", "Screens"));
    for (const descriptor of SCREEN_REGISTRY) {
      const button = node(document, "button", undefined, descriptor.title); button.type = "button";
      if (current.name === "screen" && current.screen === descriptor.id) button.setAttribute("aria-current", "page");
      button.addEventListener("click", () => this.navigate({ name: "screen", projectId, screen: descriptor.id })); nav.append(button);
    }
    const main = node(document, "main", "lss-main"); main.id = "studio-main";
    if (this.#phase === "starting") { const status = node(document, "p", "lss-status", "Connecting to the LawSynth workspace…"); status.setAttribute("role", "status"); main.append(status); }
    else if (this.#phase === "failed") { const alert = node(document, "p", "lss-status", this.#error?.message ?? "Studio failed to start."); alert.setAttribute("role", "alert"); main.append(alert); }
    else this.#renderRoute(main, document);
    layout.append(nav, main); this.#root.append(header, layout);
  }

  async stop(): Promise<void> {
    if (this.#phase === "stopped") return;
    if (this.#saveTimer !== undefined) { clearTimeout(this.#saveTimer); this.#saveTimer = undefined; await this.#save(); }
    this.#unsubscribe?.(); this.#unsubscribe = undefined;
    this.shortcuts.detach();
    this.#document?.defaultView?.removeEventListener("popstate", this.#popstate);
    this.#workspace?.close();
    this.#screens?.dispose();
    this.#screens = undefined;
    await this.options.providers.dispose();
    this.#providers = undefined;
    this.#root?.remove(); this.#style?.remove(); this.#root = undefined; this.#style = undefined; this.#host = undefined;
    this.#commit("stopped");
  }

  #renderRoute(main: HTMLElement, document: Document): void {
    const route = this.router.current;
    if (route.name === "screen") { this.#renderScreen(main, document); return; }
    const title = route.name === "home" ? "Model systems, not screenshots" : route.name === "settings" ? "Studio settings" : route.name === "project" ? this.#workspace?.snapshot.resources?.project.name ?? "Workspace" : route.name[0]!.toUpperCase() + route.name.slice(1);
    main.append(node(document, "h1", undefined, title));
    if (route.name === "home") {
      main.append(node(document, "p", undefined, "Open a project to inspect datasets, launch discovery, compare candidate laws, and audit simulation evidence."));
    } else if (route.name === "settings") {
      const grid = node(document, "div", "lss-grid");
      grid.append(this.#card(document, "API endpoint", this.#settings.apiBaseUrl), this.#card(document, "Theme", this.#settings.theme), this.#card(document, "Autosave", `${this.#settings.autosaveMs} ms`), this.#card(document, "Telemetry", this.#settings.telemetryEnabled ? "Enabled" : "Disabled")); main.append(grid);
    } else if (route.name === "project") {
      const workspace = this.#workspace?.snapshot;
      if (workspace?.phase === "loading") main.append(node(document, "p", "lss-status", "Loading project evidence…"));
      else if (workspace?.phase === "error") main.append(node(document, "p", "lss-status", workspace.error?.message ?? "Project could not be loaded."));
      else if (workspace?.resources !== undefined) {
        const grid = node(document, "div", "lss-grid");
        grid.append(this.#card(document, "World revisions", String(workspace.resources.worlds.length)), this.#card(document, "Discovery runs", String(workspace.resources.runs.length)), this.#card(document, "Loaded", new Date(workspace.resources.loadedAt).toLocaleString())); main.append(grid);
      }
    } else main.append(node(document, "p", undefined, `Active route: ${routePath(route)}`));
  }

  #card(document: Document, label: string, value: string): HTMLElement { const card = node(document, "article", "lss-card"); card.append(node(document, "strong", undefined, label), node(document, "div", undefined, value)); return card; }

  #renderScreen(main: HTMLElement, document: Document): void {
    const screens = this.#screens;
    if (screens === undefined) { main.append(node(document, "p", "lss-status", "Screens are not ready.")); return; }
    const actions: ScreenActions = {
      onSelect: (sectionId, rowId) => screens.onSelect(sectionId, rowId),
      onControl: (fieldId, value) => screens.onControl(fieldId, value),
      onAction: (actionId) => { void screens.onAction(actionId); },
    };
    main.append(renderScreenModel(document, screens.model(), actions));
  }

  async #activateRoute(route: StudioRoute): Promise<void> {
    if (route.name === "screen") { this.#screens?.setScreen(route.screen); return; }
    if ("projectId" in route && this.#workspace?.snapshot.projectId !== route.projectId) await this.#workspace?.open(route.projectId);
  }

  #onNavigate(event: CustomEvent<{ route: StudioRoute; path: string; replace: boolean }>): void {
    const view = this.#document?.defaultView;
    if (view != null) event.detail.replace ? view.history.replaceState(null, "", event.detail.path) : view.history.pushState(null, "", event.detail.path);
    this.shortcuts.setScope(event.detail.route.name === "home" || event.detail.route.name === "settings" ? "global" : "workspace");
    void this.#activateRoute(event.detail.route).catch((error) => { this.#error = errorValue(error); this.#commit("failed"); });
    this.render(); this.#emit();
  }

  #registerShortcuts(): void {
    this.shortcuts.register({ id: "home", keys: "meta+shift+h", label: "Go home", scope: "global", run: () => this.navigate({ name: "home" }) });
    this.shortcuts.register({ id: "settings", keys: "meta+,", label: "Open settings", scope: "global", run: () => this.navigate({ name: "settings" }) });
    this.shortcuts.register({ id: "cancel-run", keys: "esc", label: "Cancel active operation", scope: "workspace", run: async () => { await this.#discovery?.cancel(); await this.#simulation?.cancel(); } });
  }

  #scheduleSave(): void {
    if (this.#providers === undefined) return;
    if (this.#saveTimer !== undefined) clearTimeout(this.#saveTimer);
    this.#saveTimer = setTimeout(() => { this.#saveTimer = undefined; void this.#save(); }, this.#settings.autosaveMs);
  }

  async #save(): Promise<void> {
    if (this.#providers === undefined) return;
    try { await this.#providers.persistence.save(this.#settingsKey, JSON.stringify(this.#settings)); }
    catch (error) { this.#providers.logger.error("Studio settings save failed", { error: errorValue(error).message }); }
  }

  #commit(phase: StudioPhase): void { this.#phase = phase; this.render(); this.#emit(); }
  #emit(): void { this.dispatchEvent(new CustomEvent("change", { detail: this.snapshot })); }
}

export function createStudioApp(options: StudioAppOptions): StudioApp { return new StudioApp(options); }
