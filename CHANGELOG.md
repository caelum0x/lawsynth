# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this project has not
published a tagged release yet. Changes are recorded only when they correspond to
implemented, tested behavior.

## Unreleased

### Added

- **Product loop across CLI and SDK.** The `observe → discover → understand → use →
  compare → share → organize` loop is now a first-class product, not just an engine.
- **`explain` (Understand).** CLI `lawsynth explain` and SDK `Study.explain()` turn a
  world into readable laws with dominant terms, dependency structure, fit quality, and
  the assumptions a result is contingent on.
- **`forecast` (Use).** CLI `lawsynth forecast` (with `--horizon`, `--intervene
  NAME=VALUE@TIME`, CSV output) and SDK `Study.forecast(...)` run a world beyond the
  observed window and compare a baseline against a counterfactual, reporting divergence.
- **`compare` (Compare).** CLI `lawsynth compare A B [--json] [--html]` produces a
  structured diff of two worlds — variables, parameters, laws, and complexity.
- **Scenarios (Compare).** CLI `lawsynth scenarios --scenario NAME:k=v@t` and SDK
  `Study.add_scenario(...)` / `Study.compare_scenarios()` overlay N what-ifs against a
  baseline as a `ScenarioComparison` with per-state divergence.
- **`report` (Share).** CLI `lawsynth report` and SDK `Study.report(...)` emit a single
  self-contained HTML report — rendered equations, fit and Pareto candidates, regime
  timeline, uncertainty bands, and inline SVG trajectory/phase charts, no server or
  external assets.
- **`export` (Share).** CLI `lawsynth export --format python|latex|json` emits a
  dependency-free Python module, a LaTeX `align*` block, or a documented JSON model.
- **`library` (Organize).** CLI `lawsynth library <add|list|show|remove>` maintains a
  local, tagged, searchable index of `.lsworld` bundles (default `~/.lawsynth/library.tsv`).
- **Templates and scaffolding.** CLI `lawsynth templates` and `lawsynth new TEMPLATE`
  scaffold worlds and sample data from built-ins: `lorenz`, `lotka-volterra`,
  `pendulum`, `van-der-pol`, and `sir`.
- **`validate`.** CLI `lawsynth validate WORLD --data OBS [--holdout FRACTION]` scores a
  world against held-out observations.
- **`pipeline`.** CLI `lawsynth pipeline PIPELINE.toml` (and `--example`) drives a whole
  ingest → discover → validate → report → export flow from one deterministic config file.
- **`doctor`.** CLI `lawsynth doctor` reports install health across the command surface.
- **Discovery depth.** `discover` gained optional stages and controls: `--solver
  stlsq|sr3`, trigonometric and bounded-rational feature families, multiple derivative
  estimators (`--spline`, `--spectral`, `--savgol-window`, `--tvreg-lambda`),
  `--bootstrap`, `--symbolic-depth`, and the `--regimes`, `--pareto`, `--refine`, and
  `--causal` stages. TSV and a numeric Parquet subset join CSV as accepted inputs.
- **Python `Study` SDK.** A fluent façade over the loop: `from_csv` / `from_dataset` /
  `from_columns` / `from_source` / `load`, then `discover`, `explain`, `simulate`,
  `forecast`, `add_scenario` / `compare_scenarios`, `report`, and `save`. All returned
  objects (`Study`, `DiscoveryResult`, `Explanation`, `Forecast`, `ScenarioComparison`,
  trajectories, native `World`) render richly in Jupyter.
- **Data import.** `lawsynth.load_source` / `Study.from_source` bridge the
  `lawsynth-connectors` package (`filesystem`, `http`, `s3`, `postgres`, `sqlite`),
  coercing external records to finite floats at the SDK boundary.
- **Notebook dashboard.** The optional `lawsynth-notebook` package adds `StudyDashboard`,
  folding equations, dependencies, trajectory, uncertainty, and scenarios into one view.
- **Studio (9 screens).** The `apps/studio` visual surface ships Data Lens, Discovery
  Canvas, Equation Explorer, Structure Map, Regime Timeline, Uncertainty Lens, World Lab,
  Scenario Board, and Export, with a navigation controller over the shared World IR.
- **HTTP product endpoints.** The `/v1` services surface exposes the product loop over
  stored worlds: `GET /v1/worlds/{id}/explain`, `POST /v1/worlds/{id}/forecast`,
  `GET /v1/worlds/{id}/report`, and `POST /v1/worlds/compare`, backed by the same native
  engine as `POST /v1/worlds/{id}/simulate`.

### Notes

- Discovery remains **deterministic and offline**: identical inputs and options
  reproduce the same worlds, reports, and forecasts. `explain`, `report`, and `compare`
  read declarative structure and work without the native engine; `forecast` and
  `simulate` require it and fail clearly when it is absent. A sparse fit is not evidence
  of causality, and extrapolation beyond the observed window is not validated.
