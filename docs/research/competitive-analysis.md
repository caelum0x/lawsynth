# LawSynth — Competitive Analysis

Prepared from a code-level study (shallow clones + docs + papers) of the leading
open-source equation-discovery / symbolic-regression tools and the adjacent
forecasting, MLOps, and physics-informed-ML landscape. Competitor code was read
in scratch space only; nothing was copied into this repository. Recommendations
below are *techniques and directions* (public algorithms), not code ports.

## 1. Where LawSynth sits

LawSynth's claim — **local, deterministic discovery of governing *dynamics* from
time-series, delivered as a product** (CLI/SDK/Studio/services; explain, forecast,
intervene, compare, report, monitor, library, governance) — sits at the
intersection of four communities that are largely disconnected. No incumbent
occupies that intersection; that is the opening.

```
        Symbolic regression / dynamics          Time-series forecasting
        discovery (the ENGINE)                  products (the SURFACE)
        PySINDy · PySR · Operon · SciML          Nixtla · Darts · sktime · Prophet
        gplearn/DEAP · AI-Feynman · QLattice      (backtest, anomaly, intervals)
                          \                      /
                           \   L A W S Y N T H  /
                           /   (engine + product) \
                          /                        \
        Interpretable-ML / MLOps                 Physics-informed / neural
        MLflow · W&B (runs, registry,            DeepXDE · NeuroMANCER (PINNs:
        model-cards, governance)                 accurate but opaque)
```

## 2. The competitors (code-level)

| Tool | Category | Core method | Language | The one thing it does best |
|---|---|---|---|---|
| **PySINDy** | Dynamics discovery — *most direct* | Sparse regression over a feature library (SINDy) | Python (sklearn) | Breadth: 12+ optimizers, **weak/integral-form**, PDE, implicit/rational, Bayesian UQ, control |
| **PySR / SymbolicRegression.jl** | Symbolic regression | Multi-population GP + island migration | Python↔Julia | **Distributed multi-node search** + in-loop dimensional analysis + torch/jax export |
| **Operon** | Symbolic regression | Genetic programming (C++) | C++23 | **Raw throughput** (SIMD-batched eval, taskflow) + NSGA-II + MDL/BIC/AIC objectives |
| **SciML (DataDrivenDiffEq + MTK)** | Dynamics discovery | SINDy + **Koopman/DMD** + implicit + GP + neural-DAG, on a symbolic-numeric compiler | Julia | **Koopman/DMD**, implicit/rational nullspace, deep symbolic IR, analytic-Jacobian codegen |
| **gplearn / DEAP** | Symbolic regression | Tree GP | Python | **sklearn-native ergonomics** (gplearn); general EA toolbox (DEAP) |
| **AI-Feynman** | Physics SR | Dimensional analysis → symmetry → separability → brute force + NN probe | Python+Fortran | **Structural reductions** (units, symmetry, separability) that dissolve combinatorial search |
| **QLattice / TuringBot / Eureqa** | Commercial SR | Various | closed | Product polish; QLattice won SRBench-2022 synthetic; Eureqa is the cautionary tale |

## 3. Feature matrix (LawSynth today vs the field)

| Capability | Field leader | LawSynth status |
|---|---|---|
| Sparse regression (STLSQ/SR3/LASSO/group) | PySINDy, SciML | ✅ at/above parity (+ stability selection) |
| **Weak / integral-form** discovery | PySINDy (`_weak.py`) | ❌ **gap — top priority** (noise robustness) |
| **Koopman / DMD / EDMD** | SciML (`DataDrivenDMD`) | ❌ **gap** (linear-operator discovery) |
| **Implicit / rational (nullspace) dynamics** | SciML, PySINDy-PI | ⚠️ rational *features* only; no implicit solver |
| **Dimensional pruning in-loop** | PySR, AI-Feynman | ⚠️ `lawsynth-units` exists but **not wired into discovery** |
| Symmetry / separability reductions | AI-Feynman | ❌ gap (deterministic version feasible via our `differentiate`) |
| Optimizer breadth (FROLS/SSR/trapping/L0) | PySINDy, Operon | ⚠️ partial |
| Deterministic ensembling | (PySINDy ensembles, but stochastic) | ⚠️ have stability-selection; extend to seeded bagging |
| Control inputs (SINDYc) | PySINDy | ⚠️ interventions, not discovery-time control |
| Pareto / model selection | Operon (NSGA-II, MDL) | ✅ Pareto + **AIC/AICc/BIC** (ahead); ⚠️ O(n²) sort, no MDL objective |
| Analytic-Jacobian codegen | SciML (MTK) | ❌ export RHS+RK4 only |
| Differentiable (torch/jax) export | PySR | ❌ gap (Neural-ODE interop) |
| SymPy export | PySR, gplearn | ❌ gap (ecosystem interop) |
| **Determinism as a contract** | — (all competitors stochastic or opt-in) | ✅ **unique** (bit-exact, offline) |
| **Portable governed artifact** (`.lsworld`) | — | ✅ **unique** |
| **Regimes / causal** | — | ✅ **unique** |
| **Dynamics + simulate/forecast/intervene** | (SciML simulates; none discover *and* productize) | ✅ **unique combination** |
| Product surfaces (CLI/SDK/Studio/services) | — (all are libraries) | ✅ **unique** |
| Backtest / monitor / registry / governance | Nixtla/MLflow charge for these | ✅ have (P6–P10) |
| **Public benchmark result (SRBench)** | PySR, Operon, QLattice | ❌ **gap — #1 credibility risk** |
| sklearn-compatible estimator | PySINDy, gplearn, PySR | ❌ **gap — #1 adoption lever** |
| Distributed search | PySR (Slurm) | ❌ single-threaded (by design) |
| Warehouse / MLOps integrations | Nixtla (Snowflake), MLflow | ❌ gap |

## 4. Strategic reading

**LawSynth's moat is real but under-marketed.** No pure SR engine (PySINDy,
PySR, Operon, gplearn) ships the forecasting-product + MLOps-governance surface;
no forecasting/MLOps product (Nixtla, MLflow, W&B) has an interpretable-equation
engine; no tool combines *dynamics discovery* with *bit-exact determinism* and a
*portable governed artifact*. That intersection is defensible.

**But two gaps are existential for credibility and adoption:**
1. **No published SRBench result.** Reviewers and technical buyers will measure
   LawSynth against SRBench (Feynman exact-recovery + Strogatz ODE + black-box
   R²/size/time) whether or not it participates. Determinism is a metric no
   stochastic GP competitor can match — a selling point *if published*.
2. **No sklearn-compatible estimator.** gplearn's and PySINDy's adoption is driven
   by dropping into `Pipeline`/`GridSearchCV`. LawSynth's `Study` façade is fluent
   but not sklearn-shaped, so it can't be adopted incrementally by existing ML
   teams.

**The Eureqa lesson:** a celebrated engine (80k downloads) still died as a
standalone product — distribution + product + a regulated vertical (QLattice →
pharma) is the durable moat. LawSynth's engine-plus-platform bet is the right
correction *if* the surfaces (Studio, hosted trial, integrations) are sticky and
it wins a vertical where an **auditable, deterministic equation** has compliance
value (pharma, energy/industrial, quant finance).

## 5. Recommendations

### Adopt (close credible "but it can't do X" objections) — all deterministic/offline-safe

1. **Weak/integral-form discovery** (from PySINDy) — the single biggest
   noise-robustness win; integrate candidate terms against compactly-supported
   test functions. Erases PySINDy's key edge.
2. **Wire `lawsynth-units` into discovery** (from PySR/AI-Feynman/SciML) — near-free:
   reject dimensionally-inconsistent candidates *during* search, and add
   Buckingham-π dimensionless re-parameterization to shrink the variable space.
3. **Koopman/DMD strategy** (from SciML) — compact (~250 lines) linear-operator
   discovery (DMD/DMDc/EDMD); reproducible (SVD is deterministic).
4. **Implicit/rational nullspace solver** (from SciML/PySINDy-PI) — discover
   `f(x,ẋ)=0` forms (Michaelis-Menten) explicit regression can't express.
5. **Symmetry/separability reductions** (from AI-Feynman) — deterministic
   divide-and-conquer using our own `differentiate` crate instead of a NN probe.
6. **More optimizers** — greedy FROLS/SSR (deterministic), trapping/constrained-SR3
   (stability-guaranteed worlds), an L0/exact-cardinality path.
7. **Deterministic (seeded) ensembling** — PySINDy's ensembling benefit *with*
   reproducibility: a story PySINDy can't tell.
8. **Performance** (from Operon) — SIMD-batched columnar candidate evaluation,
   rayon parallel scoring with ordered/deterministic merge, O(n log n) Pareto
   sort, and an **MDL/description-length** objective.
9. **Codegen depth** — emit analytic Jacobians (from MTK) and a **differentiable
   torch/jax export with trainable constants + initial conditions** (from PySR),
   turning a `.lsworld` into a Neural-ODE layer. Add a **SymPy** export handle.
10. **Structured priors** (from PySR `TemplateExpressionSpec`) — let users fix
    part of a model and search the rest.

### Court adopters

11. **`LawSynthRegressor` / `LawSynthTransformer` / `LawSynthDynamics`** — a
    strict sklearn-compatible estimator (`fit`/`predict`/`score`/`get_params`/
    `__sklearn_tags__`) so LawSynth drops into `Pipeline`/`GridSearchCV`. Highest
    adoption leverage. Add auto-parsimony (`parsimony='auto'` = Cov/Var).
12. **Publish SRBench** — package a Docker + sklearn method to SRBench's
    contribution API; report Feynman + **Strogatz (dynamics — where we win
    uncontested)** + black-box, with the standard accuracy–simplicity–time Pareto
    and a **determinism headline**. Seed a benchmark whitepaper.
13. **Integrations** — warehouse (Snowflake/BigQuery/Databricks "discover in SQL")
    and MLflow/W&B export so LawSynth slots into existing stacks.
14. **Distributed-but-reproducible sweep** (P10) — an island-model-style seeded
    parallel sweep merging onto a shared Pareto/hall-of-fame; "distributed *and*
    reproducible," which PySR cannot claim.

### Lead with (differentiation to market)

- **Determinism + offline + signed `.lsworld`** as a compliance/audit wedge.
- **Dynamics you can run** — "PySR finds a formula; LawSynth finds a world you can
  simulate, forecast, and intervene on."
- **Interpretability vs neural** — against PINNs/TimeGPT: an auditable equation,
  on-prem, no data egress.
- **A regulated vertical** — pick one (pharma/energy/quant) where governance +
  model cards + reproducibility are worth paying for.

## 6. What NOT to chase

- Full PDE/spatiotemporal-ND discovery + `AxesArray` machinery (large, orthogonal
  to the ODE/executable-world sweet spot) unless a target user demands it.
- Hard dependencies that break offline/determinism (Gurobi/CVXPY/MCMC, an embedded
  Julia VM, `eval()`-based expression compilation à la DEAP).
- Out-GP-ing Operon on raw static-formula throughput inside its own game.

The concrete build-out of the "adopt" list is planned in
[`docs/roadmap/expansion-v2.md`](../roadmap/expansion-v2.md).
