import {
  SAMPLE_TRAJECTORY_IDS,
  isSampleTrajectoryId,
  renderTrajectoryChart,
  sampleTrajectory,
  sampleViewerBundleJson,
  trajectoryChartModel,
  trajectoryChartView,
} from "../src/visualize.js";
import { deepEqual, equal } from "./support.js";

// As with analysis_view.test.ts, there is no DOM library in this workspace — the
// custom dist/tests/run.js harness runs pure tests. `renderTrajectoryChart` is a
// pure builder that touches a thin slice of the DOM (createElement, and the
// namespaced createElementNS the app uses for SVG in screens/render.ts), so we
// back it with a minimal in-memory Document and assert on the produced tree.

class FakeElement {
  readonly tagName: string;
  className = "";
  #text = "";
  readonly attrs = new Map<string, string>();
  readonly children: FakeElement[] = [];
  readonly style: Record<string, string> = {};
  constructor(tag: string) {
    this.tagName = tag;
  }
  set textContent(value: string) {
    this.#text = value;
    this.children.length = 0;
  }
  get textContent(): string {
    return this.children.length === 0 ? this.#text : this.children.map((child) => child.textContent).join("");
  }
  setAttribute(key: string, value: string): void {
    this.attrs.set(key, String(value));
  }
  getAttribute(key: string): string | null {
    return this.attrs.get(key) ?? null;
  }
  append(...nodes: FakeElement[]): void {
    for (const node of nodes) this.children.push(node);
  }
  appendChild(node: FakeElement): FakeElement {
    this.children.push(node);
    return node;
  }
}

function fakeDocument(): Document {
  return {
    createElement: (tag: string) => new FakeElement(tag),
    createElementNS: (_ns: string, tag: string) => new FakeElement(tag),
  } as unknown as Document;
}

type El = FakeElement;
function walk(root: El): El[] {
  return [root, ...root.children.flatMap(walk)];
}
function byClass(root: El, cls: string): El[] {
  return walk(root).filter((el) => el.className.split(" ").includes(cls));
}
function byTag(root: El, tag: string): El[] {
  return walk(root).filter((el) => el.tagName === tag);
}
function hasText(root: El, needle: string): boolean {
  return root.textContent.includes(needle);
}

export async function visualizeTests(): Promise<void> {
  // --- bundled sample trajectory is a deterministic, RNG-free computation ---
  const damped = sampleTrajectory("damped-oscillator");
  deepEqual(damped.variables, ["x", "v"]);
  equal(damped.times.length, 121);
  equal(damped.values.length, 121);
  deepEqual(damped.values[0], [1, 0]);
  // Times are strictly increasing (normalizeTrajectory requires monotonicity).
  let previous = Number.NEGATIVE_INFINITY;
  for (const time of damped.times) { equal(time > previous, true); previous = time; }
  equal(damped.times[120], 12);
  // Recomputing yields identical output (pure, cached).
  deepEqual(sampleTrajectory("damped-oscillator").values[60], damped.values[60]);

  const lotka = sampleTrajectory("lotka-volterra");
  deepEqual(lotka.variables, ["prey", "predator"]);
  equal(lotka.times.length, 601);

  // --- chart model has one series per variable and a domain from the data ---
  const model = trajectoryChartModel(damped, "Damped oscillator");
  equal(model.title, "Damped oscillator");
  equal(model.series.length, 2);
  deepEqual(model.series.map((s) => s.id), ["x", "v"]);
  // The x-axis domain covers the trajectory's time span (padded).
  equal(model.xAxis.domain.min <= 0, true);
  equal(model.xAxis.domain.max >= 12, true);

  const view = trajectoryChartView(damped, "Damped oscillator");
  equal(view.seriesCount, 2);
  equal(view.sampleCount, 121);
  equal(view.duration, 12);
  equal(view.valueDomain.min < 0, true);

  // --- narrowing + labels ---
  equal(isSampleTrajectoryId("damped-oscillator"), true);
  equal(isSampleTrajectoryId("nope"), false);
  equal(SAMPLE_TRAJECTORY_IDS.length, 2);

  const doc = fakeDocument() as unknown as Document;
  const render = (raw: string | null, id?: "damped-oscillator" | "lotka-volterra"): El =>
    (id === undefined ? renderTrajectoryChart(doc, raw) : renderTrajectoryChart(doc, raw, id)) as unknown as El;

  // --- default (no bundle) draws the bundled demo curve, offline ---
  const demo = render(null);
  equal(byTag(demo, "svg").length, 1);
  equal(byTag(demo, "svg")[0]!.getAttribute("role"), "img");
  equal(byTag(demo, "path").length, 2); // one <path> per variable
  equal(byClass(demo, "lss-scr-legend-item").length, 2);
  equal(byClass(demo, "lss-tone-info").length, 1); // "bundled demo" notice
  equal(byClass(demo, "lss-tone-error").length, 0);
  equal(hasText(demo, "Bundled demo trajectory"), true);

  // --- the second sample renders its own two-series curve ---
  const orbit = render(null, "lotka-volterra");
  equal(byTag(orbit, "path").length, 2);
  equal(byClass(orbit, "lss-scr-legend-item").length, 2);
  equal(hasText(orbit, "prey"), true);

  // --- a real viewer bundle round-trips through parseViewerBundle and renders ---
  const bundle = render(sampleViewerBundleJson("damped-oscillator"));
  equal(byClass(bundle, "lss-tone-error").length, 0);
  equal(byTag(bundle, "path").length, 2);
  equal(byClass(bundle, "lss-tone-success").some((n) => hasText(n, "Loaded viewer bundle")), true);

  // --- honest "no engine" state: a world with no trajectory shows no curve ---
  const worldOnly = JSON.stringify({
    format: "lawsynth-viewer",
    version: 1,
    world: { id: "pendulum", name: "Pendulum", time: { kind: "continuous" }, variables: [{ id: "theta" }, { id: "omega" }], laws: [] },
  });
  const noEngine = render(worldOnly);
  equal(byTag(noEngine, "path").length, 0);
  equal(byTag(noEngine, "svg").length, 0);
  equal(byClass(noEngine, "lss-tone-error").length, 0);
  const warning = byClass(noEngine, "lss-tone-warning");
  equal(warning.length, 1);
  equal(hasText(noEngine, "Connect the LawSynth engine"), true);
  equal(hasText(noEngine, "no curve is drawn"), true);

  // --- invalid JSON → error notice with alert role, does not throw ---
  const badJson = render("{not json");
  const jsonNotice = byClass(badJson, "lss-scr-notice");
  equal(jsonNotice.length, 1);
  equal(jsonNotice[0]!.className.includes("lss-tone-error"), true);
  equal(jsonNotice[0]!.getAttribute("role"), "alert");
  equal(hasText(badJson, "not valid JSON"), true);

  // --- valid JSON, wrong shape → ViewerBundleError surfaced, not thrown ---
  const badShape = render(`{"format":"lawsynth-viewer","version":1,"world":{"id":""}}`);
  const shapeNotice = byClass(badShape, "lss-tone-error");
  equal(shapeNotice.length, 1);
  equal(hasText(badShape, "not a valid viewer bundle"), true);
}
