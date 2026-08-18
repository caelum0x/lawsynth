import { ANALYSIS_REPORTS, analysisSample, renderAnalysisReport } from "../src/analysis.js";
import { equal } from "./support.js";

// There is no DOM library in this workspace (tests are pure and run through the
// custom dist/tests/run.js harness, not jsdom). `renderAnalysisReport` is a pure
// element builder that only touches a tiny slice of the DOM API, so we back it
// with a minimal in-memory `Document` and assert on the produced tree. This keeps
// the render path — parse → view model → DOM — under test with zero dependencies.

class FakeElement {
  readonly tagName: string;
  className = "";
  #text = "";
  readonly attrs = new Map<string, string>();
  readonly children: FakeElement[] = [];
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
  return { createElement: (tag: string) => new FakeElement(tag) } as unknown as Document;
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

// VERBATIM `lawsynth stability --json` output (copied from the world-schema fixtures).
const STABILITY_CENTER = `{
  "world": "osc.lsworld",
  "states": ["v", "x"],
  "seeds_total": 25,
  "seeds_converged": 25,
  "fixed_points": [
    {
      "coordinates": [0.0, 0.0],
      "classification": "center (marginal, inconclusive)",
      "inconclusive": true,
      "eigenvalues": [{"re": 0.0, "im": -0.999984274058213995}, {"re": 0.0, "im": 0.999984274058213329}]
    }
  ]
}`;

const STABILITY_EMPTY = `{
  "world": "linear-drift.lsworld",
  "states": ["x", "y"],
  "seeds_total": 25,
  "seeds_converged": 4,
  "fixed_points": []
}`;

export async function analysisViewTests(): Promise<void> {
  const doc = fakeDocument() as unknown as Document;
  const render = (report: (typeof ANALYSIS_REPORTS)[number], json: string | null): El =>
    renderAnalysisReport(doc, report, json) as unknown as El;

  // --- placeholder: no submission yet ---
  const empty = render("stability", null);
  equal(byClass(empty, "lss-scr-empty").length, 1);
  equal(hasText(empty, "Load sample"), true);

  // --- bundled sample (stable node) → definitive success verdict ---
  const sample = render("stability", analysisSample("stability"));
  equal(byTag(sample, "table").length, 1);
  const bodyRows = byTag(byTag(sample, "tbody")[0]!, "tr");
  equal(bodyRows.length, 1);
  const classBadges = byClass(sample, "lss-badge");
  // classification badge "Stable node" (success) + verdict badge "Stable" (success).
  equal(classBadges.some((b) => b.className.includes("lss-tone-success") && b.textContent === "Stable node"), true);
  equal(classBadges.some((b) => b.className.includes("lss-tone-success") && b.textContent === "Stable"), true);
  // real eigenvalue, formatted with no imaginary part.
  equal(hasText(sample, "-2.04167"), true);
  equal(byClass(sample, "lss-scr-notice").some((n) => n.className.includes("lss-tone-success") && hasText(n, "1 fixed point")), true);
  equal(byClass(sample, "lss-tone-error").length, 0);

  // --- verbatim center → inconclusive, warning tone, a ± b i eigenvalue ---
  const center = render("stability", STABILITY_CENTER);
  const centerBadges = byClass(center, "lss-badge");
  equal(centerBadges.some((b) => b.className.includes("lss-tone-warning") && b.textContent === "Center (marginal, inconclusive)"), true);
  equal(centerBadges.some((b) => b.className.includes("lss-tone-warning") && b.textContent.includes("Inconclusive")), true);
  equal(hasText(center, "0 + 0.999984 i"), true);
  // an inconclusive point is never shown as a definitive Stable/Unstable verdict.
  equal(centerBadges.some((b) => b.textContent === "Stable" || b.textContent === "Unstable"), false);

  // --- empty box → honest "none found" state, not an error ---
  const none = render("stability", STABILITY_EMPTY);
  equal(byClass(none, "lss-scr-empty").some((n) => hasText(n, "No fixed points found in the searched region.")), true);
  equal(byTag(none, "tbody").length, 0);
  equal(byClass(none, "lss-tone-error").length, 0);

  // --- invalid JSON → error notice with alert role, does not throw ---
  const badJson = render("stability", "{not json");
  const jsonNotice = byClass(badJson, "lss-scr-notice");
  equal(jsonNotice.length, 1);
  equal(jsonNotice[0]!.className.includes("lss-tone-error"), true);
  equal(jsonNotice[0]!.getAttribute("role"), "alert");
  equal(hasText(badJson, "not valid JSON"), true);

  // --- valid JSON, wrong shape → SchemaValidationError message surfaced ---
  const badShape = render("stability", `{"world": "w"}`);
  const shapeNotice = byClass(badShape, "lss-tone-error");
  equal(shapeNotice.length, 1);
  equal(hasText(badShape, "Schema validation failed"), true);
  equal(hasText(badShape, "fixed_points"), true);

  // --- bifurcation sample → a Hopf row with a complex eigenvalue ---
  const bifurcation = render("bifurcation", analysisSample("bifurcation"));
  equal(byClass(bifurcation, "lss-tone-error").length, 0);
  equal(hasText(bifurcation, "Hopf"), true);
  equal(byTag(byTag(bifurcation, "tbody")[0]!, "tr").length, 1);

  // --- estimate sample (pole placement) → honest "no covariance" note ---
  const estimate = render("estimate", analysisSample("estimate"));
  equal(byClass(estimate, "lss-tone-error").length, 0);
  equal(byClass(estimate, "lss-scr-empty").some((n) => hasText(n, "No steady-state covariance")), true);

  // --- every bundled sample parses and renders without an error notice ---
  for (const report of ANALYSIS_REPORTS) {
    const rendered = render(report, analysisSample(report));
    equal(byClass(rendered, "lss-tone-error").length, 0);
  }
}
