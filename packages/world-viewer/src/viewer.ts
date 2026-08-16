/** Pure inspection models are exported alongside the DOM controller for SSR and workers. */
export type ViewerWorld = WorldDefinition;
export type ViewerVariable = WorldDefinition["variables"][number];
export type ViewerParameter = NonNullable<WorldDefinition["parameters"]>[number];
export type ViewerExpression = Expression;
export type ViewerLaw = Law;
export type ViewerEdge = DependencyEdge;

export interface ViewerIssue { readonly path: string; readonly message: string; readonly severity: "error" | "warning"; }
export interface ViewerInspection { readonly id: string; readonly label: string; readonly kind: "variable" | "parameter" | "law"; readonly description?: string; readonly references: readonly string[]; }
export interface WorldViewModel {
  readonly world: ViewerWorld;
  readonly title: string;
  readonly subtitle?: string;
  readonly graph: WorldGraphView;
  readonly equations: readonly EquationView[];
  readonly parameters: readonly ParameterRow[];
  readonly regimes?: RegimeTimeline;
  readonly uncertainty: UncertaintySummary;
  readonly provenance: ProvenanceView;
  readonly inspection: readonly ViewerInspection[];
  readonly issues: readonly ViewerIssue[];
  readonly trajectory?: TrajectoryView;
}

const identifier = /^[A-Za-z_-][A-Za-z0-9_-]*$/u;
const nonEmpty = (value: unknown): value is string => typeof value === "string" && value.trim().length > 0;

/** Validates viewer input without mutating it; errors make a rendering model unsafe. */
export function validateViewerWorld(world: ViewerWorld): readonly ViewerIssue[] {
  const issues: ViewerIssue[] = [];
  if (!nonEmpty(world.id) || !identifier.test(world.id)) issues.push({ path: "/id", severity: "error", message: "world id must be a valid non-empty identifier" });
  if (!nonEmpty(world.formatVersion)) issues.push({ path: "/formatVersion", severity: "error", message: "formatVersion is required" });
  if (!world.time || (world.time.kind !== "continuous" && world.time.kind !== "discrete")) issues.push({ path: "/time/kind", severity: "error", message: "time kind must be continuous or discrete" });
  if (world.time?.step !== undefined && (!Number.isFinite(world.time.step) || world.time.step <= 0)) issues.push({ path: "/time/step", severity: "error", message: "time step must be positive and finite" });
  const variables = new Set<string>();
  for (const [index, variable] of world.variables.entries()) {
    if (!nonEmpty(variable.id) || !identifier.test(variable.id) || variables.has(variable.id)) issues.push({ path: `/variables/${index}/id`, severity: "error", message: "variable ids must be unique valid identifiers" });
    variables.add(variable.id);
    if (!nonEmpty(variable.role)) issues.push({ path: `/variables/${index}/role`, severity: "error", message: "variable role is required" });
  }
  const parameters = new Set<string>();
  for (const [index, parameter] of (world.parameters ?? []).entries()) {
    if (!nonEmpty(parameter.id) || !identifier.test(parameter.id) || parameters.has(parameter.id)) issues.push({ path: `/parameters/${index}/id`, severity: "error", message: "parameter ids must be unique valid identifiers" });
    parameters.add(parameter.id);
    if (!Number.isFinite(parameter.value)) issues.push({ path: `/parameters/${index}/value`, severity: "error", message: "parameter values must be finite" });
    if (parameter.bounds !== undefined && (parameter.bounds[0] !== null && !Number.isFinite(parameter.bounds[0]) || parameter.bounds[1] !== null && !Number.isFinite(parameter.bounds[1]) || parameter.bounds[0] !== null && parameter.bounds[1] !== null && parameter.bounds[0] > parameter.bounds[1])) issues.push({ path: `/parameters/${index}/bounds`, severity: "error", message: "parameter bounds must be finite and ordered" });
  }
  const lawIds = new Set<string>();
  for (const [index, law] of world.laws.entries()) {
    if (!nonEmpty(law.id) || !identifier.test(law.id) || lawIds.has(law.id)) issues.push({ path: `/laws/${index}/id`, severity: "error", message: "law ids must be unique valid identifiers" });
    lawIds.add(law.id);
    if (!nonEmpty(law.kind) || !law.expression || !nonEmpty(law.expression.kind)) issues.push({ path: `/laws/${index}`, severity: "error", message: "laws require kind and expression" });
    if ("target" in law && !variables.has(law.target)) issues.push({ path: `/laws/${index}/target`, severity: "error", message: "law target must reference a variable" });
  }
  if (world.dependencies) {
    const nodes = new Set(world.dependencies.nodes);
    for (const node of nodes) if (!variables.has(node)) issues.push({ path: "/dependencies/nodes", severity: "warning", message: `graph node ${node} is not a declared variable` });
    for (const [index, edge] of world.dependencies.edges.entries()) if (!nodes.has(edge.source) || !nodes.has(edge.target)) issues.push({ path: `/dependencies/edges/${index}`, severity: "error", message: "dependency edge endpoints must appear in graph nodes" });
  }
  return issues;
}

function referencesForLaw(law: ViewerLaw): string[] {
  const references = new Set<string>();
  const visit = (value: unknown): void => {
    if (!value || typeof value !== "object") return;
    const node = value as Record<string, unknown>;
    if (node.kind === "symbol" && typeof node.id === "string") references.add(node.id);
    for (const child of Object.values(node)) if (child && typeof child === "object") Array.isArray(child) ? child.forEach(visit) : visit(child);
  };
  visit(law.expression); return [...references].sort();
}

export function inspectWorld(world: ViewerWorld): readonly ViewerInspection[] {
  return [
    ...world.variables.map((variable) => ({ id: variable.id, label: variable.name ?? variable.id, kind: "variable" as const, ...(variable.description === undefined ? {} : { description: variable.description }), references: [] })),
    ...(world.parameters ?? []).map((parameter) => ({ id: parameter.id, label: parameter.id, kind: "parameter" as const, ...(parameter.description === undefined ? {} : { description: parameter.description }), references: [] })),
    ...world.laws.map((law) => ({ id: law.id, label: "target" in law ? `${law.target} (${law.kind})` : law.id, kind: "law" as const, ...(law.description === undefined ? {} : { description: law.description }), references: referencesForLaw(law) })),
  ];
}

export function createWorldViewModel(world: ViewerWorld, trajectory?: TrajectoryInput): WorldViewModel {
  const issues = validateViewerWorld(world);
  if (issues.some((issue) => issue.severity === "error")) throw new TypeError(`invalid world: ${issues.filter((issue) => issue.severity === "error").map((issue) => issue.message).join("; ")}`);
  const regimes = regimeTimelineForWorld(world);
  return {
    world,
    title: world.name?.trim() || world.id,
    ...(world.description === undefined ? {} : { subtitle: world.description }),
    graph: graphForWorld(world),
    equations: equationsForWorld(world),
    parameters: parametersForWorld(world),
    ...(regimes === undefined ? {} : { regimes }),
    uncertainty: uncertaintySummary(world.uncertainty),
    provenance: provenanceView(world.provenance),
    inspection: inspectWorld(world), issues,
    ...(trajectory === undefined ? {} : { trajectory: trajectoryView(trajectory, `${world.name ?? world.id} trajectory`) }),
  };
}
import { categoricalColor } from "@lawsynth/chart-core";
import type { TrajectoryInput } from "@lawsynth/chart-core";
import type { DependencyEdge, Expression, Law, WorldDefinition } from "@lawsynth/world-schema";
import { createViewerBundle, type ViewerBundle } from "./bundle.js";
import { equationsForWorld, type EquationView } from "./equation.js";
import { copyText, downloadExport, exportSvg, exportViewerBundle } from "./export.js";
import { graphForWorld, type WorldGraphView } from "./graph.js";
import { availablePanels, type ViewerPanel } from "./layout.js";
import { parametersForWorld, type ParameterRow } from "./parameters.js";
import { provenanceView, type ProvenanceView } from "./provenance.js";
import { regimeTimelineForWorld, type RegimeTimeline } from "./regime.js";
import { resolveViewerTheme, viewerStyles, type ViewerTheme, type ViewerThemeName } from "./theme.js";
import { toolbarItems, type ToolbarAction } from "./toolbar.js";
import { trajectoryPlotGeometry, trajectoryView, type TrajectoryView } from "./trajectory.js";
import { uncertaintySummary, type UncertaintySummary } from "./uncertainty.js";

const SVG_NAMESPACE = "http://www.w3.org/2000/svg";

export interface WorldViewerOptions {
  readonly bundle?: ViewerBundle;
  readonly world?: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly panel?: ViewerPanel;
  readonly theme?: ViewerThemeName | ViewerTheme;
  readonly shadow?: boolean;
  readonly copyLink?: (panel: ViewerPanel) => string | Promise<string>;
}

export interface ViewerSnapshot {
  readonly worldId?: string;
  readonly panel: ViewerPanel;
  readonly theme: ViewerThemeName;
  readonly mounted: boolean;
  readonly phase: "empty" | "loading" | "ready" | "error";
  readonly status?: string;
}

export interface ViewerChangeDetail {
  readonly panel: ViewerPanel;
  readonly worldId: string;
}

function element<K extends keyof HTMLElementTagNameMap>(document: Document, tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className !== undefined) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function svgElement<K extends keyof SVGElementTagNameMap>(document: Document, tag: K, attributes: Readonly<Record<string, string | number>> = {}): SVGElementTagNameMap[K] {
  const node = document.createElementNS(SVG_NAMESPACE, tag);
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, String(value));
  return node;
}

function describeError(error: unknown): string { return error instanceof Error ? error.message : String(error); }

function definitionLabel(world: WorldDefinition): string {
  return world.name?.trim() || world.id;
}

/**
 * Stateful, framework-neutral viewer. It owns only the DOM subtree it mounts,
 * uses textContent for model data, and can be embedded by React/Vue/Svelte via
 * a ref without coupling those applications to its internal rendering model.
 */
export class WorldViewer extends EventTarget {
  #bundle: ViewerBundle | undefined;
  #panel: ViewerPanel;
  #theme: ViewerTheme;
  #host: HTMLElement | undefined;
  #root: HTMLElement | undefined;
  #renderRoot: HTMLElement | ShadowRoot | undefined;
  #style: HTMLStyleElement | undefined;
  #destroyed = false;
  #phase: ViewerSnapshot["phase"] = "empty";
  #status: string | undefined;
  #statusTone: "neutral" | "error" = "neutral";
  #trajectory: TrajectoryView | undefined;
  readonly #shadow: boolean;
  readonly #copyLink: WorldViewerOptions["copyLink"] | undefined;

  constructor(options: WorldViewerOptions = {}) {
    super();
    if (options.bundle !== undefined && options.world !== undefined) throw new RangeError("provide bundle or world, not both");
    const bundle = options.bundle ?? (options.world === undefined ? undefined : createViewerBundle(options.world, options.trajectory));
    if (bundle !== undefined) createWorldViewModel(bundle.world, bundle.trajectory);
    this.#bundle = bundle;
    this.#phase = bundle === undefined ? "empty" : "ready";
    this.#panel = options.panel ?? "overview";
    this.#theme = resolveViewerTheme(options.theme);
    this.#shadow = options.shadow !== false;
    this.#copyLink = options.copyLink;
    this.#normalizePanel();
    this.#prepareTrajectory();
  }

  get snapshot(): ViewerSnapshot {
    return Object.freeze({
      ...(this.#bundle === undefined ? {} : { worldId: this.#bundle.world.id }),
      panel: this.#panel,
      theme: this.#theme.name,
      mounted: this.#root !== undefined,
      phase: this.#phase,
      ...(this.#status === undefined ? {} : { status: this.#status }),
    });
  }

  get bundle(): ViewerBundle | undefined { return this.#bundle; }
  get panel(): ViewerPanel { return this.#panel; }

  mount(host: HTMLElement): this {
    if (this.#destroyed) throw new Error("a destroyed WorldViewer cannot be mounted again");
    if (this.#host !== undefined) throw new Error("WorldViewer is already mounted");
    this.#host = host;
    const document = host.ownerDocument;
    if (this.#shadow) {
      this.#renderRoot = host.shadowRoot ?? host.attachShadow({ mode: "open" });
    } else {
      this.#renderRoot = host;
    }
    this.#style = document.createElement("style");
    this.#style.dataset.lawsynthViewer = "styles";
    this.#style.textContent = viewerStyles(this.#theme);
    this.#root = element(document, "section", "lsv-root");
    this.#root.setAttribute("aria-label", "LawSynth world viewer");
    this.#renderRoot.append(this.#style, this.#root);
    this.render();
    return this;
  }

  setBundle(bundle: ViewerBundle): void {
    this.#assertAlive();
    createWorldViewModel(bundle.world, bundle.trajectory);
    this.#bundle = bundle;
    this.#phase = "ready";
    this.#root?.removeAttribute("aria-busy");
    this.#status = undefined;
    this.#prepareTrajectory();
    this.#normalizePanel();
    this.render();
    this.dispatchEvent(new CustomEvent<ViewerChangeDetail>("worldchange", { detail: { panel: this.#panel, worldId: bundle.world.id } }));
  }

  setWorld(world: WorldDefinition, trajectory?: TrajectoryInput): void {
    this.setBundle(createViewerBundle(world, trajectory));
  }

  setLoading(message = "Loading world evidence…"): void {
    this.#assertAlive();
    this.#phase = "loading";
    this.#status = message;
    this.#statusTone = "neutral";
    this.#root?.setAttribute("aria-busy", "true");
    this.render();
  }

  setError(error: unknown): void {
    this.#assertAlive();
    this.#phase = "error";
    this.#status = describeError(error);
    this.#statusTone = "error";
    this.#root?.removeAttribute("aria-busy");
    this.render();
    this.dispatchEvent(new CustomEvent("status", { detail: { message: this.#status, tone: "error" } }));
  }

  setPanel(panel: ViewerPanel): void {
    this.#assertAlive();
    if (panel === this.#panel) return;
    const available = this.#availablePanels();
    if (!available.some((candidate) => candidate.id === panel)) throw new RangeError(`panel ${panel} has no data in this world`);
    this.#panel = panel;
    this.#status = undefined;
    this.render();
    if (this.#bundle !== undefined) this.dispatchEvent(new CustomEvent<ViewerChangeDetail>("panelchange", { detail: { panel, worldId: this.#bundle.world.id } }));
  }

  setTheme(theme: ViewerThemeName | ViewerTheme): void {
    this.#assertAlive();
    this.#theme = resolveViewerTheme(theme);
    if (this.#style !== undefined) this.#style.textContent = viewerStyles(this.#theme);
    this.render();
    this.dispatchEvent(new CustomEvent("themechange", { detail: { theme: this.#theme.name } }));
  }

  render(): void {
    if (this.#root === undefined) return;
    const document = this.#root.ownerDocument;
    this.#root.replaceChildren();
    if (this.#bundle === undefined) {
      this.#renderEmpty(document, this.#phase);
      return;
    }
    const header = this.#renderHeader(document);
    const shell = element(document, "div", "lsv-shell");
    shell.append(this.#renderNavigation(document), this.#renderMain(document), this.#renderEvidence(document));
    this.#root.append(header, shell);
  }

  destroy(): void {
    if (this.#destroyed) return;
    this.#destroyed = true;
    this.#root?.remove();
    this.#style?.remove();
    this.#root = undefined;
    this.#style = undefined;
    this.#renderRoot = undefined;
    this.#host = undefined;
  }

  #assertAlive(): void {
    if (this.#destroyed) throw new Error("WorldViewer has been destroyed");
  }

  #prepareTrajectory(): void {
    this.#trajectory = undefined;
    if (this.#bundle?.trajectory === undefined) return;
    try { this.#trajectory = trajectoryView(this.#bundle.trajectory, `${definitionLabel(this.#bundle.world)} trajectory`); }
    catch (error) { this.#status = `Trajectory unavailable: ${describeError(error)}`; this.#statusTone = "error"; }
  }

  #availablePanels(): readonly { readonly id: ViewerPanel; readonly label: string }[] {
    const world = this.#bundle?.world;
    return availablePanels({
      trajectory: this.#trajectory !== undefined,
      uncertainty: (world?.uncertainty?.entries.length ?? 0) > 0,
      regimes: (world?.regimes?.regimes.length ?? 0) > 0,
      provenance: world?.provenance !== undefined,
    });
  }

  #normalizePanel(): void {
    if (!this.#availablePanels().some((candidate) => candidate.id === this.#panel)) this.#panel = "overview";
  }

  #renderEmpty(document: Document, phase: ViewerSnapshot["phase"]): void {
    const header = element(document, "header", "lsv-header");
    const names = element(document, "div");
    const title = phase === "loading" ? "Loading world" : phase === "error" ? "World unavailable" : "No world loaded";
    names.append(element(document, "div", "lsv-kicker", "LawSynth evidence viewer"), element(document, "h1", "lsv-title", title));
    header.append(names);
    const main = element(document, "main", "lsv-main");
    const detail = phase === "loading" ? this.#status ?? "Loading world evidence…" : phase === "error" ? this.#status ?? "The world could not be loaded." : "Provide a WorldDefinition or a decoded LawSynth viewer bundle to begin inspection.";
    const message = element(document, "p", phase === "error" ? "lsv-status" : "lsv-muted", detail);
    if (phase === "error") message.setAttribute("role", "alert");
    if (phase === "loading") message.setAttribute("role", "status");
    main.append(message);
    this.#root!.append(header, main);
  }

  #renderHeader(document: Document): HTMLElement {
    const world = this.#bundle!.world;
    const header = element(document, "header", "lsv-header");
    const names = element(document, "div");
    names.append(element(document, "div", "lsv-kicker", `World · ${world.formatVersion}`), element(document, "h1", "lsv-title", definitionLabel(world)));
    const kind = element(document, "span", "lsv-kind", world.time.kind);
    const toolbar = element(document, "div", "lsv-toolbar");
    toolbar.setAttribute("role", "toolbar");
    toolbar.setAttribute("aria-label", "Viewer actions");
    for (const item of toolbarItems({ panel: this.#panel, canExportSvg: this.#panel === "graph" || this.#panel === "trajectory" || this.#panel === "regimes" })) {
      const button = element(document, "button", undefined, item.label);
      button.type = "button";
      button.title = item.title;
      button.disabled = item.disabled;
      button.dataset.action = item.action;
      button.addEventListener("click", () => { void this.#performAction(item.action); });
      toolbar.append(button);
    }
    header.append(names, kind, toolbar);
    return header;
  }

  #renderNavigation(document: Document): HTMLElement {
    const navigation = element(document, "nav", "lsv-nav");
    navigation.setAttribute("aria-label", "World inspection sections");
    for (const panel of this.#availablePanels()) {
      const button = element(document, "button", undefined, panel.label);
      button.type = "button";
      if (panel.id === this.#panel) button.setAttribute("aria-current", "page");
      button.addEventListener("click", () => this.setPanel(panel.id));
      navigation.append(button);
    }
    navigation.addEventListener("keydown", (event) => {
      if (event.key !== "ArrowDown" && event.key !== "ArrowRight" && event.key !== "ArrowUp" && event.key !== "ArrowLeft") return;
      const buttons = [...navigation.querySelectorAll<HTMLButtonElement>("button:not([disabled])")];
      const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
      if (current < 0 || buttons.length === 0) return;
      event.preventDefault();
      const direction = event.key === "ArrowDown" || event.key === "ArrowRight" ? 1 : -1;
      buttons[(current + direction + buttons.length) % buttons.length]?.focus();
    });
    return navigation;
  }

  #renderMain(document: Document): HTMLElement {
    const main = element(document, "main", "lsv-main");
    main.id = "lawsynth-viewer-content";
    if (this.#status !== undefined) {
      const status = element(document, "div", "lsv-status", this.#status);
      status.setAttribute("role", this.#statusTone === "error" ? "alert" : "status");
      main.append(status);
    }
    try {
      switch (this.#panel) {
        case "overview": main.append(this.#renderOverview(document)); break;
        case "equations": main.append(this.#renderEquations(document)); break;
        case "graph": main.append(this.#renderGraph(document)); break;
        case "trajectory": main.append(this.#renderTrajectory(document)); break;
        case "parameters": main.append(this.#renderParameters(document)); break;
        case "uncertainty": main.append(this.#renderUncertainty(document)); break;
        case "regimes": main.append(this.#renderRegimes(document)); break;
        case "provenance": main.append(this.#renderProvenance(document)); break;
      }
    } catch (error) {
      const alert = element(document, "div", "lsv-status", `This view could not be rendered: ${describeError(error)}`);
      alert.setAttribute("role", "alert");
      main.append(alert);
    }
    return main;
  }

  #createPanel(document: Document, title: string): HTMLElement {
    const section = element(document, "section", "lsv-panel");
    section.setAttribute("aria-labelledby", `lsv-${this.#panel}-title`);
    const heading = element(document, "h2", undefined, title);
    heading.id = `lsv-${this.#panel}-title`;
    section.append(heading);
    return section;
  }

  #card(document: Document, label: string, value: string, detail?: string): HTMLElement {
    const card = element(document, "article", "lsv-card");
    card.append(element(document, "div", "lsv-label", label), element(document, "div", "lsv-value", value));
    if (detail !== undefined) card.append(element(document, "div", "lsv-muted", detail));
    return card;
  }

  #renderOverview(document: Document): HTMLElement {
    const world = this.#bundle!.world;
    const section = this.#createPanel(document, "Model overview");
    if (world.description !== undefined) section.append(element(document, "p", undefined, world.description));
    const grid = element(document, "div", "lsv-grid");
    grid.append(
      this.#card(document, "Variables", String(world.variables.length), `${world.variables.filter((variable) => variable.role === "state").length} state variables`),
      this.#card(document, "Laws", String(world.laws.length), `${world.laws.filter((law) => law.enabled !== false).length} enabled`),
      this.#card(document, "Parameters", String(world.parameters?.length ?? 0), "Declared coefficients"),
      this.#card(document, "Time", world.time.kind, `${world.time.symbol ?? "t"}${world.time.unit === undefined ? "" : ` · ${world.time.unit}`}`),
    );
    if (this.#trajectory !== undefined) grid.append(this.#card(document, "Samples", String(this.#trajectory.sampleCount), `duration ${this.#trajectory.duration.toPrecision(5)}`));
    if (world.uncertainty !== undefined) grid.append(this.#card(document, "Uncertainty", String(world.uncertainty.entries.length), world.uncertainty.method ?? "method not recorded"));
    section.append(grid);
    if ((world.tags?.length ?? 0) > 0) section.append(element(document, "p", "lsv-muted", `Tags: ${world.tags!.join(" · ")}`));
    return section;
  }

  #renderEquations(document: Document): HTMLElement {
    const section = this.#createPanel(document, "Governing equations");
    const equations = equationsForWorld(this.#bundle!.world);
    if (equations.length === 0) section.append(element(document, "p", "lsv-muted", "No laws are declared."));
    for (const equation of equations) {
      const article = element(document, "article", "lsv-equation");
      const label = element(document, "div", "lsv-label", `${equation.kind} · ${equation.enabled ? "enabled" : "disabled"}`);
      const code = element(document, "code", undefined, equation.text);
      if (equation.description !== undefined) code.title = equation.description;
      const dependencies = element(document, "div", "lsv-muted", equation.symbols.length === 0 ? "No symbol dependencies" : `Depends on ${equation.symbols.join(", ")}`);
      article.append(label, code, dependencies);
      section.append(article);
    }
    return section;
  }

  #renderGraph(document: Document): HTMLElement {
    const graph = graphForWorld(this.#bundle!.world);
    const section = this.#createPanel(document, "Dependency structure");
    section.append(element(document, "p", "lsv-muted", graph.inferred ? "Edges are inferred from equation symbols." : "Edges come from the declared dependency graph."));
    const svg = svgElement(document, "svg", { viewBox: `0 0 ${graph.width} ${graph.height}`, role: "img", "aria-label": "World dependency graph" });
    svg.classList.add("lsv-svg");
    const position = new Map(graph.nodes.map((node) => [node.id, node]));
    for (const edge of graph.edges) {
      const source = position.get(edge.source);
      const target = position.get(edge.target);
      if (source === undefined || target === undefined) continue;
      const path = svgElement(document, "path", { class: "lsv-edge", d: `M${source.x + source.width},${source.y + source.height / 2} C${source.x + source.width + 36},${source.y + source.height / 2} ${target.x - 36},${target.y + target.height / 2} ${target.x},${target.y + target.height / 2}` });
      const title = svgElement(document, "title");
      title.textContent = `${edge.source} to ${edge.target}, ${edge.status}`;
      path.append(title);
      svg.append(path);
    }
    for (const node of graph.nodes) {
      const group = svgElement(document, "g", { transform: `translate(${node.x} ${node.y})` });
      const rect = svgElement(document, "rect", { class: "lsv-node", width: node.width, height: node.height, rx: 3 });
      const label = svgElement(document, "text", { x: 12, y: 24, fill: "var(--lsv-text)", "font-size": 13, "font-weight": 650 });
      label.textContent = node.label;
      const role = svgElement(document, "text", { x: 12, y: 43, fill: "var(--lsv-muted)", "font-size": 11 });
      role.textContent = `${node.role}${node.unit === undefined ? "" : ` · ${node.unit}`}`;
      group.append(rect, label, role);
      svg.append(group);
    }
    section.append(svg);
    return section;
  }

  #renderTrajectory(document: Document): HTMLElement {
    const section = this.#createPanel(document, "Simulation trajectory");
    if (this.#trajectory === undefined) {
      section.append(element(document, "p", "lsv-muted", "No trajectory is attached to this viewer bundle."));
      return section;
    }
    const geometry = trajectoryPlotGeometry(this.#trajectory.chart, 900, 360, 34);
    const svg = svgElement(document, "svg", { viewBox: `0 0 ${geometry.width} ${geometry.height}`, role: "img", "aria-label": `${this.#trajectory.chart.title}; ${this.#trajectory.sampleCount} samples` });
    svg.classList.add("lsv-svg");
    for (let index = 1; index < 5; index += 1) {
      const y = (geometry.height / 5) * index;
      svg.append(svgElement(document, "line", { x1: 34, x2: geometry.width - 34, y1: y, y2: y, stroke: "var(--lsv-grid)" }));
    }
    geometry.paths.forEach((path) => {
      const line = svgElement(document, "path", { d: path.d, fill: "none", stroke: path.color ?? categoricalColor(path.id), "stroke-width": 2, "vector-effect": "non-scaling-stroke" });
      const title = svgElement(document, "title");
      title.textContent = path.label;
      line.append(title);
      svg.append(line);
    });
    section.append(svg);
    const legend = element(document, "div", "lsv-grid");
    this.#trajectory.chart.series.forEach((series) => legend.append(this.#card(document, series.label, `${series.points.length} samples`, series.unit)));
    section.append(legend);
    return section;
  }

  #renderParameters(document: Document): HTMLElement {
    const section = this.#createPanel(document, "Parameter ledger");
    const parameters = parametersForWorld(this.#bundle!.world);
    if (parameters.length === 0) {
      section.append(element(document, "p", "lsv-muted", "This world declares no parameters."));
      return section;
    }
    const table = element(document, "table", "lsv-table");
    const head = table.createTHead().insertRow();
    ["Parameter", "Value", "Bounds", "Status"].forEach((label) => { const cell = document.createElement("th"); cell.scope = "col"; cell.textContent = label; head.append(cell); });
    const body = table.createTBody();
    for (const parameter of parameters) {
      const row = body.insertRow();
      row.insertCell().textContent = parameter.id;
      row.insertCell().textContent = `${parameter.formattedValue}${parameter.unit === undefined ? "" : ` ${parameter.unit}`}`;
      row.insertCell().textContent = parameter.lower === undefined && parameter.upper === undefined ? "unbounded" : `${parameter.lower ?? "−∞"} … ${parameter.upper ?? "+∞"}`;
      row.insertCell().textContent = parameter.fixed ? "fixed" : "variable";
      if (parameter.description !== undefined) row.title = parameter.description;
    }
    section.append(table);
    return section;
  }

  #renderUncertainty(document: Document): HTMLElement {
    const section = this.#createPanel(document, "Uncertainty record");
    const summary = uncertaintySummary(this.#bundle!.world.uncertainty);
    const grid = element(document, "div", "lsv-grid");
    for (const level of ["data", "parameter", "structural", "trajectory"] as const) grid.append(this.#card(document, level, String(summary.counts[level]), "recorded entries"));
    section.append(grid);
    if (summary.method !== undefined) section.append(element(document, "p", undefined, `Method: ${summary.method}`));
    if (summary.seed !== undefined) section.append(element(document, "p", undefined, `Seed: ${summary.seed}`));
    for (const entry of summary.entries) {
      const card = element(document, "article", "lsv-card");
      card.append(element(document, "div", "lsv-label", entry.level));
      switch (entry.level) {
        case "data": card.append(element(document, "div", undefined, `${entry.variable}${entry.measurementError === undefined ? "" : ` · measurement error ${entry.measurementError}`}`)); break;
        case "parameter": card.append(element(document, "div", undefined, `${entry.parameter}${entry.interval === undefined ? "" : ` · [${entry.interval.lower}, ${entry.interval.upper}] at ${(entry.interval.confidence * 100).toPrecision(3)}%`}`)); break;
        case "structural": card.append(element(document, "div", undefined, `${entry.alternatives.length} structural alternatives`)); break;
        case "trajectory": card.append(element(document, "div", undefined, `${entry.bands.length} trajectory bands`)); break;
      }
      section.append(card);
    }
    return section;
  }

  #renderRegimes(document: Document): HTMLElement {
    const section = this.#createPanel(document, "Regime timeline");
    const timeline = regimeTimelineForWorld(this.#bundle!.world, 850);
    const model = this.#bundle!.world.regimes;
    if (model === undefined) return section;
    if (timeline === undefined) {
      const grid = element(document, "div", "lsv-grid");
      model.regimes.forEach((regime) => grid.append(this.#card(document, regime.name ?? regime.id, String(regime.lawIds?.length ?? 0), "linked laws")));
      section.append(grid);
      return section;
    }
    const svg = svgElement(document, "svg", { viewBox: `0 0 850 90`, role: "img", "aria-label": `${timeline.regimeCount} regimes from ${timeline.start} to ${timeline.end}` });
    svg.classList.add("lsv-svg");
    svg.style.minHeight = "90px";
    for (const lane of timeline.lanes) {
      const rect = svgElement(document, "rect", { x: lane.x, y: 20, width: lane.width, height: 34, fill: categoricalColor(lane.regime), opacity: lane.confidence ?? 0.88 });
      const title = svgElement(document, "title");
      title.textContent = `${lane.label}: ${lane.start}–${lane.end}${lane.confidence === undefined ? "" : `, confidence ${lane.confidence}`}`;
      rect.append(title);
      svg.append(rect);
    }
    const start = svgElement(document, "text", { x: 0, y: 78, fill: "var(--lsv-muted)", "font-size": 11 }); start.textContent = String(timeline.start);
    const end = svgElement(document, "text", { x: 850, y: 78, fill: "var(--lsv-muted)", "font-size": 11, "text-anchor": "end" }); end.textContent = String(timeline.end);
    svg.append(start, end);
    section.append(svg);
    return section;
  }

  #renderProvenance(document: Document): HTMLElement {
    const section = this.#createPanel(document, "Provenance and reproducibility");
    const provenance = provenanceView(this.#bundle!.world.provenance);
    section.append(element(document, "p", "lsv-status", provenance.reproducible ? "Core reproducibility fields are present." : "The record is missing one or more core reproducibility fields."));
    const table = element(document, "table", "lsv-table");
    const body = table.createTBody();
    for (const item of provenance.rows) { const row = body.insertRow(); row.insertCell().textContent = item.label; row.insertCell().textContent = item.value; }
    section.append(table);
    if (provenance.assumptions.length > 0) {
      section.append(element(document, "h3", undefined, "Recorded assumptions"));
      const list = element(document, "ul");
      provenance.assumptions.forEach((assumption) => list.append(element(document, "li", undefined, assumption)));
      section.append(list);
    }
    return section;
  }

  #renderEvidence(document: Document): HTMLElement {
    const aside = element(document, "aside", "lsv-evidence");
    aside.setAttribute("aria-label", "Evidence summary");
    const world = this.#bundle!.world;
    const provenance = provenanceView(world.provenance);
    aside.append(element(document, "div", "lsv-kicker", "Evidence ledger"));
    aside.append(element(document, "h3", undefined, provenance.reproducible ? "Traceable record" : "Partial record"));
    for (const item of provenance.rows.slice(0, 7)) {
      const block = element(document, "div");
      block.style.marginBottom = "14px";
      block.append(element(document, "div", "lsv-label", item.label), element(document, "div", undefined, item.value));
      aside.append(block);
    }
    if (provenance.assumptions.length > 0) aside.append(element(document, "p", "lsv-muted", `${provenance.assumptions.length} recorded assumptions`));
    return aside;
  }

  async #performAction(action: ToolbarAction): Promise<void> {
    if (this.#bundle === undefined || this.#root === undefined) return;
    try {
      switch (action) {
        case "copy-link": {
          const provided = await this.#copyLink?.(this.#panel);
          const location = this.#root.ownerDocument.defaultView?.location;
          const url = provided ?? (() => { if (location === undefined) throw new Error("no browser location is available"); const value = new URL(location.href); value.searchParams.set("panel", this.#panel); return value.toString(); })();
          await copyText(url, this.#root.ownerDocument);
          this.#setStatus("Link copied.");
          break;
        }
        case "download-json": downloadExport(exportViewerBundle(this.#bundle), this.#root.ownerDocument); this.#setStatus("JSON export prepared."); break;
        case "download-svg": {
          const svg = this.#root.querySelector("svg");
          if (svg === null || svg.namespaceURI !== SVG_NAMESPACE) throw new Error("the current panel has no exportable SVG");
          downloadExport(exportSvg(svg as SVGSVGElement, `${this.#bundle.world.id}-${this.#panel}.svg`), this.#root.ownerDocument);
          this.#setStatus("SVG export prepared.");
          break;
        }
        case "reset-view": this.#setStatus("View reset."); break;
        case "toggle-theme": this.setTheme(this.#theme.name === "paper" ? "midnight" : "paper"); break;
      }
    } catch (error) {
      this.#setStatus(describeError(error), "error");
    }
  }

  #setStatus(message: string, tone: "neutral" | "error" = "neutral"): void {
    this.#status = message;
    this.#statusTone = tone;
    this.#root?.removeAttribute("aria-busy");
    this.render();
    this.dispatchEvent(new CustomEvent("status", { detail: { message, tone } }));
  }
}

export function createWorldViewer(options: WorldViewerOptions = {}): WorldViewer {
  return new WorldViewer(options);
}
