import { codeFence, frontMatter, markdownDocument } from "./content_markdown.js";
import { ExampleRegistry, type DocumentedExample } from "./examples.js";
import { galleryEntryToExample, galleryPages, GALLERY_ENTRIES } from "./gallery.js";
import { compileSite, type DocumentationPageSource, type DocumentationSite, type SiteConfiguration } from "./site.js";

/** The product's front page: what LawSynth is and the loop it runs. */
const INTRODUCTION: DocumentationPageSource = {
  path: "/",
  section: "getting-started",
  source: markdownDocument(
    frontMatter({ title: "What is LawSynth?", description: "LawSynth discovers interpretable, executable law systems from time-series data — deterministic and offline. Read, simulate, analyze, control, and share them.", order: 1, tags: ["overview", "introduction"] }),
    "# What is LawSynth?",
    "LawSynth discovers **interpretable, executable law systems** from time-series data. Point it at a CSV and it recovers the governing equations behind the numbers — a world you can read, simulate, analyze, control, and share. If you can't read and reason about the result, it isn't a LawSynth result.",
    "## What you get",
    "- **Explicit laws, not black boxes.** Discovery returns equations like `dx/dt = σ (y − x)` with fitted parameters — not opaque weights.",
    "- **Deterministic and offline.** The same inputs produce a bit-identical world, on any machine, with no network access. Reproducibility is a [contract](/determinism), not a hope.",
    "- **One portable artifact.** Every discovery is a `.lsworld` bundle, and every downstream tool operates on that single file.",
    "## The core loop",
    codeFence("text", "observe (CSV)  →  discover (laws)  →  understand (explain)  →  use (forecast / analyze / control)  →  share (report / export / .lsworld)"),
    "Each step is a real command in the `lawsynth` CLI, mirrored by the Python and TypeScript SDKs. Discovery is a sparse fit to your data — a compact hypothesis, not proof of causality — and every world records the uncertainty and assumptions behind it.",
    "> Local-first and reproducible: the same inputs produce the same world, offline, forever.",
    "## Where to go next",
    "- Run the whole loop in five commands: [Getting started](/getting-started).",
    "- See everything LawSynth can do: [Capabilities](/capabilities).",
    "- Understand the differentiator: [Why determinism](/determinism).",
    "- Reproduce a canonical system: [Examples gallery](/examples).",
    "- Learn the vocabulary: [Core concepts](/concepts).",
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
    "- Go beyond the loop: the [Capabilities](/capabilities) page covers analysis (`stability`, `lyapunov`, `basins`, `invariants`, `bifurcation`, `sensitivity`) and control (`estimate`, `reduce`, `mpc`).",
    "- Diff two worlds or two scenarios with `lawsynth compare`, and keep a workspace navigable with `lawsynth library`.",
  ),
};

/** The capabilities overview: what LawSynth can do, by pillar, with the real command for each. */
const CAPABILITIES: DocumentationPageSource = {
  path: "/capabilities",
  section: "capabilities",
  source: markdownDocument(
    frontMatter({ title: "Capabilities", description: "Everything LawSynth can do, organized by pillar — discover laws, analyze them, design control, and share results — each with the real CLI command and SDK call.", order: 1, tags: ["capabilities", "cli", "sdk", "reference"] }),
    "# Capabilities",
    "LawSynth is a Rust-first toolkit with Python and TypeScript SDKs. Every capability below is a real command in the `lawsynth` CLI, and most are mirrored by a function in the Python SDK. They all read and write the same `.lsworld` bundle.",
    "The work falls into four pillars: **Discover**, **Analyze**, **Control**, and **Share**.",
    "## Discover",
    "Recover governing equations from observations by sparse symbolic regression over a configurable feature library.",
    "- **Sparse regression** — fit a compact law system with a choice of solver (`stlsq`, `sr3`, `frols`, `ssr`, or `trapping`), polynomial degree, and optional trigonometric or bounded-rational features.",
    codeFence("bash", "$ lawsynth discover obs.csv --time t --state x,y --output world.lsworld --solver stlsq --pareto"),
    "- **Regimes, Pareto, refinement, causality** — add `--regimes` to detect mode switches, `--pareto` for the accuracy/complexity frontier, `--refine` to jointly fit parameters, and `--causal` for dependency hypotheses.",
    "- **Controlled discovery (SINDYc)** — learn dynamics that include exogenous control inputs.",
    codeFence("bash", "$ lawsynth control obs.csv --time t --state x,v --control u"),
    "- **Network coupling** — recover which variables drive which across a multivariate system.",
    codeFence("bash", "$ lawsynth network obs.csv --state x1,x2,x3 --edge-threshold 0.1"),
    "- **Cross-validated model selection** — sweep degrees and thresholds under time-series cross-validation to choose a model that generalizes.",
    codeFence("bash", "$ lawsynth select obs.csv --state x,y --degrees 2,3 --folds 5 --scheme rolling"),
    "The same discovery flow from Python, through the `Study` façade:",
    codeFence("python", "import lawsynth\n\nstudy = lawsynth.Study.from_csv(\"obs.csv\", time=\"t\", state=[\"x\", \"y\"])\nresult = study.discover()\nresult.save(\"world.lsworld\")"),
    "## Analyze",
    "Interrogate a discovered world — its structure, its long-run behavior, and how far to trust it.",
    "- **Stability** — locate and classify fixed points over a bounded region: `lawsynth stability world.lsworld --box 0:5,0:5`.",
    "- **Bifurcation** — sweep a parameter and track how fixed points appear, merge, or lose stability: `lawsynth bifurcation world.lsworld --parameter mu --range 0:2 --box -3:3,-3:3`.",
    "- **Sensitivity** — measure how the trajectory responds to each parameter: `lawsynth sensitivity world.lsworld --parameters a,b`.",
    "- **Lyapunov exponents** — a chaos diagnostic; a positive leading exponent signals sensitive dependence: `lawsynth lyapunov world.lsworld --initial x=1,y=1,z=1`.",
    "- **Invariants** — search a bounded basis for conserved quantities: `lawsynth invariants world.lsworld --degree 2`.",
    "- **Basins of attraction** — map which initial conditions flow to which attractor: `lawsynth basins world.lsworld --box -2:2,-2:2`.",
    "- **Uncertainty** — bootstrap coefficient bounds at discovery time (`discover --bootstrap`) and propagate them into forecast bands (`forecast --confidence`).",
    "The analysis surface is mirrored in the SDK:",
    codeFence("python", "import lawsynth\n\nfixed_points = lawsynth.stability(\"world.lsworld\", box=\"0:5,0:5\")\nspectrum = lawsynth.lyapunov(\"world.lsworld\", initial={\"x\": 1.0, \"y\": 1.0, \"z\": 1.0})"),
    "## Control",
    "Turn a world into something you can estimate, reduce, or steer.",
    "- **State estimation** — design an observer by pole placement, or a Kalman filter: `lawsynth estimate world.lsworld --box -2:2,-2:2 --measure x --kalman`.",
    "- **Balanced model reduction** — approximate a higher-order world with a lower-order one: `lawsynth reduce world.lsworld --box -2:2,-2:2 --order 2`.",
    "- **Model-predictive control** — compute a control sequence that drives states to a setpoint: `lawsynth mpc world.lsworld --control u --setpoint x=1 --initial x=0`.",
    "- **Discrete-time simulation** — step a discrete-time world forward: `lawsynth simulate-discrete world.lsworld --initial x=1 --steps 100`.",
    codeFence("python", "import lawsynth\n\nplan = lawsynth.mpc(\"world.lsworld\", control=[\"u\"], setpoint={\"x\": 1.0}, initial={\"x\": 0.0})"),
    "## Share",
    "Explain, package, and hand off a world — no server, no external assets.",
    "- **Explain** — a plain-language, structured account of each law, its dominant terms, and its assumptions: `lawsynth explain world.lsworld`.",
    "- **Report** — a self-contained HTML report with rendered equations, fit, and inline trajectory charts: `lawsynth report world.lsworld --output report.html`.",
    "- **Forecast** — run the world forward, with interventions and optional confidence bands: `lawsynth forecast world.lsworld --horizon 40`.",
    "- **Export** — emit runnable code and interchange formats: `lawsynth export world.lsworld --format python` (also `c`, `onnx`, `matlab`, `latex`, `json`).",
    "- **Simplify** — algebraically reduce a law system with an e-graph: `lawsynth simplify world.lsworld`.",
    "- **Compare** — diff two worlds or two scenarios: `lawsynth compare a.lsworld b.lsworld`.",
    "- **Domain presets** — inspect and self-validate curated textbook systems: `lawsynth domains run damped-oscillator`.",
    "## Honest boundaries",
    "LawSynth is built to be trusted, which means being explicit about what it does not claim:",
    "- Discovery is a **sparse fit** to your data — a compact hypothesis, not proof of causality. `--causal` produces dependency *hypotheses*.",
    "- **Network coupling** is correlational; an edge is not proof of mechanism.",
    "- **Lyapunov** exponents are a time-averaged numerical estimate, not an analytic result.",
    "- **Invariants** are found within a bounded basis, so the absence of a result is not proof none exist.",
    "- **Determinism** is the guarantee: identical inputs, config, version, and binary produce a bit-identical world. See [Why determinism](/determinism).",
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
    "## Determinism",
    "The same inputs produce a bit-identical world, every time. This reproducibility guarantee is what makes a discovered world citable, and it is the property the rest of the system is built on. See [Why determinism](/determinism).",
  ),
};

/** The differentiator: bit-identical reproducibility as a contract, stated honestly. */
const DETERMINISM: DocumentationPageSource = {
  path: "/determinism",
  section: "concepts",
  source: markdownDocument(
    frontMatter({ title: "Why determinism", description: "LawSynth's differentiator: discovery is a fixed, seeded computation that produces a bit-identical .lsworld bundle every time — an auditable, reproducible result rather than a one-off.", order: 2, tags: ["determinism", "reproducibility", "concepts"] }),
    "# Why determinism",
    "Determinism is LawSynth's core differentiator. Discovery is a fixed, seeded computation: the same observations and the same configuration produce a **byte-identical** `.lsworld` bundle, every time, on any machine — no wall-clock reads, no unseeded randomness, no network. This is what makes a discovered world citable and auditable rather than a one-off result you can never reproduce.",
    "## What it enables",
    "- **Auditable science.** A world is a fixed function of its inputs, so a reviewer can rerun discovery and get the identical bundle — down to the bytes.",
    "- **Reproducible papers.** Cite a world by the digest of its bundle; anyone with the same data and version reproduces exactly that result.",
    "- **Trustworthy diffs.** Because bundles are stable, `lawsynth compare` differences reflect real changes in the science, not run-to-run noise.",
    "## Demonstrate it: two runs, one file",
    "Run discovery twice into different output paths, then compare the bytes — the digests match and `cmp` reports no differences:",
    codeFence("bash", "$ lawsynth discover lotka-volterra.csv --time time --state x,y --preset ecology --output a.lsworld\n$ lawsynth discover lotka-volterra.csv --time time --state x,y --preset ecology --output b.lsworld\n$ cmp a.lsworld b.lsworld && echo BYTE-IDENTICAL"),
    "Simulation is deterministic too: `lawsynth simulate` and `lawsynth forecast` print trajectories at full precision so downstream tooling can diff them exactly, and `forecast --confidence` takes an explicit `--seed` so even its bootstrap band is reproducible.",
    "## Self-validating domain presets",
    "The curated domain presets are a built-in, deterministic round-trip: synthesize a textbook law's clean trajectory, discover from it, and report the per-state error against the reference. It runs with no RNG and no clock, and doubles as a self-test.",
    codeFence("bash", "$ lawsynth domains run damped-oscillator"),
    "The honest caveat ships with the output: a good round-trip validates that the preset's search space contains the reference law, not that discovery is robust to real measurement noise.",
    "## What is — and isn't — guaranteed",
    "- **Guaranteed:** identical inputs, identical config, identical algorithm version, and identical binary produce a byte-identical `.lsworld` and identical printed trajectories.",
    "- **Not claimed:** cross-version stability. A different LawSynth version may change the algorithm and therefore the bytes; the reproducibility contract versions the algorithm so a digest is always read against a known version.",
    "- **Hardware caveat:** floating-point results can differ across fundamentally different hardware or math libraries, so the contract documents a hardware class.",
    "> Determinism is the guarantee. Everything else on the [Capabilities](/capabilities) page — discovery, analysis, control — is built on top of it.",
  ),
};

/** Every content page this site renders, in navigation order. */
export function docsContentPages(): readonly DocumentationPageSource[] {
  return Object.freeze([INTRODUCTION, GETTING_STARTED, CAPABILITIES, CONCEPTS, DETERMINISM, ...galleryPages()]);
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
