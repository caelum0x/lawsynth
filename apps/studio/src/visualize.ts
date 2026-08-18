/**
 * Studio trajectory visualization: turns a world's simulated trajectory into a
 * rendered line chart, offline and deterministically.
 *
 * There is no in-browser engine yet (the wasm integrator is a separate,
 * network-gated build step), so this surface is driven by a small set of
 * BUNDLED, RNG-free sample trajectories — computed once with a fixed RK4 loop —
 * plus an optional pasted `ViewerBundle` the user can load. The render path
 * mirrors the app's existing trajectory pipeline (`screens/render.ts` +
 * `screens/world-lab.ts`): `trajectoryView` builds a `chart-core` `ChartModel`,
 * that model is projected to SVG path geometry with `chart-core`'s `createScale`
 * primitives, and the chart is mounted as an `SVGElement` via `createElementNS`
 * — the same convention the World Lab screen uses. (`world-viewer`'s
 * `trajectoryPlotGeometry` would be the ready-made projector, but it passes an
 * inverted pixel range to `createScale`, which rejects it, so the geometry is
 * built directly here — see `renderChartSvg`.) `@lawsynth/world-viewer` also
 * exposes an `svgDocument` string helper for export; the DOM render path here is
 * `createElementNS` because that is what the app already ships.
 *
 * Every function is pure and deterministic: the samples are fixed computations,
 * nothing reads the clock, the network, or randomness. Honest outcomes are
 * surfaced honestly — a bundle that carries a world but no trajectory shows a
 * "connect the engine to simulate" note rather than a fabricated curve, and a
 * malformed bundle degrades to an error notice instead of throwing.
 */

import {
  categoricalColor,
  createScale,
  extent,
  linearTicks,
  padDomain,
  type ChartModel,
  type Domain,
  type Series,
  type TrajectoryInput,
} from "@lawsynth/chart-core";
import {
  parseViewerBundle,
  trajectoryView,
  ViewerBundleError,
  type ViewerBundle,
} from "@lawsynth/world-viewer";

// --- bundled sample trajectories --------------------------------------------

/** The bundled, offline demo trajectories, in a stable presentation order. */
export const SAMPLE_TRAJECTORY_IDS = ["damped-oscillator", "lotka-volterra"] as const;

export type SampleTrajectoryId = (typeof SAMPLE_TRAJECTORY_IDS)[number];

const SAMPLE_ID_SET: ReadonlySet<string> = new Set(SAMPLE_TRAJECTORY_IDS);

/** Narrows an arbitrary string to a known {@link SampleTrajectoryId}. */
export function isSampleTrajectoryId(value: unknown): value is SampleTrajectoryId {
  return typeof value === "string" && SAMPLE_ID_SET.has(value);
}

interface SampleMeta {
  readonly label: string;
  readonly title: string;
  readonly description: string;
}

const SAMPLE_META: Readonly<Record<SampleTrajectoryId, SampleMeta>> = Object.freeze({
  "damped-oscillator": Object.freeze({
    label: "Damped oscillator",
    title: "Damped oscillator (bundled demo)",
    description: "x'' + 2ζω·x' + ω²·x = 0, integrated with RK4 (ω = 1, ζ = 0.15).",
  }),
  "lotka-volterra": Object.freeze({
    label: "Lotka–Volterra",
    title: "Lotka–Volterra orbit (bundled demo)",
    description: "Predator–prey limit cycle, integrated with RK4 (α = 1.1, β = 0.4, δ = 0.1, γ = 0.4).",
  }),
});

/** Human label for a bundled sample id (drives the sample selector). */
export function sampleTrajectoryLabel(id: SampleTrajectoryId): string {
  return SAMPLE_META[id].label;
}

/** One-line description of what a bundled sample models. */
export function sampleTrajectoryDescription(id: SampleTrajectoryId): string {
  return SAMPLE_META[id].description;
}

/** Presentation title for a bundled sample (used as the chart heading). */
export function sampleTrajectoryTitle(id: SampleTrajectoryId): string {
  return SAMPLE_META[id].title;
}

type Derivative = (state: readonly number[]) => readonly number[];

function axpy(base: readonly number[], delta: readonly number[], scale: number): number[] {
  return base.map((value, index) => value + delta[index]! * scale);
}

/** One classical fourth-order Runge–Kutta step — deterministic, RNG-free. */
function rk4Step(f: Derivative, state: readonly number[], dt: number): number[] {
  const k1 = f(state);
  const k2 = f(axpy(state, k1, dt / 2));
  const k3 = f(axpy(state, k2, dt / 2));
  const k4 = f(axpy(state, k3, dt));
  return state.map((value, index) => value + (dt / 6) * (k1[index]! + 2 * k2[index]! + 2 * k3[index]! + k4[index]!));
}

function integrate(
  variables: readonly string[],
  f: Derivative,
  initial: readonly number[],
  dt: number,
  steps: number,
  metadata: Readonly<Record<string, string | number | boolean>>,
): TrajectoryInput {
  const times: number[] = [];
  const values: number[][] = [];
  let state: readonly number[] = [...initial];
  for (let step = 0; step <= steps; step += 1) {
    times.push(Number((step * dt).toFixed(6)));
    values.push([...state]);
    state = rk4Step(f, state, dt);
  }
  return Object.freeze({ variables: Object.freeze([...variables]), times: Object.freeze(times), values: Object.freeze(values), metadata });
}

function buildSample(id: SampleTrajectoryId): TrajectoryInput {
  if (id === "damped-oscillator") {
    const omega = 1;
    const zeta = 0.15;
    return integrate(
      ["x", "v"],
      ([x, v]) => [v!, -2 * zeta * omega * v! - omega * omega * x!],
      [1, 0],
      0.1,
      120,
      Object.freeze({ system: "damped-oscillator", integrator: "rk4", engine: "bundled-sample" }),
    );
  }
  const alpha = 1.1;
  const beta = 0.4;
  const delta = 0.1;
  const gamma = 0.4;
  return integrate(
    ["prey", "predator"],
    ([prey, predator]) => [alpha * prey! - beta * prey! * predator!, delta * prey! * predator! - gamma * predator!],
    [10, 5],
    0.02,
    600,
    Object.freeze({ system: "lotka-volterra", integrator: "rk4", engine: "bundled-sample" }),
  );
}

const SAMPLE_CACHE = new Map<SampleTrajectoryId, TrajectoryInput>();

/** The bundled, deterministic sample trajectory for an id (computed once, cached). */
export function sampleTrajectory(id: SampleTrajectoryId): TrajectoryInput {
  const cached = SAMPLE_CACHE.get(id);
  if (cached !== undefined) return cached;
  const built = buildSample(id);
  SAMPLE_CACHE.set(id, built);
  return built;
}

/**
 * Serializes a bundled sample as a valid `ViewerBundle` JSON string. This backs
 * the "Load sample bundle" action: it round-trips the demo world + trajectory
 * through `parseViewerBundle`, exactly as a real `.viewer.json` artifact would.
 */
export function sampleViewerBundleJson(id: SampleTrajectoryId): string {
  const trajectory = sampleTrajectory(id);
  const bundle = {
    format: "lawsynth-viewer",
    version: 1,
    world: {
      formatVersion: "0.1.0",
      id,
      name: SAMPLE_META[id].label,
      time: { kind: "continuous", symbol: "t", unit: "s" },
      variables: trajectory.variables.map((variable) => ({ id: variable, role: "state" })),
      parameters: [],
      laws: [],
    },
    trajectory: { variables: trajectory.variables, times: trajectory.times, values: trajectory.values },
  };
  return JSON.stringify(bundle, null, 2);
}

// --- chart view model -------------------------------------------------------

/** Presentation-ready model of a trajectory chart (drives metrics + tests). */
export interface TrajectoryChartView {
  readonly title: string;
  readonly chart: ChartModel;
  readonly variables: readonly string[];
  readonly seriesCount: number;
  readonly sampleCount: number;
  readonly duration: number;
  readonly timeDomain: Domain;
  readonly valueDomain: Domain;
}

/** Builds a `chart-core` {@link ChartModel} from a trajectory (times + values). */
export function trajectoryChartModel(input: TrajectoryInput, title = "Simulation trajectory"): ChartModel {
  return trajectoryView(input, title).chart;
}

/** Builds the full presentation model — chart plus derived metrics. */
export function trajectoryChartView(input: TrajectoryInput, title = "Simulation trajectory"): TrajectoryChartView {
  const view = trajectoryView(input, title);
  return Object.freeze({
    title,
    chart: view.chart,
    variables: Object.freeze([...view.trajectory.variables]),
    seriesCount: view.chart.series.length,
    sampleCount: view.sampleCount,
    duration: view.duration,
    timeDomain: view.chart.xAxis.domain,
    valueDomain: view.chart.yAxis.domain,
  });
}

// --- DOM + SVG rendering ----------------------------------------------------
//
// Pure element builders. `node` mints HTML nodes (mirroring analysis.ts); `svg`
// mints namespaced SVG nodes (mirroring screens/render.ts). The chart is a real
// <svg> subtree — a frame, gridline ticks, an optional zero baseline, and one
// colored <path> per variable — appended to the DOM. No state, no clock.

const SVG_NS = "http://www.w3.org/2000/svg";
const CHART_WIDTH = 720;
const CHART_HEIGHT = 320;
const CHART_PADDING = 40;

function node<K extends keyof HTMLElementTagNameMap>(
  document: Document,
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tag);
  if (className !== undefined) element.className = className;
  if (text !== undefined) element.textContent = text;
  return element;
}

function svg(document: Document, tag: string, attrs: Readonly<Record<string, string | number>>, text?: string): SVGElement {
  const element = document.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) element.setAttribute(key, String(value));
  if (text !== undefined) element.textContent = text;
  return element;
}

type NoticeTone = "success" | "warning" | "error" | "info";

function notice(document: Document, text: string, tone: NoticeTone): HTMLElement {
  const element = node(document, "p", `lss-scr-notice lss-tone-${tone}`, text);
  element.setAttribute("role", tone === "error" ? "alert" : "status");
  return element;
}

function metrics(document: Document, entries: readonly (readonly [string, string])[]): HTMLElement {
  const grid = node(document, "div", "lss-scr-metrics");
  for (const [label, value] of entries) {
    const cell = node(document, "div", "lss-scr-metric");
    cell.append(node(document, "span", "lss-scr-metric-label", label), node(document, "span", "lss-scr-metric-value", value));
    grid.append(cell);
  }
  return grid;
}

function formatTick(value: number): string {
  if (value === 0) return "0";
  if (!Number.isFinite(value)) return String(value);
  return String(Number(value.toPrecision(4)));
}

function seriesExtent(series: readonly Series[], key: "x" | "y"): Domain {
  const values = series.flatMap((entry) => entry.points.map((point) => point[key]));
  return values.length === 0 ? { min: 0, max: 1 } : padDomain(extent(values), 0.04);
}

function pathData(series: Series, xScale: (v: number) => number, yPixel: (v: number) => number): string {
  return series.points.map((point, index) => `${index === 0 ? "M" : "L"}${xScale(point.x).toFixed(2)},${yPixel(point.y).toFixed(2)}`).join(" ");
}

/**
 * Builds the chart as an accessible `<svg>` subtree with one path per series.
 *
 * The geometry is projected here with `chart-core`'s `createScale` rather than
 * `world-viewer`'s `trajectoryPlotGeometry`: that helper calls
 * `createScale(yDomain, { min: height - padding, max: padding })`, but
 * `createScale` rejects any pixel range with `min > max` ("invalid domain"), so
 * it throws for every input. We instead use an ascending pixel range and flip Y
 * ourselves (`top + bottom - yScale(v)`), keeping the pipeline within the same
 * validated primitives without touching the read-only package.
 */
function renderChartSvg(document: Document, view: TrajectoryChartView): SVGElement {
  const left = CHART_PADDING;
  const right = CHART_WIDTH - CHART_PADDING;
  const top = CHART_PADDING;
  const bottom = CHART_HEIGHT - CHART_PADDING;
  const xDomain = seriesExtent(view.chart.series, "x");
  const yDomain = seriesExtent(view.chart.series, "y");
  const xScale = createScale(xDomain, { min: left, max: right });
  const yScaleAscending = createScale(yDomain, { min: top, max: bottom });
  const yPixel = (value: number): number => top + bottom - yScaleAscending(value);

  const root = svg(document, "svg", {
    viewBox: `0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`,
    class: "lss-scr-chart",
    role: "img",
    "aria-label": `${view.title}: ${view.variables.join(", ")} versus ${view.chart.xAxis.label}`,
  });
  root.append(svg(document, "title", {}, view.title));
  root.append(svg(document, "rect", { x: left, y: top, width: right - left, height: bottom - top, fill: "#fffdf7", stroke: "#c8c6ba", "stroke-width": 1 }));

  for (const tick of linearTicks(yDomain, 5)) {
    const y = yPixel(tick);
    root.append(svg(document, "line", { x1: left, y1: y.toFixed(2), x2: right, y2: y.toFixed(2), stroke: "#e7e4d8", "stroke-width": 1 }));
    root.append(svg(document, "text", { x: left - 6, y: (y + 3).toFixed(2), "text-anchor": "end", "font-size": 10, fill: "#8a9089" }, formatTick(tick)));
  }
  for (const tick of linearTicks(xDomain, 5)) {
    const x = xScale(tick);
    root.append(svg(document, "text", { x: x.toFixed(2), y: (bottom + 15).toFixed(2), "text-anchor": "middle", "font-size": 10, fill: "#8a9089" }, formatTick(tick)));
  }
  if (yDomain.min < 0 && yDomain.max > 0) {
    const zero = yPixel(0);
    root.append(svg(document, "line", { x1: left, y1: zero.toFixed(2), x2: right, y2: zero.toFixed(2), stroke: "#59635e", "stroke-width": 1, "stroke-dasharray": "4 3" }));
  }
  for (const series of view.chart.series) {
    const color = series.color ?? categoricalColor(series.id);
    root.append(svg(document, "path", { d: pathData(series, xScale, yPixel), fill: "none", stroke: color, "stroke-width": 1.75, "stroke-linejoin": "round", "stroke-linecap": "round" }));
  }
  root.append(svg(document, "text", { x: ((left + right) / 2).toFixed(2), y: (CHART_HEIGHT - 6).toFixed(2), "text-anchor": "middle", "font-size": 11, fill: "#59635e" }, view.chart.xAxis.label));
  root.append(svg(document, "text", { x: 12, y: ((top + bottom) / 2).toFixed(2), "text-anchor": "middle", "font-size": 11, fill: "#59635e", transform: `rotate(-90 12 ${((top + bottom) / 2).toFixed(2)})` }, view.chart.yAxis.label));
  return root;
}

function renderLegend(document: Document, view: TrajectoryChartView): HTMLElement {
  const list = node(document, "div", "lss-scr-legend");
  for (const series of view.chart.series) {
    const item = node(document, "span", "lss-scr-legend-item");
    const swatch = node(document, "span", "lss-scr-legend-swatch");
    swatch.style.background = series.color ?? categoricalColor(series.id);
    item.append(swatch, node(document, "span", "lss-scr-legend-label", series.label));
    list.append(item);
  }
  return list;
}

function renderChartSection(document: Document, view: TrajectoryChartView): HTMLElement {
  const section = node(document, "section", "lss-scr-section");
  section.append(
    metrics(document, [
      ["System", view.title],
      ["Variables", view.variables.join(", ")],
      ["Samples", String(view.sampleCount)],
      ["Duration", `${formatTick(view.duration)} ${view.chart.xAxis.label}`],
      ["Value range", `[${formatTick(view.valueDomain.min)}, ${formatTick(view.valueDomain.max)}]`],
    ]),
  );
  section.append(renderChartSvg(document, view));
  section.append(renderLegend(document, view));
  return section;
}

function bundleWorldName(bundle: ViewerBundle): string {
  const name = bundle.world.name;
  return typeof name === "string" && name.trim().length > 0 ? name : bundle.world.id;
}

function renderBundle(document: Document, container: HTMLElement, bundle: ViewerBundle): void {
  const worldName = bundleWorldName(bundle);
  if (bundle.trajectory === undefined) {
    // Honest "no engine" state: a world with no simulated trajectory. We do not
    // fabricate a curve — the wasm engine has to run to produce dynamics.
    container.append(
      notice(
        document,
        `Bundle for "${worldName}" carries no trajectory. Connect the LawSynth engine (wasm) to simulate its dynamics — no curve is drawn rather than a fabricated one.`,
        "warning",
      ),
    );
    container.append(metrics(document, [["World", worldName], ["Variables", String(bundle.world.variables.length)], ["Trajectory", "not simulated"]]));
    return;
  }
  const view = trajectoryChartView(bundle.trajectory, `${worldName} trajectory`);
  container.append(notice(document, `Loaded viewer bundle for "${worldName}" — ${view.seriesCount} series, ${view.sampleCount} samples.`, "success"));
  container.append(renderChartSection(document, view));
}

/**
 * Renders the trajectory visualization region.
 *
 * With no pasted bundle (`rawBundle` null/empty) it draws the selected bundled
 * demo trajectory so the surface is demoable offline with no engine. When a
 * `ViewerBundle` is pasted it is parsed and rendered: a bundle with a trajectory
 * draws its real curve; a bundle without one shows an honest "connect the engine"
 * note; malformed input degrades to an error notice. Pure and deterministic.
 */
export function renderTrajectoryChart(
  document: Document,
  rawBundle: string | null,
  sampleId: SampleTrajectoryId = "damped-oscillator",
): HTMLElement {
  const container = node(document, "div", "lss-visualize-result");
  const trimmed = rawBundle === null ? "" : rawBundle.trim();

  if (trimmed.length === 0) {
    const view = trajectoryChartView(sampleTrajectory(sampleId), sampleTrajectoryTitle(sampleId));
    container.append(
      notice(document, `${sampleTrajectoryDescription(sampleId)} Bundled demo trajectory — no engine required. Load a .viewer.json bundle to visualize your own.`, "info"),
    );
    container.append(renderChartSection(document, view));
    return container;
  }

  let data: unknown;
  try {
    data = JSON.parse(trimmed);
  } catch (error) {
    container.append(notice(document, `That text is not valid JSON — ${error instanceof Error ? error.message : String(error)}`, "error"));
    return container;
  }

  let bundle: ViewerBundle;
  try {
    bundle = parseViewerBundle(data);
  } catch (error) {
    const message = error instanceof ViewerBundleError ? `That is not a valid viewer bundle — ${error.message}` : error instanceof Error ? error.message : String(error);
    container.append(notice(document, message, "error"));
    return container;
  }

  try {
    renderBundle(document, container, bundle);
  } catch (error) {
    container.append(notice(document, `Could not render this trajectory — ${error instanceof Error ? error.message : String(error)}`, "error"));
  }
  return container;
}
