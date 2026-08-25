# Roadmap

## Shipped (P0–P5 + product)

The local engine and the full product loop are implemented and verified:
World IR, bundle integrity, numerical simulation, reproducible discovery
(sparse + symbolic + lagged + Pareto + regimes + uncertainty + parameter
refinement + causal hypotheses), and the language bindings — plus the product
surfaces built on top: the CLI, the Python SDK (`Study`, recipes, ensemble,
backtest, monitor, `Project`, `Client`), the Studio app, the
discovery-as-a-service backend, the notebook dashboard and interactive widget,
six export targets, and self-hosting/deployment scaffolding. See
[`PRODUCT.md`](PRODUCT.md) for the product view and
[`ARCHITECTURE.md`](ARCHITECTURE.md) for the engine boundary.

Formerly-proposed areas (causal analysis, regimes, uncertainty, plugins, Studio,
services, deployment) are now implemented with source, tests, and user-facing
contracts — not proposals.

## Next

The forward plan turns a complete single-user tool into a collaborative,
extensible, governed open-source platform that users run locally or self-host.
Each phase is bounded by a **boundary specification** with a conformance suite:

| Phase | Goal | Spec |
|---|---|---|
| P6 Collaboration | teams share, review, and version models | [`specs/collaboration/`](specs/collaboration/) |
| P7 Streaming discovery | models that update on live data | [`specs/streaming-discovery/`](specs/streaming-discovery/) |
| P8 Plugin marketplace | safe community extensions | [`specs/plugin-marketplace/`](specs/plugin-marketplace/) |
| P9 Governance | auditable, accountable models | [`specs/model-governance/`](specs/model-governance/) |
| P10 specification archive | former managed-hosting boundary, retained as historical design input rather than an active service plan | [`specs/hosted-platform/`](specs/hosted-platform/) |

The full plan — goals, user problems, and sequencing — is in
[`docs/roadmap/next-phases.md`](docs/roadmap/next-phases.md).

## Proposed quantitative-research platform

[`docs/roadmap/quant-research-platform.md`](docs/roadmap/quant-research-platform.md)
defines the shared architecture and staged roadmap for LawSynth's open-source
quant family: data and experiment contracts, stochastic models, calibration,
derivatives pricing, portfolio/market/liquidity/credit risk, backtesting,
microstructure research, and independent model validation.

[`docs/roadmap/quant-diffusion-stress-engine.md`](docs/roadmap/quant-diffusion-stress-engine.md)
is the first proposed vertical inside that family. It defines a production
stress-testing product built from classical SDE and jump-diffusion baselines, an
optional PyTorch diffusion generator, portfolio valuation, risk backtests, and
governed reports. Planning does not alter the current roadmap order.
Implementation begins only after a design partner, evaluation data,
specifications, and statistical acceptance thresholds exist.

## How priorities are set

Priorities are decided through maintainership review and reproducible technical
evidence rather than calendar promises. A phase ships only what its conformance
suite can verify; capability gaps are documented, never faked. The local,
offline, deterministic core never regresses as collaboration and self-hosting
features are added. `lawsynth.dev` remains a static project website, not a
managed LawSynth application.
