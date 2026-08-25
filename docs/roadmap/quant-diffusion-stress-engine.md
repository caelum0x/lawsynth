# Quant Diffusion Stress Engine

Status: product and architecture proposal. The current repository does not ship
this end-to-end product.

## Product boundary

The working product name is **G-SMSE**, the Generative Stress Testing and Market
Simulation Engine. A risk analyst supplies market histories, portfolio positions,
and a defined macro regime. LawSynth produces reproducible classical simulations,
optional learned synthetic paths, portfolio loss distributions, and a report that
records each model, seed, assumption, source, and validation result.

G-SMSE extends LawSynth's quant-finance domain pack. It does not replace the
interpretable core. Classical models provide the baseline and a readable source
of risk. A learned diffusion model adds candidate scenarios after it passes a
separate model-risk review.

Synthetic crises do not predict a future crisis. They sample paths from a fitted
and conditioned model. Reports must state that limitation beside every generated
risk result.

## Existing LawSynth foundation

LawSynth already contains useful pieces:

- `lawsynth-sde` estimates drift and diagonal diffusion from sample paths with a
  Kramers-Moyal estimator and sparse regression.
- `lawsynth-sim` generates seeded Euler-Maruyama paths for diagonal-noise SDEs.
- The Python SDK exposes SDE discovery results.
- Regime, uncertainty, simulation, report, service, Studio, and governance
  surfaces provide integration seams.

Those components do not yet provide correlated Brownian drivers, Heston
simulation, jump processes, option valuation, a portfolio risk engine, learned
diffusion training, or a supported stochastic World bundle. The project will not
claim those capabilities until their boundary specs and conformance suites pass.

## Initial customer

The first user is a small asset manager, treasury team, risk consultant, or quant
researcher who needs auditable stress tests without buying an institutional risk
platform. The first release supports end-of-day data for three to five liquid
instruments and a linear cash/equity/bond portfolio. High-frequency order books,
complex derivatives, counterparty exposure, and regulatory capital reports wait
for separate validation.

## User workflow

1. Import licensed price histories, positions, and optional macro variables.
2. Validate calendars, currencies, corporate actions, missing values, and return
   transformations. Store the data hash and preparation record.
3. Calibrate classical baselines and run deterministic historical replay.
4. Fit an optional conditional diffusion model on a training split. Keep model
   artifacts, code revision, environment, seed, and evaluation results together.
5. Define a scenario such as a volatility band, rate shock, or correlation regime.
6. Generate seeded paths from the approved models.
7. Revalue the portfolio and calculate loss distributions, Value-at-Risk,
   Expected Shortfall, maximum drawdown, exposure contribution, and scenario P&L.
8. Compare historical replay, classical simulation, and learned simulation in one
   report. The report names failed checks and blocks unsupported conclusions.

## System structure

```text
Licensed market data + macro series + positions
                    |
            preparation contract
                    |
       +------------+-------------+
       |                          |
classical SDE/jump models    conditional diffusion model
       |                          |
seeded scenario paths        approved synthetic paths
       +------------+-------------+
                    |
          portfolio valuation
                    |
       VaR / ES / drawdown / attribution
                    |
     governed report + portable artifacts
```

## Classical simulation track

The first model set contains:

- Geometric Brownian Motion as a weak baseline.
- Ornstein-Uhlenbeck for mean-reverting factors.
- Heston stochastic volatility with correlated Brownian drivers.
- Merton jump-diffusion with an explicit Poisson jump process.
- Historical and stationary block bootstrap baselines.

Each implementation records its random generator, seed, timestep, discretization,
parameter calibration method, convergence diagnostics, and rejected paths. The
engine starts with Euler-Maruyama and adds Milstein or a model-specific scheme
only after weak and strong error tests justify it.

The pricing surface starts with European options to verify calibration and
Monte Carlo convergence, then adds barrier and lookback options. A pricing result
includes confidence intervals and a benchmark against a closed form or trusted
reference when one exists.

## Learned diffusion track

Python and PyTorch own the training boundary. The initial candidate uses a 1D
temporal U-Net or residual temporal convolution backbone with a cosine noise
schedule. Conditioning can include volatility regime, rate band, inflation band,
and asset-class label. The team must compare DDPM-style sampling with a
score-SDE implementation before choosing the supported model.

Training produces a versioned artifact that contains:

- Dataset and split hashes, feature schema, normalization parameters, and license
  metadata.
- Architecture, noise schedule, objective, optimizer, checkpoints, seed, and
  software environment.
- Training curves, held-out evaluations, privacy tests, and model card.
- Supported horizon, asset universe, conditioning range, and rejection criteria.

LawSynth treats the model as an optional generator plugin. It never serializes
learned weights into the deterministic World IR. A scenario bundle references a
content-addressed model artifact and records the generated path seed.

## Validation contract

The learned model must outperform or add measurable coverage beyond historical
resampling and calibrated classical baselines. The evaluation suite checks:

- Return distribution, quantiles, skew, kurtosis, tail index, and exceedances.
- Autocorrelation of returns, absolute returns, and squared returns.
- Volatility clustering, leverage effect, cross-asset correlation, and correlation
  breakdown under stressed conditions.
- Drawdown depth and duration, regime occupancy, transition behavior, and path
  continuity.
- Train-versus-synthetic nearest-neighbor distance, membership-inference risk,
  and memorization of rare windows.
- Downstream risk stability through train-on-synthetic/test-on-real and
  train-on-real/test-on-synthetic exercises.
- VaR exception coverage, Expected Shortfall backtests, calibration error, Monte
  Carlo standard error, and sensitivity to seeds and hyperparameters.

The engine rejects a model when it copies training windows, suppresses tails,
creates unstable correlations, or changes portfolio ranking across seeds beyond
the declared tolerance. A risk analyst approves a model version before a team can
use it in a governed report.

## Portfolio and risk engine

The valuation layer maps each generated return/factor path to position values.
Release one handles cash instruments and linear exposures. The schema reserves a
versioned pricer interface for options without claiming full derivative coverage.

Required measures include:

- Scenario and horizon P&L distribution.
- Parametric, historical, classical Monte Carlo, and learned-scenario VaR.
- Expected Shortfall with estimator and confidence information.
- Maximum drawdown and recovery time.
- Marginal and component risk contribution.
- Cross-model disagreement and sensitivity to conditioning.

Reports display risk numbers as model outputs with assumptions. They do not label
them forecasts, guarantees, or regulatory capital figures.

## Product surfaces

### CLI and Python

The CLI receives `quant prepare`, `quant calibrate`, `quant generate`, `quant
stress`, `quant value`, and `quant report` only after each command's spec passes.
The Python SDK exposes typed equivalents for notebooks and controlled pipelines.

### Self-hosted service API

The existing FastAPI service can add asynchronous quant jobs for teams that run
LawSynth on their own infrastructure. A request supplies a dataset reference,
portfolio reference, approved model version, scenario, horizon, path count, and
seed. The service returns a job identifier. Result endpoints provide summaries
and artifact downloads.

Administrators configure row, asset, horizon, path, runtime, storage, and
concurrent-job limits for their hardware. LawSynth imposes no commercial cloud
quota on local or self-hosted users. The service rejects arbitrary Python,
pickles, model code, and user-supplied executable artifacts.

### Local Studio

Studio can add a local risk lab with data quality, model calibration, path comparison,
tail-loss distribution, cross-asset correlation, drawdown, contribution, and
model-card views. Charts always expose the selected model and seed. The default
view compares learned output with historical and classical baselines.

## Data, security, and provenance

- Users retain responsibility for market-data licenses. LawSynth ships generated
  fixtures and links to approved public sources rather than redistributing
  restricted exchange data.
- Local mode keeps datasets, weights, paths, and reports on the user's machine.
- A self-hosted deployment must encrypt private artifacts, isolate tenants, use
  short-lived downloads, validate archive and content types, and record operator
  access.
- Generated paths inherit the source dataset's handling classification.
- Logs contain identifiers, timings, and aggregate sizes. They exclude positions,
  price rows, model weights, and path values.
- Reports bind data, configuration, code, model, scenario, seed, and result hashes
  into the existing governance and lineage records.

## Compute and website boundary

LawSynth remains an unrestricted open-source local and self-hosted project. Users
can run as many paths, training steps, workers, and datasets as their own hardware
and configuration allow. The project does not route simulation or PyTorch
training through Cloudflare.

Cloudflare serves `lawsynth.dev` as a static documentation and project website.
The website has no account, hosted Studio, simulation API, queue, model inference,
or private artifact storage. Static deployment keeps LawSynth outside the shared
dynamic-app budget. GitHub Releases or another open-source release channel carries
versioned binaries and public benchmark artifacts; a maintainer may mirror files
to R2 only after approving the storage cost.

## Revenue path

The open-source core remains useful offline. Revenue can come from support,
training, governed deployment work, and validated domain packs. A customer can
fund its own compute without changing the open-source engine's limits.

## Pre-code gate

The team must secure one design partner, define one asset universe and portfolio
schema, obtain licensed or public evaluation data, select the exact classical
baselines, write statistical acceptance thresholds, and define an independent
review process. The boundary specifications for correlated SDE simulation,
jump-diffusion, portfolio valuation, generative artifacts, and quant reports must
exist before implementation starts.

## Production release gate

A release candidate must reproduce all results from a clean machine using the
recorded artifacts and seeds. It must recover known parameters on generated GBM,
OU, Heston, and jump-diffusion fixtures; converge on trusted option values; pass
held-out stylized-fact tests; backtest VaR and Expected Shortfall; reject a
memorizing diffusion model; survive malformed data and artifact attacks; and
produce the same governed summary through CLI, Python, Studio, and service
surfaces.
