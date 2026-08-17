# LawSynth — Product

LawSynth turns time-series observations into **executable mathematical worlds**:
interpretable law systems you can read, simulate, stress-test, and share. This
document is the product view — what a user actually does with LawSynth — layered
on top of the engine and repository blueprint in `ARCHITECTURE.md` and
`LawSynth_Production_Architecture.md`.

## Who it's for

- **Scientists & engineers** who have measurements and want a *mechanistic* model
  (equations, not a black box) they can trust and reason about.
- **Analysts & quants** who need to forecast, run interventions, and compare
  scenarios on a model they can inspect.
- **Teams** who want reproducible, shareable, local-first results — no data
  leaves the machine.

## The core loop

```
observe (CSV)  →  discover (laws)  →  understand (explain)  →  use (simulate /
forecast / intervene)  →  compare  →  share (report / .lsworld bundle)
```

Every step is deterministic and offline. A discovery is a portable `.lsworld`
bundle; everything downstream operates on that artifact.

## Product surfaces

| Surface | For | Entry point |
| --- | --- | --- |
| **CLI** | power users, automation, CI | `lawsynth <command>` |
| **Python SDK** | notebooks, pipelines | `import lawsynth` |
| **Studio** | interactive exploration | web app (`apps/studio`) |
| **Services** | teams, self-hosting | API + gateway |

## Feature areas (product depth)

### 1. Discover — already deep
Sparse dynamics, symbolic search, lagged structure, Pareto frontier, regimes,
uncertainty, joint parameter refinement, causal hypotheses. Tunable per run.

### 2. Understand — *`explain`*
Turn a world into meaning: plain-language description of each law, the dominant
terms, discovered regimes, dependency/causal hypotheses, fit quality, and the
assumptions a result is contingent on. Answers "what did it find, and can I
trust it?"

### 3. Use — *`forecast` / `simulate` / `intervene`*
Run the world forward, forecast beyond the observed window, and ask what-if:
change a parameter or input on a schedule and see how the trajectory responds.
Scenario objects make interventions first-class and comparable.

### 4. Compare — *`compare`*
Diff two worlds (or two runs): structure, parameters, complexity, and fit —
and diff two *scenarios* of the same world. Model selection made legible.

### 5. Share — *`report`*
Generate a self-contained HTML report from a `.lsworld`: rendered equations,
fit and Pareto candidates, regime timeline, uncertainty bands, and inline SVG
trajectory + phase-portrait charts. No server, no external assets — a single
file a colleague can open.

### 6. Organize — *`library`*
A local world library: register, tag, search, and describe `.lsworld` bundles
so a workspace of discoveries stays navigable.

### 7. Explore — **Studio**
The visual product: a discovery canvas, equation explorer, regime timeline,
uncertainty lens, and world lab, driven by the shared TypeScript packages.

## Principles

- **Interpretable first.** If a user can't read and reason about the result, it
  isn't a LawSynth result.
- **Local-first & reproducible.** Same inputs → same world, offline, forever.
- **Honest about uncertainty.** Boundaries, assumptions, and confidence are
  surfaced, never hidden.
- **Composable.** CLI, SDK, Studio, and services all operate on the same
  validated World IR and `.lsworld` bundles.

## Roadmap (product)

- **Now:** `report`, `explain`, `compare`, `forecast`/`intervene`, `library`
  across CLI + SDK; Studio discovery/equation/regime screens.
- **Next:** scenario boards, model-selection views, notebook-native rich display,
  world templates.
- **Later:** collaborative workspaces, hosted Studio, plugin marketplace.
