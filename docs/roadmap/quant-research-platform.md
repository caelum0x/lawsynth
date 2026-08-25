# Quantitative Research Platform Roadmap

Status: proposed open-source product family. Nothing in this document is a
claim that the complete quant platform ships today.

## Direction

LawSynth should become a reproducible quantitative-research workbench, not a
collection of unrelated finance demos. The shared product promise is:

> Import licensed market data, state assumptions explicitly, calibrate or
> discover a model, run deterministic experiments, validate the result against
> independent baselines, and export an auditable research bundle.

The first vertical remains the [Quant Diffusion Stress
Engine](quant-diffusion-stress-engine.md). This roadmap places that project in a
larger architecture and adds the adjacent quant capabilities needed to make its
models, prices, risk measures, and experiments credible.

LawSynth remains Apache-2.0, local-first, and self-hostable. `lawsynth.dev` is a
static documentation and project website. It does not run private portfolios,
training jobs, backtests, or trading strategies.

## Product boundary

The quant family is for researchers, students, small asset managers, treasury
teams, risk consultants, and engineering teams that need inspectable research
software. It is not a broker, exchange, portfolio custodian, signal-selling
service, execution venue, or source of investment advice.

Initial releases use end-of-day data and generated fixtures. Intraday feeds,
order books, complex exotics, counterparty exposure, and live execution enter
only behind separate data-license, numerical-validation, operational-risk, and
security reviews.

## Shared architecture

```text
licensed/public data + generated fixtures + portfolio definitions
                              |
                  quant preparation contract
        calendars / actions / units / currencies / hashes
                              |
        +---------------------+---------------------+
        |                     |                     |
 stochastic models      market models        empirical methods
 SDE / jumps / regimes   curves / vol / credit  bootstrap / factors
        +---------------------+---------------------+
                              |
              calibration + validation registry
                              |
       +----------------------+----------------------+
       |                      |                      |
    pricing               portfolio risk       research/backtest
 value / Greeks          stress / VaR / ES      costs / leakage
       +----------------------+----------------------+
                              |
           governed experiment and report bundle
```

Rust owns deterministic numerical kernels, schemas, validation, simulation,
pricing, risk aggregation, and reproducible artifact construction. Python owns
notebook ergonomics, statistical diagnostics, calibration workflows that benefit
from the scientific Python ecosystem, and optional PyTorch training. Neither
surface may implement a second definition of prices, returns, calendars, units,
or risk metrics.

The CLI, Python SDK, local Studio, and self-hosted API consume the same versioned
contracts. Large matrices and paths use Arrow-compatible columnar data or
Parquet; governed metadata stays in a small inspectable manifest. Model weights
are content-addressed external artifacts rather than opaque data embedded in a
`.lsworld` file.

## Q0 — Quant foundation

Every later project depends on one shared foundation:

- Trading calendars, time zones, observation cutoffs, and strictly ordered
  event time.
- Raw, adjusted, and total-return price semantics with explicit corporate-action
  records.
- Currencies, FX conversion policy, day-count conventions, compounding, units,
  and decimal precision.
- Long and wide market-data schemas with stable instrument identifiers and
  vendor/source/license metadata.
- Portfolio, position, trade, cash-flow, benchmark, and scenario schemas.
- Seeded random streams with documented algorithms and independent substreams.
- Experiment manifests binding data, code, configuration, environment, model,
  seed, and output hashes.
- A fixture registry containing generated paths and small redistributable public
  datasets with known expected results.

The foundation rejects ambiguous timestamps, mixed adjusted/unadjusted prices,
unknown currencies, duplicate observations, look-ahead joins, and unlicensed
redistribution. It never guesses a calendar or corporate action.

## Q1 — Stochastic process and calibration lab

Build a reusable process library rather than one-off simulation notebooks:

- Brownian motion, correlated Brownian drivers, GBM, OU, and CIR.
- Heston stochastic volatility, Merton jump diffusion, and regime-switching
  processes.
- Later candidates: local volatility, SABR, rough-volatility research, Hawkes
  events, and Lévy processes. These do not ship before their own references and
  convergence tests exist.
- Euler-Maruyama, Milstein where valid, full-truncation Heston, exact schemes
  where available, and explicit rejection/repair policies.
- Maximum likelihood, generalized method of moments, characteristic-function,
  and simulation-based calibration behind one diagnostic contract.

Every calibration returns parameter bounds, optimizer status, objective value,
identifiability warnings, uncertainty estimates, residual checks, and a
train/validation split. Every simulator records its algorithm, seed, timestep,
correlation repair, rejected paths, and weak/strong convergence evidence.

## Q2 — Derivatives pricing and volatility

Use pricing as a numerical truth test and a useful product in its own right:

- Discount and forward curves with explicit day-count, calendar, interpolation,
  and bootstrapping conventions.
- Black-Scholes and Bachelier analytic references.
- Binomial/trinomial trees, Monte Carlo and quasi-Monte Carlo, finite-difference
  PDEs, and variance-reduction techniques.
- European, American, Asian, barrier, and lookback options added in that order.
- Implied volatility, arbitrage-aware smiles and surfaces, SVI/SABR calibration,
  and static-arbitrage diagnostics.
- Delta, gamma, vega, theta, and rho with method and error information; pathwise,
  likelihood-ratio, adjoint, or automatic differentiation only after independent
  finite-difference checks.

A pricer is released only when it converges to analytic or independently trusted
references across a declared parameter grid. Reports include confidence
intervals, discretization settings, numerical error estimates, and unsupported
regions.

## Q3 — Portfolio, market, liquidity, and credit risk

The shared portfolio engine revalues positions under historical and simulated
scenarios. Capability grows in controlled layers:

1. Cash, spot instruments, linear factor exposures, and vanilla options.
2. Historical replay, parametric risk, Monte Carlo VaR, Expected Shortfall,
   drawdown, marginal/component risk, and factor attribution.
3. Liquidity horizons, bid/ask and impact add-ons, concentration, wrong-way
   scenario research, and model disagreement.
4. Fixed-income curve risk, duration/convexity, key-rate duration, bond and swap
   cash flows.
5. Credit transition and default models, hazard curves, recovery assumptions,
   migration matrices, and later counterparty exposure/CVA research.

VaR implementations require exception tests; Expected Shortfall requires its own
backtesting and uncertainty treatment. Credit and liquidity outputs must state
data scarcity, calibration assumptions, and the difference between research
metrics and regulatory capital.

## Q4 — Generative markets and synthetic stress

This track is specified in the [G-SMSE roadmap](quant-diffusion-stress-engine.md):
classical stochastic baselines plus optional conditional score/diffusion models
generate multi-asset crisis paths for portfolio stress testing.

Additional research projects may include:

- Tail-conditioned generation and controlled correlation breakdown.
- Synthetic limit-order-book or event streams after the daily-data model passes.
- Rare-event sampling and importance-weighted stress generation.
- Privacy and memorization measurement for proprietary training data.
- Distillation or accelerated samplers with measured fidelity/cost tradeoffs.

Generated data is always labeled synthetic. It is never presented as a forecast
or used to hide weak performance on real held-out data.

## Q5 — Research, portfolio construction, and backtesting

Provide a small, correct experimental harness rather than a strategy marketplace:

- Return, factor, signal, target, universe, rebalance, and order-intent contracts.
- Walk-forward and purged/embargoed cross-validation for overlapping labels.
- Buy-and-hold, equal-weight, volatility-target, minimum-variance, risk-parity,
  and constrained mean-variance baselines.
- Turnover, spread, fees, borrow cost, market impact, capacity, and partial-fill
  models with every assumption visible.
- Corporate-action-safe holdings accounting, cash, dividends, splits, delistings,
  FX, and benchmark comparison.
- Deflated performance statistics, multiple-testing controls, stability across
  subperiods, and parameter-sensitivity reports.

The harness blocks future data, same-bar fill assumptions without an explicit
model, survivorship-biased universes, and silent reuse of a test period. It does
not connect to a brokerage account.

## Q6 — Market microstructure and execution research

After the end-of-day foundation is stable, add a separately bounded research
pack for timestamped trades and quotes:

- Normalized trade/quote and limit-order-book event schemas.
- Deterministic replay, clock-skew checks, sequence-gap detection, and session
  boundaries.
- Spread, depth, imbalance, order-flow, realized-volatility, and toxicity
  measures.
- Almgren-Chriss, TWAP, VWAP, POV, and implementation-shortfall baselines.
- Queue and fill models evaluated against replayed data, with latency and impact
  sensitivity.

This phase produces research schedules and simulated fills only. Live routing,
exchange credentials, and unattended execution are explicit non-goals.

## Q7 — Model-risk and benchmark laboratory

All quant projects share one independent validation layer:

- Property and metamorphic tests for invariants such as no-arbitrage bounds,
  monotonicity, conservation, and unit/currency consistency.
- Analytic fixtures, published reference values, cross-language parity, and
  differential tests against independently implemented libraries.
- Statistical tests with power analyses and tolerances chosen before evaluation.
- Determinism tests across threads and supported platforms, plus declared
  tolerances for accelerated math.
- Accuracy/performance frontiers rather than performance-only leaderboards.
- Data leakage, overfitting, multiple testing, memorization, and unstable-seed
  adversarial cases.
- Model cards, approval states, limitations, reproducibility commands, and
  signed release evidence.

Validation code must not import the implementation under test when an independent
reference is required. A benchmark result includes hardware, software, data
hashes, warm-up policy, repetitions, and uncertainty.

## Product surfaces

The eventual command family is intentionally regular:

```text
lawsynth quant data ...
lawsynth quant calibrate ...
lawsynth quant simulate ...
lawsynth quant price ...
lawsynth quant risk ...
lawsynth quant generate ...
lawsynth quant backtest ...
lawsynth quant validate ...
lawsynth quant report ...
```

These commands are reserved names, not shipped claims. A command appears in the
CLI only when its schema, fixtures, conformance tests, Python parity, error model,
and documentation pass.

Local Studio may visualize data quality, calibration, paths, volatility
surfaces, P&L distributions, risk attribution, and validation evidence. The
self-hosted service may schedule bounded jobs over preconfigured datasets and
model registries. Neither surface accepts arbitrary Python, pickle files,
untrusted native plugins, or executable model artifacts.

## Dependency order

| Release | Scope | Required evidence |
| --- | --- | --- |
| QR0 | Quant data, money/time conventions, portfolios, seeds, experiment bundles | schema fixtures, round trips, leakage tests, deterministic hashes |
| QR1 | Processes, calibration, Monte Carlo, analytic vanilla pricing | parameter recovery, convergence grids, independent price references |
| QR2 | Portfolio valuation and market-risk measures | P&L reconciliation, VaR/ES backtests, attribution identities |
| QR3 | G-SMSE classical stress engine | historical replay, Heston/jump fixtures, governed stress reports |
| QR4 | Conditional generative diffusion | held-out stylized facts, privacy tests, downstream risk stability |
| QR5 | Volatility/fixed-income/credit packs | per-pack curve, calibration, no-arbitrage, and reference suites |
| QR6 | Backtesting and microstructure research | leakage adversaries, accounting reconciliation, replay/fill validation |

QR0–QR2 form the minimum coherent platform. Deep generative work must not jump
ahead of classical baselines, correct portfolio accounting, or model-risk
infrastructure.

## Cost and operations

The default path runs on the user's hardware. CPU kernels are the reference;
optional GPU acceleration is feature-gated and benchmarked against the reference.
Jobs enforce operator-configured limits for rows, assets, paths, horizon,
memory, runtime, output size, and concurrency.

CI uses compact generated fixtures. Expensive nightly tests publish only
redistributable, content-addressed benchmark summaries. The static website stays
within the existing Cloudflare free-tier deployment; quant compute and private
artifact storage are not moved to Cloudflare.

## Pre-code gates

Before QR0 implementation, maintainers must approve:

- One first user workflow and an end-of-day asset universe.
- Data licenses and redistributable validation fixtures.
- Canonical schemas and numerical conventions.
- Independent references and statistical tolerances.
- Supported hardware and reproducibility envelope.
- Threat model for datasets, archives, model artifacts, reports, and plugins.

Each later release receives its own boundary specification and conformance suite.
Research notebooks may explore ideas, but a notebook is never production evidence.

## Portfolio-level release gate

The quant family is production-ready only when a clean machine can reproduce the
published reference experiments from pinned data and artifact hashes; Rust,
Python, CLI, Studio, and self-hosted API agree on supported outputs; malformed
and adversarial inputs fail safely; numerical and statistical acceptance suites
pass; and every report states sources, assumptions, uncertainty, limitations,
and the exact command needed to reproduce it.
