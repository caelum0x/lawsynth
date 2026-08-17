import { codeFence, frontMatter, markdownDocument } from "./content_markdown.js";
import { ExampleRegistry, type DocumentedExample } from "./examples.js";
import { galleryEntryToExample, galleryPages, GALLERY_ENTRIES } from "./gallery.js";
import { compileSite, type DocumentationPageSource, type DocumentationSite, type SiteConfiguration } from "./site.js";

/** The product's front page: what LawSynth is and the loop it runs. */
const INTRODUCTION: DocumentationPageSource = {
  path: "/",
  section: "getting-started",
  source: markdownDocument(
    frontMatter({ title: "What is LawSynth?", description: "LawSynth turns time-series observations into executable mathematical worlds — interpretable law systems you can read, simulate, and share.", order: 1, tags: ["overview", "introduction"] }),
    "# What is LawSynth?",
    "LawSynth turns time-series observations into **executable mathematical worlds**: interpretable law systems you can read, simulate, stress-test, and share. If you can't read and reason about the result, it isn't a LawSynth result.",
    "## The core loop",
    codeFence("text", "observe (CSV)  →  discover (laws)  →  understand (explain)  →  use (forecast / intervene)  →  share (report / .lsworld)"),
    "Every step is deterministic and offline. A discovery is a portable `.lsworld` bundle; everything downstream operates on that one artifact.",
    "> Local-first and reproducible: the same inputs produce the same world, offline, forever.",
    "Continue with the [Getting started](/getting-started) walkthrough, browse the [Examples gallery](/examples), or read the [Core concepts](/concepts).",
  ),
};

/** The end-to-end walkthrough: install → discover → explain → forecast → report. */
const GETTING_STARTED: DocumentationPageSource = {
  path: "/getting-started",
  section: "getting-started",
  source: markdownDocument(
    frontMatter({ title: "Getting started", description: "Install LawSynth and go from a CSV to a discovered, explained, forecast, and shareable world in five commands.", order: 2, tags: ["guide", "quickstart", "cli", "sdk"] }),
    "# Getting started",
    "This walkthrough takes you through the whole loop — **install, discover, understand, use, share** — with the real CLI commands and the equivalent Python SDK. Everything runs locally; no data leaves your machine.",
    "## 1. Install",
    "Install the CLI with Cargo, or the Python SDK with pip:",
    codeFence("bash", "$ cargo install lawsynth-cli\n$ pip install lawsynth"),
    "## 2. Discover",
    "Point `discover` at a CSV, name the time column and the state variables whose dynamics you want to model, and write a `.lsworld` bundle:",
    codeFence("bash", "$ lawsynth discover obs.csv --time t --state x,y --output world.lsworld"),
    "The same run from the Python SDK, using the fluent `Study` façade:",
    codeFence("python", "import lawsynth\n\nstudy = lawsynth.Study.from_csv(\"obs.csv\", time=\"t\", state=[\"x\", \"y\"])\nresult = study.discover()\n\nfor target, equation in result.equations.items():\n    print(f\"d{target}/dt = {equation}\")"),
    "Discovery is tunable: add `--regimes` to detect regime switches, `--pareto` to report the accuracy/complexity frontier, `--refine` to jointly fit parameters, and `--causal` for dependency hypotheses.",
    "## 3. Understand",
    "`explain` turns a world into meaning: a plain-language description of each law, the dominant terms, discovered regimes, fit quality, and the assumptions a result is contingent on.",
    codeFence("bash", "$ lawsynth explain world.lsworld"),
    codeFence("python", "explanation = result.explain()\nprint(explanation.to_text())"),
    "## 4. Use",
    "`forecast` runs the world forward beyond the observed window. Override parameters, set initial values, and schedule interventions to ask what-if:",
    codeFence("bash", "$ lawsynth forecast world.lsworld --horizon 40 --step 0.05 \\\n    --initial x=1.0 --intervene y=0.5@20 --output forecast.csv"),
    codeFence("python", "forecast = result.forecast({}, horizon=40, step=0.05)"),
    "## 5. Share",
    "`report` renders a self-contained HTML report — rendered equations, fit and Pareto candidates, regime timeline, uncertainty bands, and inline SVG trajectory charts. No server, no external assets: one file a colleague can open.",
    codeFence("bash", "$ lawsynth report world.lsworld --output report.html"),
    codeFence("python", "result.report(\"report.html\")\nresult.save(\"world.lsworld\")  # the portable bundle everything else operates on"),
    "## Next steps",
    "- Reproduce a canonical system from the [Examples gallery](/examples).",
    "- Learn the vocabulary in [Core concepts](/concepts).",
    "- Diff two worlds or two scenarios with `lawsynth compare`, and keep a workspace navigable with `lawsynth library`.",
  ),
};

/** The concept vocabulary: World IR, laws, bundles, regimes, uncertainty. */
const CONCEPTS: DocumentationPageSource = {
  path: "/concepts",
  section: "concepts",
  source: markdownDocument(
    frontMatter({ title: "Core concepts", description: "The vocabulary behind LawSynth: the World IR, laws, .lsworld bundles, regimes, and uncertainty.", order: 1, tags: ["concepts", "world-ir", "reference"] }),
    "# Core concepts",
    "A handful of ideas recur across the CLI, SDK, Studio, and services. They all operate on the same validated representation.",
    "## World IR",
    "The **World IR** is a single typed, executable representation of a dynamical system. Instead of a private object graph per algorithm, every stage — discovery, explanation, simulation, reporting — exchanges the same document: variables, laws, parameters, dependencies, regimes, and uncertainty in one place.",
    "## Laws",
    "A **law** is one equation with a kind. A `continuous` law defines a state variable's time derivative (`dx/dt = …`); other kinds cover discrete updates, algebraic constraints, observations, and events. Each law carries a target and an expression tree built from constants, symbols, and a fixed operator set — the same tree the Rust core evaluates.",
    "## .lsworld bundles",
    "A discovery is a portable **`.lsworld` bundle**: a validated World IR plus its provenance. It is the unit of exchange — `explain`, `forecast`, `compare`, and `report` all take a bundle in. Same bundle, same result, on any machine, offline.",
    "> Composable: CLI, SDK, Studio, and services all read and write the same `.lsworld` bundle.",
    "## Regimes",
    "Real systems switch behavior. A **regime** is a distinct mode with its own effective dynamics — a spring that yields, a market that flips, a population that crashes. Discovery can detect regime boundaries (`--regimes`) and surface them as a timeline so a single world can honestly describe a piecewise process.",
    "## Uncertainty",
    "LawSynth is honest about what it does not know. **Uncertainty** is first-class: parameter confidence, bootstrap bounds, and the assumptions a result depends on are recorded on the world and rendered as bands in forecasts and reports — never hidden.",
  ),
};

/** Every content page this site renders, in navigation order. */
export function docsContentPages(): readonly DocumentationPageSource[] {
  return Object.freeze([INTRODUCTION, GETTING_STARTED, CONCEPTS, ...galleryPages()]);
}

/** The getting-started CLI/SDK snippets as documented examples. */
const GETTING_STARTED_EXAMPLES: readonly DocumentedExample[] = Object.freeze([
  { id: "cli-install", title: "Install LawSynth", description: "Install the CLI with Cargo or the Python SDK with pip.", language: "bash", source: "$ cargo install lawsynth-cli\n$ pip install lawsynth", runnable: true, capabilities: ["install"] },
  { id: "cli-discover", title: "Discover a world", description: "Recover a world from a CSV, naming the time and state columns.", language: "bash", source: "$ lawsynth discover obs.csv --time t --state x,y --output world.lsworld", runnable: true, capabilities: ["discover"] },
  { id: "sdk-quickstart", title: "SDK quickstart", description: "The discover → explain → forecast → report loop from the Python SDK.", language: "python", source: "import lawsynth\n\nstudy = lawsynth.Study.from_csv(\"obs.csv\", time=\"t\", state=[\"x\", \"y\"])\nresult = study.discover()\nprint(result.explain().to_text())\nforecast = result.forecast({}, horizon=40)\nresult.report(\"report.html\")", runnable: true, capabilities: ["discover", "explain", "forecast", "report"] },
  { id: "cli-explain", title: "Explain a world", description: "Turn a bundle into a plain-language, structured explanation.", language: "bash", source: "$ lawsynth explain world.lsworld", runnable: true, capabilities: ["explain"] },
  { id: "cli-forecast", title: "Forecast forward", description: "Run a world beyond the observed window with an intervention.", language: "bash", source: "$ lawsynth forecast world.lsworld --horizon 40 --step 0.05 --intervene y=0.5@20 --output forecast.csv", runnable: true, capabilities: ["forecast", "intervene"] },
  { id: "cli-report", title: "Share a report", description: "Render a self-contained HTML report from a bundle.", language: "bash", source: "$ lawsynth report world.lsworld --output report.html", runnable: true, capabilities: ["report"] },
]);

/** An `ExampleRegistry` holding the getting-started snippets and every gallery discovery command. */
export function docsExampleRegistry(): ExampleRegistry {
  const registry = new ExampleRegistry();
  for (const example of GETTING_STARTED_EXAMPLES) registry.add(example);
  for (const entry of GALLERY_ENTRIES) registry.add(galleryEntryToExample(entry));
  return registry;
}

/**
 * Compiles the full documentation site — introduction, getting-started,
 * concepts, and the examples gallery — through the existing `compileSite`
 * pipeline, which builds navigation and the search index from these pages.
 */
export function buildDocsSite(configuration: SiteConfiguration): DocumentationSite {
  return compileSite(docsContentPages(), configuration);
}
