import { codeBlock, type CodeBlock } from "./code.js";
import { codeFence, frontMatter, markdownDocument } from "./content_markdown.js";
import type { DocumentedExample } from "./examples.js";
import type { NavigationPage } from "./navigation.js";
import type { DocumentationPageSource } from "./site.js";

export type GalleryCategory = "chaos" | "ecology" | "mechanics" | "epidemiology";

export interface RecoveredLaw {
  /** Left-hand side, e.g. `dx/dt`. */
  readonly target: string;
  /** Right-hand side as recovered, e.g. `σ (y − x)`. */
  readonly expression: string;
}

/**
 * One canonical system in the examples gallery. It bundles the short pitch, the
 * exact `lawsynth discover` command, and the law system a clean run recovers —
 * the three things a reader needs to reproduce and trust the result.
 */
export interface GalleryEntry {
  readonly id: string;
  readonly title: string;
  /** Path segment under `/examples/`, e.g. `lorenz`. */
  readonly slug: string;
  readonly category: GalleryCategory;
  readonly summary: string;
  readonly datasetFile: string;
  readonly timeColumn: string;
  readonly stateColumns: readonly string[];
  readonly discoverCommand: string;
  readonly recoveredLaws: readonly RecoveredLaw[];
  readonly recoveredParameters: readonly string[];
  readonly tags: readonly string[];
  readonly order: number;
}

function discover(entry: Pick<GalleryEntry, "datasetFile" | "timeColumn" | "stateColumns" | "slug">): string {
  return `lawsynth discover ${entry.datasetFile} --time ${entry.timeColumn} --state ${entry.stateColumns.join(",")} --output ${entry.slug}.lsworld`;
}

/** The canonical systems shipped with the documentation gallery. */
export const GALLERY_ENTRIES: readonly GalleryEntry[] = Object.freeze([
  {
    id: "lorenz",
    title: "Lorenz system",
    slug: "lorenz",
    category: "chaos",
    summary: "A three-variable convection model and the textbook example of deterministic chaos and a strange attractor.",
    datasetFile: "lorenz.csv",
    timeColumn: "t",
    stateColumns: ["x", "y", "z"],
    discoverCommand: discover({ datasetFile: "lorenz.csv", timeColumn: "t", stateColumns: ["x", "y", "z"], slug: "lorenz" }),
    recoveredLaws: [
      { target: "dx/dt", expression: "σ (y − x)" },
      { target: "dy/dt", expression: "x (ρ − z) − y" },
      { target: "dz/dt", expression: "x y − β z" },
    ],
    recoveredParameters: ["σ ≈ 10.0", "ρ ≈ 28.0", "β ≈ 2.667"],
    tags: ["chaos", "attractor", "3d"],
    order: 2,
  },
  {
    id: "lotka-volterra",
    title: "Lotka–Volterra",
    slug: "lotka-volterra",
    category: "ecology",
    summary: "Predator–prey dynamics recovered from the classic Hudson's Bay lynx–hare pelt record: coupled populations that cycle out of phase.",
    datasetFile: "lynx-hare.csv",
    timeColumn: "year",
    stateColumns: ["hare", "lynx"],
    discoverCommand: discover({ datasetFile: "lynx-hare.csv", timeColumn: "year", stateColumns: ["hare", "lynx"], slug: "lotka-volterra" }),
    recoveredLaws: [
      { target: "d(hare)/dt", expression: "α·hare − β·hare·lynx" },
      { target: "d(lynx)/dt", expression: "δ·hare·lynx − γ·lynx" },
    ],
    recoveredParameters: ["α ≈ 0.55", "β ≈ 0.028", "δ ≈ 0.026", "γ ≈ 0.84"],
    tags: ["ecology", "predator-prey", "oscillation"],
    order: 3,
  },
  {
    id: "damped-oscillator",
    title: "Damped oscillator",
    slug: "damped-oscillator",
    category: "mechanics",
    summary: "A mass on a spring bleeding energy to friction — the simplest system where a recovered parameter (ζ) maps directly to a physical regime.",
    datasetFile: "oscillator.csv",
    timeColumn: "t",
    stateColumns: ["x", "v"],
    discoverCommand: discover({ datasetFile: "oscillator.csv", timeColumn: "t", stateColumns: ["x", "v"], slug: "damped-oscillator" }),
    recoveredLaws: [
      { target: "dx/dt", expression: "v" },
      { target: "dv/dt", expression: "−ω² x − 2 ζ ω v" },
    ],
    recoveredParameters: ["ω ≈ 2.0", "ζ ≈ 0.15"],
    tags: ["mechanics", "pendulum", "damping"],
    order: 4,
  },
  {
    id: "van-der-pol",
    title: "Van der Pol oscillator",
    slug: "van-der-pol",
    category: "mechanics",
    summary: "A nonlinear oscillator with a self-sustaining limit cycle — the amplitude-dependent damping term is where symbolic search earns its keep.",
    datasetFile: "vanderpol.csv",
    timeColumn: "t",
    stateColumns: ["x", "v"],
    discoverCommand: discover({ datasetFile: "vanderpol.csv", timeColumn: "t", stateColumns: ["x", "v"], slug: "van-der-pol" }),
    recoveredLaws: [
      { target: "dx/dt", expression: "v" },
      { target: "dv/dt", expression: "μ (1 − x²) v − x" },
    ],
    recoveredParameters: ["μ ≈ 1.0"],
    tags: ["mechanics", "limit-cycle", "nonlinear"],
    order: 5,
  },
  {
    id: "sir",
    title: "SIR epidemic",
    slug: "sir",
    category: "epidemiology",
    summary: "The compartmental susceptible–infected–recovered model recovered from a synthetic outbreak curve, including the mass-action infection term.",
    datasetFile: "outbreak.csv",
    timeColumn: "day",
    stateColumns: ["S", "I", "R"],
    discoverCommand: discover({ datasetFile: "outbreak.csv", timeColumn: "day", stateColumns: ["S", "I", "R"], slug: "sir" }),
    recoveredLaws: [
      { target: "dS/dt", expression: "−β S I / N" },
      { target: "dI/dt", expression: "β S I / N − γ I" },
      { target: "dR/dt", expression: "γ I" },
    ],
    recoveredParameters: ["β ≈ 0.30", "γ ≈ 0.10", "R₀ = β/γ ≈ 3.0"],
    tags: ["epidemiology", "compartmental", "sir"],
    order: 6,
  },
]);

/** The discovery command for a gallery entry as a highlighted `CodeBlock`. */
export function galleryDiscoverBlock(entry: GalleryEntry): CodeBlock {
  return codeBlock(`$ ${entry.discoverCommand}`, "bash", `Discover the ${entry.title.toLowerCase()} world`);
}

/** The recovered law system for a gallery entry as a highlighted `CodeBlock`. */
export function galleryLawBlock(entry: GalleryEntry): CodeBlock {
  const body = entry.recoveredLaws.map((law) => `${law.target} = ${law.expression}`).join("\n");
  return codeBlock(body, "text", "Recovered law system");
}

/** Projects a gallery entry into an `ExampleRegistry`-compatible documented example. */
export function galleryEntryToExample(entry: GalleryEntry): DocumentedExample {
  return Object.freeze({
    id: entry.id,
    title: entry.title,
    description: entry.summary,
    language: "bash",
    source: `$ ${entry.discoverCommand}`,
    runnable: true,
    capabilities: Object.freeze(["discover", entry.category]),
  });
}

/** The gallery index page plus one page per canonical system, ready for `compileSite`. */
export function galleryPages(): readonly DocumentationPageSource[] {
  const index: DocumentationPageSource = {
    path: "/examples",
    section: "examples",
    source: markdownDocument(
      frontMatter({ title: "Examples gallery", description: "Canonical dynamical systems recovered end-to-end with a single discovery command.", order: 1, tags: ["examples", "gallery"] }),
      "# Examples gallery",
      "Each entry is a canonical system with a short description, the exact `lawsynth discover` command that recovers it, and the law system a clean run produces. Every result is a portable `.lsworld` bundle you can `explain`, `forecast`, and `report` on.",
      ...GALLERY_ENTRIES.map((entry) => `- **[${entry.title}](/examples/${entry.slug})** — ${entry.summary}`),
    ),
  };

  const pages = GALLERY_ENTRIES.map((entry): DocumentationPageSource => ({
    path: `/examples/${entry.slug}`,
    section: "examples",
    source: markdownDocument(
      frontMatter({ title: entry.title, description: entry.summary, order: entry.order, tags: ["examples", ...entry.tags] }),
      `# ${entry.title}`,
      entry.summary,
      "## Discover",
      "Recover the world from observations with a single deterministic command:",
      codeFence("bash", `$ ${entry.discoverCommand}`),
      "## Recovered law system",
      "A clean run recovers the following continuous laws:",
      codeFence("text", entry.recoveredLaws.map((law) => `${law.target} = ${law.expression}`).join("\n")),
      `Fitted parameters: ${entry.recoveredParameters.join(", ")}.`,
      "## Next",
      "Inspect and use the bundle:",
      codeFence("bash", [
        `$ lawsynth explain ${entry.slug}.lsworld`,
        `$ lawsynth forecast ${entry.slug}.lsworld --horizon 40 --step 0.05 --output ${entry.slug}-forecast.csv`,
        `$ lawsynth report ${entry.slug}.lsworld --output ${entry.slug}.html`,
      ].join("\n")),
    ),
  }));

  return Object.freeze([index, ...pages]);
}

/** The gallery as explicit navigation pages, keeping `navigation.ts`'s `NavigationPage` shape. */
export function galleryNavigationPages(): readonly NavigationPage[] {
  return Object.freeze([
    { path: "/examples", title: "Examples gallery", section: "examples", order: 1 },
    ...GALLERY_ENTRIES.map((entry): NavigationPage => ({ path: `/examples/${entry.slug}`, title: entry.title, section: "examples", order: entry.order })),
  ]);
}
