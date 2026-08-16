# LawSynth

## Production architecture and repository blueprint

**Working name:** LawSynth  
**Tagline:** Discover governing laws from data. Run them as executable worlds.  
**Document status:** Product and architecture decision, revision 1  
**Target license:** Apache-2.0 for the complete core, UI, SDKs, file formats, and self-hosted services  
**Primary languages:** Rust, Python, and TypeScript  
**Date:** 16 August 2026

---

## 1. Executive decision

LawSynth is worth building, with one important condition: it must not be marketed as a general “AI scientist” or a chat wrapper. It should begin as a precise engine for one hard, visually impressive job:

> Given multivariate observations, discover compact candidate laws governing the system, assemble them into an executable world model, expose uncertainty and regime changes, and simulate interventions or alternative futures.

This is not another model-training platform, agent framework, notebook, testing suite, observability product, or optimization wrapper. The product itself performs mathematical discovery.

The initial wedge is numerical time-series data because it permits a coherent end-to-end result:

1. infer variables, units, sampling behavior, and usable windows;
2. estimate latent derivatives and delays;
3. search for differential or difference equations;
4. discover candidate directed dependencies;
5. detect changing regimes and events;
6. quantify parameter and trajectory uncertainty;
7. compile the result into an executable world model;
8. simulate interventions, shocks, and counterfactual scenarios;
9. present the result in a visual discovery studio.

The individual ingredients have strong open-source implementations, including [PySR](https://github.com/MilesCranmer/PySR), [PySINDy](https://github.com/dynamicslab/pysindy), [DoWhy](https://github.com/py-why/dowhy), [Tigramite](https://github.com/jakobrunge/tigramite), [pgmpy](https://github.com/pgmpy/pgmpy), and [DeepCausality](https://github.com/deepcausality-rs/deep_causality). The opportunity is not pretending those fields do not exist. The opportunity is the missing product layer: one typed, executable representation that combines equations, causal hypotheses, regimes, uncertainty, interventions, and simulation.

### Verdict

| Dimension | Assessment |
|---|---:|
| Technical depth | 9.5/10 |
| Demo and GitHub appeal | 9/10 |
| OSS usefulness | 9/10 |
| Differentiation if fully integrated | 8.5/10 |
| Differentiation as “symbolic regression plus UI” | 4/10 |
| Solo-founder feasibility for focused alpha | 7/10 |
| Solo-founder feasibility for the whole vision at once | 2/10 |
| Plausible path to 10k stars | Strong, but execution-dependent |

**Build decision: yes.** Build the engine and open format first; earn the larger scientific-discovery claim through working examples.

---

## 2. Name and identity

### Chosen name: LawSynth

“Law” describes a governing mathematical relationship. “Synth” describes discovering and assembling multiple laws into a runnable system. The name is short, understandable, and aligns with both the research and the product demo.

Suggested identity:

- GitHub organization: `lawsynth`
- main repository: `lawsynth/lawsynth`
- Python distribution: `lawsynth`
- Rust CLI and meta-crate: `lawsynth`
- executable: `lawsynth`
- documentation: `docs.lawsynth.org`
- open model bundle extension: `.lsworld`
- visual application: **LawSynth Studio**
- core runtime: **LawSynth Engine**
- model representation: **World IR**

Short descriptions:

- **One line:** An open-source engine that discovers governing equations and executable world models from data.
- **Developer:** LawSynth turns multivariate time series into typed equations, causal hypotheses, regimes, uncertainty, and intervention-ready simulations.
- **README hero:** Upload observations. Recover the laws. Change the world.

The initial web and package search found no obvious exact-name project occupying this scientific-discovery position, but this is only a preliminary namespace check, not legal or trademark clearance. Reserve the GitHub organization, PyPI name, crates.io name, and primary domains before public announcement.

---

## 3. What LawSynth is—and is not

### It is

- a discovery engine for dynamics in observed data;
- an interpretable world-model compiler;
- a scenario and intervention simulator;
- a Python scientific library with a high-performance Rust core;
- a local-first visual studio;
- a versioned open model format;
- an extensible research platform for new discovery algorithms;
- a self-hostable multi-user service at production maturity.

### It is not

- a generic LLM agent framework;
- a literature-review chatbot;
- a causal-certainty machine;
- a replacement for domain knowledge or controlled experiments;
- a generic MLOps/model-registry product;
- a no-code AutoML dashboard;
- a finance-only quant platform;
- a digital-twin system requiring hardware;
- an IDE, CI tool, testing product, or observability suite;
- a foundation model trained at enormous cost.

### Scientific claim discipline

LawSynth should say **candidate law**, **candidate causal structure**, and **model under assumptions**. Observational data alone rarely identifies a unique causal mechanism. The software must display assumptions, alternative explanations, sensitivity, uncertainty, and extrapolation boundaries rather than hiding them.

---

## 4. Product experience

### Ten-second demo

1. Drag in a noisy Lorenz-system CSV with columns `t`, `x`, `y`, and `z`.
2. LawSynth animates a search over compact equations.
3. The discovered equations appear beside the observed attractor.
4. Toggle a parameter or inject a shock.
5. The reconstructed world evolves immediately with uncertainty bands.

This is the README GIF. It demonstrates real mathematical work, not chat.

### First useful workflow

```python
import lawsynth as ls

data = ls.read("market_dynamics.parquet", time="timestamp")

world = ls.discover(
    data,
    state=["demand", "price", "inventory"],
    controls=["promotion"],
    assumptions=ls.Assumptions(
        max_delay="14d",
        allowed_operators=["+", "-", "*", "/", "log", "exp"],
        regime_changes=True,
    ),
)

print(world.equations)
print(world.dependencies)
print(world.regimes)

baseline = world.simulate(horizon="90d")
scenario = world.intervene(promotion=0.20).simulate(horizon="90d")

world.save("demand-system.lsworld")
scenario.plot(compare=baseline)
```

### CLI

```bash
lawsynth discover observations.parquet \
  --time timestamp \
  --state demand,price,inventory \
  --control promotion \
  --output demand-system.lsworld

lawsynth inspect demand-system.lsworld
lawsynth simulate demand-system.lsworld --horizon 90d
lawsynth intervene demand-system.lsworld --set promotion=0.20
lawsynth studio demand-system.lsworld
lawsynth serve --host 0.0.0.0 --port 7310
```

### Studio screens

1. **Workspace** — projects, datasets, worlds, and runs.
2. **Data Lens** — variables, units, missingness, sampling, distributions, and time alignment.
3. **Discovery Canvas** — search progress, candidate frontier, constraints, and assumptions.
4. **Equation Explorer** — editable equations, dimensions, parameters, residuals, and alternatives.
5. **Structure Map** — lagged dependency and candidate causal graph.
6. **Regime Timeline** — change points, state labels, and regime-specific equations.
7. **World Lab** — initial conditions, interventions, shocks, events, and trajectory comparison.
8. **Uncertainty Lens** — posterior/ensemble parameter ranges and trajectory envelopes.
9. **Provenance** — data fingerprint, configuration, algorithm versions, seeds, and artifacts.
10. **Export** — Python, Rust, JSON, LaTeX, ONNX where applicable, and `.lsworld`.

---

## 5. The core abstraction: World IR

The moat is not a collection of algorithms. It is a stable, typed intermediate representation that every discovery method can produce and every simulator can execute.

### 5.1 World definition

A `World` contains:

- variables and roles;
- units and dimensions;
- time semantics;
- state transition laws;
- observation laws;
- algebraic constraints;
- probability distributions;
- parameters and priors;
- exogenous inputs and controls;
- candidate directed dependencies;
- regimes and transition conditions;
- discrete events;
- interventions;
- uncertainty representation;
- fitted evidence and provenance;
- compilation targets.

### 5.2 Variable roles

```text
State       value evolves inside the world
Observed    measured projection of state
Control     deliberately chosen input
Exogenous   input generated outside the world
Parameter   fixed or inferred coefficient
Latent      inferred but not directly observed
Derived     expression computed from other variables
Noise       stochastic process or residual source
Event       discrete occurrence that changes behavior
```

### 5.3 Law kinds

```text
Continuous       dx/dt = f(x, u, e, p, t)
Discrete         x[t+1] = f(x[t-k:t], u, e, p, t)
Algebraic        0 = g(x, u, e, p, t)
Observation      y = h(x, p) + noise
Stochastic       dx = f(...)dt + g(...)dW
Event            when predicate(...) then transition(...)
Regime           if regime == r use law set L_r
Constraint       expression relation and admissible domain
```

### 5.4 Expression IR

The expression layer must be language-neutral and deterministic. It should support:

- scalar, vector, matrix, boolean, categorical, duration, and timestamp types;
- integer and floating constants with explicit precision;
- variables, parameters, and indexed delays;
- arithmetic and comparison operators;
- common elementary functions;
- piecewise expressions;
- aggregations and window operations;
- derivative and integral nodes;
- random variables and distributions;
- unit annotations;
- source spans and provenance;
- canonicalization and stable hashing.

Internally, Rust owns the canonical AST and e-graph representation. Python and TypeScript consume generated bindings from the schema. SymPy import/export is an adapter, not the canonical storage format.

### 5.5 Open bundle format: `.lsworld`

An `.lsworld` file is a deterministic ZIP64-compatible archive:

```text
demand-system.lsworld/
├── manifest.json
├── world/
│   ├── world.json
│   ├── expressions.cbor
│   ├── variables.json
│   ├── parameters.parquet
│   ├── dependencies.json
│   ├── regimes.json
│   ├── events.json
│   └── interventions.json
├── evidence/
│   ├── data-manifest.json
│   ├── data-profile.parquet
│   ├── fit-metrics.parquet
│   ├── residuals.parquet
│   ├── uncertainty.parquet
│   └── candidate-frontier.parquet
├── provenance/
│   ├── run.json
│   ├── environment.json
│   ├── algorithms.json
│   ├── assumptions.json
│   └── checksums.sha256
├── previews/
│   ├── summary.svg
│   └── thumbnail.webp
└── signatures/
    └── bundle.sig
```

Rules:

- JSON for inspectable metadata and schemas;
- CBOR for compact expression graphs;
- Arrow/Parquet for tabular numeric artifacts;
- content-addressed files and stable SHA-256 hashes;
- semantic format version independent from software version;
- unknown optional fields preserved during round trips;
- migrations are explicit and loss-aware;
- signatures optional, verification built into CLI;
- specification and conformance fixtures fully open source.

---

## 6. Architecture principles

1. **Local-first core.** `pip install lawsynth` must do useful work without accounts or services.
2. **Rust owns correctness-sensitive execution.** IR, search primitives, evaluation, simulation, and serialization live in Rust.
3. **Python owns scientific composition.** User API, adapters, experiment orchestration, and research extensibility live in Python.
4. **TypeScript owns the interface.** The Studio is a React application consuming generated API types.
5. **One world representation.** Algorithms exchange World IR, not private object graphs.
6. **No mandatory LLM.** Language models can propose priors or explain results, but core discovery works offline.
7. **Artifacts over hidden state.** Every run produces an inspectable, reproducible bundle.
8. **CPU-first.** Great multithreaded CPU execution before GPU complexity.
9. **Columnar data end to end.** Arrow arrays and Parquet minimize copies across languages.
10. **Modular algorithms, opinionated workflow.** Researchers can replace components; normal users get sensible defaults.
11. **Honest scientific semantics.** Assumptions and uncertainty are part of the data model.
12. **Complete OSS.** No proprietary “enterprise algorithm” behind an API.

---

## 7. High-level system architecture

```text
                            ┌───────────────────────────┐
                            │      LawSynth Studio      │
                            │ React + TypeScript + WASM │
                            └─────────────┬─────────────┘
                                          │ HTTP/SSE/WebSocket
                            ┌─────────────▼─────────────┐
                            │  Python API / Local Daemon│
                            │ workflow, auth, projects  │
                            └───────┬──────────┬────────┘
                                    │          │
                       PyO3/Arrow   │          │ jobs/artifacts
                 ┌──────────────────▼──┐   ┌───▼─────────────────┐
                 │ LawSynth Rust Engine│   │ OSS service layer   │
                 │                     │   │ scheduler, workers, │
                 │ World IR            │   │ registry, storage   │
                 │ discovery pipelines │   └───┬─────────────────┘
                 │ simulation runtime  │       │
                 │ bundle compiler     │   ┌───▼─────────────────┐
                 └──────────┬──────────┘   │ Postgres + object   │
                            │              │ store + NATS        │
                 ┌──────────▼──────────┐   └─────────────────────┘
                 │ Arrow / Parquet /   │
                 │ .lsworld artifacts  │
                 └─────────────────────┘
```

### Deployment modes

| Mode | Audience | Processes | Persistence |
|---|---|---|---|
| Embedded library | notebooks and Python apps | Python + native extension | caller controlled |
| Local Studio | individual user | local daemon + browser | SQLite and local object directory |
| Single-node server | lab or small company | API + worker + Postgres + MinIO | durable shared storage |
| Distributed | organization | API replicas + scheduler + worker pools | Postgres, S3, NATS |
| WASM viewer | docs and shared demos | browser only | read-only bundle |

Do not begin with Kubernetes. The first public alpha supports embedded library and Local Studio. Single-node Docker Compose follows. Distributed deployment begins only after jobs exceed one machine.

---

## 8. Discovery pipeline

### 8.1 Pipeline graph

```text
ingest
  → schema and semantic profiling
  → time alignment and usable-window selection
  → preprocessing variants
  → derivative/delay/state reconstruction variants
  → parallel discovery strategies
      ├── sparse dynamics discovery
      ├── symbolic equation search
      ├── lagged structure discovery
      └── regime discovery
  → candidate normalization into World IR
  → joint parameter refinement
  → uncertainty and stability estimation
  → Pareto frontier construction
  → world compilation
  → simulation and intervention analysis
  → `.lsworld` artifact
```

### 8.2 Ingestion and profiling

Initial formats:

- CSV and TSV;
- Parquet;
- Arrow IPC;
- pandas and Polars DataFrames;
- NumPy arrays;
- xarray Dataset for labeled multidimensional series.

Later adapters:

- DuckDB query;
- SQLAlchemy/ADBC sources;
- Delta Lake and Iceberg snapshots;
- Kafka-compatible bounded captures;
- finance adapters maintained outside the core.

Profiling computes:

- type and role candidates;
- unit metadata and conflicts;
- sampling cadence and gaps;
- missingness mechanisms and patterns;
- scale, skew, tails, discreteness, and bounds;
- autocorrelation and candidate delays;
- collinearity and redundancy;
- nonstationarity indicators;
- candidate change points;
- usable windows and exclusion reasons.

### 8.3 Preprocessing as branches, not silent mutation

Every transformation is an immutable operation in the run graph:

- selection and filtering;
- time-zone normalization;
- resampling and alignment;
- missing-data policy;
- robust scaling;
- detrending;
- seasonal adjustment;
- smoothing;
- outlier policy;
- differencing;
- log or Box-Cox transforms;
- unit conversion.

LawSynth may explore several preprocessing branches and report which branch produced each candidate. Original data is never overwritten.

### 8.4 State and derivative reconstruction

Alpha methods:

- finite difference variants;
- Savitzky–Golay derivatives;
- smoothing splines;
- total-variation regularized differentiation;
- explicit delay coordinates;
- user-supplied derivative columns.

Later methods:

- Gaussian-process derivative posterior;
- neural interpolation for irregular time series;
- latent state-space reconstruction;
- Koopman embeddings;
- weak-form integral discovery to avoid direct differentiation.

### 8.5 Equation discovery

Two complementary engines ship early:

**Sparse library discovery**

- polynomial, trigonometric, rational, and user-defined feature libraries;
- STLSQ and SR3-family sparse selection;
- ensemble and bootstrap support;
- constraints and shared terms across equations;
- discrete-time and continuous-time variants.

**Symbolic search**

- typed expression grammar;
- genetic programming baseline;
- e-graph simplification and equivalence classes;
- constant optimization;
- multi-objective ranking: fit, complexity, stability, units, and extrapolation;
- island populations and deterministic seeds;
- warm starts from sparse discovery;
- optional neural or LLM-proposed priors later.

No single “best equation” should be forced. The output is a Pareto frontier of materially different candidates.

### 8.6 Dependency and causal hypothesis discovery

Initial features:

- lagged association graph;
- Granger-style predictive direction;
- conditional independence tests;
- time-order constraints;
- forbidden and required edges;
- exogeneity declarations;
- bootstrap edge stability;
- alternative graph equivalence classes.

Later features:

- PCMCI-family discovery;
- differentiable DAG structure learning;
- nonlinear additive-noise approaches;
- invariant causal prediction across regimes/environments;
- causal effect identification through adapters to DoWhy/pgmpy;
- interventional dataset fusion.

The graph is labeled **candidate structure** unless the data and assumptions support stronger identification.

### 8.7 Regimes and events

Alpha:

- PELT and binary segmentation change points;
- Bayesian online change-point detection;
- hidden Markov regime labels;
- regime-specific coefficients;
- manual regime annotations;
- event markers from input data.

Beta:

- Markov-switching equations;
- guard-condition discovery;
- hysteresis;
- regime transition probabilities;
- shared symbolic structure with regime-specific parameters;
- event-triggered state resets.

### 8.8 Uncertainty

Uncertainty is represented at four levels:

1. **data uncertainty** — missingness, measurement error, and resampling;
2. **parameter uncertainty** — intervals or posterior samples;
3. **structural uncertainty** — alternative equations and graphs;
4. **trajectory uncertainty** — simulated future distributions.

Initial methods:

- block bootstrap;
- candidate ensembles;
- profile likelihood and covariance approximation;
- residual resampling;
- stability selection;
- scenario bands across structural alternatives.

Later:

- Hamiltonian Monte Carlo through an adapter;
- variational inference;
- stochastic differential equation inference;
- Bayesian model averaging;
- conformal trajectory envelopes where assumptions fit.

### 8.9 Compilation and simulation

World IR compiles to:

- native Rust interpreter;
- vectorized CPU executor;
- Python callable;
- WASM read-only simulator;
- generated Rust module;
- generated Python/NumPy function;
- LaTeX and MathML presentation;
- ONNX only for compatible learned components.

Simulation types:

- initial-value ODE;
- discrete recurrence;
- stochastic trajectories;
- event-driven hybrid systems;
- regime-switching systems;
- Monte Carlo parameter ensembles;
- interventions on controls, parameters, equations, and graph edges.

---

## 9. Technology choices

### Rust

- stable Rust edition current at implementation time;
- Cargo workspace;
- PyO3 and maturin for Python bindings;
- Arrow and Parquet crates for zero/low-copy data exchange;
- Rayon for CPU parallelism;
- Tokio only in service and I/O crates, not the mathematical core;
- Serde for metadata;
- e-graph library or a small purpose-built equality-saturation layer;
- nalgebra/faer and ndarray selected behind narrow internal traits;
- tracing for internal diagnostics;
- criterion for microbenchmarks;
- cargo-nextest for test execution.

### Python

- Python 3.11+ at alpha, 3.12+ when ecosystem support permits;
- uv workspaces and locked dependency groups;
- typed public API with py.typed;
- Pydantic only at API/config boundaries;
- NumPy, pandas, Polars, Arrow, xarray adapters;
- FastAPI for local/server API;
- scientific integrations isolated behind optional extras;
- Ruff, mypy or Pyright, pytest, and Hypothesis for internal quality.

### TypeScript

- React and Vite for Studio;
- pnpm workspace;
- TanStack Query and Router;
- Zustand or a small event store for local UI state;
- Apache ECharts/Plotly for general charts;
- custom Canvas/WebGL only for large trajectories and graphs;
- generated API client and schema types;
- Playwright and Vitest for internal quality.

### Infrastructure

- SQLite for local project metadata;
- Postgres for server metadata;
- filesystem locally and S3-compatible storage on server;
- NATS JetStream for distributed jobs/events when needed;
- OpenTelemetry-compatible telemetry, disabled or local by default;
- Docker Compose before Kubernetes;
- OCI images and signed release artifacts;
- GitHub Actions for public CI/release.

### Explicit non-choices

- no microservices in alpha;
- no mandatory Redis plus Celery stack;
- no JVM service;
- no proprietary cloud database requirement;
- no GPU-first CUDA dependency;
- no LLM required for normal discovery;
- no plugin system based on executing arbitrary Python inside the API process.

---

## 10. Process model

### Local Studio

```text
lawsynth studio
  ├── starts Python local daemon
  ├── loads Rust extension in process
  ├── stores metadata in SQLite
  ├── stores artifacts under project directory
  ├── serves compiled Studio assets
  └── opens browser
```

Discovery runs start in a child process so a crash or native error does not destroy the daemon. Resource limits and cancellation are enforced at the process boundary.

### Distributed server

```text
API request
  → validate and persist RunSpec
  → publish job envelope
  → scheduler assigns compatible worker pool
  → worker leases job
  → worker streams progress events
  → worker uploads content-addressed artifacts
  → transaction records final artifact references
  → API notifies Studio through SSE/WebSocket
```

Job semantics:

- idempotent run submission with client token;
- lease and heartbeat;
- cooperative cancellation plus hard timeout;
- resumable checkpoints for long searches;
- content-addressed immutable artifacts;
- deterministic seed plan;
- append-only run events;
- explicit retry classification;
- no duplicate finalization;
- CPU/memory/time quotas.

---

## 11. Public API design

### Python object model

```text
Dataset
DataSpec
Variable
Unit
Assumptions
DiscoveryPlan
DiscoveryRun
Candidate
CandidateFrontier
World
Equation
DependencyGraph
Regime
Event
Intervention
Scenario
Trajectory
Uncertainty
WorldBundle
```

### API layers

1. **One-call API** for first success.
2. **Builder API** for explicit configuration.
3. **Pipeline API** for researchers replacing stages.
4. **Rust API** for native embedding.
5. **HTTP API** for Studio and distributed operation.

### Builder example

```python
plan = (
    ls.DiscoveryPlan()
    .data(data, time="t")
    .roles(state=["x", "y", "z"])
    .differentiate(method="tvreg")
    .equations(methods=["sindy", "symbolic"])
    .structure(method="time_ordered")
    .regimes(methods=["pelt", "hmm"])
    .uncertainty(method="block_bootstrap", samples=200)
    .rank(by=["fit", "complexity", "stability", "units"])
)

run = plan.start()
for event in run.events():
    print(event.stage, event.progress, event.message)

world = run.best_world()
```

### HTTP resources

```text
GET    /v1/health
GET    /v1/version
POST   /v1/projects
GET    /v1/projects/{project_id}
POST   /v1/datasets
GET    /v1/datasets/{dataset_id}
POST   /v1/datasets/{dataset_id}/profile
POST   /v1/runs
GET    /v1/runs/{run_id}
POST   /v1/runs/{run_id}/cancel
GET    /v1/runs/{run_id}/events
GET    /v1/runs/{run_id}/candidates
POST   /v1/worlds
GET    /v1/worlds/{world_id}
POST   /v1/worlds/{world_id}/simulate
POST   /v1/worlds/{world_id}/intervene
GET    /v1/worlds/{world_id}/bundle
POST   /v1/bundles/import
GET    /v1/artifacts/{artifact_id}
```

SSE is sufficient for alpha progress streams. WebSocket is added only for collaborative editing or bidirectional live control.

---

## 12. Monorepo overview

```text
lawsynth/
├── .cargo/                 Cargo defaults and aliases
├── .config/                repository-wide tool configuration
├── .github/                issue forms, workflows, release automation
├── .vscode/                recommended editor setup
├── apps/                   user-facing applications
├── assets/                 brand and shared visual assets
├── benchmarks/             performance and scientific benchmarks
├── bindings/               generated cross-language bindings
├── crates/                 Rust workspace
├── datasets/               small open demo data and generators
├── deploy/                 self-hosting and production deployment
├── docs/                   documentation source
├── examples/               end-to-end examples
├── infra/                  deployment modules and operational config
├── packages/               TypeScript shared packages
├── plugins/                first-party extension examples
├── proto/                  service contracts
├── python/                 Python workspace
├── schemas/                language-neutral data schemas
├── scripts/                developer and release commands
├── services/               self-hosted service processes
├── specs/                  World IR and open-format specifications
├── tests/                  cross-language and system suites
├── tools/                  repository-owned generators and utilities
└── xtask/                  Rust build orchestration
```

The next section is the canonical production tree. It lists the intended hand-authored modules and the generated file families. A mature repository should contain thousands of meaningful files, but generating thousands of empty placeholders would make the architecture worse, not better.

---

## 13. Canonical folder and file directory

### 13.1 Root, governance, and automation

```text
lawsynth/
├── README.md
├── README.tr.md
├── LICENSE
├── NOTICE
├── CITATION.cff
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── GOVERNANCE.md
├── MAINTAINERS.md
├── SECURITY.md
├── SUPPORT.md
├── ROADMAP.md
├── CHANGELOG.md
├── ARCHITECTURE.md
├── AUTHORS.md
├── Cargo.toml
├── Cargo.lock
├── pyproject.toml
├── uv.lock
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── deny.toml
├── typos.toml
├── lefthook.yml
├── justfile
├── Makefile
├── Dockerfile
├── docker-compose.yml
├── codecov.yml
├── release-plz.toml
├── .editorconfig
├── .env.example
├── .gitattributes
├── .gitignore
├── .gitmodules
├── .pre-commit-config.yaml
├── .python-version
├── .node-version
├── .cargo/
│   └── config.toml
├── .config/
│   ├── cargo-nextest.toml
│   ├── markdownlint.json
│   ├── pyrightconfig.json
│   ├── pytest.ini
│   ├── ruff.toml
│   ├── taplo.toml
│   └── vitest.workspace.ts
├── .github/
│   ├── CODEOWNERS
│   ├── FUNDING.yml
│   ├── dependabot.yml
│   ├── labeler.yml
│   ├── pull_request_template.md
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug.yml
│   │   ├── config.yml
│   │   ├── documentation.yml
│   │   ├── feature.yml
│   │   ├── research-method.yml
│   │   └── scientific-result.yml
│   └── workflows/
│       ├── benchmark-comment.yml
│       ├── codeql.yml
│       ├── docs.yml
│       ├── fuzz.yml
│       ├── labels.yml
│       ├── nightly-science.yml
│       ├── pr-python.yml
│       ├── pr-rust.yml
│       ├── pr-typescript.yml
│       ├── release-cli.yml
│       ├── release-containers.yml
│       ├── release-python.yml
│       ├── release-rust.yml
│       ├── release-studio.yml
│       ├── scorecard.yml
│       └── stale.yml
└── .vscode/
    ├── extensions.json
    ├── launch.json
    ├── settings.json
    └── tasks.json
```

### 13.2 Specifications, schemas, and contracts

```text
specs/
├── README.md
├── world-ir/
│   ├── 000-overview.md
│   ├── 010-identifiers.md
│   ├── 020-types.md
│   ├── 030-units.md
│   ├── 040-variables.md
│   ├── 050-expressions.md
│   ├── 060-laws.md
│   ├── 070-distributions.md
│   ├── 080-regimes.md
│   ├── 090-events.md
│   ├── 100-dependencies.md
│   ├── 110-interventions.md
│   ├── 120-uncertainty.md
│   ├── 130-provenance.md
│   ├── 140-execution.md
│   ├── 150-canonicalization.md
│   ├── 160-hashing.md
│   └── changelog.md
├── bundle/
│   ├── 000-container.md
│   ├── 010-manifest.md
│   ├── 020-layout.md
│   ├── 030-content-types.md
│   ├── 040-checksums.md
│   ├── 050-signatures.md
│   ├── 060-migrations.md
│   ├── 070-forward-compatibility.md
│   ├── 080-security.md
│   └── changelog.md
├── discovery/
│   ├── run-spec.md
│   ├── stage-contract.md
│   ├── candidate-contract.md
│   ├── score-contract.md
│   ├── checkpoint-contract.md
│   └── event-contract.md
├── api/
│   ├── errors.md
│   ├── pagination.md
│   ├── idempotency.md
│   ├── versioning.md
│   └── streaming.md
├── rfc/
│   ├── README.md
│   ├── 0000-template.md
│   ├── 0001-world-ir.md
│   ├── 0002-lsworld-bundle.md
│   ├── 0003-plugin-abi.md
│   └── 0004-reproducible-runs.md
└── conformance/
    ├── README.md
    ├── required-cases.toml
    └── compatibility-matrix.toml

schemas/
├── README.md
├── jsonschema/
│   ├── manifest.schema.json
│   ├── world.schema.json
│   ├── variable.schema.json
│   ├── expression.schema.json
│   ├── law.schema.json
│   ├── distribution.schema.json
│   ├── dependency.schema.json
│   ├── regime.schema.json
│   ├── event.schema.json
│   ├── intervention.schema.json
│   ├── uncertainty.schema.json
│   ├── assumptions.schema.json
│   ├── data-manifest.schema.json
│   ├── discovery-plan.schema.json
│   ├── run.schema.json
│   ├── run-event.schema.json
│   ├── artifact.schema.json
│   └── plugin-manifest.schema.json
├── arrow/
│   ├── parameters.json
│   ├── profiles.json
│   ├── residuals.json
│   ├── trajectories.json
│   ├── uncertainty.json
│   ├── candidate-frontier.json
│   └── run-events.json
├── examples/
│   ├── minimal-world.json
│   ├── lorenz-world.json
│   ├── regime-world.json
│   ├── stochastic-world.json
│   └── hybrid-world.json
└── migrations/
    ├── v0_to_v1.json
    └── README.md

proto/
├── buf.gen.yaml
├── buf.lock
├── buf.yaml
└── lawsynth/
    └── v1/
        ├── common.proto
        ├── dataset.proto
        ├── project.proto
        ├── run.proto
        ├── scheduler.proto
        ├── worker.proto
        ├── artifact.proto
        ├── world.proto
        ├── simulation.proto
        └── events.proto

bindings/
├── README.md
├── python/                 generated schema and protobuf classes
├── rust/                   generated protobuf modules
└── typescript/             generated schema and API types
```

### 13.3 Rust workspace

Each Rust crate normally contains `Cargo.toml`, `README.md`, `src/lib.rs`, `src/error.rs`, focused modules, unit tests near the code, `tests/` for public behavior, and `benches/` where performance matters. The files below define the intended module boundaries.

```text
crates/
├── lawsynth-core/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── id.rs
│       ├── version.rs
│       ├── hash.rs
│       ├── seed.rs
│       ├── cancel.rs
│       ├── progress.rs
│       ├── resource.rs
│       └── prelude.rs
├── lawsynth-expr/
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ast.rs
│   │   ├── node.rs
│   │   ├── operator.rs
│   │   ├── literal.rs
│   │   ├── symbol.rs
│   │   ├── types.rs
│   │   ├── shape.rs
│   │   ├── domain.rs
│   │   ├── visitor.rs
│   │   ├── rewrite.rs
│   │   ├── simplify.rs
│   │   ├── canonical.rs
│   │   ├── hash.rs
│   │   ├── parser.rs
│   │   ├── printer.rs
│   │   ├── latex.rs
│   │   ├── mathml.rs
│   │   ├── evaluate.rs
│   │   ├── differentiate.rs
│   │   ├── interval.rs
│   │   └── error.rs
│   ├── tests/
│   │   ├── canonicalization.rs
│   │   ├── roundtrip.rs
│   │   ├── differentiation.rs
│   │   └── fixtures.rs
│   └── benches/
│       ├── evaluate.rs
│       └── simplify.rs
├── lawsynth-egraph/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── language.rs
│       ├── analysis.rs
│       ├── rules.rs
│       ├── schedule.rs
│       ├── extract.rs
│       ├── cost.rs
│       ├── proof.rs
│       └── limits.rs
├── lawsynth-units/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── dimension.rs
│       ├── unit.rs
│       ├── registry.rs
│       ├── parse.rs
│       ├── convert.rs
│       ├── infer.rs
│       ├── check.rs
│       └── builtins.rs
├── lawsynth-world/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── world.rs
│       ├── builder.rs
│       ├── variable.rs
│       ├── role.rs
│       ├── time.rs
│       ├── parameter.rs
│       ├── law.rs
│       ├── continuous.rs
│       ├── discrete.rs
│       ├── algebraic.rs
│       ├── stochastic.rs
│       ├── observation.rs
│       ├── graph.rs
│       ├── regime.rs
│       ├── event.rs
│       ├── constraint.rs
│       ├── intervention.rs
│       ├── uncertainty.rs
│       ├── provenance.rs
│       ├── validate.rs
│       └── fingerprint.rs
├── lawsynth-data/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── dataset.rs
│       ├── schema.rs
│       ├── column.rs
│       ├── roles.rs
│       ├── time_axis.rs
│       ├── window.rs
│       ├── batch.rs
│       ├── stream.rs
│       ├── scan.rs
│       ├── csv.rs
│       ├── parquet.rs
│       ├── arrow_ipc.rs
│       ├── projection.rs
│       ├── filter.rs
│       ├── fingerprint.rs
│       └── error.rs
├── lawsynth-profile/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── profiler.rs
│       ├── column_profile.rs
│       ├── time_profile.rs
│       ├── missingness.rs
│       ├── distribution.rs
│       ├── dependence.rs
│       ├── autocorrelation.rs
│       ├── delays.rs
│       ├── stationarity.rs
│       ├── quality_flags.rs
│       └── report.rs
├── lawsynth-preprocess/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── pipeline.rs
│       ├── transform.rs
│       ├── select.rs
│       ├── align.rs
│       ├── resample.rs
│       ├── impute.rs
│       ├── scale.rs
│       ├── detrend.rs
│       ├── seasonal.rs
│       ├── smooth.rs
│       ├── outlier.rs
│       ├── difference.rs
│       ├── power.rs
│       ├── unit_convert.rs
│       └── provenance.rs
├── lawsynth-stats/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── moments.rs
│       ├── quantile.rs
│       ├── covariance.rs
│       ├── correlation.rs
│       ├── robust.rs
│       ├── distance.rs
│       ├── distributions.rs
│       ├── bootstrap.rs
│       ├── block_bootstrap.rs
│       ├── information.rs
│       ├── hypothesis.rs
│       ├── multiple_testing.rs
│       ├── sampling.rs
│       └── rng.rs
├── lawsynth-differentiate/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── method.rs
│       ├── finite.rs
│       ├── savgol.rs
│       ├── spline.rs
│       ├── tvreg.rs
│       ├── spectral.rs
│       ├── weak_form.rs
│       ├── irregular.rs
│       ├── boundary.rs
│       └── diagnostics.rs
├── lawsynth-features/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── library.rs
│       ├── term.rs
│       ├── polynomial.rs
│       ├── trigonometric.rs
│       ├── rational.rs
│       ├── delay.rs
│       ├── interaction.rs
│       ├── custom.rs
│       ├── constraints.rs
│       ├── matrix.rs
│       └── cache.rs
├── lawsynth-opt/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── objective.rs
│       ├── bounds.rs
│       ├── result.rs
│       ├── least_squares.rs
│       ├── lbfgs.rs
│       ├── nelder_mead.rs
│       ├── coordinate.rs
│       ├── mixed.rs
│       └── termination.rs
```

`lawsynth-opt` is an internal numerical primitive, not the product focus. It exists because symbolic constants and world parameters must be fitted efficiently.

```text
crates/
├── lawsynth-sparse/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── problem.rs
│       ├── standardize.rs
│       ├── stlsq.rs
│       ├── sr3.rs
│       ├── lasso.rs
│       ├── group.rs
│       ├── constrained.rs
│       ├── ensemble.rs
│       ├── stability.rs
│       └── result.rs
├── lawsynth-symbolic/
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── grammar.rs
│   │   ├── population.rs
│   │   ├── individual.rs
│   │   ├── initialize.rs
│   │   ├── mutate.rs
│   │   ├── crossover.rs
│   │   ├── select.rs
│   │   ├── migrate.rs
│   │   ├── evaluate.rs
│   │   ├── constants.rs
│   │   ├── simplify.rs
│   │   ├── equivalence.rs
│   │   ├── constraints.rs
│   │   ├── complexity.rs
│   │   ├── stability.rs
│   │   ├── frontier.rs
│   │   ├── checkpoint.rs
│   │   ├── search.rs
│   │   └── result.rs
│   ├── tests/
│   │   ├── grammar.rs
│   │   ├── operators.rs
│   │   ├── determinism.rs
│   │   ├── recovery.rs
│   │   └── checkpoints.rs
│   └── benches/
│       ├── population.rs
│       └── evaluate.rs
├── lawsynth-dynamics/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── problem.rs
│       ├── continuous.rs
│       ├── discrete.rs
│       ├── delay.rs
│       ├── implicit.rs
│       ├── control.rs
│       ├── shared_structure.rs
│       ├── discover.rs
│       ├── refine.rs
│       ├── score.rs
│       └── result.rs
├── lawsynth-causal/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── graph.rs
│       ├── edge.rs
│       ├── assumptions.rs
│       ├── time_order.rs
│       ├── lagged.rs
│       ├── granger.rs
│       ├── conditional_independence.rs
│       ├── score_based.rs
│       ├── equivalence.rs
│       ├── bootstrap.rs
│       ├── stability.rs
│       ├── effects.rs
│       ├── identification.rs
│       ├── sensitivity.rs
│       └── result.rs
├── lawsynth-regime/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── segmentation.rs
│       ├── cost.rs
│       ├── pelt.rs
│       ├── binary.rs
│       ├── bocpd.rs
│       ├── hmm.rs
│       ├── markov.rs
│       ├── transitions.rs
│       ├── annotations.rs
│       ├── shared_laws.rs
│       ├── regime_laws.rs
│       └── result.rs
├── lawsynth-uncertainty/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── source.rs
│       ├── interval.rs
│       ├── samples.rs
│       ├── covariance.rs
│       ├── profile.rs
│       ├── residual.rs
│       ├── bootstrap.rs
│       ├── ensemble.rs
│       ├── structural.rs
│       ├── trajectory.rs
│       ├── propagate.rs
│       ├── summarize.rs
│       └── result.rs
├── lawsynth-sim/
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── state.rs
│   │   ├── context.rs
│   │   ├── compile.rs
│   │   ├── interpreter.rs
│   │   ├── vectorized.rs
│   │   ├── discrete.rs
│   │   ├── ode.rs
│   │   ├── sde.rs
│   │   ├── algebraic.rs
│   │   ├── hybrid.rs
│   │   ├── events.rs
│   │   ├── regimes.rs
│   │   ├── noise.rs
│   │   ├── ensemble.rs
│   │   ├── intervention.rs
│   │   ├── trajectory.rs
│   │   ├── diagnostics.rs
│   │   └── error.rs
│   ├── tests/
│   │   ├── ode_reference.rs
│   │   ├── discrete_reference.rs
│   │   ├── sde_statistics.rs
│   │   ├── events.rs
│   │   └── interventions.rs
│   └── benches/
│       ├── ode.rs
│       ├── expression.rs
│       └── ensemble.rs
├── lawsynth-score/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── metric.rs
│       ├── fit.rs
│       ├── complexity.rs
│       ├── stability.rs
│       ├── dimensionality.rs
│       ├── residual.rs
│       ├── forecast.rs
│       ├── extrapolation.rs
│       ├── pareto.rs
│       └── rank.rs
├── lawsynth-discovery/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── plan.rs
│       ├── assumptions.rs
│       ├── stage.rs
│       ├── graph.rs
│       ├── context.rs
│       ├── candidate.rs
│       ├── frontier.rs
│       ├── branch.rs
│       ├── scheduler.rs
│       ├── execute.rs
│       ├── event.rs
│       ├── checkpoint.rs
│       ├── resume.rs
│       ├── artifact.rs
│       ├── default_plan.rs
│       └── error.rs
├── lawsynth-bundle/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── manifest.rs
│       ├── layout.rs
│       ├── reader.rs
│       ├── writer.rs
│       ├── canonical.rs
│       ├── checksum.rs
│       ├── signature.rs
│       ├── migration.rs
│       ├── preview.rs
│       ├── limits.rs
│       └── error.rs
├── lawsynth-store/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── store.rs
│       ├── object.rs
│       ├── local.rs
│       ├── memory.rs
│       ├── s3.rs
│       ├── multipart.rs
│       ├── cache.rs
│       ├── gc.rs
│       └── error.rs
├── lawsynth-plugin-api/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── manifest.rs
│       ├── capability.rs
│       ├── algorithm.rs
│       ├── data_adapter.rs
│       ├── simulator.rs
│       ├── protocol.rs
│       ├── host.rs
│       ├── limits.rs
│       └── error.rs
├── lawsynth-plugin-host/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── discover.rs
│       ├── registry.rs
│       ├── process.rs
│       ├── wasi.rs
│       ├── rpc.rs
│       ├── permissions.rs
│       ├── resources.rs
│       └── lifecycle.rs
├── lawsynth-runner/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── run.rs
│       ├── process.rs
│       ├── envelope.rs
│       ├── resources.rs
│       ├── limits.rs
│       ├── heartbeat.rs
│       ├── checkpoint.rs
│       ├── cancellation.rs
│       └── events.rs
├── lawsynth-api-types/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── project.rs
│       ├── dataset.rs
│       ├── run.rs
│       ├── world.rs
│       ├── simulation.rs
│       ├── artifact.rs
│       ├── pagination.rs
│       └── error.rs
├── lawsynth-cli/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── main.rs
│       ├── args.rs
│       ├── output.rs
│       ├── config.rs
│       ├── command/
│       │   ├── mod.rs
│       │   ├── discover.rs
│       │   ├── inspect.rs
│       │   ├── profile.rs
│       │   ├── simulate.rs
│       │   ├── intervene.rs
│       │   ├── convert.rs
│       │   ├── bundle.rs
│       │   ├── plugin.rs
│       │   ├── studio.rs
│       │   └── serve.rs
│       └── ui/
│           ├── mod.rs
│           ├── progress.rs
│           ├── table.rs
│           └── equation.rs
├── lawsynth-python/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── convert.rs
│       ├── errors.rs
│       ├── py_dataset.rs
│       ├── py_plan.rs
│       ├── py_run.rs
│       ├── py_world.rs
│       ├── py_simulation.rs
│       ├── py_bundle.rs
│       └── py_events.rs
├── lawsynth-wasm/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── world.rs
│       ├── simulate.rs
│       ├── bundle.rs
│       └── errors.rs
└── lawsynth-test-support/
    ├── Cargo.toml
    ├── README.md
    └── src/
        ├── lib.rs
        ├── approx.rs
        ├── generators.rs
        ├── fixtures.rs
        ├── synthetic.rs
        ├── snapshots.rs
        └── temp.rs
```

### 13.4 Python workspace

The Python API is intentionally thin over the native engine for hot paths, but rich in adapters, declarative configuration, visualization, and research composition.

```text
python/
├── lawsynth/
│   ├── pyproject.toml
│   ├── README.md
│   ├── LICENSE
│   ├── src/lawsynth/
│   │   ├── __init__.py
│   │   ├── py.typed
│   │   ├── _native.pyi
│   │   ├── _version.py
│   │   ├── errors.py
│   │   ├── typing.py
│   │   ├── config.py
│   │   ├── logging.py
│   │   ├── dataset.py
│   │   ├── variable.py
│   │   ├── units.py
│   │   ├── assumptions.py
│   │   ├── plan.py
│   │   ├── run.py
│   │   ├── candidate.py
│   │   ├── frontier.py
│   │   ├── equation.py
│   │   ├── graph.py
│   │   ├── regime.py
│   │   ├── event.py
│   │   ├── uncertainty.py
│   │   ├── world.py
│   │   ├── intervention.py
│   │   ├── scenario.py
│   │   ├── trajectory.py
│   │   ├── bundle.py
│   │   ├── discover.py
│   │   ├── simulate.py
│   │   ├── inspect.py
│   │   ├── datasets/
│   │   │   ├── __init__.py
│   │   │   ├── readers.py
│   │   │   ├── pandas.py
│   │   │   ├── polars.py
│   │   │   ├── numpy.py
│   │   │   ├── arrow.py
│   │   │   ├── xarray.py
│   │   │   └── duckdb.py
│   │   ├── methods/
│   │   │   ├── __init__.py
│   │   │   ├── base.py
│   │   │   ├── differentiation.py
│   │   │   ├── equations.py
│   │   │   ├── structure.py
│   │   │   ├── regimes.py
│   │   │   ├── uncertainty.py
│   │   │   └── registry.py
│   │   ├── pipeline/
│   │   │   ├── __init__.py
│   │   │   ├── stage.py
│   │   │   ├── context.py
│   │   │   ├── artifact.py
│   │   │   ├── events.py
│   │   │   ├── graph.py
│   │   │   └── custom.py
│   │   ├── integrations/
│   │   │   ├── __init__.py
│   │   │   ├── sympy.py
│   │   │   ├── scipy.py
│   │   │   ├── sklearn.py
│   │   │   ├── torch.py
│   │   │   ├── jax.py
│   │   │   ├── dowhy.py
│   │   │   ├── pgmpy.py
│   │   │   └── mlflow.py
│   │   ├── plotting/
│   │   │   ├── __init__.py
│   │   │   ├── theme.py
│   │   │   ├── equation.py
│   │   │   ├── frontier.py
│   │   │   ├── graph.py
│   │   │   ├── regime.py
│   │   │   ├── residual.py
│   │   │   ├── trajectory.py
│   │   │   └── uncertainty.py
│   │   ├── report/
│   │   │   ├── __init__.py
│   │   │   ├── model.py
│   │   │   ├── render.py
│   │   │   ├── markdown.py
│   │   │   ├── html.py
│   │   │   └── latex.py
│   │   └── experimental/
│   │       ├── __init__.py
│   │       ├── weak_form.py
│   │       ├── koopman.py
│   │       ├── neural_prior.py
│   │       └── latent_state.py
│   └── tests/
│       ├── conftest.py
│       ├── test_import.py
│       ├── test_dataset.py
│       ├── test_plan.py
│       ├── test_discover.py
│       ├── test_world.py
│       ├── test_simulate.py
│       ├── test_bundle.py
│       ├── test_typing.py
│       ├── adapters/
│       ├── integrations/
│       └── snapshots/
├── lawsynth-server/
│   ├── pyproject.toml
│   ├── README.md
│   ├── src/lawsynth_server/
│   │   ├── __init__.py
│   │   ├── app.py
│   │   ├── lifespan.py
│   │   ├── settings.py
│   │   ├── dependencies.py
│   │   ├── errors.py
│   │   ├── auth.py
│   │   ├── pagination.py
│   │   ├── idempotency.py
│   │   ├── events.py
│   │   ├── routes/
│   │   ├── models/
│   │   ├── repositories/
│   │   ├── services/
│   │   └── middleware/
│   └── tests/
├── lawsynth-connectors/
│   ├── pyproject.toml
│   ├── README.md
│   ├── src/lawsynth_connectors/
│   │   ├── __init__.py
│   │   ├── base.py
│   │   ├── registry.py
│   │   ├── filesystem.py
│   │   ├── http.py
│   │   ├── sql.py
│   │   ├── duckdb.py
│   │   ├── postgres.py
│   │   ├── s3.py
│   │   ├── delta.py
│   │   └── iceberg.py
│   └── tests/
├── lawsynth-bench/
│   ├── pyproject.toml
│   ├── README.md
│   ├── src/lawsynth_bench/
│   │   ├── __init__.py
│   │   ├── registry.py
│   │   ├── problem.py
│   │   ├── dataset.py
│   │   ├── runner.py
│   │   ├── metrics.py
│   │   ├── leaderboard.py
│   │   ├── report.py
│   │   └── cli.py
│   └── tests/
└── lawsynth-notebook/
    ├── pyproject.toml
    ├── README.md
    ├── src/lawsynth_notebook/
    │   ├── __init__.py
    │   ├── display.py
    │   ├── widget.py
    │   ├── events.py
    │   └── assets.py
    └── tests/
```

### 13.5 Studio and TypeScript packages

```text
apps/
├── studio/
│   ├── README.md
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── vitest.config.ts
│   ├── playwright.config.ts
│   ├── public/
│   │   ├── favicon.svg
│   │   ├── manifest.webmanifest
│   │   ├── robots.txt
│   │   └── examples/
│   └── src/
│       ├── main.tsx
│       ├── app.tsx
│       ├── routes.tsx
│       ├── env.ts
│       ├── styles.css
│       ├── vite-env.d.ts
│       ├── app/
│       │   ├── providers.tsx
│       │   ├── error-boundary.tsx
│       │   ├── shortcuts.ts
│       │   ├── command-menu.tsx
│       │   └── update-banner.tsx
│       ├── api/
│       │   ├── client.ts
│       │   ├── errors.ts
│       │   ├── events.ts
│       │   ├── query-keys.ts
│       │   └── generated/
│       ├── auth/
│       │   ├── provider.tsx
│       │   ├── session.ts
│       │   ├── login-page.tsx
│       │   └── guard.tsx
│       ├── layout/
│       │   ├── app-shell.tsx
│       │   ├── sidebar.tsx
│       │   ├── header.tsx
│       │   ├── inspector.tsx
│       │   ├── status-bar.tsx
│       │   └── mobile-nav.tsx
│       ├── pages/
│       │   ├── home-page.tsx
│       │   ├── projects-page.tsx
│       │   ├── project-page.tsx
│       │   ├── dataset-page.tsx
│       │   ├── discovery-page.tsx
│       │   ├── candidate-page.tsx
│       │   ├── world-page.tsx
│       │   ├── simulation-page.tsx
│       │   ├── settings-page.tsx
│       │   └── not-found-page.tsx
│       ├── features/
│       │   ├── projects/
│       │   │   ├── api.ts
│       │   │   ├── model.ts
│       │   │   ├── project-card.tsx
│       │   │   ├── project-list.tsx
│       │   │   ├── project-form.tsx
│       │   │   └── project-menu.tsx
│       │   ├── datasets/
│       │   │   ├── api.ts
│       │   │   ├── model.ts
│       │   │   ├── dropzone.tsx
│       │   │   ├── import-dialog.tsx
│       │   │   ├── schema-table.tsx
│       │   │   ├── variable-editor.tsx
│       │   │   ├── role-editor.tsx
│       │   │   ├── unit-editor.tsx
│       │   │   ├── missingness-view.tsx
│       │   │   ├── distribution-view.tsx
│       │   │   ├── time-axis-view.tsx
│       │   │   ├── correlation-view.tsx
│       │   │   └── profile-summary.tsx
│       │   ├── discovery/
│       │   │   ├── api.ts
│       │   │   ├── model.ts
│       │   │   ├── plan-builder.tsx
│       │   │   ├── assumption-editor.tsx
│       │   │   ├── operator-editor.tsx
│       │   │   ├── constraints-editor.tsx
│       │   │   ├── stage-graph.tsx
│       │   │   ├── run-controls.tsx
│       │   │   ├── progress-stream.tsx
│       │   │   ├── branch-list.tsx
│       │   │   ├── candidate-table.tsx
│       │   │   ├── pareto-frontier.tsx
│       │   │   └── checkpoint-list.tsx
│       │   ├── equations/
│       │   │   ├── model.ts
│       │   │   ├── equation-card.tsx
│       │   │   ├── equation-editor.tsx
│       │   │   ├── equation-tree.tsx
│       │   │   ├── parameter-table.tsx
│       │   │   ├── units-badge.tsx
│       │   │   ├── residual-view.tsx
│       │   │   ├── term-importance.tsx
│       │   │   ├── alternative-list.tsx
│       │   │   └── latex-view.tsx
│       │   ├── structure/
│       │   │   ├── model.ts
│       │   │   ├── graph-canvas.tsx
│       │   │   ├── graph-toolbar.tsx
│       │   │   ├── edge-inspector.tsx
│       │   │   ├── lag-matrix.tsx
│       │   │   ├── stability-view.tsx
│       │   │   └── assumption-panel.tsx
│       │   ├── regimes/
│       │   │   ├── model.ts
│       │   │   ├── regime-timeline.tsx
│       │   │   ├── regime-card.tsx
│       │   │   ├── transition-matrix.tsx
│       │   │   ├── change-point-table.tsx
│       │   │   └── law-comparison.tsx
│       │   ├── simulation/
│       │   │   ├── api.ts
│       │   │   ├── model.ts
│       │   │   ├── world-lab.tsx
│       │   │   ├── initial-state-editor.tsx
│       │   │   ├── horizon-editor.tsx
│       │   │   ├── intervention-editor.tsx
│       │   │   ├── shock-editor.tsx
│       │   │   ├── event-editor.tsx
│       │   │   ├── scenario-list.tsx
│       │   │   ├── trajectory-chart.tsx
│       │   │   ├── phase-portrait.tsx
│       │   │   ├── ensemble-view.tsx
│       │   │   └── compare-view.tsx
│       │   ├── uncertainty/
│       │   │   ├── model.ts
│       │   │   ├── parameter-intervals.tsx
│       │   │   ├── structural-alternatives.tsx
│       │   │   ├── trajectory-envelope.tsx
│       │   │   └── sensitivity-view.tsx
│       │   ├── provenance/
│       │   │   ├── model.ts
│       │   │   ├── run-summary.tsx
│       │   │   ├── data-fingerprint.tsx
│       │   │   ├── environment-view.tsx
│       │   │   ├── assumptions-view.tsx
│       │   │   └── artifact-tree.tsx
│       │   └── export/
│       │       ├── api.ts
│       │       ├── export-dialog.tsx
│       │       ├── format-picker.tsx
│       │       └── share-dialog.tsx
│       ├── components/
│       │   ├── data-table/
│       │   ├── equation/
│       │   ├── graph/
│       │   ├── charts/
│       │   ├── forms/
│       │   ├── feedback/
│       │   └── primitives/
│       ├── hooks/
│       │   ├── use-event-stream.ts
│       │   ├── use-keyboard.ts
│       │   ├── use-local-storage.ts
│       │   ├── use-resize-observer.ts
│       │   └── use-theme.ts
│       ├── lib/
│       │   ├── format.ts
│       │   ├── download.ts
│       │   ├── math.ts
│       │   ├── color.ts
│       │   ├── date.ts
│       │   ├── units.ts
│       │   └── invariant.ts
│       ├── store/
│       │   ├── workspace.ts
│       │   ├── selection.ts
│       │   ├── panels.ts
│       │   └── preferences.ts
│       └── workers/
│           ├── bundle.worker.ts
│           ├── layout.worker.ts
│           └── simulation.worker.ts
├── docs-site/
│   ├── README.md
│   ├── package.json
│   ├── astro.config.mjs
│   ├── src/
│   └── public/
└── playground/
    ├── README.md
    ├── package.json
    ├── src/
    └── public/

packages/
├── api-client/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── client.ts
│       ├── errors.ts
│       ├── stream.ts
│       └── generated/
├── world-schema/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── validators.ts
│       ├── migrations.ts
│       └── generated/
├── world-viewer/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── viewer.tsx
│       ├── bundle.ts
│       ├── equation.tsx
│       ├── graph.tsx
│       └── trajectory.tsx
├── design-system/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── tokens.css
│       ├── theme.ts
│       ├── icons/
│       └── components/
├── chart-core/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── scales.ts
│       ├── axis.ts
│       ├── tooltip.ts
│       ├── downsample.ts
│       └── palette.ts
├── eslint-config/
│   ├── package.json
│   └── index.js
└── tsconfig/
    ├── package.json
    ├── base.json
    ├── react.json
    └── library.json
```

### 13.6 Services

Alpha uses `lawsynth-server` as one modular process. These service directories become separately deployable only when operational scale justifies it.

```text
services/
├── api/
│   ├── README.md
│   ├── Dockerfile
│   ├── pyproject.toml
│   ├── migrations/
│   │   ├── env.py
│   │   ├── script.py.mako
│   │   └── versions/
│   ├── src/lawsynth_api/
│   │   ├── __init__.py
│   │   ├── main.py
│   │   ├── app.py
│   │   ├── settings.py
│   │   ├── lifespan.py
│   │   ├── auth/
│   │   ├── db/
│   │   ├── routes/
│   │   ├── repositories/
│   │   ├── domain/
│   │   ├── middleware/
│   │   └── telemetry/
│   └── tests/
├── scheduler/
│   ├── README.md
│   ├── Dockerfile
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── scheduler.rs
│       ├── queue.rs
│       ├── lease.rs
│       ├── policy.rs
│       ├── pool.rs
│       ├── quota.rs
│       ├── recovery.rs
│       └── metrics.rs
├── worker/
│   ├── README.md
│   ├── Dockerfile
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── worker.rs
│       ├── lease.rs
│       ├── heartbeat.rs
│       ├── execute.rs
│       ├── checkpoint.rs
│       ├── upload.rs
│       ├── resources.rs
│       └── shutdown.rs
├── artifact/
│   ├── README.md
│   ├── Dockerfile
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── routes.rs
│       ├── object.rs
│       ├── upload.rs
│       ├── download.rs
│       ├── metadata.rs
│       ├── checksum.rs
│       ├── retention.rs
│       └── gc.rs
├── migration/
│   ├── README.md
│   ├── Dockerfile
│   ├── pyproject.toml
│   └── src/lawsynth_migration/
└── gateway/
    ├── README.md
    ├── Dockerfile
    └── config/
        ├── local.yaml
        └── production.yaml
```

### 13.7 Documentation and examples

```text
docs/
├── index.md
├── getting-started/
│   ├── installation.md
│   ├── ten-minute-world.md
│   ├── concepts.md
│   ├── studio.md
│   ├── python.md
│   ├── cli.md
│   └── self-hosting.md
├── concepts/
│   ├── world-model.md
│   ├── world-ir.md
│   ├── variables.md
│   ├── laws.md
│   ├── assumptions.md
│   ├── candidate-frontier.md
│   ├── causal-hypotheses.md
│   ├── regimes.md
│   ├── interventions.md
│   ├── uncertainty.md
│   └── provenance.md
├── guides/
│   ├── preparing-time-series.md
│   ├── units-and-dimensions.md
│   ├── irregular-sampling.md
│   ├── choosing-operators.md
│   ├── discovering-odes.md
│   ├── discovering-difference-equations.md
│   ├── finding-delays.md
│   ├── detecting-regimes.md
│   ├── comparing-candidates.md
│   ├── simulating-interventions.md
│   ├── handling-uncertainty.md
│   ├── exporting-worlds.md
│   ├── writing-a-method-plugin.md
│   └── reproducible-runs.md
├── methods/
│   ├── differentiation/
│   ├── sparse-discovery/
│   ├── symbolic-search/
│   ├── causal-structure/
│   ├── regimes/
│   ├── uncertainty/
│   └── simulation/
├── reference/
│   ├── python/
│   ├── rust/
│   ├── cli/
│   ├── http-api/
│   ├── configuration/
│   ├── world-ir/
│   └── bundle-format/
├── self-hosting/
│   ├── architecture.md
│   ├── docker-compose.md
│   ├── kubernetes.md
│   ├── storage.md
│   ├── database.md
│   ├── workers.md
│   ├── authentication.md
│   ├── backup.md
│   ├── upgrade.md
│   └── air-gapped.md
├── research/
│   ├── algorithm-notes.md
│   ├── benchmark-methodology.md
│   ├── scientific-limitations.md
│   ├── reading-list.md
│   └── citations.bib
├── contributing/
│   ├── development.md
│   ├── architecture.md
│   ├── adding-an-operator.md
│   ├── adding-an-algorithm.md
│   ├── adding-a-dataset.md
│   ├── documentation.md
│   ├── releases.md
│   └── governance.md
└── assets/
    ├── architecture.svg
    ├── world-ir.svg
    ├── pipeline.svg
    └── studio/

examples/
├── README.md
├── 00_quickstart/
│   ├── README.md
│   ├── discover.py
│   ├── simulate.py
│   └── expected/
├── 01_lorenz/
│   ├── README.md
│   ├── generate.py
│   ├── discover.py
│   ├── studio-project.json
│   └── expected/
├── 02_predator_prey/
├── 03_pendulum/
├── 04_epidemic/
├── 05_regime_switching/
├── 06_delayed_feedback/
├── 07_stochastic_volatility/
├── 08_supply_demand/
├── 09_inventory_control/
├── 10_energy_load/
├── 11_customer_growth/
├── 12_macro_dynamics/
├── 13_market_microstructure/
├── 14_synthetic_control/
├── 15_user_constraints/
├── 16_custom_operator/
├── 17_custom_stage/
├── 18_bundle_interchange/
├── 19_server_api/
└── notebooks/
    ├── quickstart.ipynb
    ├── equation-discovery.ipynb
    ├── regime-discovery.ipynb
    ├── intervention-lab.ipynb
    └── uncertainty.ipynb
```

### 13.8 Benchmarks, datasets, and scientific reference cases

```text
benchmarks/
├── README.md
├── registry.toml
├── methodology.md
├── environments/
│   ├── cpu-small.toml
│   ├── cpu-large.toml
│   └── reproducible.toml
├── equation-recovery/
│   ├── README.md
│   ├── problems/
│   │   ├── algebraic/
│   │   ├── rational/
│   │   ├── transcendental/
│   │   └── dimensional/
│   ├── manifests/
│   └── baselines/
├── dynamics/
│   ├── README.md
│   ├── ode/
│   ├── discrete/
│   ├── delay/
│   ├── stochastic/
│   ├── hybrid/
│   └── baselines/
├── causal-structure/
│   ├── README.md
│   ├── linear/
│   ├── nonlinear/
│   ├── lagged/
│   ├── confounded/
│   └── baselines/
├── regimes/
│   ├── README.md
│   ├── change-points/
│   ├── switching/
│   └── event-driven/
├── uncertainty/
│   ├── README.md
│   ├── parameter-coverage/
│   ├── structural-recovery/
│   └── trajectory-coverage/
├── performance/
│   ├── README.md
│   ├── expression-eval/
│   ├── symbolic-search/
│   ├── simulation/
│   ├── bundle-io/
│   └── end-to-end/
├── reports/
│   ├── schema.json
│   ├── templates/
│   └── published/
└── scripts/
    ├── run_suite.py
    ├── compare_baseline.py
    ├── aggregate.py
    ├── render_report.py
    └── verify_reproducibility.py

datasets/
├── README.md
├── LICENSES.md
├── registry.toml
├── cards/
│   ├── template.md
│   ├── lorenz.md
│   ├── lotka-volterra.md
│   ├── damped-pendulum.md
│   ├── sir.md
│   ├── regime-ar.md
│   ├── stochastic-volatility.md
│   └── supply-demand.md
├── generators/
│   ├── common.py
│   ├── lorenz.py
│   ├── lotka_volterra.py
│   ├── pendulum.py
│   ├── sir.py
│   ├── delay_feedback.py
│   ├── regime_ar.py
│   ├── stochastic_volatility.py
│   ├── supply_demand.py
│   ├── inventory.py
│   └── confounded_system.py
├── manifests/
│   ├── lorenz.toml
│   ├── lotka-volterra.toml
│   ├── pendulum.toml
│   ├── sir.toml
│   ├── regime-ar.toml
│   └── supply-demand.toml
└── small/
    ├── lorenz.parquet
    ├── lotka-volterra.parquet
    ├── pendulum.parquet
    ├── sir.parquet
    ├── regime-ar.parquet
    └── supply-demand.parquet
```

Large data must not live in Git. The registry records download URL, license, checksum, generator version, citation, intended purpose, and known limitations.

### 13.9 Cross-language and system verification

These are normal engineering quality assets. They are necessary for a mathematical engine, but they are not the product itself.

```text
tests/
├── README.md
├── conformance/
│   ├── README.md
│   ├── runner.py
│   ├── manifest.toml
│   ├── valid/
│   │   ├── minimal/
│   │   ├── continuous/
│   │   ├── discrete/
│   │   ├── stochastic/
│   │   ├── regime/
│   │   ├── hybrid/
│   │   └── signed/
│   └── invalid/
│       ├── bad-schema/
│       ├── bad-expression/
│       ├── bad-units/
│       ├── bad-hash/
│       └── unsafe-archive/
├── cross-language/
│   ├── README.md
│   ├── python_to_rust.py
│   ├── rust_to_python.rs
│   ├── typescript_to_rust.ts
│   ├── bundle_roundtrip.py
│   └── expected/
├── scientific/
│   ├── README.md
│   ├── equation_recovery.py
│   ├── trajectory_accuracy.py
│   ├── graph_recovery.py
│   ├── regime_recovery.py
│   ├── uncertainty_coverage.py
│   ├── adversarial_noise.py
│   ├── irregular_sampling.py
│   └── expected/
├── end-to-end/
│   ├── README.md
│   ├── local_library.py
│   ├── cli.sh
│   ├── local_studio.spec.ts
│   ├── server_run.spec.ts
│   ├── cancellation.spec.ts
│   ├── resume.spec.ts
│   ├── export.spec.ts
│   └── fixtures/
├── compatibility/
│   ├── README.md
│   ├── matrix.toml
│   ├── previous-bundles/
│   └── migration-cases/
├── chaos/
│   ├── README.md
│   ├── worker_loss.py
│   ├── storage_timeout.py
│   ├── duplicate_delivery.py
│   └── api_restart.py
├── security/
│   ├── archive_traversal.rs
│   ├── decompression_limits.rs
│   ├── plugin_permissions.rs
│   ├── expression_limits.rs
│   └── authorization.py
└── performance/
    ├── budgets.toml
    ├── compare.py
    └── datasets/

fuzz/
├── Cargo.toml
├── fuzz_targets/
│   ├── parse_expression.rs
│   ├── parse_bundle.rs
│   ├── migrate_bundle.rs
│   ├── evaluate_expression.rs
│   └── deserialize_world.rs
└── corpus/
```

### 13.10 Plugins and extension examples

```text
plugins/
├── README.md
├── sdk/
│   ├── rust/
│   └── python/
├── examples/
│   ├── custom-operator-rust/
│   │   ├── Cargo.toml
│   │   ├── plugin.toml
│   │   └── src/lib.rs
│   ├── custom-stage-python/
│   │   ├── pyproject.toml
│   │   ├── plugin.toml
│   │   └── src/custom_stage/
│   ├── csv-variant-adapter/
│   ├── external-simulator/
│   └── report-exporter/
└── registry/
    ├── schema.json
    └── index.json
```

Plugin execution rules:

- trusted native Rust plugins are explicit opt-in;
- Python plugins run out of process;
- portable plugins prefer WASI component boundaries;
- manifests declare filesystem, network, CPU, memory, and artifact permissions;
- plugin output is validated before entering World IR;
- server administrators can disable all plugins;
- the stable extension point is the protocol, not Rust ABI compatibility.

### 13.11 Deployment and infrastructure

```text
deploy/
├── README.md
├── compose/
│   ├── .env.example
│   ├── compose.yaml
│   ├── compose.gpu.yaml
│   ├── compose.observability.yaml
│   ├── init/
│   └── config/
├── docker/
│   ├── api.Dockerfile
│   ├── scheduler.Dockerfile
│   ├── worker.Dockerfile
│   ├── artifact.Dockerfile
│   ├── studio.Dockerfile
│   └── development.Dockerfile
├── helm/
│   └── lawsynth/
│       ├── Chart.yaml
│       ├── values.yaml
│       ├── values.schema.json
│       ├── templates/
│       │   ├── _helpers.tpl
│       │   ├── api-deployment.yaml
│       │   ├── api-service.yaml
│       │   ├── scheduler-deployment.yaml
│       │   ├── worker-deployment.yaml
│       │   ├── artifact-deployment.yaml
│       │   ├── ingress.yaml
│       │   ├── service-account.yaml
│       │   ├── configmap.yaml
│       │   ├── secrets.yaml
│       │   ├── network-policy.yaml
│       │   ├── pod-disruption-budget.yaml
│       │   └── migration-job.yaml
│       └── tests/
├── systemd/
│   ├── lawsynth.service
│   ├── lawsynth-worker.service
│   └── README.md
└── airgap/
    ├── README.md
    ├── manifest.yaml
    └── verify.sh

infra/
├── README.md
├── terraform/
│   ├── modules/
│   │   ├── network/
│   │   ├── database/
│   │   ├── object-store/
│   │   ├── nats/
│   │   ├── cluster/
│   │   ├── observability/
│   │   └── secrets/
│   └── examples/
│       ├── local/
│       ├── aws/
│       ├── gcp/
│       └── azure/
├── kubernetes/
│   ├── base/
│   └── overlays/
│       ├── development/
│       ├── staging/
│       └── production/
├── grafana/
│   ├── dashboards/
│   └── provisioning/
├── prometheus/
│   ├── rules.yaml
│   └── alerts.yaml
└── otel/
    ├── collector.yaml
    └── semantic-conventions.md
```

### 13.12 Repository tools and scripts

```text
tools/
├── schema-gen/
│   ├── Cargo.toml
│   └── src/main.rs
├── bundle-inspector/
│   ├── Cargo.toml
│   └── src/main.rs
├── benchmark-site/
│   ├── package.json
│   └── src/
├── license-check/
│   ├── pyproject.toml
│   └── src/
└── release-notes/
    ├── pyproject.toml
    └── src/

scripts/
├── bootstrap.sh
├── check.sh
├── test.sh
├── bench.sh
├── build-wheels.sh
├── build-studio.sh
├── generate-bindings.sh
├── generate-schemas.sh
├── generate-docs.sh
├── package-bundle-fixtures.sh
├── update-citations.sh
├── update-licenses.sh
├── verify-generated.sh
├── verify-reproducibility.sh
└── release-smoke.sh

xtask/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── command.rs
    ├── codegen.rs
    ├── conformance.rs
    ├── docs.rs
    ├── package.rs
    ├── release.rs
    └── verify.rs

assets/
├── brand/
│   ├── logo.svg
│   ├── logo-mark.svg
│   ├── wordmark.svg
│   ├── palette.json
│   └── typography.md
├── readme/
│   ├── hero.webp
│   ├── lorenz-demo.gif
│   ├── pipeline.svg
│   └── studio.webp
└── social/
    ├── github-card.png
    ├── announcement.png
    └── demo-thumbnail.png
```

---

## 14. File inventory and code-size estimate

File count is a planning signal, not a quality target. The repo should grow through real modules, documentation, examples, schemas, conformance cases, and scientific fixtures. It should never be padded to look large.

### Mature v1 tracked-file budget

| Area | Hand-authored | Fixtures/generated/snapshots | Total |
|---|---:|---:|---:|
| Root, governance, GitHub automation | 62 | 0 | 62 |
| Specs, schemas, protocol contracts | 78 | 64 | 142 |
| Rust engine and native tools | 540 | 150 | 690 |
| Python packages and adapters | 260 | 95 | 355 |
| Studio and TypeScript packages | 285 | 105 | 390 |
| Service processes and migrations | 165 | 70 | 235 |
| Documentation and site content | 185 | 45 | 230 |
| Examples and notebooks | 110 | 70 | 180 |
| Benchmarks and dataset manifests | 125 | 145 | 270 |
| Cross-language/system suites | 115 | 190 | 305 |
| Plugins and SDK examples | 58 | 22 | 80 |
| Deployment and infrastructure | 115 | 15 | 130 |
| Repository tools and brand assets | 74 | 18 | 92 |
| **Total mature v1** | **2,172** | **989** | **3,161** |

This is a mature production repository, not the alpha starting point.

### Growth by release

| Milestone | Tracked files | Hand-written LOC, excluding generated/vendor | What exists |
|---|---:|---:|---|
| Architecture scaffold | 120–180 | 5k–12k | specs, IR skeleton, build, one example |
| v0.1 engine alpha | 350–500 | 45k–75k | Python API, ODE/discrete discovery, simulation, CLI |
| v0.2 Studio alpha | 650–850 | 85k–130k | local Studio, artifacts, candidate explorer |
| v0.5 public beta | 1,200–1,600 | 150k–230k | regimes, uncertainty, bundles, server mode |
| v1.0 production | 2,600–3,200 | 260k–390k | stable format, distributed jobs, broad docs/benchmarks |
| v2 research ecosystem | 4,000+ | 450k+ | PDE/latent/neural extensions, plugin ecosystem |

### Approximate mature v1 file composition

```text
Rust source and crate metadata             ~690
Python source and package metadata         ~355
TypeScript/CSS/UI assets                   ~390
Schemas/protobuf/generated bindings        ~142
Docs/examples/notebooks                    ~410
Scientific cases and fixtures              ~575
Services/deployment/operations             ~365
Governance/tools/brand                     ~234
                                         ------
                                          ~3,161
```

The first commit should contain roughly 120 files, not 3,000. The remaining files are earned release by release.

---

## 15. Production data model

### Metadata database

Local mode maps the same domain model to SQLite. Server mode uses Postgres.

```text
organizations
users
memberships
api_keys

projects
project_members
project_settings

datasets
dataset_versions
dataset_variables
dataset_profiles
dataset_artifacts

discovery_plans
runs
run_attempts
run_stages
run_events
run_checkpoints
run_artifacts

candidates
candidate_scores
candidate_equations
candidate_dependencies
candidate_regimes

worlds
world_versions
world_artifacts
world_tags

scenarios
interventions
simulation_runs
trajectory_artifacts

plugins
plugin_versions
plugin_installations

worker_pools
workers
job_leases
resource_quotas

audit_events
idempotency_keys
outbox_events
```

Design rules:

- UUIDv7 or sortable identifiers;
- immutable dataset and world versions;
- mutable human names point to immutable versions;
- large numerical payloads live in object storage, never database blobs;
- database rows reference artifacts by content hash;
- soft deletion for user-facing objects, retention workflow for artifacts;
- transactional outbox for reliable job/event publication;
- explicit tenant/organization key on every server-owned row;
- row-level authorization in application logic and optional Postgres RLS;
- append-only audit events for security-relevant server actions;
- schema migrations tested against the previous two minor versions.

### Object storage layout

```text
objects/sha256/ab/cd/<full-hash>
uploads/<organization>/<upload-id>/<part>
exports/<organization>/<export-id>/<file>
previews/<world-version>/<asset>
checkpoints/<run-attempt>/<sequence>.checkpoint
```

Logical paths are metadata only. Content hashes control physical deduplication. Garbage collection marks live references from the database, waits through a safety window, then sweeps unreferenced objects.

### Event envelope

```json
{
  "spec_version": "1.0",
  "event_id": "019...",
  "organization_id": "019...",
  "run_id": "019...",
  "attempt_id": "019...",
  "sequence": 184,
  "occurred_at": "2026-08-16T12:00:00Z",
  "kind": "candidate.discovered",
  "stage": "symbolic_search",
  "progress": 0.42,
  "payload": {},
  "trace_id": "..."
}
```

Progress is monotonic within a stage but not across replanned branches. The UI uses stage weights and event sequence, not arrival time, to render status.

---

## 16. Algorithm contracts

Every discovery stage implements a small protocol and consumes immutable inputs.

```text
StageDescriptor
  id
  version
  capabilities
  accepted_inputs
  produced_outputs
  resource_profile
  determinism_class

StageRequest
  run_context
  input_artifacts
  configuration
  assumptions
  seed_stream
  cancellation

StageResult
  output_artifacts
  candidates
  measurements
  warnings
  provenance
  checkpoint
```

### Determinism classes

- `deterministic` — same inputs and version produce the same bytes;
- `seeded` — same inputs, version, hardware class, and seed produce the same result;
- `numerically_stable` — small platform variation permitted within declared tolerance;
- `external` — remote/provider behavior cannot be guaranteed.

### Candidate score vector

Candidates retain individual measurements instead of collapsing immediately to a magic scalar:

```text
fit_train
fit_holdout
trajectory_error
complexity_nodes
complexity_description_length
parameter_count
unit_consistency
residual_structure
bootstrap_stability
regime_stability
graph_stability
extrapolation_risk
simulation_failure_rate
compute_cost
```

The frontier uses Pareto dominance. A configurable final selector may rank frontier members, but the UI always exposes the trade-off.

### Assumption object

Assumptions are first-class and hashable:

- variable roles;
- temporal direction;
- allowed and forbidden edges;
- allowed operators and constants;
- monotonicity and sign constraints;
- bounds and conservation relations;
- units;
- maximum lags;
- known interventions;
- environment/regime labels;
- missingness and measurement-error model;
- stochasticity choice;
- smoothness and sparsity preferences.

Changing assumptions creates a new run. It never mutates old evidence.

---

## 17. MVP boundary

### v0.1 must support

- Linux, macOS, and Windows Python wheels;
- Python 3.11–3.13 where build ecosystem permits;
- CSV, Parquet, pandas, Polars, NumPy, and Arrow inputs;
- numeric multivariate time series with one time axis;
- regular and moderately irregular sampling;
- observed states and exogenous/control variables;
- continuous ODE and discrete recurrence discovery;
- Savitzky–Golay, spline, and TV-regularized derivative estimates;
- polynomial/trigonometric feature libraries;
- STLSQ/SR3 sparse discovery;
- typed genetic symbolic search;
- constant fitting and equation simplification;
- holdout trajectory score and Pareto frontier;
- deterministic native simulation;
- parameter interventions and state shocks;
- bootstrap parameter/trajectory uncertainty;
- `.lsworld` read/write;
- Python API and CLI;
- Lorenz, Lotka–Volterra, pendulum, SIR, and regime examples.

### v0.1 explicitly does not support

- PDE discovery;
- image, text, molecule, or graph observations;
- autonomous literature research;
- general hidden-state identification;
- production causal-effect claims;
- arbitrary Python expressions inside Rust execution;
- GPU execution;
- distributed workers;
- collaborative editing;
- enterprise identity providers;
- real-time unbounded streaming;
- LLM-generated equations as a required path.

### v0.2 adds the “star-worthy” product layer

- Local Studio;
- dataset profiler;
- live discovery progress;
- equation and Pareto explorer;
- editable assumptions;
- phase portrait and trajectory comparison;
- intervention lab;
- shareable read-only world viewer;
- one-command demo data;
- browser-hosted WASM simulation for exported examples.

### v0.5 adds model richness

- change-point and regime discovery;
- candidate dependency graph;
- structural alternatives;
- stronger uncertainty propagation;
- event/hybrid simulation;
- plugin protocol preview;
- Docker Compose server mode;
- stable bundle release candidate.

### v1.0 contract

- World IR 1.0 and `.lsworld` 1.0 compatibility promise;
- production local and single-node self-hosting;
- resumable jobs;
- multi-user projects;
- object storage and Postgres;
- versioned worlds and scenario history;
- published scientific benchmark methodology;
- language bindings generated from stable schemas;
- documented upgrade and backup procedure;
- security review of archive, expression, and plugin boundaries;
- at least three external research groups using the extension API.

---

## 18. Roadmap

The calendar assumes one strong full-time engineer, selective external contributors, and disciplined scope. Add 30–50% buffer if this is not the primary project.

### Phase 0 — reserve and specify (weeks 1–2)

Deliverables:

- reserve name and package namespaces;
- publish design manifesto and non-goals;
- create Apache-2.0 repository and governance basics;
- specify World IR draft and bundle draft;
- establish Rust/Python workspace and PyO3 round trip;
- implement minimal expression AST, variables, laws, and world;
- store/load a hand-written Lorenz world;
- set quantitative performance and scientific-recovery budgets.

Exit test: Python creates a World, Rust simulates it, the bundle round-trips byte-stably, and a plot matches the reference trajectory.

### Phase 1 — executable world core (weeks 3–6)

Deliverables:

- Arrow dataset boundary;
- expression parser, evaluator, simplifier, and derivative;
- units and dimensional consistency;
- continuous and discrete world semantics;
- ODE/discrete simulator;
- interventions and trajectory output;
- Python `World` and `Scenario` API;
- initial CLI;
- conformance fixtures.

Exit test: five known worlds simulate correctly from Rust, Python, and serialized bundles.

### Phase 2 — discovery alpha (weeks 7–12)

Deliverables:

- profiling and preprocessing provenance;
- derivative methods;
- feature libraries;
- STLSQ and SR3;
- typed symbolic search;
- constant optimization;
- Pareto scoring;
- checkpoints and cancellation;
- bootstrap uncertainty;
- `ls.discover()`.

Exit test: recover recognizable Lorenz and Lotka–Volterra structures from noisy data within a documented CPU budget.

### Phase 3 — public engine release (weeks 13–15)

Deliverables:

- wheels for major platforms;
- CLI installers;
- quickstart and API documentation;
- benchmark runner and baseline report;
- five polished examples;
- deterministic release process;
- issue templates and contribution guides;
- v0.1 release.

Exit test: a new user goes from install to recovered equations in under ten minutes without repository checkout.

### Phase 4 — Studio and launch demo (weeks 16–22)

Deliverables:

- local server mode;
- Studio shell and project storage;
- data upload/profile screens;
- run plan and progress stream;
- equation/Pareto explorer;
- World Lab and comparison views;
- WASM world viewer;
- excellent README animation and launch site;
- v0.2 release.

Exit test: a user completes the entire Lorenz demo without writing code and exports a self-contained world.

### Phase 5 — dynamic worlds (weeks 23–30)

Deliverables:

- PELT, BOCPD, and HMM regimes;
- regime-specific law sets;
- lag/dependency discovery;
- graph stability;
- event and hybrid simulation;
- structural uncertainty display;
- assumption editor;
- v0.3 release.

Exit test: discover both change points and different dynamics in a switching synthetic system.

### Phase 6 — production beta (weeks 31–40)

Deliverables:

- modular service process;
- Postgres and S3-compatible artifact storage;
- worker isolation;
- lease, heartbeat, retry, cancellation, and checkpoint semantics;
- multi-user projects and API keys;
- Compose deployment;
- plugin protocol preview;
- compatibility migrations;
- v0.5 release.

Exit test: interrupt and resume a multi-hour run without corrupting results; restore a deployment from backup.

### Phase 7 — v1 stabilization (weeks 41–52)

Deliverables:

- freeze World IR 1.0 and bundle 1.0;
- publish conformance kit;
- performance work and memory budgets;
- broad scientific reference cases;
- complete operations and security documentation;
- independent reproduction of benchmark results;
- extension API adopters;
- v1.0 release.

Exit test: previous release bundles load without loss, three external plugins pass conformance, and published examples reproduce on clean machines.

---

## 19. Priority stack

### P0: project-defining work

1. World IR semantics.
2. Reliable simulation.
3. Equation discovery that works on real noisy cases.
4. Pareto alternatives and honest uncertainty.
5. Python experience.
6. Visual intervention demo.
7. Open bundle and reproducibility.

### P1: differentiation

1. Regime-aware law discovery.
2. Equation + graph + regime integration.
3. Constraints and units in the search.
4. Compiled portable worlds.
5. Research extension protocol.
6. Beautiful candidate and scenario exploration.

### P2: production adoption

1. resumable long-running jobs;
2. multi-user self-hosting;
3. object storage and lifecycle;
4. organization auth and quotas;
5. broad connectors;
6. deployment automation.

### Avoid until evidence demands it

- generic chat interface;
- multi-agent “scientist team”;
- custom foundation model;
- dozens of industry templates;
- GPU cluster scheduler;
- custom dataframe engine;
- custom database;
- PDE and spatial worlds;
- mobile application;
- proprietary hosted-only features;
- marketplace before a real plugin ecosystem.

---

## 20. Competitive position

### Adjacent OSS projects

| Project | Strength | LawSynth should not duplicate | Remaining gap for LawSynth |
|---|---|---|---|
| [PySR](https://github.com/MilesCranmer/PySR) | powerful symbolic regression | generic symbolic-expression search alone | dynamic multi-equation world, regimes, interventions, portable runtime |
| [PySINDy](https://github.com/dynamicslab/pysindy) | sparse nonlinear dynamics discovery | SINDy method collection alone | full world representation, causal hypotheses, Studio, deployment |
| [DoWhy](https://github.com/py-why/dowhy) | causal assumptions and effect workflow | broad causal inference reimplementation | dynamics-first discovery and executable simulation |
| [Tigramite](https://github.com/jakobrunge/tigramite) | causal discovery for time series | full causal-method catalog | equation synthesis and unified world execution |
| [pgmpy](https://github.com/pgmpy/pgmpy) | graphical and probabilistic models | probabilistic graph toolkit | discovered law system and scenario runtime |
| [DeepCausality](https://github.com/deepcausality-rs/deep_causality) | Rust causality and dynamic context | generic causal library | end-user discovery workflow and equation engine |
| [AI-Descartes](https://github.com/IBM/AI-Descartes) | scientific equation discovery with theory | paper-specific discovery pipeline | general OSS product, world format, simulator, Studio |
| [LLM-SR](https://github.com/deep-symbolic-mathematics/LLM-SR) | LLM-guided scientific regression | LLM-only equation generation | offline deterministic engine and integrated worlds |
| [SciForge](https://github.com/AGI4Sci/SciForge) | broad AI scientific workbench | general research agent/workbench | mathematical dynamics engine as the product |
| [DiscoveryWorld](https://github.com/allenai/discoveryworld) | environment for scientific-discovery agents | simulated agent benchmark | learning runnable laws from user observations |

### Defensible thesis

No single algorithm is defensible forever. LawSynth’s durable position is:

1. a well-designed open World IR;
2. interoperability across discovery methods;
3. a high-performance execution and intervention runtime;
4. accumulated scientific benchmarks and conformance cases;
5. the best visual interface for inspecting discovered dynamics;
6. a contributor ecosystem adding methods without rewriting the product.

### One sentence to avoid

“There are no competitors.”

### Better sentence

“Excellent open-source tools solve individual parts of equation discovery, causal inference, and simulation; LawSynth makes their outputs composable as one inspectable, executable world.”

---

## 21. Performance budgets

Set budgets before implementation so architectural mistakes are visible.

### Alpha reference machine

Use a published 8-core x86-64 CPU, 32 GB RAM, and no GPU as the standard benchmark class.

| Operation | Alpha target |
|---|---:|
| Import `lawsynth` | under 1.0 s warm |
| Load 1M-row, 12-column Parquet | under 3 s and under 2× file-size peak memory |
| Profile 1M × 12 numeric table | under 8 s |
| Evaluate 10k-node expression batch | over 10M scalar node-evaluations/s |
| Simulate 3-state ODE, 100k steps | under 1 s for deterministic fixed-step path |
| Bundle open, metadata only | under 100 ms for small world |
| Bundle deterministic round trip | byte-stable except declared preview timestamps |
| Cancel a discovery run | cooperative stop under 2 s; hard stop under 10 s |
| Studio first useful paint | under 2 s on local connection |
| Progress event to UI | p95 under 250 ms locally |

Scientific budgets matter more than microbenchmarks:

- structure recovery rate on published synthetic suite;
- trajectory error on withheld initial conditions;
- parameter interval coverage;
- false edge rate under known confounding cases;
- regime boundary accuracy;
- simulation failure rate outside training support;
- run-to-run stability across seeds;
- complexity at matched prediction quality.

Do not advertise a single aggregate accuracy number. Publish the problem matrix and failure cases.

---

## 22. Security and trust boundaries

Although LawSynth is not a security product, it processes untrusted datasets, archives, plugins, and expressions.

### Main threats

- malicious `.lsworld` archive paths or decompression bombs;
- oversized/recursive expression graphs;
- user plugins reading data or network without permission;
- arbitrary Python code smuggled through serialized objects;
- tenant artifact leakage;
- forged run provenance;
- job resource exhaustion;
- unsafe generated code;
- vulnerable scientific dependencies;
- leaked data through opt-out telemetry mistakes.

### Required controls

- never use pickle for portable artifacts;
- normalize and reject unsafe archive paths;
- cap archive entries, expanded bytes, expression depth, and node count;
- validate schemas before allocating large structures;
- run Python plugins and generated code out of process;
- deny plugin network/filesystem by default;
- enforce organization-scoped authorization at repository boundaries;
- use short-lived signed artifact download URLs in server mode;
- content checksums and optional bundle signatures;
- resource limits per run and worker pool;
- secret-free job envelopes;
- dependency advisories and signed release provenance;
- telemetry disabled by default for local mode;
- no dataset content in telemetry or logs;
- documented security contact and coordinated disclosure.

### Trust levels

```text
Level 0  data only, no executable extensions
Level 1  World IR expressions executed by bounded native interpreter
Level 2  WASI plugin with declared capabilities
Level 3  out-of-process Python plugin
Level 4  trusted native plugin or generated native code
```

Studio must show the trust level before opening or executing a bundle.

---

## 23. Reliability and operations

### Service-level objectives after v1

| Surface | Suggested SLO |
|---|---:|
| API availability | 99.9% monthly for self-hosted reference deployment |
| accepted run persistence | 99.99% once API returns accepted |
| artifact checksum integrity | 100% detected on read |
| event ordering | monotonic per run attempt |
| job cancellation acknowledgment | p95 under 2 s |
| metadata backup RPO | 15 min reference configuration |
| metadata restore RTO | under 2 h documented procedure |

### Failure behavior

- API outage does not corrupt active worker artifacts;
- worker loss returns the job to schedulable state after lease expiry;
- checkpoint-compatible jobs resume, others restart explicitly;
- artifact upload finalizes only after checksum verification;
- scheduler is reconstructable from database state;
- duplicate messages are safe through idempotent handlers;
- schema migration failure stops rollout before new application replicas;
- partial exports are never presented as complete bundles;
- Studio can reconnect and replay events from the last sequence.

### Internal observability

Track:

- runs by stage, state, method, and failure class;
- queue delay and lease age;
- CPU time, peak RSS, artifact bytes, and candidate counts;
- expression-evaluation and simulation latency;
- checkpoint size and recovery success;
- API latency and error class;
- object-store and database saturation;
- scientific warnings such as unstable candidates or unsupported extrapolation.

Dataset names, column values, equations, and user prompts must not appear in telemetry by default.

---

## 24. Licensing and governance

### License recommendation

Use **Apache License 2.0** for:

- Rust engine;
- Python API;
- Studio;
- service layer;
- schemas and format specifications;
- SDKs and first-party plugins.

Why:

- permissive commercial adoption;
- explicit patent grant;
- familiar to infrastructure and scientific users;
- permits companies to embed LawSynth while still encouraging upstream collaboration.

Documentation can use CC BY 4.0 and datasets keep their source-specific licenses. Do not silently relicense imported benchmark data.

### “Genuinely open source” promise

- all discovery algorithms developed by the project are in the public repo;
- Studio is open, not a crippled community UI;
- distributed services and deployment manifests are open;
- `.lsworld` is vendor-neutral and documented;
- no hosted-only model format;
- contributors retain copyright under a DCO, with no copyright assignment;
- optional paid hosting may sell operations, scale, support, and convenience—not hidden core capability.

### Governance progression

1. founder-led maintainership through v0.x;
2. public RFCs for World IR and bundle changes;
3. subsystem maintainers after sustained contribution;
4. technical steering group after at least three independent organizations contribute;
5. consider neutral foundation only after the ecosystem exists.

---

## 25. GitHub-star and adoption strategy

Ten thousand stars are not an engineering requirement, but the project can be designed for attention without becoming shallow.

### Launch assets

- 8–12 second Lorenz discovery GIF above the fold;
- one-line install and one-screen Python example;
- interactive browser world where visitors change a parameter;
- “equations recovered from noisy data” visual comparison;
- public benchmark dashboard including failures;
- clear statement of what is new and what existing projects inspired;
- polished architecture diagram;
- five copy-paste examples across science, finance, and business dynamics;
- no login and no API key for the core demo.

### Release story

**v0.1:** “An open-source compiler from time series to executable equations.”  
**v0.2:** “Upload data, discover the dynamics, change a parameter, watch the world respond.”  
**v0.3:** “One system can obey different laws: LawSynth now discovers regimes.”  
**v1.0:** “The open format and runtime for discovered world models.”

### Community loops

- monthly “discover this system” challenge;
- benchmark contributions with visible attribution;
- gallery of community `.lsworld` models;
- method plugins linked to papers;
- reproduction badges for examples;
- small, well-scoped `good first issue` modules;
- public design calls for World IR RFCs;
- integrations with PySR, PySINDy, DoWhy, and scientific notebooks rather than hostile replacement claims.

### Star-killing mistakes

- README begins with architecture instead of result;
- first install compiles Rust for twenty minutes;
- demo requires an LLM API key;
- “AI scientist” claim produces only chat text;
- UI exists but math does not recover known systems;
- only toy noiseless data works;
- causality is overstated;
- repository launches with thousands of empty files;
- core features are immediately moved behind a hosted paywall.

---

## 26. Principal risks and pivots

| Risk | Early signal | Mitigation |
|---|---|---|
| Scope becomes “solve all science” | roadmap fills with chemistry, PDEs, papers, agents | enforce numeric time-series v0.x boundary |
| Symbolic search is too slow | Lorenz demo needs hours | warm-start from sparse discovery; bounded grammar; Rust evaluator; checkpoints |
| Real data produces unstable laws | candidate rankings change by seed/window | structural ensembles, stability scores, alternative frontier, constraints |
| Product overclaims causality | users treat graph as truth | assumption-centered UI and candidate wording |
| Rust slows research iteration | every new method needs complex native code | Python stage protocol; promote stable hot paths to Rust later |
| Python/Rust copies dominate | high peak memory on large data | Arrow boundary and batch processing |
| UI consumes whole project | engine quality stalls | launch CLI/library first; Studio only after recovery targets pass |
| Package build friction | users cannot install wheels | release wheels before announcement; limited supported matrix |
| Existing libraries move into integration | competitor adds simulation/UI | win through open World IR, composition, and UX; integrate their algorithms |
| Single maintainer burnout | reviews and support block research | narrow issue templates, maintainers, roadmap limits, funded milestones |

### Healthy pivots that preserve the core

If full discovery is weaker than expected, do not pivot to generic agents. Better pivots are:

1. **World IR and runtime first:** an open interchange and execution format for discovered dynamical models.
2. **Regime discovery studio:** the best visual product for systems whose laws change over time.
3. **Equation compiler:** compile symbolic dynamics to fast Rust/Python/WASM simulators.
4. **Scientific model gallery:** executable, versioned, intervention-ready system models.
5. **Method workbench:** composable scientific-discovery pipelines that still output World IR.

Each pivot reuses the engine instead of discarding it.

---

## 27. First 30 implementation issues

These issues create a coherent vertical slice rather than scaffolding every future directory.

1. Establish Cargo, uv, and pnpm workspaces.
2. Add license, governance, contribution, and architecture documents.
3. Define IDs, versions, stable hashing, and error taxonomy.
4. Implement scalar expression AST.
5. Implement expression parser and canonical printer.
6. Implement expression evaluator.
7. Implement simplification and canonical hashing.
8. Define dimensions and unit checking.
9. Define variables, roles, parameters, and time semantics.
10. Define continuous and discrete laws.
11. Define `World` builder and semantic validation.
12. Define bundle manifest draft.
13. Implement deterministic bundle writer and safe reader.
14. Add Lorenz world fixture.
15. Implement fixed-step RK4 simulator.
16. Implement adaptive ODE solver adapter.
17. Implement discrete recurrence simulator.
18. Implement parameter and state interventions.
19. Expose World and simulation through PyO3.
20. Build Python `World`, `Scenario`, and `Trajectory` API.
21. Add Arrow Dataset boundary.
22. Implement sampling/time-axis profile.
23. Implement Savitzky–Golay differentiation.
24. Implement polynomial feature library.
25. Implement STLSQ discovery.
26. Convert sparse result into World IR.
27. Add trajectory and complexity scoring.
28. Implement candidate Pareto frontier.
29. Ship `lawsynth discover` and `lawsynth simulate` CLI.
30. Publish noisy Lorenz recovery benchmark and animated demo.

At issue 30, the project is real. Only then add more crates from the production tree.

---

## 28. Release acceptance criteria

### v0.1 engine alpha

- clean `pip install lawsynth` on supported platforms;
- no compiler required for end users;
- Python quickstart completes under ten minutes;
- at least four of five reference systems recover recognizable terms under published noise settings;
- every result includes seed, data hash, plan hash, and algorithm versions;
- bundle round-trip conformance passes across Rust and Python;
- cancellation leaves no corrupt artifact;
- unsupported/unstable cases emit clear warnings;
- docs explain observational and causal limitations.

### v0.2 Studio alpha

- zero-code Lorenz workflow completes in browser;
- user can compare at least three frontier candidates;
- equations render and remain linked to metrics;
- user can apply an intervention and compare trajectories;
- refresh/restart preserves project and run history;
- UI handles run failure and reconnection;
- exported bundle reopens locally and in read-only viewer.

### v1.0

- World IR and bundle conformance suite public;
- upgrade from latest v0.5 without data loss;
- jobs survive worker restart through retry or checkpoint;
- server authorization has no known cross-tenant leaks;
- backups and restore exercised in release procedure;
- performance budgets measured on published hardware;
- scientific benchmark report independently reproducible;
- public compatibility policy and deprecation window;
- at least three non-core example plugins;
- at least one real dataset case from finance/business and one from physical or biological science.

---

## 29. Recommended first-year team shape

The alpha can be founder-built. A credible v1 benefits from:

- **1 systems/scientific founder:** Rust core, World IR, discovery architecture;
- **1 scientific Python contributor:** algorithms, adapters, benchmarks, examples;
- **1 product/frontend engineer:** Studio and visual model exploration;
- **part-time statistical advisor:** assumptions, causal language, uncertainty methodology;
- **part-time design/documentation help:** README, demos, tutorials, visual identity.

If only one person is available, sequence the roles rather than doing all tracks simultaneously:

1. engine and Python API;
2. benchmark and documentation;
3. Studio;
4. server mode;
5. distributed production.

---

## 30. Final recommendation

Build **LawSynth** as an open-source **world-model compiler**, not as a vague AI scientist.

The project earns attention through a powerful visual promise—data goes in, governing equations and an explorable world come out—but earns long-term value through a serious architecture:

- typed World IR;
- Rust discovery and simulation engine;
- excellent Python scientific API;
- local-first Studio;
- honest uncertainty and assumptions;
- open `.lsworld` artifact;
- reproducible benchmarks;
- OSS services and plugin protocol.

Start with approximately 120–180 files and one undeniable end-to-end demo. Reach 350–500 files at the engine alpha, 650–850 with Studio, 1,200–1,600 in beta, and roughly **3,161 meaningful tracked files** at a mature production v1. The target is not repository size. The target is a system researchers and companies can trust, extend, inspect, and run without asking permission.

---

## Appendix A. Research and competitor references

- [PySR — symbolic regression](https://github.com/MilesCranmer/PySR)
- [PySINDy — sparse identification of nonlinear dynamics](https://github.com/dynamicslab/pysindy)
- [DoWhy — causal inference](https://github.com/py-why/dowhy)
- [Tigramite — causal discovery for time series](https://github.com/jakobrunge/tigramite)
- [pgmpy — probabilistic and causal graphical models](https://github.com/pgmpy/pgmpy)
- [DeepCausality — Rust causality library](https://github.com/deepcausality-rs/deep_causality)
- [AI-Descartes — scientific discovery combining data and theory](https://github.com/IBM/AI-Descartes)
- [LLM-SR — LLM-guided scientific regression](https://github.com/deep-symbolic-mathematics/LLM-SR)
- [LLM-SRBench — equation-discovery benchmark](https://github.com/deep-symbolic-mathematics/llm-srbench)
- [CausalDynamics — benchmark for causal discovery in dynamical systems](https://github.com/kausable/CausalDynamics)
- [SciForge — AI scientific workbench](https://github.com/AGI4Sci/SciForge)
- [DiscoveryWorld — environment for automated scientific discovery](https://github.com/allenai/discoveryworld)
- [CodeScientist — code-experiment discovery system](https://github.com/allenai/codescientist)
- [Apache Arrow — language-independent columnar memory format](https://github.com/apache/arrow)
- [PyO3 — Rust bindings for Python](https://github.com/PyO3/pyo3)
- [maturin — building and publishing Rust-based Python packages](https://github.com/PyO3/maturin)

## Appendix B. Decision log

| Decision | Chosen | Rejected for now | Reason |
|---|---|---|---|
| Product identity | world-model compiler | general AI scientist | precise, buildable, demonstrable |
| Initial data | multivariate numeric time series | all scientific modalities | coherent end-to-end path |
| Core runtime | Rust | Python-only | search/simulation performance and portable runtime |
| User API | Python | Rust-only | scientific ecosystem and contributor velocity |
| Interface | React/TypeScript Studio | desktop-native UI | web sharing and visualization ecosystem |
| Data boundary | Arrow/Parquet | custom dataframe | interoperability and lower copying |
| Artifact | open `.lsworld` bundle | database-only model | portability and reproducibility |
| Architecture | modular monolith first | microservices first | lower complexity and faster iteration |
| Jobs | child process locally; leased workers later | in-request execution | cancellation and fault isolation |
| Discovery result | Pareto frontier | one opaque score | scientific honesty and user choice |
| LLM role | optional prior/explanation plugin | mandatory engine | offline use and non-wrapper identity |
| License | Apache-2.0 | open core | genuine OSS and patent clarity |
