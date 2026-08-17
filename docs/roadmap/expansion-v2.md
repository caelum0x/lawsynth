# LawSynth — Expansion v2 (toward a ~10,000-file platform)

The original manifest (`LawSynth_3161_Repository_Manifest.md`) planned P0–P5 at
**3,161 files**; the repo is now ~3,460 with P6–P10 built. This document plans the
**next expansion** — the competitor-informed capabilities from
[`competitive-analysis.md`](../research/competitive-analysis.md) built out across
*every surface* (spec → crate → tests → conformance → benchmark → SDK → CLI →
Studio → service → docs → cookbook → example). Building each capability across all
surfaces is exactly what multiplies file count from ~3.5k toward **~10k** — the
same discipline the original manifest used, applied to a bigger scope.

**Contract (unchanged):** every capability ships as a *boundary spec + conformance
suite*, is deterministic and offline, and never regresses the local core. Files
are added only when the milestone is reached; this is a plan, not a scaffold.

## The multiplier: one capability, ten surfaces

Each engine capability (e.g. "weak-form discovery") lands as, roughly:
`spec/ (3) + crate src (8) + crate tests (6) + conformance cases (5) + benchmark
cases (6) + SDK module+tests (4) + CLI command+tests (3) + Studio screen (3) +
service endpoint+tests (3) + docs+cookbook+example (5)` ≈ **~46 files**.
That is why ~30 new capabilities → ~1,400 files just for the engine, and the
ecosystem/data/platform tiers carry the rest.

## Milestones

### v2-A — Engine breadth (close the discovery gaps) · ~1,600 files

New Rust crates (each: `src/` 6–12, `tests/` 5–10, `benches/` 1–2) + a spec dir +
conformance cases + benchmark family + SDK/CLI/Studio/service surfaces:

| Capability | New crate(s) | Spec | ~files |
|---|---|---|---:|
| **Weak / integral-form** discovery | `lawsynth-weakform` | `specs/weak-form/` | 55 |
| **Koopman / DMD / EDMD** | `lawsynth-koopman` | `specs/koopman/` | 60 |
| **Implicit / rational nullspace** | `lawsynth-implicit` | `specs/implicit-dynamics/` | 55 |
| **Units-in-discovery** (dim. pruning + Buckingham-π) | extend `lawsynth-units`, `lawsynth-discovery` | `specs/dimensional-search/` | 45 |
| **Symmetry / separability** reductions | `lawsynth-reduce` | `specs/structural-reductions/` | 55 |
| Optimizer breadth (FROLS/SSR/trapping/L0) | extend `lawsynth-sparse` | `specs/optimizers/` | 50 |
| Deterministic ensembling | `lawsynth-ensemble` | `specs/ensembling/` | 45 |
| Control inputs (SINDYc discovery) | extend `lawsynth-discovery`, `-world` | `specs/control-discovery/` | 45 |
| MDL / description-length objective | extend `lawsynth-score` | `specs/model-selection/` | 35 |
| Analytic-Jacobian symbolic diff + codegen | `lawsynth-jacobian` | `specs/jacobian/` | 50 |
| Richer e-graph rewrite system | extend `lawsynth-egraph` | `specs/egraph-rules/` | 45 |
| Structured/template priors | `lawsynth-template` | `specs/template-priors/` | 45 |
| Performance: SIMD eval + rayon sweep + O(n log n) Pareto | extend `lawsynth-features`,`-score`,`-runner` | `specs/parallel-determinism/` | 55 |

### v2-B — Ecosystem & adoption · ~1,900 files

| Workstream | Where | ~files |
|---|---|---:|
| **sklearn adapter** (`LawSynthRegressor`/`Transformer`/`Dynamics`) + tests + docs | `python/lawsynth-sklearn/` | 60 |
| **SymPy / torch / jax** differentiable export (trainable constants) | `python/lawsynth/` + `crates/lawsynth-report` | 50 |
| **SRBench harness** (Docker + sklearn method + result reporting) | `benchmarks/srbench/` | 80 |
| **Feynman** equation family (public formulas, our own generators) | `benchmarks/feynman/` (~120 cases) | 380 |
| **Strogatz ODE** family (dynamics — we win) | `benchmarks/strogatz/` (~15 cases) | 90 |
| **SRSD / black-box** families | `benchmarks/srsd/`, `benchmarks/blackbox/` | 260 |
| Warehouse connectors (Snowflake/BigQuery/Databricks) | `python/lawsynth-connectors/` + `plugins/` | 180 |
| MLflow / W&B interop (export + plugin) | `plugins/mlflow-export/`, `plugins/wandb-export/` | 90 |
| **Benchmark whitepaper + reproducibility artifacts** | `docs/benchmarks/`, `papers/` | 60 |
| Plugin marketplace content (10 → 40 example plugins) | `plugins/` | 420 |
| Connector expansion (kafka/iceberg/delta/arrow depth + 10 new sources) | `python/lawsynth-connectors/` | 230 |

### v2-C — Platform, hosting & distributed · ~1,700 files

| Workstream | Where | ~files |
|---|---|---:|
| **Distributed-but-reproducible sweep** (island model, seeded, shared Pareto) | `crates/lawsynth-sweep` + `services/scheduler` | 90 |
| Real OIDC/SSO providers behind the P10 seam (Okta/Auth0/Azure AD) | `services/gateway/` | 70 |
| Multi-tenant quota/metering/billing hooks + usage dashboards | `services/api/`, `apps/console/` (new admin app) | 220 |
| Hosted self-serve trial (browser Studio + sample datasets + onboarding) | `apps/trial/` (new) | 180 |
| **Admin console** app (tenants, members, audit, plugins, usage) | `apps/console/` (new) | 260 |
| Distributed worker fleet ops (autoscale, placement, drain) | `services/scheduler`,`worker`, `deploy/` | 160 |
| Collaboration depth (P6): real-time presence, comments UI, review UX | `apps/studio/`, `packages/collab/` (new) | 190 |
| Streaming platform (P7): long-lived stream runs, dashboards | `services/`, `apps/studio/` | 150 |
| Governance depth (P9): approval workflows UI, audit explorer, compliance exports | `apps/console/`, `services/api/` | 180 |
| Deployment depth (helm/terraform/k8s for the fleet + observability) | `deploy/` | 200 |

### v2-D — Domain packs & verticals · ~1,100 files

Vertical wedges where an auditable deterministic equation has compliance value.
Each pack: curated templates + example datasets + recipes + tutorials + a Studio
"lab" preset + a benchmark family.

| Pack | Where | ~files |
|---|---|---:|
| **Pharma / systems-biology** (PK/PD, enzyme kinetics, gene regulation) | `domains/biology/` | 240 |
| **Energy / industrial** (power systems, thermal, control loops) | `domains/energy/` | 220 |
| **Quant finance** (mean-reversion, volatility, regime-switching) | `domains/finance/` | 220 |
| **Climate / earth** (compartment models, oscillators) | `domains/climate/` | 200 |
| **Epidemiology** (SIR/SEIR families, interventions) | `domains/epidemiology/` | 180 |

### v2-E — Docs, education & credibility · ~700 files

Cookbook expansion (10 → 40 recipes), tutorials (3 → 20), a full API reference
site build, a benchmark leaderboard page, video/notebook galleries, migration
guides ("coming from PySINDy/PySR/gplearn"), and academic artifacts.

## Rollup

| Milestone | Focus | ~files |
|---|---|---:|
| v2-A | Engine breadth | 1,600 |
| v2-B | Ecosystem & adoption | 1,900 |
| v2-C | Platform & distributed | 1,700 |
| v2-D | Domain packs | 1,100 |
| v2-E | Docs & credibility | 700 |
| **Subtotal (new)** | | **~7,000** |
| Current repo | | ~3,460 |
| **Target** | | **~10,460** |

## Sequencing (what to build first, and why)

1. **Units-in-discovery + weak-form** (v2-A) — highest algorithmic ROI, both
   deterministic, both directly neutralize a competitor's headline advantage.
2. **sklearn adapter + SRBench harness + Strogatz/Feynman benchmarks** (v2-B) —
   the credibility + adoption unlock; without a published benchmark, engine work
   is invisible to the market.
3. **Koopman/DMD + implicit** (v2-A) — remove the two most common "it can't even
   do X" objections from the SciML/PySINDy crowd.
4. **Distributed-but-reproducible sweep** (v2-C) — scales discovery while keeping
   determinism, our flagship differentiator.
5. **One vertical pack** (v2-D) — convert the platform into paying value.

Each item ships behind a boundary spec with a conformance suite, deterministic
and offline, on at least the CLI + SDK, with docs + a cookbook recipe — the same
contract that carried P0–P10.

> Build order note: the immediate next commits implement the **sequencing #1**
> items (units-in-discovery dimensional pruning, then weak-form) as real,
> tested crates — starting now.
