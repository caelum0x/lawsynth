# LawSynth — Exact 3,161-file repository manifest

**Status:** Canonical production repository plan  
**Scope:** Every planned directory and every planned tracked file  
**Rule:** These are meaningful target files, not empty scaffold placeholders  
**Companion:** `LawSynth_Production_Architecture.md`

## 1. Contract

This manifest turns the production architecture into an exact repository inventory. Every file path is unique. Every parent directory is derived and listed. File count is fixed at **3,161** for mature v1; implementation should add these files only when the corresponding milestone is reached.

Milestones:

- **P0:** repository, specifications, and build foundation
- **P1:** executable World IR, bundle, simulation, and language boundary
- **P2:** discovery engine, Python API, examples, and scientific benchmarks
- **P3:** regimes, uncertainty, WASM, Studio, and public documentation
- **P4:** storage, server API types, connectors, and developer tooling
- **P5:** distributed services, plugins, and production deployment

## 2. Exact totals

| Subsystem | Files |
|---|---:|
| Root and governance | 62 |
| Specifications and contracts | 142 |
| Rust workspace | 690 |
| Python workspace | 355 |
| Studio and TypeScript packages | 390 |
| Backend services | 235 |
| Documentation | 230 |
| End-to-end examples | 180 |
| Scientific benchmarks | 270 |
| Cross-language and system suites | 305 |
| Plugin ecosystem | 80 |
| Deployment and infrastructure | 130 |
| Repository tools and assets | 92 |
| **Total** | **3,161** |

**Derived directories:** 682

### Milestone distribution

| Milestone | Files |
|---|---:|
| P0 | 204 |
| P1 | 489 |
| P2 | 820 |
| P3 | 866 |
| P4 | 291 |
| P5 | 491 |

### Ownership distribution

| Owner | Files |
|---|---:|
| Architecture | 142 |
| Backend Platform | 235 |
| Design | 12 |
| Developer Experience | 80 |
| Documentation | 230 |
| Extension Ecosystem | 80 |
| Maintainers | 62 |
| Platform Engineering | 130 |
| Python SDK | 355 |
| Quality Engineering | 305 |
| Research Benchmarks | 270 |
| Rust Core | 690 |
| Scientific Examples | 180 |
| Web Studio | 390 |

### Primary file types

| Type | Files |
|---|---:|
| `md` | 715 |
| `rs` | 680 |
| `py` | 606 |
| `json` | 442 |
| `ts` | 311 |
| `toml` | 193 |
| `yaml` | 88 |
| `tf` | 24 |
| `license` | 16 |
| `dockerfile` | 13 |
| `yml` | 12 |
| `example` | 9 |
| `sh` | 8 |
| `typed` | 5 |
| `service` | 5 |
| `svg` | 4 |
| `tfvars` | 3 |
| `txt` | 3 |
| `png` | 3 |
| `lock` | 2 |
| `webp` | 2 |
| `notice` | 1 |
| `cff` | 1 |
| `justfile` | 1 |
| `makefile` | 1 |
| `editorconfig` | 1 |
| `gitattributes` | 1 |
| `gitignore` | 1 |
| `python-version` | 1 |
| `node-version` | 1 |
| `ini` | 1 |
| `metadata` | 1 |
| `version` | 1 |
| `hcl` | 1 |
| `dockerignore` | 1 |
| `target` | 1 |
| `sha256` | 1 |
| `gif` | 1 |

## 3. Every directory

Direct files count only files immediately inside the directory; subtree files include descendants.

| Directory | Direct files | Subtree files |
|---|---:|---:|
| `.cargo/` | 1 | 1 |
| `.config/` | 7 | 7 |
| `.github/` | 5 | 11 |
| `.github/ISSUE_TEMPLATE/` | 3 | 3 |
| `.github/workflows/` | 3 | 3 |
| `.vscode/` | 2 | 2 |
| `apps/` | 0 | 117 |
| `apps/docs-site/` | 3 | 39 |
| `apps/docs-site/examples/` | 5 | 5 |
| `apps/docs-site/fixtures/` | 5 | 5 |
| `apps/docs-site/src/` | 16 | 16 |
| `apps/docs-site/tests/` | 10 | 10 |
| `apps/playground/` | 3 | 39 |
| `apps/playground/examples/` | 5 | 5 |
| `apps/playground/fixtures/` | 5 | 5 |
| `apps/playground/src/` | 16 | 16 |
| `apps/playground/tests/` | 10 | 10 |
| `apps/studio/` | 3 | 39 |
| `apps/studio/examples/` | 5 | 5 |
| `apps/studio/fixtures/` | 5 | 5 |
| `apps/studio/src/` | 16 | 16 |
| `apps/studio/tests/` | 10 | 10 |
| `assets/` | 0 | 12 |
| `assets/brand/` | 5 | 5 |
| `assets/readme/` | 4 | 4 |
| `assets/social/` | 3 | 3 |
| `benchmarks/` | 0 | 270 |
| `benchmarks/causal/` | 0 | 45 |
| `benchmarks/causal/confounded/` | 9 | 9 |
| `benchmarks/causal/interventional/` | 9 | 9 |
| `benchmarks/causal/lagged/` | 9 | 9 |
| `benchmarks/causal/linear/` | 9 | 9 |
| `benchmarks/causal/nonlinear/` | 9 | 9 |
| `benchmarks/dynamics/` | 0 | 54 |
| `benchmarks/dynamics/delay/` | 9 | 9 |
| `benchmarks/dynamics/discrete/` | 9 | 9 |
| `benchmarks/dynamics/hybrid/` | 9 | 9 |
| `benchmarks/dynamics/ode-chaotic/` | 9 | 9 |
| `benchmarks/dynamics/ode-small/` | 9 | 9 |
| `benchmarks/dynamics/stochastic/` | 9 | 9 |
| `benchmarks/equation/` | 0 | 45 |
| `benchmarks/equation/algebraic-clean/` | 9 | 9 |
| `benchmarks/equation/algebraic-noisy/` | 9 | 9 |
| `benchmarks/equation/dimensional/` | 9 | 9 |
| `benchmarks/equation/rational/` | 9 | 9 |
| `benchmarks/equation/transcendental/` | 9 | 9 |
| `benchmarks/performance/` | 0 | 63 |
| `benchmarks/performance/bundle-io/` | 9 | 9 |
| `benchmarks/performance/end-to-end/` | 9 | 9 |
| `benchmarks/performance/expression-eval/` | 9 | 9 |
| `benchmarks/performance/python-boundary/` | 9 | 9 |
| `benchmarks/performance/simulation/` | 9 | 9 |
| `benchmarks/performance/sparse-discovery/` | 9 | 9 |
| `benchmarks/performance/symbolic-search/` | 9 | 9 |
| `benchmarks/regime/` | 0 | 36 |
| `benchmarks/regime/change-point/` | 9 | 9 |
| `benchmarks/regime/event-driven/` | 9 | 9 |
| `benchmarks/regime/hmm/` | 9 | 9 |
| `benchmarks/regime/markov-switching/` | 9 | 9 |
| `benchmarks/uncertainty/` | 0 | 27 |
| `benchmarks/uncertainty/parameter-coverage/` | 9 | 9 |
| `benchmarks/uncertainty/structural-recovery/` | 9 | 9 |
| `benchmarks/uncertainty/trajectory-coverage/` | 9 | 9 |
| `crates/` | 0 | 690 |
| `crates/lawsynth-api-types/` | 2 | 23 |
| `crates/lawsynth-api-types/benches/` | 2 | 2 |
| `crates/lawsynth-api-types/examples/` | 1 | 1 |
| `crates/lawsynth-api-types/fixtures/` | 0 | 3 |
| `crates/lawsynth-api-types/fixtures/events/` | 3 | 3 |
| `crates/lawsynth-api-types/src/` | 11 | 11 |
| `crates/lawsynth-api-types/tests/` | 4 | 4 |
| `crates/lawsynth-bundle/` | 2 | 23 |
| `crates/lawsynth-bundle/benches/` | 2 | 2 |
| `crates/lawsynth-bundle/examples/` | 1 | 1 |
| `crates/lawsynth-bundle/fixtures/` | 0 | 3 |
| `crates/lawsynth-bundle/fixtures/migration/` | 3 | 3 |
| `crates/lawsynth-bundle/src/` | 11 | 11 |
| `crates/lawsynth-bundle/tests/` | 4 | 4 |
| `crates/lawsynth-causal/` | 2 | 23 |
| `crates/lawsynth-causal/benches/` | 2 | 2 |
| `crates/lawsynth-causal/examples/` | 1 | 1 |
| `crates/lawsynth-causal/fixtures/` | 0 | 3 |
| `crates/lawsynth-causal/fixtures/sensitivity/` | 3 | 3 |
| `crates/lawsynth-causal/src/` | 11 | 11 |
| `crates/lawsynth-causal/tests/` | 4 | 4 |
| `crates/lawsynth-cli/` | 2 | 23 |
| `crates/lawsynth-cli/benches/` | 2 | 2 |
| `crates/lawsynth-cli/examples/` | 1 | 1 |
| `crates/lawsynth-cli/fixtures/` | 0 | 3 |
| `crates/lawsynth-cli/fixtures/serve/` | 3 | 3 |
| `crates/lawsynth-cli/src/` | 11 | 11 |
| `crates/lawsynth-cli/tests/` | 4 | 4 |
| `crates/lawsynth-core/` | 2 | 23 |
| `crates/lawsynth-core/benches/` | 2 | 2 |
| `crates/lawsynth-core/examples/` | 1 | 1 |
| `crates/lawsynth-core/fixtures/` | 0 | 3 |
| `crates/lawsynth-core/fixtures/diagnostics/` | 3 | 3 |
| `crates/lawsynth-core/src/` | 11 | 11 |
| `crates/lawsynth-core/tests/` | 4 | 4 |
| `crates/lawsynth-data/` | 2 | 23 |
| `crates/lawsynth-data/benches/` | 2 | 2 |
| `crates/lawsynth-data/examples/` | 1 | 1 |
| `crates/lawsynth-data/fixtures/` | 0 | 3 |
| `crates/lawsynth-data/fixtures/fingerprint/` | 3 | 3 |
| `crates/lawsynth-data/src/` | 11 | 11 |
| `crates/lawsynth-data/tests/` | 4 | 4 |
| `crates/lawsynth-differentiate/` | 2 | 23 |
| `crates/lawsynth-differentiate/benches/` | 2 | 2 |
| `crates/lawsynth-differentiate/examples/` | 1 | 1 |
| `crates/lawsynth-differentiate/fixtures/` | 0 | 3 |
| `crates/lawsynth-differentiate/fixtures/irregular/` | 3 | 3 |
| `crates/lawsynth-differentiate/src/` | 11 | 11 |
| `crates/lawsynth-differentiate/tests/` | 4 | 4 |
| `crates/lawsynth-discovery/` | 2 | 23 |
| `crates/lawsynth-discovery/benches/` | 2 | 2 |
| `crates/lawsynth-discovery/examples/` | 1 | 1 |
| `crates/lawsynth-discovery/fixtures/` | 0 | 3 |
| `crates/lawsynth-discovery/fixtures/execute/` | 3 | 3 |
| `crates/lawsynth-discovery/src/` | 11 | 11 |
| `crates/lawsynth-discovery/tests/` | 4 | 4 |
| `crates/lawsynth-dynamics/` | 2 | 23 |
| `crates/lawsynth-dynamics/benches/` | 2 | 2 |
| `crates/lawsynth-dynamics/examples/` | 1 | 1 |
| `crates/lawsynth-dynamics/fixtures/` | 0 | 3 |
| `crates/lawsynth-dynamics/fixtures/result/` | 3 | 3 |
| `crates/lawsynth-dynamics/src/` | 11 | 11 |
| `crates/lawsynth-dynamics/tests/` | 4 | 4 |
| `crates/lawsynth-egraph/` | 2 | 23 |
| `crates/lawsynth-egraph/benches/` | 2 | 2 |
| `crates/lawsynth-egraph/examples/` | 1 | 1 |
| `crates/lawsynth-egraph/fixtures/` | 0 | 3 |
| `crates/lawsynth-egraph/fixtures/limits/` | 3 | 3 |
| `crates/lawsynth-egraph/src/` | 11 | 11 |
| `crates/lawsynth-egraph/tests/` | 4 | 4 |
| `crates/lawsynth-expr/` | 2 | 23 |
| `crates/lawsynth-expr/benches/` | 2 | 2 |
| `crates/lawsynth-expr/examples/` | 1 | 1 |
| `crates/lawsynth-expr/fixtures/` | 0 | 3 |
| `crates/lawsynth-expr/fixtures/evaluate/` | 3 | 3 |
| `crates/lawsynth-expr/src/` | 11 | 11 |
| `crates/lawsynth-expr/tests/` | 4 | 4 |
| `crates/lawsynth-features/` | 2 | 23 |
| `crates/lawsynth-features/benches/` | 2 | 2 |
| `crates/lawsynth-features/examples/` | 1 | 1 |
| `crates/lawsynth-features/fixtures/` | 0 | 3 |
| `crates/lawsynth-features/fixtures/constraints/` | 3 | 3 |
| `crates/lawsynth-features/src/` | 11 | 11 |
| `crates/lawsynth-features/tests/` | 4 | 4 |
| `crates/lawsynth-opt/` | 2 | 23 |
| `crates/lawsynth-opt/benches/` | 2 | 2 |
| `crates/lawsynth-opt/examples/` | 1 | 1 |
| `crates/lawsynth-opt/fixtures/` | 0 | 3 |
| `crates/lawsynth-opt/fixtures/termination/` | 3 | 3 |
| `crates/lawsynth-opt/src/` | 11 | 11 |
| `crates/lawsynth-opt/tests/` | 4 | 4 |
| `crates/lawsynth-plugin-api/` | 2 | 23 |
| `crates/lawsynth-plugin-api/benches/` | 2 | 2 |
| `crates/lawsynth-plugin-api/examples/` | 1 | 1 |
| `crates/lawsynth-plugin-api/fixtures/` | 0 | 3 |
| `crates/lawsynth-plugin-api/fixtures/lifecycle/` | 3 | 3 |
| `crates/lawsynth-plugin-api/src/` | 11 | 11 |
| `crates/lawsynth-plugin-api/tests/` | 4 | 4 |
| `crates/lawsynth-plugin-host/` | 2 | 23 |
| `crates/lawsynth-plugin-host/benches/` | 2 | 2 |
| `crates/lawsynth-plugin-host/examples/` | 1 | 1 |
| `crates/lawsynth-plugin-host/fixtures/` | 0 | 3 |
| `crates/lawsynth-plugin-host/fixtures/lifecycle/` | 3 | 3 |
| `crates/lawsynth-plugin-host/src/` | 11 | 11 |
| `crates/lawsynth-plugin-host/tests/` | 4 | 4 |
| `crates/lawsynth-preprocess/` | 2 | 23 |
| `crates/lawsynth-preprocess/benches/` | 2 | 2 |
| `crates/lawsynth-preprocess/examples/` | 1 | 1 |
| `crates/lawsynth-preprocess/fixtures/` | 0 | 3 |
| `crates/lawsynth-preprocess/fixtures/smooth/` | 3 | 3 |
| `crates/lawsynth-preprocess/src/` | 11 | 11 |
| `crates/lawsynth-preprocess/tests/` | 4 | 4 |
| `crates/lawsynth-profile/` | 2 | 23 |
| `crates/lawsynth-profile/benches/` | 2 | 2 |
| `crates/lawsynth-profile/examples/` | 1 | 1 |
| `crates/lawsynth-profile/fixtures/` | 0 | 3 |
| `crates/lawsynth-profile/fixtures/quality_flags/` | 3 | 3 |
| `crates/lawsynth-profile/src/` | 11 | 11 |
| `crates/lawsynth-profile/tests/` | 4 | 4 |
| `crates/lawsynth-python/` | 2 | 23 |
| `crates/lawsynth-python/benches/` | 2 | 2 |
| `crates/lawsynth-python/examples/` | 1 | 1 |
| `crates/lawsynth-python/fixtures/` | 0 | 3 |
| `crates/lawsynth-python/fixtures/py_events/` | 3 | 3 |
| `crates/lawsynth-python/src/` | 11 | 11 |
| `crates/lawsynth-python/tests/` | 4 | 4 |
| `crates/lawsynth-regime/` | 2 | 23 |
| `crates/lawsynth-regime/benches/` | 2 | 2 |
| `crates/lawsynth-regime/examples/` | 1 | 1 |
| `crates/lawsynth-regime/fixtures/` | 0 | 3 |
| `crates/lawsynth-regime/fixtures/regime_laws/` | 3 | 3 |
| `crates/lawsynth-regime/src/` | 11 | 11 |
| `crates/lawsynth-regime/tests/` | 4 | 4 |
| `crates/lawsynth-runner/` | 2 | 23 |
| `crates/lawsynth-runner/benches/` | 2 | 2 |
| `crates/lawsynth-runner/examples/` | 1 | 1 |
| `crates/lawsynth-runner/fixtures/` | 0 | 3 |
| `crates/lawsynth-runner/fixtures/cancellation/` | 3 | 3 |
| `crates/lawsynth-runner/src/` | 11 | 11 |
| `crates/lawsynth-runner/tests/` | 4 | 4 |
| `crates/lawsynth-score/` | 2 | 23 |
| `crates/lawsynth-score/benches/` | 2 | 2 |
| `crates/lawsynth-score/examples/` | 1 | 1 |
| `crates/lawsynth-score/fixtures/` | 0 | 3 |
| `crates/lawsynth-score/fixtures/rank/` | 3 | 3 |
| `crates/lawsynth-score/src/` | 11 | 11 |
| `crates/lawsynth-score/tests/` | 4 | 4 |
| `crates/lawsynth-sim/` | 2 | 23 |
| `crates/lawsynth-sim/benches/` | 2 | 2 |
| `crates/lawsynth-sim/examples/` | 1 | 1 |
| `crates/lawsynth-sim/fixtures/` | 0 | 3 |
| `crates/lawsynth-sim/fixtures/hybrid/` | 3 | 3 |
| `crates/lawsynth-sim/src/` | 11 | 11 |
| `crates/lawsynth-sim/tests/` | 4 | 4 |
| `crates/lawsynth-sparse/` | 2 | 23 |
| `crates/lawsynth-sparse/benches/` | 2 | 2 |
| `crates/lawsynth-sparse/examples/` | 1 | 1 |
| `crates/lawsynth-sparse/fixtures/` | 0 | 3 |
| `crates/lawsynth-sparse/fixtures/stability/` | 3 | 3 |
| `crates/lawsynth-sparse/src/` | 11 | 11 |
| `crates/lawsynth-sparse/tests/` | 4 | 4 |
| `crates/lawsynth-stats/` | 2 | 23 |
| `crates/lawsynth-stats/benches/` | 2 | 2 |
| `crates/lawsynth-stats/examples/` | 1 | 1 |
| `crates/lawsynth-stats/fixtures/` | 0 | 3 |
| `crates/lawsynth-stats/fixtures/sampling/` | 3 | 3 |
| `crates/lawsynth-stats/src/` | 11 | 11 |
| `crates/lawsynth-stats/tests/` | 4 | 4 |
| `crates/lawsynth-store/` | 2 | 23 |
| `crates/lawsynth-store/benches/` | 2 | 2 |
| `crates/lawsynth-store/examples/` | 1 | 1 |
| `crates/lawsynth-store/fixtures/` | 0 | 3 |
| `crates/lawsynth-store/fixtures/gc/` | 3 | 3 |
| `crates/lawsynth-store/src/` | 11 | 11 |
| `crates/lawsynth-store/tests/` | 4 | 4 |
| `crates/lawsynth-symbolic/` | 2 | 23 |
| `crates/lawsynth-symbolic/benches/` | 2 | 2 |
| `crates/lawsynth-symbolic/examples/` | 1 | 1 |
| `crates/lawsynth-symbolic/fixtures/` | 0 | 3 |
| `crates/lawsynth-symbolic/fixtures/frontier/` | 3 | 3 |
| `crates/lawsynth-symbolic/src/` | 11 | 11 |
| `crates/lawsynth-symbolic/tests/` | 4 | 4 |
| `crates/lawsynth-uncertainty/` | 2 | 23 |
| `crates/lawsynth-uncertainty/benches/` | 2 | 2 |
| `crates/lawsynth-uncertainty/examples/` | 1 | 1 |
| `crates/lawsynth-uncertainty/fixtures/` | 0 | 3 |
| `crates/lawsynth-uncertainty/fixtures/propagate/` | 3 | 3 |
| `crates/lawsynth-uncertainty/src/` | 11 | 11 |
| `crates/lawsynth-uncertainty/tests/` | 4 | 4 |
| `crates/lawsynth-units/` | 2 | 23 |
| `crates/lawsynth-units/benches/` | 2 | 2 |
| `crates/lawsynth-units/examples/` | 1 | 1 |
| `crates/lawsynth-units/fixtures/` | 0 | 3 |
| `crates/lawsynth-units/fixtures/builtins/` | 3 | 3 |
| `crates/lawsynth-units/src/` | 11 | 11 |
| `crates/lawsynth-units/tests/` | 4 | 4 |
| `crates/lawsynth-wasm/` | 2 | 23 |
| `crates/lawsynth-wasm/benches/` | 2 | 2 |
| `crates/lawsynth-wasm/examples/` | 1 | 1 |
| `crates/lawsynth-wasm/fixtures/` | 0 | 3 |
| `crates/lawsynth-wasm/fixtures/errors/` | 3 | 3 |
| `crates/lawsynth-wasm/src/` | 11 | 11 |
| `crates/lawsynth-wasm/tests/` | 4 | 4 |
| `crates/lawsynth-world/` | 2 | 23 |
| `crates/lawsynth-world/benches/` | 2 | 2 |
| `crates/lawsynth-world/examples/` | 1 | 1 |
| `crates/lawsynth-world/fixtures/` | 0 | 3 |
| `crates/lawsynth-world/fixtures/intervention/` | 3 | 3 |
| `crates/lawsynth-world/src/` | 11 | 11 |
| `crates/lawsynth-world/tests/` | 4 | 4 |
| `deploy/` | 0 | 130 |
| `deploy/airgap/` | 0 | 10 |
| `deploy/airgap/bundle/` | 10 | 10 |
| `deploy/compose/` | 0 | 20 |
| `deploy/compose/local/` | 10 | 10 |
| `deploy/compose/production/` | 10 | 10 |
| `deploy/docker/` | 0 | 10 |
| `deploy/docker/images/` | 10 | 10 |
| `deploy/helm/` | 0 | 10 |
| `deploy/helm/lawsynth/` | 10 | 10 |
| `deploy/kubernetes/` | 0 | 30 |
| `deploy/kubernetes/base/` | 10 | 10 |
| `deploy/kubernetes/production/` | 10 | 10 |
| `deploy/kubernetes/staging/` | 10 | 10 |
| `deploy/observability/` | 0 | 10 |
| `deploy/observability/reference/` | 10 | 10 |
| `deploy/systemd/` | 0 | 10 |
| `deploy/systemd/single-node/` | 10 | 10 |
| `deploy/terraform/` | 0 | 30 |
| `deploy/terraform/aws/` | 10 | 10 |
| `deploy/terraform/azure/` | 10 | 10 |
| `deploy/terraform/gcp/` | 10 | 10 |
| `docs/` | 0 | 230 |
| `docs/concepts/` | 0 | 50 |
| `docs/concepts/causality/` | 10 | 10 |
| `docs/concepts/equations/` | 10 | 10 |
| `docs/concepts/regimes/` | 10 | 10 |
| `docs/concepts/uncertainty/` | 10 | 10 |
| `docs/concepts/world-ir/` | 10 | 10 |
| `docs/contributing/` | 10 | 10 |
| `docs/getting-started/` | 10 | 10 |
| `docs/guides/` | 0 | 40 |
| `docs/guides/data/` | 10 | 10 |
| `docs/guides/discovery/` | 10 | 10 |
| `docs/guides/simulation/` | 10 | 10 |
| `docs/guides/studio/` | 10 | 10 |
| `docs/methods/` | 0 | 70 |
| `docs/methods/causal/` | 10 | 10 |
| `docs/methods/differentiation/` | 10 | 10 |
| `docs/methods/regime/` | 10 | 10 |
| `docs/methods/simulation/` | 10 | 10 |
| `docs/methods/sparse/` | 10 | 10 |
| `docs/methods/symbolic/` | 10 | 10 |
| `docs/methods/uncertainty/` | 10 | 10 |
| `docs/reference/` | 0 | 30 |
| `docs/reference/cli/` | 10 | 10 |
| `docs/reference/python/` | 10 | 10 |
| `docs/reference/rust/` | 10 | 10 |
| `docs/research/` | 10 | 10 |
| `docs/self-hosting/` | 10 | 10 |
| `examples/` | 0 | 180 |
| `examples/00-quickstart/` | 7 | 9 |
| `examples/00-quickstart/expected/` | 2 | 2 |
| `examples/01-lorenz/` | 7 | 9 |
| `examples/01-lorenz/expected/` | 2 | 2 |
| `examples/02-lotka-volterra/` | 7 | 9 |
| `examples/02-lotka-volterra/expected/` | 2 | 2 |
| `examples/03-damped-pendulum/` | 7 | 9 |
| `examples/03-damped-pendulum/expected/` | 2 | 2 |
| `examples/04-sir-epidemic/` | 7 | 9 |
| `examples/04-sir-epidemic/expected/` | 2 | 2 |
| `examples/05-regime-switching/` | 7 | 9 |
| `examples/05-regime-switching/expected/` | 2 | 2 |
| `examples/06-delayed-feedback/` | 7 | 9 |
| `examples/06-delayed-feedback/expected/` | 2 | 2 |
| `examples/07-stochastic-volatility/` | 7 | 9 |
| `examples/07-stochastic-volatility/expected/` | 2 | 2 |
| `examples/08-supply-demand/` | 7 | 9 |
| `examples/08-supply-demand/expected/` | 2 | 2 |
| `examples/09-inventory-control/` | 7 | 9 |
| `examples/09-inventory-control/expected/` | 2 | 2 |
| `examples/10-energy-load/` | 7 | 9 |
| `examples/10-energy-load/expected/` | 2 | 2 |
| `examples/11-customer-growth/` | 7 | 9 |
| `examples/11-customer-growth/expected/` | 2 | 2 |
| `examples/12-macro-dynamics/` | 7 | 9 |
| `examples/12-macro-dynamics/expected/` | 2 | 2 |
| `examples/13-market-microstructure/` | 7 | 9 |
| `examples/13-market-microstructure/expected/` | 2 | 2 |
| `examples/14-synthetic-control/` | 7 | 9 |
| `examples/14-synthetic-control/expected/` | 2 | 2 |
| `examples/15-user-constraints/` | 7 | 9 |
| `examples/15-user-constraints/expected/` | 2 | 2 |
| `examples/16-custom-operator/` | 7 | 9 |
| `examples/16-custom-operator/expected/` | 2 | 2 |
| `examples/17-custom-stage/` | 7 | 9 |
| `examples/17-custom-stage/expected/` | 2 | 2 |
| `examples/18-bundle-interchange/` | 7 | 9 |
| `examples/18-bundle-interchange/expected/` | 2 | 2 |
| `examples/19-server-api/` | 7 | 9 |
| `examples/19-server-api/expected/` | 2 | 2 |
| `packages/` | 0 | 273 |
| `packages/api-client/` | 3 | 39 |
| `packages/api-client/examples/` | 5 | 5 |
| `packages/api-client/fixtures/` | 5 | 5 |
| `packages/api-client/src/` | 16 | 16 |
| `packages/api-client/tests/` | 10 | 10 |
| `packages/chart-core/` | 3 | 39 |
| `packages/chart-core/examples/` | 5 | 5 |
| `packages/chart-core/fixtures/` | 5 | 5 |
| `packages/chart-core/src/` | 16 | 16 |
| `packages/chart-core/tests/` | 10 | 10 |
| `packages/design-system/` | 3 | 39 |
| `packages/design-system/examples/` | 5 | 5 |
| `packages/design-system/fixtures/` | 5 | 5 |
| `packages/design-system/src/` | 16 | 16 |
| `packages/design-system/tests/` | 10 | 10 |
| `packages/layout-engine/` | 3 | 39 |
| `packages/layout-engine/examples/` | 5 | 5 |
| `packages/layout-engine/fixtures/` | 5 | 5 |
| `packages/layout-engine/src/` | 16 | 16 |
| `packages/layout-engine/tests/` | 10 | 10 |
| `packages/state-store/` | 3 | 39 |
| `packages/state-store/examples/` | 5 | 5 |
| `packages/state-store/fixtures/` | 5 | 5 |
| `packages/state-store/src/` | 16 | 16 |
| `packages/state-store/tests/` | 10 | 10 |
| `packages/world-schema/` | 3 | 39 |
| `packages/world-schema/examples/` | 5 | 5 |
| `packages/world-schema/fixtures/` | 5 | 5 |
| `packages/world-schema/src/` | 16 | 16 |
| `packages/world-schema/tests/` | 10 | 10 |
| `packages/world-viewer/` | 3 | 39 |
| `packages/world-viewer/examples/` | 5 | 5 |
| `packages/world-viewer/fixtures/` | 5 | 5 |
| `packages/world-viewer/src/` | 16 | 16 |
| `packages/world-viewer/tests/` | 10 | 10 |
| `plugins/` | 0 | 80 |
| `plugins/csv-variant-adapter/` | 4 | 8 |
| `plugins/csv-variant-adapter/docs/` | 1 | 1 |
| `plugins/csv-variant-adapter/examples/` | 1 | 1 |
| `plugins/csv-variant-adapter/src/` | 0 | 1 |
| `plugins/csv-variant-adapter/src/csv_variant_adapter/` | 1 | 1 |
| `plugins/csv-variant-adapter/tests/` | 1 | 1 |
| `plugins/custom-operator-rust/` | 4 | 8 |
| `plugins/custom-operator-rust/docs/` | 1 | 1 |
| `plugins/custom-operator-rust/examples/` | 1 | 1 |
| `plugins/custom-operator-rust/src/` | 1 | 1 |
| `plugins/custom-operator-rust/tests/` | 1 | 1 |
| `plugins/custom-stage-python/` | 4 | 8 |
| `plugins/custom-stage-python/docs/` | 1 | 1 |
| `plugins/custom-stage-python/examples/` | 1 | 1 |
| `plugins/custom-stage-python/src/` | 0 | 1 |
| `plugins/custom-stage-python/src/custom_stage_python/` | 1 | 1 |
| `plugins/custom-stage-python/tests/` | 1 | 1 |
| `plugins/duckdb-source/` | 4 | 8 |
| `plugins/duckdb-source/docs/` | 1 | 1 |
| `plugins/duckdb-source/examples/` | 1 | 1 |
| `plugins/duckdb-source/src/` | 0 | 1 |
| `plugins/duckdb-source/src/duckdb_source/` | 1 | 1 |
| `plugins/duckdb-source/tests/` | 1 | 1 |
| `plugins/external-simulator/` | 4 | 8 |
| `plugins/external-simulator/docs/` | 1 | 1 |
| `plugins/external-simulator/examples/` | 1 | 1 |
| `plugins/external-simulator/src/` | 1 | 1 |
| `plugins/external-simulator/tests/` | 1 | 1 |
| `plugins/finance-data-adapter/` | 4 | 8 |
| `plugins/finance-data-adapter/docs/` | 1 | 1 |
| `plugins/finance-data-adapter/examples/` | 1 | 1 |
| `plugins/finance-data-adapter/src/` | 0 | 1 |
| `plugins/finance-data-adapter/src/finance_data_adapter/` | 1 | 1 |
| `plugins/finance-data-adapter/tests/` | 1 | 1 |
| `plugins/neural-prior/` | 4 | 8 |
| `plugins/neural-prior/docs/` | 1 | 1 |
| `plugins/neural-prior/examples/` | 1 | 1 |
| `plugins/neural-prior/src/` | 0 | 1 |
| `plugins/neural-prior/src/neural_prior/` | 1 | 1 |
| `plugins/neural-prior/tests/` | 1 | 1 |
| `plugins/report-exporter/` | 4 | 8 |
| `plugins/report-exporter/docs/` | 1 | 1 |
| `plugins/report-exporter/examples/` | 1 | 1 |
| `plugins/report-exporter/src/` | 0 | 1 |
| `plugins/report-exporter/src/report_exporter/` | 1 | 1 |
| `plugins/report-exporter/tests/` | 1 | 1 |
| `plugins/scenario-exporter/` | 4 | 8 |
| `plugins/scenario-exporter/docs/` | 1 | 1 |
| `plugins/scenario-exporter/examples/` | 1 | 1 |
| `plugins/scenario-exporter/src/` | 1 | 1 |
| `plugins/scenario-exporter/tests/` | 1 | 1 |
| `plugins/world-validator-wasi/` | 4 | 8 |
| `plugins/world-validator-wasi/docs/` | 1 | 1 |
| `plugins/world-validator-wasi/examples/` | 1 | 1 |
| `plugins/world-validator-wasi/src/` | 1 | 1 |
| `plugins/world-validator-wasi/tests/` | 1 | 1 |
| `python/` | 0 | 355 |
| `python/lawsynth/` | 3 | 71 |
| `python/lawsynth-bench/` | 3 | 71 |
| `python/lawsynth-bench/docs/` | 10 | 10 |
| `python/lawsynth-bench/fixtures/` | 0 | 10 |
| `python/lawsynth-bench/fixtures/baseline/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/cli/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/dataset/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/environment/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/leaderboard/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/metrics/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/problem/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/registry/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/report/` | 1 | 1 |
| `python/lawsynth-bench/fixtures/runner/` | 1 | 1 |
| `python/lawsynth-bench/src/` | 0 | 26 |
| `python/lawsynth-bench/src/lawsynth_bench/` | 26 | 26 |
| `python/lawsynth-bench/tests/` | 22 | 22 |
| `python/lawsynth-connectors/` | 3 | 71 |
| `python/lawsynth-connectors/docs/` | 10 | 10 |
| `python/lawsynth-connectors/fixtures/` | 0 | 10 |
| `python/lawsynth-connectors/fixtures/base/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/delta/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/duckdb/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/filesystem/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/http/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/iceberg/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/postgres/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/registry/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/s3/` | 1 | 1 |
| `python/lawsynth-connectors/fixtures/sql/` | 1 | 1 |
| `python/lawsynth-connectors/src/` | 0 | 26 |
| `python/lawsynth-connectors/src/lawsynth_connectors/` | 26 | 26 |
| `python/lawsynth-connectors/tests/` | 22 | 22 |
| `python/lawsynth-notebook/` | 3 | 71 |
| `python/lawsynth-notebook/docs/` | 10 | 10 |
| `python/lawsynth-notebook/fixtures/` | 0 | 10 |
| `python/lawsynth-notebook/fixtures/assets/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/display/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/equation_view/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/events/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/frontier_view/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/graph_view/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/regime_view/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/trajectory_view/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/uncertainty_view/` | 1 | 1 |
| `python/lawsynth-notebook/fixtures/widget/` | 1 | 1 |
| `python/lawsynth-notebook/src/` | 0 | 26 |
| `python/lawsynth-notebook/src/lawsynth_notebook/` | 26 | 26 |
| `python/lawsynth-notebook/tests/` | 22 | 22 |
| `python/lawsynth-server/` | 3 | 71 |
| `python/lawsynth-server/docs/` | 10 | 10 |
| `python/lawsynth-server/fixtures/` | 0 | 10 |
| `python/lawsynth-server/fixtures/app/` | 1 | 1 |
| `python/lawsynth-server/fixtures/auth/` | 1 | 1 |
| `python/lawsynth-server/fixtures/datasets/` | 1 | 1 |
| `python/lawsynth-server/fixtures/dependencies/` | 1 | 1 |
| `python/lawsynth-server/fixtures/events/` | 1 | 1 |
| `python/lawsynth-server/fixtures/idempotency/` | 1 | 1 |
| `python/lawsynth-server/fixtures/lifespan/` | 1 | 1 |
| `python/lawsynth-server/fixtures/pagination/` | 1 | 1 |
| `python/lawsynth-server/fixtures/projects/` | 1 | 1 |
| `python/lawsynth-server/fixtures/settings/` | 1 | 1 |
| `python/lawsynth-server/src/` | 0 | 26 |
| `python/lawsynth-server/src/lawsynth_server/` | 26 | 26 |
| `python/lawsynth-server/tests/` | 22 | 22 |
| `python/lawsynth/docs/` | 10 | 10 |
| `python/lawsynth/fixtures/` | 0 | 10 |
| `python/lawsynth/fixtures/assumptions/` | 1 | 1 |
| `python/lawsynth/fixtures/candidate/` | 1 | 1 |
| `python/lawsynth/fixtures/dataset/` | 1 | 1 |
| `python/lawsynth/fixtures/equation/` | 1 | 1 |
| `python/lawsynth/fixtures/frontier/` | 1 | 1 |
| `python/lawsynth/fixtures/graph/` | 1 | 1 |
| `python/lawsynth/fixtures/plan/` | 1 | 1 |
| `python/lawsynth/fixtures/run/` | 1 | 1 |
| `python/lawsynth/fixtures/units/` | 1 | 1 |
| `python/lawsynth/fixtures/variable/` | 1 | 1 |
| `python/lawsynth/src/` | 0 | 26 |
| `python/lawsynth/src/lawsynth/` | 26 | 26 |
| `python/lawsynth/tests/` | 22 | 22 |
| `services/` | 0 | 235 |
| `services/api/` | 4 | 47 |
| `services/api/config/` | 6 | 6 |
| `services/api/docs/` | 5 | 5 |
| `services/api/src/` | 0 | 20 |
| `services/api/src/lawsynth_api/` | 20 | 20 |
| `services/api/tests/` | 12 | 12 |
| `services/artifact/` | 4 | 47 |
| `services/artifact/config/` | 6 | 6 |
| `services/artifact/docs/` | 5 | 5 |
| `services/artifact/src/` | 20 | 20 |
| `services/artifact/tests/` | 12 | 12 |
| `services/gateway/` | 4 | 47 |
| `services/gateway/config/` | 6 | 6 |
| `services/gateway/docs/` | 5 | 5 |
| `services/gateway/src/` | 20 | 20 |
| `services/gateway/tests/` | 12 | 12 |
| `services/scheduler/` | 4 | 47 |
| `services/scheduler/config/` | 6 | 6 |
| `services/scheduler/docs/` | 5 | 5 |
| `services/scheduler/src/` | 20 | 20 |
| `services/scheduler/tests/` | 12 | 12 |
| `services/worker/` | 4 | 47 |
| `services/worker/config/` | 6 | 6 |
| `services/worker/docs/` | 5 | 5 |
| `services/worker/src/` | 20 | 20 |
| `services/worker/tests/` | 12 | 12 |
| `specs/` | 2 | 142 |
| `specs/bundle/` | 10 | 10 |
| `specs/causal-contract/` | 10 | 10 |
| `specs/dataset-contract/` | 10 | 10 |
| `specs/discovery-run/` | 10 | 10 |
| `specs/event-protocol/` | 10 | 10 |
| `specs/expression-language/` | 10 | 10 |
| `specs/plugin-protocol/` | 10 | 10 |
| `specs/regime-contract/` | 10 | 10 |
| `specs/reproducibility/` | 10 | 10 |
| `specs/security-model/` | 10 | 10 |
| `specs/service-api/` | 10 | 10 |
| `specs/simulation-contract/` | 10 | 10 |
| `specs/uncertainty-contract/` | 10 | 10 |
| `specs/world-ir/` | 10 | 10 |
| `tests/` | 0 | 305 |
| `tests/chaos/` | 0 | 25 |
| `tests/chaos/api-restart/` | 5 | 5 |
| `tests/chaos/duplicate-delivery/` | 5 | 5 |
| `tests/chaos/scheduler-restart/` | 5 | 5 |
| `tests/chaos/storage-timeout/` | 5 | 5 |
| `tests/chaos/worker-loss/` | 5 | 5 |
| `tests/compatibility/` | 0 | 20 |
| `tests/compatibility/forward-fields/` | 5 | 5 |
| `tests/compatibility/plugin-protocol/` | 5 | 5 |
| `tests/compatibility/v0-bundles/` | 5 | 5 |
| `tests/compatibility/v1-migrations/` | 5 | 5 |
| `tests/conformance/` | 0 | 60 |
| `tests/conformance/bad-expression/` | 5 | 5 |
| `tests/conformance/bad-hash/` | 5 | 5 |
| `tests/conformance/bad-schema/` | 5 | 5 |
| `tests/conformance/bad-units/` | 5 | 5 |
| `tests/conformance/continuous-world/` | 5 | 5 |
| `tests/conformance/discrete-world/` | 5 | 5 |
| `tests/conformance/hybrid-world/` | 5 | 5 |
| `tests/conformance/minimal-world/` | 5 | 5 |
| `tests/conformance/regime-world/` | 5 | 5 |
| `tests/conformance/signed-bundle/` | 5 | 5 |
| `tests/conformance/stochastic-world/` | 5 | 5 |
| `tests/conformance/unsafe-archive/` | 5 | 5 |
| `tests/cross-language/` | 0 | 25 |
| `tests/cross-language/bundle-roundtrip/` | 5 | 5 |
| `tests/cross-language/python-rust/` | 5 | 5 |
| `tests/cross-language/rust-python/` | 5 | 5 |
| `tests/cross-language/schema-roundtrip/` | 5 | 5 |
| `tests/cross-language/typescript-rust/` | 5 | 5 |
| `tests/end-to-end/` | 0 | 45 |
| `tests/end-to-end/cancellation/` | 5 | 5 |
| `tests/end-to-end/cli-discover/` | 5 | 5 |
| `tests/end-to-end/cli-simulate/` | 5 | 5 |
| `tests/end-to-end/export/` | 5 | 5 |
| `tests/end-to-end/import/` | 5 | 5 |
| `tests/end-to-end/local-library/` | 5 | 5 |
| `tests/end-to-end/local-studio/` | 5 | 5 |
| `tests/end-to-end/resume/` | 5 | 5 |
| `tests/end-to-end/server-run/` | 5 | 5 |
| `tests/performance/` | 0 | 50 |
| `tests/performance/bundle-open/` | 5 | 5 |
| `tests/performance/cancellation-latency/` | 5 | 5 |
| `tests/performance/event-latency/` | 5 | 5 |
| `tests/performance/expression-throughput/` | 5 | 5 |
| `tests/performance/import-time/` | 5 | 5 |
| `tests/performance/memory-budget/` | 5 | 5 |
| `tests/performance/ode-simulation/` | 5 | 5 |
| `tests/performance/parquet-load/` | 5 | 5 |
| `tests/performance/profile-million/` | 5 | 5 |
| `tests/performance/studio-paint/` | 5 | 5 |
| `tests/scientific/` | 0 | 50 |
| `tests/scientific/adversarial-noise/` | 5 | 5 |
| `tests/scientific/irregular-sampling/` | 5 | 5 |
| `tests/scientific/lorenz-recovery/` | 5 | 5 |
| `tests/scientific/lotka-volterra-recovery/` | 5 | 5 |
| `tests/scientific/missing-data/` | 5 | 5 |
| `tests/scientific/pendulum-recovery/` | 5 | 5 |
| `tests/scientific/regime-recovery/` | 5 | 5 |
| `tests/scientific/sir-recovery/` | 5 | 5 |
| `tests/scientific/uncertainty-coverage/` | 5 | 5 |
| `tests/scientific/unit-consistency/` | 5 | 5 |
| `tests/security/` | 0 | 30 |
| `tests/security/archive-traversal/` | 5 | 5 |
| `tests/security/authorization/` | 5 | 5 |
| `tests/security/decompression-limits/` | 5 | 5 |
| `tests/security/expression-limits/` | 5 | 5 |
| `tests/security/plugin-permissions/` | 5 | 5 |
| `tests/security/tenant-isolation/` | 5 | 5 |
| `tools/` | 0 | 80 |
| `tools/api-doc-gen/` | 2 | 8 |
| `tools/api-doc-gen/src/` | 5 | 5 |
| `tools/api-doc-gen/tests/` | 1 | 1 |
| `tools/benchmark-site/` | 2 | 8 |
| `tools/benchmark-site/src/` | 5 | 5 |
| `tools/benchmark-site/tests/` | 1 | 1 |
| `tools/binding-gen/` | 2 | 8 |
| `tools/binding-gen/src/` | 5 | 5 |
| `tools/binding-gen/tests/` | 1 | 1 |
| `tools/bundle-inspector/` | 2 | 8 |
| `tools/bundle-inspector/src/` | 5 | 5 |
| `tools/bundle-inspector/tests/` | 1 | 1 |
| `tools/conformance-runner/` | 2 | 8 |
| `tools/conformance-runner/src/` | 5 | 5 |
| `tools/conformance-runner/tests/` | 1 | 1 |
| `tools/dataset-registry/` | 2 | 8 |
| `tools/dataset-registry/src/` | 5 | 5 |
| `tools/dataset-registry/tests/` | 1 | 1 |
| `tools/fixture-builder/` | 2 | 8 |
| `tools/fixture-builder/src/` | 5 | 5 |
| `tools/fixture-builder/tests/` | 1 | 1 |
| `tools/license-check/` | 2 | 8 |
| `tools/license-check/src/` | 5 | 5 |
| `tools/license-check/tests/` | 1 | 1 |
| `tools/release-notes/` | 2 | 8 |
| `tools/release-notes/src/` | 5 | 5 |
| `tools/release-notes/tests/` | 1 | 1 |
| `tools/schema-gen/` | 2 | 8 |
| `tools/schema-gen/src/` | 5 | 5 |
| `tools/schema-gen/tests/` | 1 | 1 |

## 4. Complete repository tree

The following tree contains every directory and all 3,161 files.

```text
lawsynth/
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
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug.yml
│   │   ├── feature.yml
│   │   └── research-method.yml
│   ├── workflows/
│   │   ├── nightly-science.yml
│   │   ├── pr.yml
│   │   └── release.yml
│   ├── CODEOWNERS
│   ├── dependabot.yml
│   ├── FUNDING.yml
│   ├── labeler.yml
│   └── pull_request_template.md
├── .vscode/
│   ├── extensions.json
│   └── settings.json
├── apps/
│   ├── docs-site/
│   │   ├── examples/
│   │   │   ├── analytics.example.ts
│   │   │   ├── redirects.example.ts
│   │   │   ├── seo.example.ts
│   │   │   ├── theme.example.ts
│   │   │   └── versions.example.ts
│   │   ├── fixtures/
│   │   │   ├── code.json
│   │   │   ├── markdown.json
│   │   │   ├── navigation.json
│   │   │   ├── search.json
│   │   │   └── site.json
│   │   ├── src/
│   │   │   ├── analytics.ts
│   │   │   ├── api_reference.ts
│   │   │   ├── benchmarks.ts
│   │   │   ├── blog.ts
│   │   │   ├── code.ts
│   │   │   ├── equations.ts
│   │   │   ├── examples.ts
│   │   │   ├── index.ts
│   │   │   ├── markdown.ts
│   │   │   ├── navigation.ts
│   │   │   ├── redirects.ts
│   │   │   ├── search.ts
│   │   │   ├── seo.ts
│   │   │   ├── site.ts
│   │   │   ├── theme.ts
│   │   │   └── versions.ts
│   │   ├── tests/
│   │   │   ├── api_reference.test.ts
│   │   │   ├── benchmarks.test.ts
│   │   │   ├── blog.test.ts
│   │   │   ├── code.test.ts
│   │   │   ├── equations.test.ts
│   │   │   ├── examples.test.ts
│   │   │   ├── markdown.test.ts
│   │   │   ├── navigation.test.ts
│   │   │   ├── search.test.ts
│   │   │   └── site.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── playground/
│   │   ├── examples/
│   │   │   ├── embed.example.ts
│   │   │   ├── errors.example.ts
│   │   │   ├── storage.example.ts
│   │   │   ├── theme.example.ts
│   │   │   └── worker.example.ts
│   │   ├── fixtures/
│   │   │   ├── dataset_picker.json
│   │   │   ├── editor.json
│   │   │   ├── parameter_panel.json
│   │   │   ├── playground.json
│   │   │   └── world_picker.json
│   │   ├── src/
│   │   │   ├── charts.ts
│   │   │   ├── dataset_picker.ts
│   │   │   ├── editor.ts
│   │   │   ├── embed.ts
│   │   │   ├── errors.ts
│   │   │   ├── examples.ts
│   │   │   ├── index.ts
│   │   │   ├── parameter_panel.ts
│   │   │   ├── playground.ts
│   │   │   ├── share.ts
│   │   │   ├── simulation.ts
│   │   │   ├── storage.ts
│   │   │   ├── theme.ts
│   │   │   ├── wasm.ts
│   │   │   ├── worker.ts
│   │   │   └── world_picker.ts
│   │   ├── tests/
│   │   │   ├── charts.test.ts
│   │   │   ├── dataset_picker.test.ts
│   │   │   ├── editor.test.ts
│   │   │   ├── examples.test.ts
│   │   │   ├── parameter_panel.test.ts
│   │   │   ├── playground.test.ts
│   │   │   ├── share.test.ts
│   │   │   ├── simulation.test.ts
│   │   │   ├── wasm.test.ts
│   │   │   └── world_picker.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   └── studio/
│       ├── examples/
│       │   ├── export.example.ts
│       │   ├── provenance.example.ts
│       │   ├── settings.example.ts
│       │   ├── shortcuts.example.ts
│       │   └── uncertainty.example.ts
│       ├── fixtures/
│       │   ├── app.json
│       │   ├── dataset.json
│       │   ├── providers.json
│       │   ├── routes.json
│       │   └── workspace.json
│       ├── src/
│       │   ├── app.ts
│       │   ├── dataset.ts
│       │   ├── discovery.ts
│       │   ├── equations.ts
│       │   ├── export.ts
│       │   ├── index.ts
│       │   ├── provenance.ts
│       │   ├── providers.ts
│       │   ├── regimes.ts
│       │   ├── routes.ts
│       │   ├── settings.ts
│       │   ├── shortcuts.ts
│       │   ├── simulation.ts
│       │   ├── structure.ts
│       │   ├── uncertainty.ts
│       │   └── workspace.ts
│       ├── tests/
│       │   ├── app.test.ts
│       │   ├── dataset.test.ts
│       │   ├── discovery.test.ts
│       │   ├── equations.test.ts
│       │   ├── providers.test.ts
│       │   ├── regimes.test.ts
│       │   ├── routes.test.ts
│       │   ├── simulation.test.ts
│       │   ├── structure.test.ts
│       │   └── workspace.test.ts
│       ├── package.json
│       ├── README.md
│       └── tsconfig.json
├── assets/
│   ├── brand/
│   │   ├── logo-mark.svg
│   │   ├── logo.svg
│   │   ├── palette.json
│   │   ├── typography.md
│   │   └── wordmark.svg
│   ├── readme/
│   │   ├── hero.webp
│   │   ├── lorenz-demo.gif
│   │   ├── pipeline.svg
│   │   └── studio.webp
│   └── social/
│       ├── announcement.png
│       ├── demo-thumbnail.png
│       └── github-card.png
├── benchmarks/
│   ├── causal/
│   │   ├── confounded/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── interventional/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── lagged/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── linear/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   └── nonlinear/
│   │       ├── baseline.json
│   │       ├── benchmark.toml
│   │       ├── expected.json
│   │       ├── generate.py
│   │       ├── README.md
│   │       ├── report.md
│   │       ├── run.py
│   │       ├── score.py
│   │       └── test_benchmark.py
│   ├── dynamics/
│   │   ├── delay/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── discrete/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── hybrid/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── ode-chaotic/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── ode-small/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   └── stochastic/
│   │       ├── baseline.json
│   │       ├── benchmark.toml
│   │       ├── expected.json
│   │       ├── generate.py
│   │       ├── README.md
│   │       ├── report.md
│   │       ├── run.py
│   │       ├── score.py
│   │       └── test_benchmark.py
│   ├── equation/
│   │   ├── algebraic-clean/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── algebraic-noisy/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── dimensional/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── rational/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   └── transcendental/
│   │       ├── baseline.json
│   │       ├── benchmark.toml
│   │       ├── expected.json
│   │       ├── generate.py
│   │       ├── README.md
│   │       ├── report.md
│   │       ├── run.py
│   │       ├── score.py
│   │       └── test_benchmark.py
│   ├── performance/
│   │   ├── bundle-io/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── end-to-end/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── expression-eval/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── python-boundary/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── simulation/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── sparse-discovery/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   └── symbolic-search/
│   │       ├── baseline.json
│   │       ├── benchmark.toml
│   │       ├── expected.json
│   │       ├── generate.py
│   │       ├── README.md
│   │       ├── report.md
│   │       ├── run.py
│   │       ├── score.py
│   │       └── test_benchmark.py
│   ├── regime/
│   │   ├── change-point/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── event-driven/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   ├── hmm/
│   │   │   ├── baseline.json
│   │   │   ├── benchmark.toml
│   │   │   ├── expected.json
│   │   │   ├── generate.py
│   │   │   ├── README.md
│   │   │   ├── report.md
│   │   │   ├── run.py
│   │   │   ├── score.py
│   │   │   └── test_benchmark.py
│   │   └── markov-switching/
│   │       ├── baseline.json
│   │       ├── benchmark.toml
│   │       ├── expected.json
│   │       ├── generate.py
│   │       ├── README.md
│   │       ├── report.md
│   │       ├── run.py
│   │       ├── score.py
│   │       └── test_benchmark.py
│   └── uncertainty/
│       ├── parameter-coverage/
│       │   ├── baseline.json
│       │   ├── benchmark.toml
│       │   ├── expected.json
│       │   ├── generate.py
│       │   ├── README.md
│       │   ├── report.md
│       │   ├── run.py
│       │   ├── score.py
│       │   └── test_benchmark.py
│       ├── structural-recovery/
│       │   ├── baseline.json
│       │   ├── benchmark.toml
│       │   ├── expected.json
│       │   ├── generate.py
│       │   ├── README.md
│       │   ├── report.md
│       │   ├── run.py
│       │   ├── score.py
│       │   └── test_benchmark.py
│       └── trajectory-coverage/
│           ├── baseline.json
│           ├── benchmark.toml
│           ├── expected.json
│           ├── generate.py
│           ├── README.md
│           ├── report.md
│           ├── run.py
│           ├── score.py
│           └── test_benchmark.py
├── crates/
│   ├── lawsynth-api-types/
│   │   ├── benches/
│   │   │   ├── artifact_latency.rs
│   │   │   └── simulation_throughput.rs
│   │   ├── examples/
│   │   │   └── pagination_basic.rs
│   │   ├── fixtures/
│   │   │   └── events/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── artifact.rs
│   │   │   ├── config.rs
│   │   │   ├── dataset.rs
│   │   │   ├── error.rs
│   │   │   ├── events.rs
│   │   │   ├── lib.rs
│   │   │   ├── pagination.rs
│   │   │   ├── project.rs
│   │   │   ├── run.rs
│   │   │   ├── simulation.rs
│   │   │   └── world.rs
│   │   ├── tests/
│   │   │   ├── dataset_integration.rs
│   │   │   ├── project_unit.rs
│   │   │   ├── run_property.rs
│   │   │   └── world_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-bundle/
│   │   ├── benches/
│   │   │   ├── canonical_throughput.rs
│   │   │   └── checksum_latency.rs
│   │   ├── examples/
│   │   │   └── signature_basic.rs
│   │   ├── fixtures/
│   │   │   └── migration/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── canonical.rs
│   │   │   ├── checksum.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── layout.rs
│   │   │   ├── lib.rs
│   │   │   ├── manifest.rs
│   │   │   ├── migration.rs
│   │   │   ├── reader.rs
│   │   │   ├── signature.rs
│   │   │   └── writer.rs
│   │   ├── tests/
│   │   │   ├── layout_integration.rs
│   │   │   ├── manifest_unit.rs
│   │   │   ├── reader_property.rs
│   │   │   └── writer_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-causal/
│   │   ├── benches/
│   │   │   ├── granger_throughput.rs
│   │   │   └── independence_latency.rs
│   │   ├── examples/
│   │   │   └── equivalence_basic.rs
│   │   ├── fixtures/
│   │   │   └── sensitivity/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── assumptions.rs
│   │   │   ├── config.rs
│   │   │   ├── equivalence.rs
│   │   │   ├── error.rs
│   │   │   ├── granger.rs
│   │   │   ├── graph.rs
│   │   │   ├── independence.rs
│   │   │   ├── lagged.rs
│   │   │   ├── lib.rs
│   │   │   ├── sensitivity.rs
│   │   │   └── time_order.rs
│   │   ├── tests/
│   │   │   ├── assumptions_integration.rs
│   │   │   ├── graph_unit.rs
│   │   │   ├── lagged_roundtrip.rs
│   │   │   └── time_order_property.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-cli/
│   │   ├── benches/
│   │   │   ├── profile_throughput.rs
│   │   │   └── simulate_latency.rs
│   │   ├── examples/
│   │   │   └── intervene_basic.rs
│   │   ├── fixtures/
│   │   │   └── serve/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── args.rs
│   │   │   ├── config.rs
│   │   │   ├── discover.rs
│   │   │   ├── error.rs
│   │   │   ├── inspect.rs
│   │   │   ├── intervene.rs
│   │   │   ├── lib.rs
│   │   │   ├── output.rs
│   │   │   ├── profile.rs
│   │   │   ├── serve.rs
│   │   │   └── simulate.rs
│   │   ├── tests/
│   │   │   ├── args_unit.rs
│   │   │   ├── discover_property.rs
│   │   │   ├── inspect_roundtrip.rs
│   │   │   └── output_integration.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-core/
│   │   ├── benches/
│   │   │   ├── cancel_throughput.rs
│   │   │   └── resource_latency.rs
│   │   ├── examples/
│   │   │   └── progress_basic.rs
│   │   ├── fixtures/
│   │   │   └── diagnostics/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── cancel.rs
│   │   │   ├── config.rs
│   │   │   ├── diagnostics.rs
│   │   │   ├── error.rs
│   │   │   ├── hash.rs
│   │   │   ├── id.rs
│   │   │   ├── lib.rs
│   │   │   ├── progress.rs
│   │   │   ├── resource.rs
│   │   │   ├── seed.rs
│   │   │   └── version.rs
│   │   ├── tests/
│   │   │   ├── hash_property.rs
│   │   │   ├── id_unit.rs
│   │   │   ├── seed_roundtrip.rs
│   │   │   └── version_integration.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-data/
│   │   ├── benches/
│   │   │   ├── batch_latency.rs
│   │   │   └── window_throughput.rs
│   │   ├── examples/
│   │   │   └── parquet_basic.rs
│   │   ├── fixtures/
│   │   │   └── fingerprint/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── batch.rs
│   │   │   ├── column.rs
│   │   │   ├── config.rs
│   │   │   ├── dataset.rs
│   │   │   ├── error.rs
│   │   │   ├── fingerprint.rs
│   │   │   ├── lib.rs
│   │   │   ├── parquet.rs
│   │   │   ├── schema.rs
│   │   │   ├── time_axis.rs
│   │   │   └── window.rs
│   │   ├── tests/
│   │   │   ├── column_property.rs
│   │   │   ├── dataset_unit.rs
│   │   │   ├── schema_integration.rs
│   │   │   └── time_axis_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-differentiate/
│   │   ├── benches/
│   │   │   ├── spectral_latency.rs
│   │   │   └── tvreg_throughput.rs
│   │   ├── examples/
│   │   │   └── weak_form_basic.rs
│   │   ├── fixtures/
│   │   │   └── irregular/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── finite.rs
│   │   │   ├── irregular.rs
│   │   │   ├── lib.rs
│   │   │   ├── method.rs
│   │   │   ├── savgol.rs
│   │   │   ├── spectral.rs
│   │   │   ├── spline.rs
│   │   │   ├── tvreg.rs
│   │   │   └── weak_form.rs
│   │   ├── tests/
│   │   │   ├── finite_integration.rs
│   │   │   ├── method_unit.rs
│   │   │   ├── savgol_property.rs
│   │   │   └── spline_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-discovery/
│   │   ├── benches/
│   │   │   ├── branch_latency.rs
│   │   │   └── candidate_throughput.rs
│   │   ├── examples/
│   │   │   └── checkpoint_basic.rs
│   │   ├── fixtures/
│   │   │   └── execute/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── assumptions.rs
│   │   │   ├── branch.rs
│   │   │   ├── candidate.rs
│   │   │   ├── checkpoint.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── execute.rs
│   │   │   ├── graph.rs
│   │   │   ├── lib.rs
│   │   │   ├── plan.rs
│   │   │   └── stage.rs
│   │   ├── tests/
│   │   │   ├── assumptions_integration.rs
│   │   │   ├── graph_roundtrip.rs
│   │   │   ├── plan_unit.rs
│   │   │   └── stage_property.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-dynamics/
│   │   ├── benches/
│   │   │   ├── control_latency.rs
│   │   │   └── implicit_throughput.rs
│   │   ├── examples/
│   │   │   └── refine_basic.rs
│   │   ├── fixtures/
│   │   │   └── result/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── continuous.rs
│   │   │   ├── control.rs
│   │   │   ├── delay.rs
│   │   │   ├── discrete.rs
│   │   │   ├── error.rs
│   │   │   ├── implicit.rs
│   │   │   ├── lib.rs
│   │   │   ├── problem.rs
│   │   │   ├── refine.rs
│   │   │   └── result.rs
│   │   ├── tests/
│   │   │   ├── continuous_integration.rs
│   │   │   ├── delay_roundtrip.rs
│   │   │   ├── discrete_property.rs
│   │   │   └── problem_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-egraph/
│   │   ├── benches/
│   │   │   ├── cost_latency.rs
│   │   │   └── extract_throughput.rs
│   │   ├── examples/
│   │   │   └── proof_basic.rs
│   │   ├── fixtures/
│   │   │   └── limits/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── analysis.rs
│   │   │   ├── config.rs
│   │   │   ├── cost.rs
│   │   │   ├── error.rs
│   │   │   ├── extract.rs
│   │   │   ├── language.rs
│   │   │   ├── lib.rs
│   │   │   ├── limits.rs
│   │   │   ├── proof.rs
│   │   │   ├── rules.rs
│   │   │   └── schedule.rs
│   │   ├── tests/
│   │   │   ├── analysis_integration.rs
│   │   │   ├── language_unit.rs
│   │   │   ├── rules_property.rs
│   │   │   └── schedule_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-expr/
│   │   ├── benches/
│   │   │   ├── parser_latency.rs
│   │   │   └── symbol_throughput.rs
│   │   ├── examples/
│   │   │   └── printer_basic.rs
│   │   ├── fixtures/
│   │   │   └── evaluate/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── ast.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── evaluate.rs
│   │   │   ├── lib.rs
│   │   │   ├── literal.rs
│   │   │   ├── node.rs
│   │   │   ├── operator.rs
│   │   │   ├── parser.rs
│   │   │   ├── printer.rs
│   │   │   └── symbol.rs
│   │   ├── tests/
│   │   │   ├── ast_unit.rs
│   │   │   ├── literal_roundtrip.rs
│   │   │   ├── node_integration.rs
│   │   │   └── operator_property.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-features/
│   │   ├── benches/
│   │   │   ├── delay_latency.rs
│   │   │   └── rational_throughput.rs
│   │   ├── examples/
│   │   │   └── interaction_basic.rs
│   │   ├── fixtures/
│   │   │   └── constraints/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── constraints.rs
│   │   │   ├── delay.rs
│   │   │   ├── error.rs
│   │   │   ├── interaction.rs
│   │   │   ├── lib.rs
│   │   │   ├── library.rs
│   │   │   ├── polynomial.rs
│   │   │   ├── rational.rs
│   │   │   ├── term.rs
│   │   │   └── trigonometric.rs
│   │   ├── tests/
│   │   │   ├── library_unit.rs
│   │   │   ├── polynomial_property.rs
│   │   │   ├── term_integration.rs
│   │   │   └── trigonometric_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-opt/
│   │   ├── benches/
│   │   │   ├── coordinate_latency.rs
│   │   │   └── nelder_mead_throughput.rs
│   │   ├── examples/
│   │   │   └── mixed_basic.rs
│   │   ├── fixtures/
│   │   │   └── termination/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── bounds.rs
│   │   │   ├── config.rs
│   │   │   ├── coordinate.rs
│   │   │   ├── error.rs
│   │   │   ├── lbfgs.rs
│   │   │   ├── least_squares.rs
│   │   │   ├── lib.rs
│   │   │   ├── mixed.rs
│   │   │   ├── nelder_mead.rs
│   │   │   ├── objective.rs
│   │   │   └── termination.rs
│   │   ├── tests/
│   │   │   ├── bounds_integration.rs
│   │   │   ├── lbfgs_roundtrip.rs
│   │   │   ├── least_squares_property.rs
│   │   │   └── objective_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-plugin-api/
│   │   ├── benches/
│   │   │   ├── protocol_latency.rs
│   │   │   └── simulator_throughput.rs
│   │   ├── examples/
│   │   │   └── limits_basic.rs
│   │   ├── fixtures/
│   │   │   └── lifecycle/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── algorithm.rs
│   │   │   ├── capability.rs
│   │   │   ├── config.rs
│   │   │   ├── data_adapter.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── lifecycle.rs
│   │   │   ├── limits.rs
│   │   │   ├── manifest.rs
│   │   │   ├── protocol.rs
│   │   │   └── simulator.rs
│   │   ├── tests/
│   │   │   ├── algorithm_property.rs
│   │   │   ├── capability_integration.rs
│   │   │   ├── data_adapter_roundtrip.rs
│   │   │   └── manifest_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-plugin-host/
│   │   ├── benches/
│   │   │   ├── permissions_latency.rs
│   │   │   └── rpc_throughput.rs
│   │   ├── examples/
│   │   │   └── resources_basic.rs
│   │   ├── fixtures/
│   │   │   └── lifecycle/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── discover.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── lifecycle.rs
│   │   │   ├── permissions.rs
│   │   │   ├── process.rs
│   │   │   ├── registry.rs
│   │   │   ├── resources.rs
│   │   │   ├── rpc.rs
│   │   │   └── wasi.rs
│   │   ├── tests/
│   │   │   ├── discover_unit.rs
│   │   │   ├── process_property.rs
│   │   │   ├── registry_integration.rs
│   │   │   └── wasi_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-preprocess/
│   │   ├── benches/
│   │   │   ├── impute_throughput.rs
│   │   │   └── scale_latency.rs
│   │   ├── examples/
│   │   │   └── detrend_basic.rs
│   │   ├── fixtures/
│   │   │   └── smooth/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── align.rs
│   │   │   ├── config.rs
│   │   │   ├── detrend.rs
│   │   │   ├── error.rs
│   │   │   ├── impute.rs
│   │   │   ├── lib.rs
│   │   │   ├── pipeline.rs
│   │   │   ├── resample.rs
│   │   │   ├── scale.rs
│   │   │   ├── smooth.rs
│   │   │   └── transform.rs
│   │   ├── tests/
│   │   │   ├── align_property.rs
│   │   │   ├── pipeline_unit.rs
│   │   │   ├── resample_roundtrip.rs
│   │   │   └── transform_integration.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-profile/
│   │   ├── benches/
│   │   │   ├── dependence_latency.rs
│   │   │   └── distribution_throughput.rs
│   │   ├── examples/
│   │   │   └── delays_basic.rs
│   │   ├── fixtures/
│   │   │   └── quality_flags/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── column_profile.rs
│   │   │   ├── config.rs
│   │   │   ├── delays.rs
│   │   │   ├── dependence.rs
│   │   │   ├── distribution.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── missingness.rs
│   │   │   ├── profiler.rs
│   │   │   ├── quality_flags.rs
│   │   │   └── time_profile.rs
│   │   ├── tests/
│   │   │   ├── column_profile_integration.rs
│   │   │   ├── missingness_roundtrip.rs
│   │   │   ├── profiler_unit.rs
│   │   │   └── time_profile_property.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-python/
│   │   ├── benches/
│   │   │   ├── py_simulation_latency.rs
│   │   │   └── py_world_throughput.rs
│   │   ├── examples/
│   │   │   └── py_bundle_basic.rs
│   │   ├── fixtures/
│   │   │   └── py_events/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── convert.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── py_bundle.rs
│   │   │   ├── py_dataset.rs
│   │   │   ├── py_events.rs
│   │   │   ├── py_plan.rs
│   │   │   ├── py_run.rs
│   │   │   ├── py_simulation.rs
│   │   │   └── py_world.rs
│   │   ├── tests/
│   │   │   ├── convert_unit.rs
│   │   │   ├── py_dataset_integration.rs
│   │   │   ├── py_plan_property.rs
│   │   │   └── py_run_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-regime/
│   │   ├── benches/
│   │   │   ├── bocpd_throughput.rs
│   │   │   └── hmm_latency.rs
│   │   ├── examples/
│   │   │   └── transitions_basic.rs
│   │   ├── fixtures/
│   │   │   └── regime_laws/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── binary.rs
│   │   │   ├── bocpd.rs
│   │   │   ├── config.rs
│   │   │   ├── cost.rs
│   │   │   ├── error.rs
│   │   │   ├── hmm.rs
│   │   │   ├── lib.rs
│   │   │   ├── pelt.rs
│   │   │   ├── regime_laws.rs
│   │   │   ├── segmentation.rs
│   │   │   └── transitions.rs
│   │   ├── tests/
│   │   │   ├── binary_roundtrip.rs
│   │   │   ├── cost_integration.rs
│   │   │   ├── pelt_property.rs
│   │   │   └── segmentation_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-runner/
│   │   ├── benches/
│   │   │   ├── heartbeat_latency.rs
│   │   │   └── limits_throughput.rs
│   │   ├── examples/
│   │   │   └── checkpoint_basic.rs
│   │   ├── fixtures/
│   │   │   └── cancellation/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── cancellation.rs
│   │   │   ├── checkpoint.rs
│   │   │   ├── config.rs
│   │   │   ├── envelope.rs
│   │   │   ├── error.rs
│   │   │   ├── heartbeat.rs
│   │   │   ├── lib.rs
│   │   │   ├── limits.rs
│   │   │   ├── process.rs
│   │   │   ├── resources.rs
│   │   │   └── run.rs
│   │   ├── tests/
│   │   │   ├── envelope_property.rs
│   │   │   ├── process_integration.rs
│   │   │   ├── resources_roundtrip.rs
│   │   │   └── run_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-score/
│   │   ├── benches/
│   │   │   ├── dimensionality_throughput.rs
│   │   │   └── residual_latency.rs
│   │   ├── examples/
│   │   │   └── pareto_basic.rs
│   │   ├── fixtures/
│   │   │   └── rank/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── complexity.rs
│   │   │   ├── config.rs
│   │   │   ├── dimensionality.rs
│   │   │   ├── error.rs
│   │   │   ├── fit.rs
│   │   │   ├── lib.rs
│   │   │   ├── metric.rs
│   │   │   ├── pareto.rs
│   │   │   ├── rank.rs
│   │   │   ├── residual.rs
│   │   │   └── stability.rs
│   │   ├── tests/
│   │   │   ├── complexity_property.rs
│   │   │   ├── fit_integration.rs
│   │   │   ├── metric_unit.rs
│   │   │   └── stability_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-sim/
│   │   ├── benches/
│   │   │   ├── discrete_throughput.rs
│   │   │   └── ode_latency.rs
│   │   ├── examples/
│   │   │   └── sde_basic.rs
│   │   ├── fixtures/
│   │   │   └── hybrid/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── compile.rs
│   │   │   ├── config.rs
│   │   │   ├── context.rs
│   │   │   ├── discrete.rs
│   │   │   ├── error.rs
│   │   │   ├── hybrid.rs
│   │   │   ├── interpreter.rs
│   │   │   ├── lib.rs
│   │   │   ├── ode.rs
│   │   │   ├── sde.rs
│   │   │   └── state.rs
│   │   ├── tests/
│   │   │   ├── compile_property.rs
│   │   │   ├── context_integration.rs
│   │   │   ├── interpreter_roundtrip.rs
│   │   │   └── state_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-sparse/
│   │   ├── benches/
│   │   │   ├── group_latency.rs
│   │   │   └── lasso_throughput.rs
│   │   ├── examples/
│   │   │   └── constrained_basic.rs
│   │   ├── fixtures/
│   │   │   └── stability/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── constrained.rs
│   │   │   ├── error.rs
│   │   │   ├── group.rs
│   │   │   ├── lasso.rs
│   │   │   ├── lib.rs
│   │   │   ├── problem.rs
│   │   │   ├── sr3.rs
│   │   │   ├── stability.rs
│   │   │   ├── standardize.rs
│   │   │   └── stlsq.rs
│   │   ├── tests/
│   │   │   ├── problem_unit.rs
│   │   │   ├── sr3_roundtrip.rs
│   │   │   ├── standardize_integration.rs
│   │   │   └── stlsq_property.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-stats/
│   │   ├── benches/
│   │   │   ├── bootstrap_latency.rs
│   │   │   └── distributions_throughput.rs
│   │   ├── examples/
│   │   │   └── information_basic.rs
│   │   ├── fixtures/
│   │   │   └── sampling/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── bootstrap.rs
│   │   │   ├── config.rs
│   │   │   ├── covariance.rs
│   │   │   ├── distributions.rs
│   │   │   ├── error.rs
│   │   │   ├── information.rs
│   │   │   ├── lib.rs
│   │   │   ├── moments.rs
│   │   │   ├── quantile.rs
│   │   │   ├── robust.rs
│   │   │   └── sampling.rs
│   │   ├── tests/
│   │   │   ├── covariance_property.rs
│   │   │   ├── moments_unit.rs
│   │   │   ├── quantile_integration.rs
│   │   │   └── robust_roundtrip.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-store/
│   │   ├── benches/
│   │   │   ├── multipart_latency.rs
│   │   │   └── s3_throughput.rs
│   │   ├── examples/
│   │   │   └── cache_basic.rs
│   │   ├── fixtures/
│   │   │   └── gc/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── cache.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── gc.rs
│   │   │   ├── lib.rs
│   │   │   ├── local.rs
│   │   │   ├── memory.rs
│   │   │   ├── multipart.rs
│   │   │   ├── object.rs
│   │   │   ├── s3.rs
│   │   │   └── store.rs
│   │   ├── tests/
│   │   │   ├── local_property.rs
│   │   │   ├── memory_roundtrip.rs
│   │   │   ├── object_integration.rs
│   │   │   └── store_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-symbolic/
│   │   ├── benches/
│   │   │   ├── constants_latency.rs
│   │   │   └── crossover_throughput.rs
│   │   ├── examples/
│   │   │   └── simplify_basic.rs
│   │   ├── fixtures/
│   │   │   └── frontier/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── constants.rs
│   │   │   ├── crossover.rs
│   │   │   ├── error.rs
│   │   │   ├── frontier.rs
│   │   │   ├── grammar.rs
│   │   │   ├── initialize.rs
│   │   │   ├── lib.rs
│   │   │   ├── mutate.rs
│   │   │   ├── population.rs
│   │   │   └── simplify.rs
│   │   ├── tests/
│   │   │   ├── grammar_unit.rs
│   │   │   ├── initialize_property.rs
│   │   │   ├── mutate_roundtrip.rs
│   │   │   └── population_integration.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-uncertainty/
│   │   ├── benches/
│   │   │   ├── bootstrap_latency.rs
│   │   │   └── profile_throughput.rs
│   │   ├── examples/
│   │   │   └── structural_basic.rs
│   │   ├── fixtures/
│   │   │   └── propagate/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── bootstrap.rs
│   │   │   ├── config.rs
│   │   │   ├── covariance.rs
│   │   │   ├── error.rs
│   │   │   ├── interval.rs
│   │   │   ├── lib.rs
│   │   │   ├── profile.rs
│   │   │   ├── propagate.rs
│   │   │   ├── samples.rs
│   │   │   ├── source.rs
│   │   │   └── structural.rs
│   │   ├── tests/
│   │   │   ├── covariance_roundtrip.rs
│   │   │   ├── interval_integration.rs
│   │   │   ├── samples_property.rs
│   │   │   └── source_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-units/
│   │   ├── benches/
│   │   │   ├── convert_throughput.rs
│   │   │   └── infer_latency.rs
│   │   ├── examples/
│   │   │   └── check_basic.rs
│   │   ├── fixtures/
│   │   │   └── builtins/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── builtins.rs
│   │   │   ├── check.rs
│   │   │   ├── config.rs
│   │   │   ├── convert.rs
│   │   │   ├── dimension.rs
│   │   │   ├── error.rs
│   │   │   ├── infer.rs
│   │   │   ├── lib.rs
│   │   │   ├── parse.rs
│   │   │   ├── registry.rs
│   │   │   └── unit.rs
│   │   ├── tests/
│   │   │   ├── dimension_unit.rs
│   │   │   ├── parse_roundtrip.rs
│   │   │   ├── registry_property.rs
│   │   │   └── unit_integration.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── lawsynth-wasm/
│   │   ├── benches/
│   │   │   ├── events_latency.rs
│   │   │   └── trajectory_throughput.rs
│   │   ├── examples/
│   │   │   └── memory_basic.rs
│   │   ├── fixtures/
│   │   │   └── errors/
│   │   │       ├── edge_case.json
│   │   │       ├── minimal.json
│   │   │       └── typical.json
│   │   ├── src/
│   │   │   ├── bundle.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── errors.rs
│   │   │   ├── events.rs
│   │   │   ├── expression.rs
│   │   │   ├── lib.rs
│   │   │   ├── memory.rs
│   │   │   ├── simulate.rs
│   │   │   ├── trajectory.rs
│   │   │   └── world.rs
│   │   ├── tests/
│   │   │   ├── bundle_property.rs
│   │   │   ├── expression_integration.rs
│   │   │   ├── simulate_roundtrip.rs
│   │   │   └── world_unit.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   └── lawsynth-world/
│       ├── benches/
│       │   ├── graph_throughput.rs
│       │   └── regime_latency.rs
│       ├── examples/
│       │   └── event_basic.rs
│       ├── fixtures/
│       │   └── intervention/
│       │       ├── edge_case.json
│       │       ├── minimal.json
│       │       └── typical.json
│       ├── src/
│       │   ├── config.rs
│       │   ├── error.rs
│       │   ├── event.rs
│       │   ├── graph.rs
│       │   ├── intervention.rs
│       │   ├── law.rs
│       │   ├── lib.rs
│       │   ├── parameter.rs
│       │   ├── regime.rs
│       │   ├── variable.rs
│       │   └── world.rs
│       ├── tests/
│       │   ├── law_roundtrip.rs
│       │   ├── parameter_property.rs
│       │   ├── variable_integration.rs
│       │   └── world_unit.rs
│       ├── Cargo.toml
│       └── README.md
├── deploy/
│   ├── airgap/
│   │   └── bundle/
│   │       ├── checksums.sha256
│   │       ├── datasets.txt
│   │       ├── export.sh
│   │       ├── images.txt
│   │       ├── import.sh
│   │       ├── install.sh
│   │       ├── manifest.yaml
│   │       ├── packages.txt
│   │       ├── README.md
│   │       └── verify.sh
│   ├── compose/
│   │   ├── local/
│   │   │   ├── .env.example
│   │   │   ├── api.yaml
│   │   │   ├── compose.yaml
│   │   │   ├── healthcheck.sh
│   │   │   ├── minio.yaml
│   │   │   ├── nats.yaml
│   │   │   ├── postgres.yaml
│   │   │   ├── README.md
│   │   │   ├── volumes.yaml
│   │   │   └── worker.yaml
│   │   └── production/
│   │       ├── .env.example
│   │       ├── api.yaml
│   │       ├── backup.sh
│   │       ├── compose.yaml
│   │       ├── nats.yaml
│   │       ├── object-store.yaml
│   │       ├── postgres.yaml
│   │       ├── proxy.yaml
│   │       ├── README.md
│   │       └── worker.yaml
│   ├── docker/
│   │   └── images/
│   │       ├── .dockerignore
│   │       ├── api.Dockerfile
│   │       ├── artifact.Dockerfile
│   │       ├── build.hcl
│   │       ├── development.Dockerfile
│   │       ├── gateway.Dockerfile
│   │       ├── README.md
│   │       ├── scheduler.Dockerfile
│   │       ├── studio.Dockerfile
│   │       └── worker.Dockerfile
│   ├── helm/
│   │   └── lawsynth/
│   │       ├── Chart.yaml
│   │       ├── README.md
│   │       ├── templates-api.yaml
│   │       ├── templates-ingress.yaml
│   │       ├── templates-migration.yaml
│   │       ├── templates-rbac.yaml
│   │       ├── templates-storage.yaml
│   │       ├── templates-worker.yaml
│   │       ├── values.schema.json
│   │       └── values.yaml
│   ├── kubernetes/
│   │   ├── base/
│   │   │   ├── api.yaml
│   │   │   ├── artifact.yaml
│   │   │   ├── configmap.yaml
│   │   │   ├── gateway.yaml
│   │   │   ├── kustomization.yaml
│   │   │   ├── namespace.yaml
│   │   │   ├── rbac.yaml
│   │   │   ├── README.md
│   │   │   ├── scheduler.yaml
│   │   │   └── worker.yaml
│   │   ├── production/
│   │   │   ├── backup-cronjob.yaml
│   │   │   ├── config.yaml
│   │   │   ├── disruption-budget.yaml
│   │   │   ├── ingress.yaml
│   │   │   ├── kustomization.yaml
│   │   │   ├── network-policy.yaml
│   │   │   ├── README.md
│   │   │   ├── replicas.yaml
│   │   │   ├── resources.yaml
│   │   │   └── secrets.example.yaml
│   │   └── staging/
│   │       ├── alerts.yaml
│   │       ├── config.yaml
│   │       ├── ingress.yaml
│   │       ├── kustomization.yaml
│   │       ├── network-policy.yaml
│   │       ├── README.md
│   │       ├── replicas.yaml
│   │       ├── resources.yaml
│   │       ├── secrets.example.yaml
│   │       └── smoke-job.yaml
│   ├── observability/
│   │   └── reference/
│   │       ├── alerts.yaml
│   │       ├── api-dashboard.json
│   │       ├── grafana-datasources.yaml
│   │       ├── logging.yaml
│   │       ├── otel-collector.yaml
│   │       ├── prometheus.yaml
│   │       ├── README.md
│   │       ├── runbook.md
│   │       ├── science-dashboard.json
│   │       └── worker-dashboard.json
│   ├── systemd/
│   │   └── single-node/
│   │       ├── environment.example
│   │       ├── install.sh
│   │       ├── lawsynth-api.service
│   │       ├── lawsynth-artifact.service
│   │       ├── lawsynth-gateway.service
│   │       ├── lawsynth-scheduler.service
│   │       ├── lawsynth-worker.service
│   │       ├── lawsynth.target
│   │       ├── README.md
│   │       └── uninstall.sh
│   └── terraform/
│       ├── aws/
│       │   ├── cluster.tf
│       │   ├── database.tf
│       │   ├── example.tfvars
│       │   ├── main.tf
│       │   ├── network.tf
│       │   ├── outputs.tf
│       │   ├── README.md
│       │   ├── storage.tf
│       │   ├── variables.tf
│       │   └── versions.tf
│       ├── azure/
│       │   ├── cluster.tf
│       │   ├── database.tf
│       │   ├── example.tfvars
│       │   ├── main.tf
│       │   ├── network.tf
│       │   ├── outputs.tf
│       │   ├── README.md
│       │   ├── storage.tf
│       │   ├── variables.tf
│       │   └── versions.tf
│       └── gcp/
│           ├── cluster.tf
│           ├── database.tf
│           ├── example.tfvars
│           ├── main.tf
│           ├── network.tf
│           ├── outputs.tf
│           ├── README.md
│           ├── storage.tf
│           ├── variables.tf
│           └── versions.tf
├── docs/
│   ├── concepts/
│   │   ├── causality/
│   │   │   ├── assumptions.md
│   │   │   ├── counterfactuals.md
│   │   │   ├── equivalence.md
│   │   │   ├── graphs.md
│   │   │   ├── identification.md
│   │   │   ├── interventions.md
│   │   │   ├── lags.md
│   │   │   ├── limitations.md
│   │   │   ├── README.md
│   │   │   └── stability.md
│   │   ├── equations/
│   │   │   ├── algebraic.md
│   │   │   ├── alternatives.md
│   │   │   ├── constraints.md
│   │   │   ├── continuous.md
│   │   │   ├── discrete.md
│   │   │   ├── expressions.md
│   │   │   ├── operators.md
│   │   │   ├── README.md
│   │   │   ├── simplification.md
│   │   │   └── stochastic.md
│   │   ├── regimes/
│   │   │   ├── change-points.md
│   │   │   ├── events.md
│   │   │   ├── guards.md
│   │   │   ├── probabilities.md
│   │   │   ├── README.md
│   │   │   ├── shared-laws.md
│   │   │   ├── specific-laws.md
│   │   │   ├── states.md
│   │   │   ├── transitions.md
│   │   │   └── visualization.md
│   │   ├── uncertainty/
│   │   │   ├── bootstrap.md
│   │   │   ├── communication.md
│   │   │   ├── coverage.md
│   │   │   ├── ensembles.md
│   │   │   ├── parameters.md
│   │   │   ├── propagation.md
│   │   │   ├── README.md
│   │   │   ├── sources.md
│   │   │   ├── structure.md
│   │   │   └── trajectories.md
│   │   └── world-ir/
│   │       ├── events.md
│   │       ├── laws.md
│   │       ├── provenance.md
│   │       ├── README.md
│   │       ├── time.md
│   │       ├── types.md
│   │       ├── units.md
│   │       ├── variables.md
│   │       ├── versioning.md
│   │       └── world.md
│   ├── contributing/
│   │   ├── algorithms.md
│   │   ├── architecture.md
│   │   ├── benchmarks.md
│   │   ├── datasets.md
│   │   ├── development.md
│   │   ├── documentation.md
│   │   ├── governance.md
│   │   ├── operators.md
│   │   ├── README.md
│   │   └── releases.md
│   ├── getting-started/
│   │   ├── cli.md
│   │   ├── concepts.md
│   │   ├── examples.md
│   │   ├── first-world.md
│   │   ├── installation.md
│   │   ├── python.md
│   │   ├── quickstart.md
│   │   ├── README.md
│   │   ├── studio.md
│   │   └── troubleshooting.md
│   ├── guides/
│   │   ├── data/
│   │   │   ├── arrow.md
│   │   │   ├── csv.md
│   │   │   ├── irregular-time.md
│   │   │   ├── missing-data.md
│   │   │   ├── pandas.md
│   │   │   ├── parquet.md
│   │   │   ├── polars.md
│   │   │   ├── README.md
│   │   │   ├── units.md
│   │   │   └── xarray.md
│   │   ├── discovery/
│   │   │   ├── checkpoints.md
│   │   │   ├── constraints.md
│   │   │   ├── derivatives.md
│   │   │   ├── operators.md
│   │   │   ├── planning.md
│   │   │   ├── ranking.md
│   │   │   ├── README.md
│   │   │   ├── reproducibility.md
│   │   │   ├── sparse.md
│   │   │   └── symbolic.md
│   │   ├── simulation/
│   │   │   ├── comparison.md
│   │   │   ├── controls.md
│   │   │   ├── ensembles.md
│   │   │   ├── events.md
│   │   │   ├── export.md
│   │   │   ├── horizon.md
│   │   │   ├── initial-state.md
│   │   │   ├── interventions.md
│   │   │   ├── README.md
│   │   │   └── shocks.md
│   │   └── studio/
│   │       ├── data-lens.md
│   │       ├── discovery-canvas.md
│   │       ├── equation-explorer.md
│   │       ├── export.md
│   │       ├── README.md
│   │       ├── regime-timeline.md
│   │       ├── structure-map.md
│   │       ├── uncertainty-lens.md
│   │       ├── workspace.md
│   │       └── world-lab.md
│   ├── methods/
│   │   ├── causal/
│   │   │   ├── bootstrap.md
│   │   │   ├── effects.md
│   │   │   ├── granger.md
│   │   │   ├── independence.md
│   │   │   ├── lagged.md
│   │   │   ├── limits.md
│   │   │   ├── README.md
│   │   │   ├── score-based.md
│   │   │   ├── sensitivity.md
│   │   │   └── time-order.md
│   │   ├── differentiation/
│   │   │   ├── boundary.md
│   │   │   ├── finite.md
│   │   │   ├── irregular.md
│   │   │   ├── README.md
│   │   │   ├── savgol.md
│   │   │   ├── selection.md
│   │   │   ├── spectral.md
│   │   │   ├── spline.md
│   │   │   ├── tvreg.md
│   │   │   └── weak-form.md
│   │   ├── regime/
│   │   │   ├── binary.md
│   │   │   ├── bocpd.md
│   │   │   ├── guards.md
│   │   │   ├── hmm.md
│   │   │   ├── markov.md
│   │   │   ├── pelt.md
│   │   │   ├── README.md
│   │   │   ├── selection.md
│   │   │   ├── shared-structure.md
│   │   │   └── transitions.md
│   │   ├── simulation/
│   │   │   ├── diagnostics.md
│   │   │   ├── discrete.md
│   │   │   ├── ensembles.md
│   │   │   ├── events.md
│   │   │   ├── hybrid.md
│   │   │   ├── interventions.md
│   │   │   ├── noise.md
│   │   │   ├── ode.md
│   │   │   ├── README.md
│   │   │   └── sde.md
│   │   ├── sparse/
│   │   │   ├── constraints.md
│   │   │   ├── ensembles.md
│   │   │   ├── groups.md
│   │   │   ├── lasso.md
│   │   │   ├── libraries.md
│   │   │   ├── README.md
│   │   │   ├── selection.md
│   │   │   ├── sr3.md
│   │   │   ├── stability.md
│   │   │   └── stlsq.md
│   │   ├── symbolic/
│   │   │   ├── constants.md
│   │   │   ├── crossover.md
│   │   │   ├── egraphs.md
│   │   │   ├── frontiers.md
│   │   │   ├── grammar.md
│   │   │   ├── initialization.md
│   │   │   ├── mutation.md
│   │   │   ├── performance.md
│   │   │   ├── README.md
│   │   │   └── simplification.md
│   │   └── uncertainty/
│   │       ├── bootstrap.md
│   │       ├── calibration.md
│   │       ├── covariance.md
│   │       ├── ensembles.md
│   │       ├── profile.md
│   │       ├── README.md
│   │       ├── residual.md
│   │       ├── structural.md
│   │       ├── summaries.md
│   │       └── trajectory.md
│   ├── reference/
│   │   ├── cli/
│   │   │   ├── bundle.md
│   │   │   ├── discover.md
│   │   │   ├── inspect.md
│   │   │   ├── intervene.md
│   │   │   ├── plugin.md
│   │   │   ├── profile.md
│   │   │   ├── README.md
│   │   │   ├── serve.md
│   │   │   ├── simulate.md
│   │   │   └── studio.md
│   │   ├── python/
│   │   │   ├── bundle.md
│   │   │   ├── candidate.md
│   │   │   ├── dataset.md
│   │   │   ├── errors.md
│   │   │   ├── plan.md
│   │   │   ├── README.md
│   │   │   ├── run.md
│   │   │   ├── scenario.md
│   │   │   ├── trajectory.md
│   │   │   └── world.md
│   │   └── rust/
│   │       ├── bundle.md
│   │       ├── core.md
│   │       ├── data.md
│   │       ├── discovery.md
│   │       ├── errors.md
│   │       ├── expr.md
│   │       ├── plugins.md
│   │       ├── README.md
│   │       ├── simulation.md
│   │       └── world.md
│   ├── research/
│   │   ├── benchmarks.md
│   │   ├── citations.md
│   │   ├── collaboration.md
│   │   ├── failure-cases.md
│   │   ├── limitations.md
│   │   ├── methodology.md
│   │   ├── reading-list.md
│   │   ├── README.md
│   │   ├── reproducibility.md
│   │   └── roadmap.md
│   └── self-hosting/
│       ├── airgap.md
│       ├── architecture.md
│       ├── authentication.md
│       ├── backup.md
│       ├── compose.md
│       ├── database.md
│       ├── README.md
│       ├── storage.md
│       ├── upgrade.md
│       └── workers.md
├── examples/
│   ├── 00-quickstart/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 01-lorenz/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 02-lotka-volterra/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 03-damped-pendulum/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 04-sir-epidemic/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 05-regime-switching/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 06-delayed-feedback/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 07-stochastic-volatility/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 08-supply-demand/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 09-inventory-control/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 10-energy-load/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 11-customer-growth/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 12-macro-dynamics/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 13-market-microstructure/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 14-synthetic-control/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 15-user-constraints/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 16-custom-operator/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 17-custom-stage/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   ├── 18-bundle-interchange/
│   │   ├── expected/
│   │   │   ├── metrics.json
│   │   │   └── world.json
│   │   ├── config.toml
│   │   ├── dataset-card.md
│   │   ├── discover.py
│   │   ├── generate.py
│   │   ├── README.md
│   │   ├── simulate.py
│   │   └── test_example.py
│   └── 19-server-api/
│       ├── expected/
│       │   ├── metrics.json
│       │   └── world.json
│       ├── config.toml
│       ├── dataset-card.md
│       ├── discover.py
│       ├── generate.py
│       ├── README.md
│       ├── simulate.py
│       └── test_example.py
├── packages/
│   ├── api-client/
│   │   ├── examples/
│   │   │   ├── artifacts.example.ts
│   │   │   ├── downloads.example.ts
│   │   │   ├── events.example.ts
│   │   │   ├── generated.example.ts
│   │   │   └── uploads.example.ts
│   │   ├── fixtures/
│   │   │   ├── auth.json
│   │   │   ├── client.json
│   │   │   ├── errors.json
│   │   │   ├── pagination.json
│   │   │   └── transport.json
│   │   ├── src/
│   │   │   ├── artifacts.ts
│   │   │   ├── auth.ts
│   │   │   ├── client.ts
│   │   │   ├── datasets.ts
│   │   │   ├── downloads.ts
│   │   │   ├── errors.ts
│   │   │   ├── events.ts
│   │   │   ├── generated.ts
│   │   │   ├── index.ts
│   │   │   ├── pagination.ts
│   │   │   ├── projects.ts
│   │   │   ├── runs.ts
│   │   │   ├── simulations.ts
│   │   │   ├── transport.ts
│   │   │   ├── uploads.ts
│   │   │   └── worlds.ts
│   │   ├── tests/
│   │   │   ├── auth.test.ts
│   │   │   ├── client.test.ts
│   │   │   ├── datasets.test.ts
│   │   │   ├── errors.test.ts
│   │   │   ├── pagination.test.ts
│   │   │   ├── projects.test.ts
│   │   │   ├── runs.test.ts
│   │   │   ├── simulations.test.ts
│   │   │   ├── transport.test.ts
│   │   │   └── worlds.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── chart-core/
│   │   ├── examples/
│   │   │   ├── export.example.ts
│   │   │   ├── graph.example.ts
│   │   │   ├── heatmap.example.ts
│   │   │   ├── phase_portrait.example.ts
│   │   │   └── trajectory.example.ts
│   │   ├── fixtures/
│   │   │   ├── axis.json
│   │   │   ├── chart.json
│   │   │   ├── scales.json
│   │   │   ├── series.json
│   │   │   └── tooltip.json
│   │   ├── src/
│   │   │   ├── axis.ts
│   │   │   ├── brush.ts
│   │   │   ├── chart.ts
│   │   │   ├── downsample.ts
│   │   │   ├── export.ts
│   │   │   ├── graph.ts
│   │   │   ├── heatmap.ts
│   │   │   ├── index.ts
│   │   │   ├── legend.ts
│   │   │   ├── palette.ts
│   │   │   ├── phase_portrait.ts
│   │   │   ├── scales.ts
│   │   │   ├── series.ts
│   │   │   ├── tooltip.ts
│   │   │   ├── trajectory.ts
│   │   │   └── zoom.ts
│   │   ├── tests/
│   │   │   ├── axis.test.ts
│   │   │   ├── brush.test.ts
│   │   │   ├── chart.test.ts
│   │   │   ├── downsample.test.ts
│   │   │   ├── legend.test.ts
│   │   │   ├── palette.test.ts
│   │   │   ├── scales.test.ts
│   │   │   ├── series.test.ts
│   │   │   ├── tooltip.test.ts
│   │   │   └── zoom.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── design-system/
│   │   ├── examples/
│   │   │   ├── icons.example.ts
│   │   │   ├── progress.example.ts
│   │   │   ├── theme.example.ts
│   │   │   ├── toast.example.ts
│   │   │   └── tokens.example.ts
│   │   ├── fixtures/
│   │   │   ├── button.json
│   │   │   ├── dialog.json
│   │   │   ├── input.json
│   │   │   ├── popover.json
│   │   │   └── select.json
│   │   ├── src/
│   │   │   ├── badge.ts
│   │   │   ├── button.ts
│   │   │   ├── dialog.ts
│   │   │   ├── icons.ts
│   │   │   ├── index.ts
│   │   │   ├── input.ts
│   │   │   ├── panel.ts
│   │   │   ├── popover.ts
│   │   │   ├── progress.ts
│   │   │   ├── select.ts
│   │   │   ├── table.ts
│   │   │   ├── tabs.ts
│   │   │   ├── theme.ts
│   │   │   ├── toast.ts
│   │   │   ├── tokens.ts
│   │   │   └── tooltip.ts
│   │   ├── tests/
│   │   │   ├── badge.test.ts
│   │   │   ├── button.test.ts
│   │   │   ├── dialog.test.ts
│   │   │   ├── input.test.ts
│   │   │   ├── panel.test.ts
│   │   │   ├── popover.test.ts
│   │   │   ├── select.test.ts
│   │   │   ├── table.test.ts
│   │   │   ├── tabs.test.ts
│   │   │   └── tooltip.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── layout-engine/
│   │   ├── examples/
│   │   │   ├── animation.example.ts
│   │   │   ├── cache.example.ts
│   │   │   ├── constraints.example.ts
│   │   │   ├── viewport.example.ts
│   │   │   └── worker.example.ts
│   │   ├── fixtures/
│   │   │   ├── dag.json
│   │   │   ├── force.json
│   │   │   ├── graph_layout.json
│   │   │   ├── layout.json
│   │   │   └── timeline.json
│   │   ├── src/
│   │   │   ├── animation.ts
│   │   │   ├── cache.ts
│   │   │   ├── collision.ts
│   │   │   ├── constraints.ts
│   │   │   ├── dag.ts
│   │   │   ├── force.ts
│   │   │   ├── graph_layout.ts
│   │   │   ├── grid.ts
│   │   │   ├── index.ts
│   │   │   ├── labels.ts
│   │   │   ├── layout.ts
│   │   │   ├── measure.ts
│   │   │   ├── routing.ts
│   │   │   ├── timeline.ts
│   │   │   ├── viewport.ts
│   │   │   └── worker.ts
│   │   ├── tests/
│   │   │   ├── collision.test.ts
│   │   │   ├── dag.test.ts
│   │   │   ├── force.test.ts
│   │   │   ├── graph_layout.test.ts
│   │   │   ├── grid.test.ts
│   │   │   ├── labels.test.ts
│   │   │   ├── layout.test.ts
│   │   │   ├── measure.test.ts
│   │   │   ├── routing.test.ts
│   │   │   └── timeline.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── state-store/
│   │   ├── examples/
│   │   │   ├── errors.example.ts
│   │   │   ├── mutations.example.ts
│   │   │   ├── optimistic.example.ts
│   │   │   ├── queries.example.ts
│   │   │   └── undo.example.ts
│   │   ├── fixtures/
│   │   │   ├── panels.json
│   │   │   ├── preferences.json
│   │   │   ├── selection.json
│   │   │   ├── store.json
│   │   │   └── workspace.json
│   │   ├── src/
│   │   │   ├── commands.ts
│   │   │   ├── errors.ts
│   │   │   ├── events.ts
│   │   │   ├── history.ts
│   │   │   ├── index.ts
│   │   │   ├── mutations.ts
│   │   │   ├── optimistic.ts
│   │   │   ├── panels.ts
│   │   │   ├── persistence.ts
│   │   │   ├── preferences.ts
│   │   │   ├── queries.ts
│   │   │   ├── selection.ts
│   │   │   ├── store.ts
│   │   │   ├── sync.ts
│   │   │   ├── undo.ts
│   │   │   └── workspace.ts
│   │   ├── tests/
│   │   │   ├── commands.test.ts
│   │   │   ├── events.test.ts
│   │   │   ├── history.test.ts
│   │   │   ├── panels.test.ts
│   │   │   ├── persistence.test.ts
│   │   │   ├── preferences.test.ts
│   │   │   ├── selection.test.ts
│   │   │   ├── store.test.ts
│   │   │   ├── sync.test.ts
│   │   │   └── workspace.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── world-schema/
│   │   ├── examples/
│   │   │   ├── generated.example.ts
│   │   │   ├── hash.example.ts
│   │   │   ├── migrations.example.ts
│   │   │   ├── provenance.example.ts
│   │   │   └── uncertainty.example.ts
│   │   ├── fixtures/
│   │   │   ├── expression.json
│   │   │   ├── manifest.json
│   │   │   ├── types.json
│   │   │   ├── validators.json
│   │   │   └── world.json
│   │   ├── src/
│   │   │   ├── event.ts
│   │   │   ├── expression.ts
│   │   │   ├── generated.ts
│   │   │   ├── graph.ts
│   │   │   ├── hash.ts
│   │   │   ├── index.ts
│   │   │   ├── intervention.ts
│   │   │   ├── law.ts
│   │   │   ├── manifest.ts
│   │   │   ├── migrations.ts
│   │   │   ├── provenance.ts
│   │   │   ├── regime.ts
│   │   │   ├── types.ts
│   │   │   ├── uncertainty.ts
│   │   │   ├── validators.ts
│   │   │   └── world.ts
│   │   ├── tests/
│   │   │   ├── event.test.ts
│   │   │   ├── expression.test.ts
│   │   │   ├── graph.test.ts
│   │   │   ├── intervention.test.ts
│   │   │   ├── law.test.ts
│   │   │   ├── manifest.test.ts
│   │   │   ├── regime.test.ts
│   │   │   ├── types.test.ts
│   │   │   ├── validators.test.ts
│   │   │   └── world.test.ts
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   └── world-viewer/
│       ├── examples/
│       │   ├── embed.example.ts
│       │   ├── export.example.ts
│       │   ├── layout.example.ts
│       │   ├── theme.example.ts
│       │   └── worker.example.ts
│       ├── fixtures/
│       │   ├── bundle.json
│       │   ├── equation.json
│       │   ├── graph.json
│       │   ├── regime.json
│       │   └── viewer.json
│       ├── src/
│       │   ├── bundle.ts
│       │   ├── embed.ts
│       │   ├── equation.ts
│       │   ├── export.ts
│       │   ├── graph.ts
│       │   ├── index.ts
│       │   ├── layout.ts
│       │   ├── parameters.ts
│       │   ├── provenance.ts
│       │   ├── regime.ts
│       │   ├── theme.ts
│       │   ├── toolbar.ts
│       │   ├── trajectory.ts
│       │   ├── uncertainty.ts
│       │   ├── viewer.ts
│       │   └── worker.ts
│       ├── tests/
│       │   ├── bundle.test.ts
│       │   ├── equation.test.ts
│       │   ├── graph.test.ts
│       │   ├── parameters.test.ts
│       │   ├── provenance.test.ts
│       │   ├── regime.test.ts
│       │   ├── toolbar.test.ts
│       │   ├── trajectory.test.ts
│       │   ├── uncertainty.test.ts
│       │   └── viewer.test.ts
│       ├── package.json
│       ├── README.md
│       └── tsconfig.json
├── plugins/
│   ├── csv-variant-adapter/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.py
│   │   ├── src/
│   │   │   └── csv_variant_adapter/
│   │   │       └── plugin.py
│   │   ├── tests/
│   │   │   └── test_plugin.py
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── custom-operator-rust/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.rs
│   │   ├── src/
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   └── plugin_test.rs
│   │   ├── Cargo.toml
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   └── README.md
│   ├── custom-stage-python/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.py
│   │   ├── src/
│   │   │   └── custom_stage_python/
│   │   │       └── plugin.py
│   │   ├── tests/
│   │   │   └── test_plugin.py
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── duckdb-source/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.py
│   │   ├── src/
│   │   │   └── duckdb_source/
│   │   │       └── plugin.py
│   │   ├── tests/
│   │   │   └── test_plugin.py
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── external-simulator/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.rs
│   │   ├── src/
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   └── plugin_test.rs
│   │   ├── Cargo.toml
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   └── README.md
│   ├── finance-data-adapter/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.py
│   │   ├── src/
│   │   │   └── finance_data_adapter/
│   │   │       └── plugin.py
│   │   ├── tests/
│   │   │   └── test_plugin.py
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── neural-prior/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.py
│   │   ├── src/
│   │   │   └── neural_prior/
│   │   │       └── plugin.py
│   │   ├── tests/
│   │   │   └── test_plugin.py
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── report-exporter/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.py
│   │   ├── src/
│   │   │   └── report_exporter/
│   │   │       └── plugin.py
│   │   ├── tests/
│   │   │   └── test_plugin.py
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── scenario-exporter/
│   │   ├── docs/
│   │   │   └── usage.md
│   │   ├── examples/
│   │   │   └── basic.rs
│   │   ├── src/
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   └── plugin_test.rs
│   │   ├── Cargo.toml
│   │   ├── LICENSE
│   │   ├── plugin.toml
│   │   └── README.md
│   └── world-validator-wasi/
│       ├── docs/
│       │   └── usage.md
│       ├── examples/
│       │   └── basic.rs
│       ├── src/
│       │   └── lib.rs
│       ├── tests/
│       │   └── plugin_test.rs
│       ├── Cargo.toml
│       ├── LICENSE
│       ├── plugin.toml
│       └── README.md
├── python/
│   ├── lawsynth-bench/
│   │   ├── docs/
│   │   │   ├── aggregation.md
│   │   │   ├── equation_recovery.md
│   │   │   ├── errors_bench.md
│   │   │   ├── graph_recovery.md
│   │   │   ├── performance.md
│   │   │   ├── publish.md
│   │   │   ├── regime_recovery.md
│   │   │   ├── render.md
│   │   │   ├── trajectory_accuracy.md
│   │   │   └── uncertainty_coverage.md
│   │   ├── fixtures/
│   │   │   ├── baseline/
│   │   │   │   └── sample.json
│   │   │   ├── cli/
│   │   │   │   └── sample.json
│   │   │   ├── dataset/
│   │   │   │   └── sample.json
│   │   │   ├── environment/
│   │   │   │   └── sample.json
│   │   │   ├── leaderboard/
│   │   │   │   └── sample.json
│   │   │   ├── metrics/
│   │   │   │   └── sample.json
│   │   │   ├── problem/
│   │   │   │   └── sample.json
│   │   │   ├── registry/
│   │   │   │   └── sample.json
│   │   │   ├── report/
│   │   │   │   └── sample.json
│   │   │   └── runner/
│   │   │       └── sample.json
│   │   ├── src/
│   │   │   └── lawsynth_bench/
│   │   │       ├── __init__.py
│   │   │       ├── _version.py
│   │   │       ├── aggregation.py
│   │   │       ├── baseline.py
│   │   │       ├── cli.py
│   │   │       ├── config.py
│   │   │       ├── dataset.py
│   │   │       ├── environment.py
│   │   │       ├── equation_recovery.py
│   │   │       ├── errors.py
│   │   │       ├── errors_bench.py
│   │   │       ├── graph_recovery.py
│   │   │       ├── leaderboard.py
│   │   │       ├── metrics.py
│   │   │       ├── performance.py
│   │   │       ├── problem.py
│   │   │       ├── publish.py
│   │   │       ├── py.typed
│   │   │       ├── regime_recovery.py
│   │   │       ├── registry.py
│   │   │       ├── render.py
│   │   │       ├── report.py
│   │   │       ├── reproduce.py
│   │   │       ├── runner.py
│   │   │       ├── trajectory_accuracy.py
│   │   │       └── uncertainty_coverage.py
│   │   ├── tests/
│   │   │   ├── conftest.py
│   │   │   ├── test_aggregation.py
│   │   │   ├── test_baseline.py
│   │   │   ├── test_cli.py
│   │   │   ├── test_dataset.py
│   │   │   ├── test_environment.py
│   │   │   ├── test_equation_recovery.py
│   │   │   ├── test_errors_bench.py
│   │   │   ├── test_graph_recovery.py
│   │   │   ├── test_leaderboard.py
│   │   │   ├── test_metrics.py
│   │   │   ├── test_performance.py
│   │   │   ├── test_problem.py
│   │   │   ├── test_publish.py
│   │   │   ├── test_regime_recovery.py
│   │   │   ├── test_registry.py
│   │   │   ├── test_render.py
│   │   │   ├── test_report.py
│   │   │   ├── test_reproduce.py
│   │   │   ├── test_runner.py
│   │   │   ├── test_trajectory_accuracy.py
│   │   │   └── test_uncertainty_coverage.py
│   │   ├── LICENSE
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── lawsynth-connectors/
│   │   ├── docs/
│   │   │   ├── arrow.md
│   │   │   ├── credentials.md
│   │   │   ├── errors_connector.md
│   │   │   ├── fingerprints.md
│   │   │   ├── kafka.md
│   │   │   ├── pagination.md
│   │   │   ├── partitioning.md
│   │   │   ├── polars.md
│   │   │   ├── validation.md
│   │   │   └── xarray.md
│   │   ├── fixtures/
│   │   │   ├── base/
│   │   │   │   └── sample.json
│   │   │   ├── delta/
│   │   │   │   └── sample.json
│   │   │   ├── duckdb/
│   │   │   │   └── sample.json
│   │   │   ├── filesystem/
│   │   │   │   └── sample.json
│   │   │   ├── http/
│   │   │   │   └── sample.json
│   │   │   ├── iceberg/
│   │   │   │   └── sample.json
│   │   │   ├── postgres/
│   │   │   │   └── sample.json
│   │   │   ├── registry/
│   │   │   │   └── sample.json
│   │   │   ├── s3/
│   │   │   │   └── sample.json
│   │   │   └── sql/
│   │   │       └── sample.json
│   │   ├── src/
│   │   │   └── lawsynth_connectors/
│   │   │       ├── __init__.py
│   │   │       ├── _version.py
│   │   │       ├── arrow.py
│   │   │       ├── base.py
│   │   │       ├── config.py
│   │   │       ├── credentials.py
│   │   │       ├── delta.py
│   │   │       ├── duckdb.py
│   │   │       ├── errors.py
│   │   │       ├── errors_connector.py
│   │   │       ├── filesystem.py
│   │   │       ├── fingerprints.py
│   │   │       ├── http.py
│   │   │       ├── iceberg.py
│   │   │       ├── kafka.py
│   │   │       ├── pagination.py
│   │   │       ├── pandas.py
│   │   │       ├── partitioning.py
│   │   │       ├── polars.py
│   │   │       ├── postgres.py
│   │   │       ├── py.typed
│   │   │       ├── registry.py
│   │   │       ├── s3.py
│   │   │       ├── sql.py
│   │   │       ├── validation.py
│   │   │       └── xarray.py
│   │   ├── tests/
│   │   │   ├── conftest.py
│   │   │   ├── test_arrow.py
│   │   │   ├── test_base.py
│   │   │   ├── test_credentials.py
│   │   │   ├── test_delta.py
│   │   │   ├── test_duckdb.py
│   │   │   ├── test_errors_connector.py
│   │   │   ├── test_filesystem.py
│   │   │   ├── test_fingerprints.py
│   │   │   ├── test_http.py
│   │   │   ├── test_iceberg.py
│   │   │   ├── test_kafka.py
│   │   │   ├── test_pagination.py
│   │   │   ├── test_pandas.py
│   │   │   ├── test_partitioning.py
│   │   │   ├── test_polars.py
│   │   │   ├── test_postgres.py
│   │   │   ├── test_registry.py
│   │   │   ├── test_s3.py
│   │   │   ├── test_sql.py
│   │   │   ├── test_validation.py
│   │   │   └── test_xarray.py
│   │   ├── LICENSE
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── lawsynth-notebook/
│   │   ├── docs/
│   │   │   ├── comm.md
│   │   │   ├── compatibility.md
│   │   │   ├── controls.md
│   │   │   ├── errors_notebook.md
│   │   │   ├── export.md
│   │   │   ├── extension.md
│   │   │   ├── serialization.md
│   │   │   ├── server_proxy.md
│   │   │   ├── templates.md
│   │   │   └── themes.md
│   │   ├── fixtures/
│   │   │   ├── assets/
│   │   │   │   └── sample.json
│   │   │   ├── display/
│   │   │   │   └── sample.json
│   │   │   ├── equation_view/
│   │   │   │   └── sample.json
│   │   │   ├── events/
│   │   │   │   └── sample.json
│   │   │   ├── frontier_view/
│   │   │   │   └── sample.json
│   │   │   ├── graph_view/
│   │   │   │   └── sample.json
│   │   │   ├── regime_view/
│   │   │   │   └── sample.json
│   │   │   ├── trajectory_view/
│   │   │   │   └── sample.json
│   │   │   ├── uncertainty_view/
│   │   │   │   └── sample.json
│   │   │   └── widget/
│   │   │       └── sample.json
│   │   ├── src/
│   │   │   └── lawsynth_notebook/
│   │   │       ├── __init__.py
│   │   │       ├── _version.py
│   │   │       ├── assets.py
│   │   │       ├── comm.py
│   │   │       ├── compatibility.py
│   │   │       ├── config.py
│   │   │       ├── controls.py
│   │   │       ├── display.py
│   │   │       ├── equation_view.py
│   │   │       ├── errors.py
│   │   │       ├── errors_notebook.py
│   │   │       ├── events.py
│   │   │       ├── export.py
│   │   │       ├── extension.py
│   │   │       ├── frontier_view.py
│   │   │       ├── graph_view.py
│   │   │       ├── progress.py
│   │   │       ├── py.typed
│   │   │       ├── regime_view.py
│   │   │       ├── serialization.py
│   │   │       ├── server_proxy.py
│   │   │       ├── templates.py
│   │   │       ├── themes.py
│   │   │       ├── trajectory_view.py
│   │   │       ├── uncertainty_view.py
│   │   │       └── widget.py
│   │   ├── tests/
│   │   │   ├── conftest.py
│   │   │   ├── test_assets.py
│   │   │   ├── test_comm.py
│   │   │   ├── test_compatibility.py
│   │   │   ├── test_controls.py
│   │   │   ├── test_display.py
│   │   │   ├── test_equation_view.py
│   │   │   ├── test_errors_notebook.py
│   │   │   ├── test_events.py
│   │   │   ├── test_export.py
│   │   │   ├── test_extension.py
│   │   │   ├── test_frontier_view.py
│   │   │   ├── test_graph_view.py
│   │   │   ├── test_progress.py
│   │   │   ├── test_regime_view.py
│   │   │   ├── test_serialization.py
│   │   │   ├── test_server_proxy.py
│   │   │   ├── test_templates.py
│   │   │   ├── test_themes.py
│   │   │   ├── test_trajectory_view.py
│   │   │   ├── test_uncertainty_view.py
│   │   │   └── test_widget.py
│   │   ├── LICENSE
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── lawsynth-server/
│   │   ├── docs/
│   │   │   ├── artifacts.md
│   │   │   ├── database.md
│   │   │   ├── errors_api.md
│   │   │   ├── health.md
│   │   │   ├── middleware.md
│   │   │   ├── repositories.md
│   │   │   ├── simulations.md
│   │   │   ├── storage.md
│   │   │   ├── telemetry.md
│   │   │   └── worlds.md
│   │   ├── fixtures/
│   │   │   ├── app/
│   │   │   │   └── sample.json
│   │   │   ├── auth/
│   │   │   │   └── sample.json
│   │   │   ├── datasets/
│   │   │   │   └── sample.json
│   │   │   ├── dependencies/
│   │   │   │   └── sample.json
│   │   │   ├── events/
│   │   │   │   └── sample.json
│   │   │   ├── idempotency/
│   │   │   │   └── sample.json
│   │   │   ├── lifespan/
│   │   │   │   └── sample.json
│   │   │   ├── pagination/
│   │   │   │   └── sample.json
│   │   │   ├── projects/
│   │   │   │   └── sample.json
│   │   │   └── settings/
│   │   │       └── sample.json
│   │   ├── src/
│   │   │   └── lawsynth_server/
│   │   │       ├── __init__.py
│   │   │       ├── _version.py
│   │   │       ├── app.py
│   │   │       ├── artifacts.py
│   │   │       ├── auth.py
│   │   │       ├── config.py
│   │   │       ├── database.py
│   │   │       ├── datasets.py
│   │   │       ├── dependencies.py
│   │   │       ├── errors.py
│   │   │       ├── errors_api.py
│   │   │       ├── events.py
│   │   │       ├── health.py
│   │   │       ├── idempotency.py
│   │   │       ├── lifespan.py
│   │   │       ├── middleware.py
│   │   │       ├── pagination.py
│   │   │       ├── projects.py
│   │   │       ├── py.typed
│   │   │       ├── repositories.py
│   │   │       ├── runs.py
│   │   │       ├── settings.py
│   │   │       ├── simulations.py
│   │   │       ├── storage.py
│   │   │       ├── telemetry.py
│   │   │       └── worlds.py
│   │   ├── tests/
│   │   │   ├── conftest.py
│   │   │   ├── test_app.py
│   │   │   ├── test_artifacts.py
│   │   │   ├── test_auth.py
│   │   │   ├── test_database.py
│   │   │   ├── test_datasets.py
│   │   │   ├── test_dependencies.py
│   │   │   ├── test_errors_api.py
│   │   │   ├── test_events.py
│   │   │   ├── test_health.py
│   │   │   ├── test_idempotency.py
│   │   │   ├── test_lifespan.py
│   │   │   ├── test_middleware.py
│   │   │   ├── test_pagination.py
│   │   │   ├── test_projects.py
│   │   │   ├── test_repositories.py
│   │   │   ├── test_runs.py
│   │   │   ├── test_settings.py
│   │   │   ├── test_simulations.py
│   │   │   ├── test_storage.py
│   │   │   ├── test_telemetry.py
│   │   │   └── test_worlds.py
│   │   ├── LICENSE
│   │   ├── pyproject.toml
│   │   └── README.md
│   └── lawsynth/
│       ├── docs/
│       │   ├── bundle.md
│       │   ├── discover.md
│       │   ├── event.md
│       │   ├── inspect.md
│       │   ├── intervention.md
│       │   ├── scenario.md
│       │   ├── simulate.md
│       │   ├── trajectory.md
│       │   ├── uncertainty.md
│       │   └── world.md
│       ├── fixtures/
│       │   ├── assumptions/
│       │   │   └── sample.json
│       │   ├── candidate/
│       │   │   └── sample.json
│       │   ├── dataset/
│       │   │   └── sample.json
│       │   ├── equation/
│       │   │   └── sample.json
│       │   ├── frontier/
│       │   │   └── sample.json
│       │   ├── graph/
│       │   │   └── sample.json
│       │   ├── plan/
│       │   │   └── sample.json
│       │   ├── run/
│       │   │   └── sample.json
│       │   ├── units/
│       │   │   └── sample.json
│       │   └── variable/
│       │       └── sample.json
│       ├── src/
│       │   └── lawsynth/
│       │       ├── __init__.py
│       │       ├── _version.py
│       │       ├── assumptions.py
│       │       ├── bundle.py
│       │       ├── candidate.py
│       │       ├── config.py
│       │       ├── dataset.py
│       │       ├── discover.py
│       │       ├── equation.py
│       │       ├── errors.py
│       │       ├── event.py
│       │       ├── frontier.py
│       │       ├── graph.py
│       │       ├── inspect.py
│       │       ├── intervention.py
│       │       ├── plan.py
│       │       ├── py.typed
│       │       ├── regime.py
│       │       ├── run.py
│       │       ├── scenario.py
│       │       ├── simulate.py
│       │       ├── trajectory.py
│       │       ├── uncertainty.py
│       │       ├── units.py
│       │       ├── variable.py
│       │       └── world.py
│       ├── tests/
│       │   ├── conftest.py
│       │   ├── test_assumptions.py
│       │   ├── test_bundle.py
│       │   ├── test_candidate.py
│       │   ├── test_dataset.py
│       │   ├── test_discover.py
│       │   ├── test_equation.py
│       │   ├── test_event.py
│       │   ├── test_frontier.py
│       │   ├── test_graph.py
│       │   ├── test_inspect.py
│       │   ├── test_intervention.py
│       │   ├── test_plan.py
│       │   ├── test_regime.py
│       │   ├── test_run.py
│       │   ├── test_scenario.py
│       │   ├── test_simulate.py
│       │   ├── test_trajectory.py
│       │   ├── test_uncertainty.py
│       │   ├── test_units.py
│       │   ├── test_variable.py
│       │   └── test_world.py
│       ├── LICENSE
│       ├── pyproject.toml
│       └── README.md
├── services/
│   ├── api/
│   │   ├── config/
│   │   │   ├── development.yaml
│   │   │   ├── limits.yaml
│   │   │   ├── logging.yaml
│   │   │   ├── production.yaml
│   │   │   ├── staging.yaml
│   │   │   └── test.yaml
│   │   ├── docs/
│   │   │   ├── api.md
│   │   │   ├── architecture.md
│   │   │   ├── failures.md
│   │   │   ├── operations.md
│   │   │   └── security.md
│   │   ├── src/
│   │   │   └── lawsynth_api/
│   │   │       ├── app.py
│   │   │       ├── artifacts.py
│   │   │       ├── auth.py
│   │   │       ├── authorization.py
│   │   │       ├── database.py
│   │   │       ├── datasets.py
│   │   │       ├── downloads.py
│   │   │       ├── events.py
│   │   │       ├── lifespan.py
│   │   │       ├── main.py
│   │   │       ├── middleware.py
│   │   │       ├── projects.py
│   │   │       ├── repositories.py
│   │   │       ├── runs.py
│   │   │       ├── settings.py
│   │   │       ├── simulations.py
│   │   │       ├── storage.py
│   │   │       ├── telemetry.py
│   │   │       ├── uploads.py
│   │   │       └── worlds.py
│   │   ├── tests/
│   │   │   ├── app_test.py
│   │   │   ├── artifacts_test.py
│   │   │   ├── auth_test.py
│   │   │   ├── authorization_test.py
│   │   │   ├── datasets_test.py
│   │   │   ├── lifespan_test.py
│   │   │   ├── main_test.py
│   │   │   ├── projects_test.py
│   │   │   ├── runs_test.py
│   │   │   ├── settings_test.py
│   │   │   ├── simulations_test.py
│   │   │   └── worlds_test.py
│   │   ├── .env.example
│   │   ├── Dockerfile
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── artifact/
│   │   ├── config/
│   │   │   ├── development.yaml
│   │   │   ├── limits.yaml
│   │   │   ├── logging.yaml
│   │   │   ├── production.yaml
│   │   │   ├── staging.yaml
│   │   │   └── test.yaml
│   │   ├── docs/
│   │   │   ├── api.md
│   │   │   ├── architecture.md
│   │   │   ├── failures.md
│   │   │   ├── operations.md
│   │   │   └── security.md
│   │   ├── src/
│   │   │   ├── authorization.rs
│   │   │   ├── cache.rs
│   │   │   ├── checksum.rs
│   │   │   ├── config.rs
│   │   │   ├── database.rs
│   │   │   ├── download.rs
│   │   │   ├── errors.rs
│   │   │   ├── gc.rs
│   │   │   ├── health.rs
│   │   │   ├── limits.rs
│   │   │   ├── main.rs
│   │   │   ├── metadata.rs
│   │   │   ├── multipart.rs
│   │   │   ├── object.rs
│   │   │   ├── retention.rs
│   │   │   ├── routes.rs
│   │   │   ├── signature.rs
│   │   │   ├── storage.rs
│   │   │   ├── telemetry.rs
│   │   │   └── upload.rs
│   │   ├── tests/
│   │   │   ├── checksum_test.rs
│   │   │   ├── config_test.rs
│   │   │   ├── download_test.rs
│   │   │   ├── gc_test.rs
│   │   │   ├── main_test.rs
│   │   │   ├── metadata_test.rs
│   │   │   ├── multipart_test.rs
│   │   │   ├── object_test.rs
│   │   │   ├── retention_test.rs
│   │   │   ├── routes_test.rs
│   │   │   ├── signature_test.rs
│   │   │   └── upload_test.rs
│   │   ├── .env.example
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── README.md
│   ├── gateway/
│   │   ├── config/
│   │   │   ├── development.yaml
│   │   │   ├── limits.yaml
│   │   │   ├── logging.yaml
│   │   │   ├── production.yaml
│   │   │   ├── staging.yaml
│   │   │   └── test.yaml
│   │   ├── docs/
│   │   │   ├── api.md
│   │   │   ├── architecture.md
│   │   │   ├── failures.md
│   │   │   ├── operations.md
│   │   │   └── security.md
│   │   ├── src/
│   │   │   ├── auth.rs
│   │   │   ├── body_limits.rs
│   │   │   ├── config.rs
│   │   │   ├── cors.rs
│   │   │   ├── downloads.rs
│   │   │   ├── errors.rs
│   │   │   ├── events.rs
│   │   │   ├── headers.rs
│   │   │   ├── health.rs
│   │   │   ├── main.rs
│   │   │   ├── metrics.rs
│   │   │   ├── proxy.rs
│   │   │   ├── rate_limit.rs
│   │   │   ├── retry.rs
│   │   │   ├── routing.rs
│   │   │   ├── shutdown.rs
│   │   │   ├── timeouts.rs
│   │   │   ├── tls.rs
│   │   │   ├── tracing.rs
│   │   │   └── uploads.rs
│   │   ├── tests/
│   │   │   ├── auth_test.rs
│   │   │   ├── config_test.rs
│   │   │   ├── cors_test.rs
│   │   │   ├── headers_test.rs
│   │   │   ├── health_test.rs
│   │   │   ├── main_test.rs
│   │   │   ├── metrics_test.rs
│   │   │   ├── proxy_test.rs
│   │   │   ├── rate_limit_test.rs
│   │   │   ├── routing_test.rs
│   │   │   ├── tls_test.rs
│   │   │   └── tracing_test.rs
│   │   ├── .env.example
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── README.md
│   ├── scheduler/
│   │   ├── config/
│   │   │   ├── development.yaml
│   │   │   ├── limits.yaml
│   │   │   ├── logging.yaml
│   │   │   ├── production.yaml
│   │   │   ├── staging.yaml
│   │   │   └── test.yaml
│   │   ├── docs/
│   │   │   ├── api.md
│   │   │   ├── architecture.md
│   │   │   ├── failures.md
│   │   │   ├── operations.md
│   │   │   └── security.md
│   │   ├── src/
│   │   │   ├── backoff.rs
│   │   │   ├── config.rs
│   │   │   ├── database.rs
│   │   │   ├── errors.rs
│   │   │   ├── events.rs
│   │   │   ├── fairness.rs
│   │   │   ├── health.rs
│   │   │   ├── lease.rs
│   │   │   ├── main.rs
│   │   │   ├── metrics.rs
│   │   │   ├── nats.rs
│   │   │   ├── placement.rs
│   │   │   ├── policy.rs
│   │   │   ├── pool.rs
│   │   │   ├── priority.rs
│   │   │   ├── queue.rs
│   │   │   ├── quota.rs
│   │   │   ├── recovery.rs
│   │   │   ├── scheduler.rs
│   │   │   └── shutdown.rs
│   │   ├── tests/
│   │   │   ├── config_test.rs
│   │   │   ├── database_test.rs
│   │   │   ├── events_test.rs
│   │   │   ├── lease_test.rs
│   │   │   ├── main_test.rs
│   │   │   ├── metrics_test.rs
│   │   │   ├── policy_test.rs
│   │   │   ├── pool_test.rs
│   │   │   ├── queue_test.rs
│   │   │   ├── quota_test.rs
│   │   │   ├── recovery_test.rs
│   │   │   └── scheduler_test.rs
│   │   ├── .env.example
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── README.md
│   └── worker/
│       ├── config/
│       │   ├── development.yaml
│       │   ├── limits.yaml
│       │   ├── logging.yaml
│       │   ├── production.yaml
│       │   ├── staging.yaml
│       │   └── test.yaml
│       ├── docs/
│       │   ├── api.md
│       │   ├── architecture.md
│       │   ├── failures.md
│       │   ├── operations.md
│       │   └── security.md
│       ├── src/
│       │   ├── artifacts.rs
│       │   ├── checkpoint.rs
│       │   ├── cleanup.rs
│       │   ├── config.rs
│       │   ├── errors.rs
│       │   ├── events.rs
│       │   ├── execute.rs
│       │   ├── health.rs
│       │   ├── heartbeat.rs
│       │   ├── lease.rs
│       │   ├── limits.rs
│       │   ├── main.rs
│       │   ├── plugins.rs
│       │   ├── recovery.rs
│       │   ├── resources.rs
│       │   ├── sandbox.rs
│       │   ├── shutdown.rs
│       │   ├── telemetry.rs
│       │   ├── upload.rs
│       │   └── worker.rs
│       ├── tests/
│       │   ├── checkpoint_test.rs
│       │   ├── config_test.rs
│       │   ├── execute_test.rs
│       │   ├── heartbeat_test.rs
│       │   ├── lease_test.rs
│       │   ├── limits_test.rs
│       │   ├── main_test.rs
│       │   ├── plugins_test.rs
│       │   ├── resources_test.rs
│       │   ├── sandbox_test.rs
│       │   ├── upload_test.rs
│       │   └── worker_test.rs
│       ├── .env.example
│       ├── Cargo.toml
│       ├── Dockerfile
│       └── README.md
├── specs/
│   ├── bundle/
│   │   ├── changelog.md
│   │   ├── checksums.md
│   │   ├── compatibility.md
│   │   ├── content-types.md
│   │   ├── layout.md
│   │   ├── limits.md
│   │   ├── manifest.md
│   │   ├── migrations.md
│   │   ├── README.md
│   │   └── signatures.md
│   ├── causal-contract/
│   │   ├── assumptions.md
│   │   ├── changelog.md
│   │   ├── equivalence.md
│   │   ├── graphs.md
│   │   ├── identification.md
│   │   ├── interventions.md
│   │   ├── lags.md
│   │   ├── README.md
│   │   ├── sensitivity.md
│   │   └── stability.md
│   ├── dataset-contract/
│   │   ├── changelog.md
│   │   ├── fingerprints.md
│   │   ├── missingness.md
│   │   ├── partitions.md
│   │   ├── provenance.md
│   │   ├── README.md
│   │   ├── schema.md
│   │   ├── time-axis.md
│   │   ├── units.md
│   │   └── variables.md
│   ├── discovery-run/
│   │   ├── candidate-contract.md
│   │   ├── changelog.md
│   │   ├── checkpoint-contract.md
│   │   ├── determinism.md
│   │   ├── event-contract.md
│   │   ├── README.md
│   │   ├── resources.md
│   │   ├── run-spec.md
│   │   ├── score-contract.md
│   │   └── stage-contract.md
│   ├── event-protocol/
│   │   ├── artifact-events.md
│   │   ├── audit-events.md
│   │   ├── changelog.md
│   │   ├── envelope.md
│   │   ├── job-events.md
│   │   ├── ordering.md
│   │   ├── progress.md
│   │   ├── README.md
│   │   ├── replay.md
│   │   └── world-events.md
│   ├── expression-language/
│   │   ├── canonicalization.md
│   │   ├── changelog.md
│   │   ├── differentiation.md
│   │   ├── domains.md
│   │   ├── evaluation.md
│   │   ├── grammar.md
│   │   ├── operators.md
│   │   ├── README.md
│   │   ├── serialization.md
│   │   └── typing.md
│   ├── plugin-protocol/
│   │   ├── capabilities.md
│   │   ├── changelog.md
│   │   ├── compatibility.md
│   │   ├── errors.md
│   │   ├── lifecycle.md
│   │   ├── manifest.md
│   │   ├── permissions.md
│   │   ├── README.md
│   │   ├── resources.md
│   │   └── transport.md
│   ├── regime-contract/
│   │   ├── change-points.md
│   │   ├── changelog.md
│   │   ├── events.md
│   │   ├── guards.md
│   │   ├── README.md
│   │   ├── regime-laws.md
│   │   ├── segments.md
│   │   ├── shared-laws.md
│   │   ├── states.md
│   │   └── transitions.md
│   ├── reproducibility/
│   │   ├── algorithm-version.md
│   │   ├── artifacts.md
│   │   ├── changelog.md
│   │   ├── citations.md
│   │   ├── data-hash.md
│   │   ├── environment.md
│   │   ├── hardware-class.md
│   │   ├── plan-hash.md
│   │   ├── README.md
│   │   └── seed-plan.md
│   ├── security-model/
│   │   ├── archives.md
│   │   ├── changelog.md
│   │   ├── expressions.md
│   │   ├── plugins.md
│   │   ├── README.md
│   │   ├── resource-limits.md
│   │   ├── signatures.md
│   │   ├── telemetry.md
│   │   ├── tenancy.md
│   │   └── trust-levels.md
│   ├── service-api/
│   │   ├── authentication.md
│   │   ├── authorization.md
│   │   ├── changelog.md
│   │   ├── errors.md
│   │   ├── idempotency.md
│   │   ├── pagination.md
│   │   ├── README.md
│   │   ├── resources.md
│   │   ├── streaming.md
│   │   └── versioning.md
│   ├── simulation-contract/
│   │   ├── changelog.md
│   │   ├── diagnostics.md
│   │   ├── events.md
│   │   ├── initial-state.md
│   │   ├── interventions.md
│   │   ├── noise.md
│   │   ├── README.md
│   │   ├── solvers.md
│   │   ├── time-grid.md
│   │   └── trajectories.md
│   ├── uncertainty-contract/
│   │   ├── changelog.md
│   │   ├── intervals.md
│   │   ├── parameter.md
│   │   ├── propagation.md
│   │   ├── README.md
│   │   ├── samples.md
│   │   ├── sources.md
│   │   ├── structural.md
│   │   ├── summaries.md
│   │   └── trajectory.md
│   ├── world-ir/
│   │   ├── changelog.md
│   │   ├── events.md
│   │   ├── identifiers.md
│   │   ├── laws.md
│   │   ├── provenance.md
│   │   ├── README.md
│   │   ├── regimes.md
│   │   ├── types.md
│   │   ├── units.md
│   │   └── variables.md
│   ├── README.md
│   └── VERSION
├── tests/
│   ├── chaos/
│   │   ├── api-restart/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── duplicate-delivery/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── scheduler-restart/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── storage-timeout/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── worker-loss/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   ├── compatibility/
│   │   ├── forward-fields/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── plugin-protocol/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── v0-bundles/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── v1-migrations/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   ├── conformance/
│   │   ├── bad-expression/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── bad-hash/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── bad-schema/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── bad-units/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── continuous-world/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── discrete-world/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── hybrid-world/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── minimal-world/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── regime-world/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── signed-bundle/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── stochastic-world/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── unsafe-archive/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   ├── cross-language/
│   │   ├── bundle-roundtrip/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── python-rust/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── rust-python/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── schema-roundtrip/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── typescript-rust/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   ├── end-to-end/
│   │   ├── cancellation/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── cli-discover/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── cli-simulate/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── export/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── import/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── local-library/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── local-studio/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── resume/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── server-run/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   ├── performance/
│   │   ├── bundle-open/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── cancellation-latency/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── event-latency/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── expression-throughput/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── import-time/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── memory-budget/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── ode-simulation/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── parquet-load/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── profile-million/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── studio-paint/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   ├── scientific/
│   │   ├── adversarial-noise/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── irregular-sampling/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── lorenz-recovery/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── lotka-volterra-recovery/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── missing-data/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── pendulum-recovery/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── regime-recovery/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── sir-recovery/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   ├── uncertainty-coverage/
│   │   │   ├── case.toml
│   │   │   ├── expected.json
│   │   │   ├── input.json
│   │   │   ├── README.md
│   │   │   └── run.py
│   │   └── unit-consistency/
│   │       ├── case.toml
│   │       ├── expected.json
│   │       ├── input.json
│   │       ├── README.md
│   │       └── run.py
│   └── security/
│       ├── archive-traversal/
│       │   ├── case.toml
│       │   ├── expected.json
│       │   ├── input.json
│       │   ├── README.md
│       │   └── run.py
│       ├── authorization/
│       │   ├── case.toml
│       │   ├── expected.json
│       │   ├── input.json
│       │   ├── README.md
│       │   └── run.py
│       ├── decompression-limits/
│       │   ├── case.toml
│       │   ├── expected.json
│       │   ├── input.json
│       │   ├── README.md
│       │   └── run.py
│       ├── expression-limits/
│       │   ├── case.toml
│       │   ├── expected.json
│       │   ├── input.json
│       │   ├── README.md
│       │   └── run.py
│       ├── plugin-permissions/
│       │   ├── case.toml
│       │   ├── expected.json
│       │   ├── input.json
│       │   ├── README.md
│       │   └── run.py
│       └── tenant-isolation/
│           ├── case.toml
│           ├── expected.json
│           ├── input.json
│           ├── README.md
│           └── run.py
├── tools/
│   ├── api-doc-gen/
│   │   ├── src/
│   │   │   ├── main.py
│   │   │   ├── openapi.py
│   │   │   ├── python.py
│   │   │   ├── rust.py
│   │   │   └── typescript.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── benchmark-site/
│   │   ├── src/
│   │   │   ├── charts.py
│   │   │   ├── compare.py
│   │   │   ├── main.py
│   │   │   ├── publish.py
│   │   │   └── results.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── binding-gen/
│   │   ├── src/
│   │   │   ├── main.py
│   │   │   ├── protobuf.py
│   │   │   ├── python.py
│   │   │   ├── rust.py
│   │   │   └── typescript.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── bundle-inspector/
│   │   ├── src/
│   │   │   ├── archive.py
│   │   │   ├── checksum.py
│   │   │   ├── main.py
│   │   │   ├── manifest.py
│   │   │   └── report.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── conformance-runner/
│   │   ├── src/
│   │   │   ├── compare.py
│   │   │   ├── discover.py
│   │   │   ├── execute.py
│   │   │   ├── main.py
│   │   │   └── report.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── dataset-registry/
│   │   ├── src/
│   │   │   ├── card.py
│   │   │   ├── download.py
│   │   │   ├── main.py
│   │   │   ├── manifest.py
│   │   │   └── verify.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── fixture-builder/
│   │   ├── src/
│   │   │   ├── canonicalize.py
│   │   │   ├── checksum.py
│   │   │   ├── generate.py
│   │   │   ├── main.py
│   │   │   └── package.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── license-check/
│   │   ├── src/
│   │   │   ├── main.py
│   │   │   ├── notice.py
│   │   │   ├── policy.py
│   │   │   ├── report.py
│   │   │   └── scan.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   ├── release-notes/
│   │   ├── src/
│   │   │   ├── changes.py
│   │   │   ├── commits.py
│   │   │   ├── main.py
│   │   │   ├── publish.py
│   │   │   └── render.py
│   │   ├── tests/
│   │   │   └── test_cli.py
│   │   ├── pyproject.toml
│   │   └── README.md
│   └── schema-gen/
│       ├── src/
│       │   ├── jsonschema.py
│       │   ├── main.py
│       │   ├── python.py
│       │   ├── schema.py
│       │   └── typescript.py
│       ├── tests/
│       │   └── test_cli.py
│       ├── pyproject.toml
│       └── README.md
├── .editorconfig
├── .env.example
├── .gitattributes
├── .gitignore
├── .node-version
├── .pre-commit-config.yaml
├── .python-version
├── ARCHITECTURE.md
├── AUTHORS.md
├── Cargo.lock
├── Cargo.toml
├── CHANGELOG.md
├── CITATION.cff
├── clippy.toml
├── CODE_OF_CONDUCT.md
├── codecov.yml
├── CONTRIBUTING.md
├── deny.toml
├── docker-compose.yml
├── Dockerfile
├── GOVERNANCE.md
├── justfile
├── lefthook.yml
├── LICENSE
├── MAINTAINERS.md
├── Makefile
├── NOTICE
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── pyproject.toml
├── README.md
├── README.tr.md
├── release-plz.toml
├── ROADMAP.md
├── rust-toolchain.toml
├── rustfmt.toml
├── SECURITY.md
├── SUPPORT.md
├── typos.toml
└── uv.lock
```

## 5. Every file with responsibility

| # | Path | Subsystem | Component | Phase | Owner | Responsibility |
|---:|---|---|---|---|---|---|
| 1 | `README.md` | root | repository | P0 | Maintainers | Documents the purpose, boundaries, and usage of repository. |
| 2 | `README.tr.md` | root | repository | P0 | Maintainers | Documents README.tr for repository. |
| 3 | `LICENSE` | root | repository | P0 | Maintainers | Declares legal terms and notices for repository. |
| 4 | `NOTICE` | root | repository | P0 | Maintainers | Declares legal terms and notices for repository. |
| 5 | `CITATION.cff` | root | repository | P0 | Maintainers | Provides CITATION for repository. |
| 6 | `CODE_OF_CONDUCT.md` | root | repository | P0 | Maintainers | Documents CODE OF CONDUCT for repository. |
| 7 | `CONTRIBUTING.md` | root | repository | P0 | Maintainers | Documents CONTRIBUTING for repository. |
| 8 | `GOVERNANCE.md` | root | repository | P0 | Maintainers | Documents GOVERNANCE for repository. |
| 9 | `MAINTAINERS.md` | root | repository | P0 | Maintainers | Documents MAINTAINERS for repository. |
| 10 | `SECURITY.md` | root | repository | P0 | Maintainers | Documents SECURITY for repository. |
| 11 | `SUPPORT.md` | root | repository | P0 | Maintainers | Documents SUPPORT for repository. |
| 12 | `ROADMAP.md` | root | repository | P0 | Maintainers | Documents ROADMAP for repository. |
| 13 | `CHANGELOG.md` | root | repository | P0 | Maintainers | Documents CHANGELOG for repository. |
| 14 | `ARCHITECTURE.md` | root | repository | P0 | Maintainers | Documents ARCHITECTURE for repository. |
| 15 | `AUTHORS.md` | root | repository | P0 | Maintainers | Documents AUTHORS for repository. |
| 16 | `Cargo.toml` | root | repository | P0 | Maintainers | Declares the build, dependencies, and package metadata for repository. |
| 17 | `Cargo.lock` | root | repository | P0 | Maintainers | Locks reproducible dependencies for repository. |
| 18 | `pyproject.toml` | root | repository | P0 | Maintainers | Declares the build, dependencies, and package metadata for repository. |
| 19 | `uv.lock` | root | repository | P0 | Maintainers | Locks reproducible dependencies for repository. |
| 20 | `package.json` | root | repository | P0 | Maintainers | Declares the build, dependencies, and package metadata for repository. |
| 21 | `pnpm-lock.yaml` | root | repository | P0 | Maintainers | Locks reproducible dependencies for repository. |
| 22 | `pnpm-workspace.yaml` | root | repository | P0 | Maintainers | Configures or declares pnpm workspace for repository. |
| 23 | `rust-toolchain.toml` | root | repository | P0 | Maintainers | Configures or declares rust toolchain for repository. |
| 24 | `rustfmt.toml` | root | repository | P0 | Maintainers | Configures or declares rustfmt for repository. |
| 25 | `clippy.toml` | root | repository | P0 | Maintainers | Configures or declares clippy for repository. |
| 26 | `deny.toml` | root | repository | P0 | Maintainers | Configures or declares deny for repository. |
| 27 | `typos.toml` | root | repository | P0 | Maintainers | Configures or declares typos for repository. |
| 28 | `lefthook.yml` | root | repository | P0 | Maintainers | Configures or declares lefthook for repository. |
| 29 | `justfile` | root | repository | P0 | Maintainers | Provides justfile for repository. |
| 30 | `Makefile` | root | repository | P0 | Maintainers | Provides Makefile for repository. |
| 31 | `Dockerfile` | root | repository | P0 | Maintainers | Provides Dockerfile for repository. |
| 32 | `docker-compose.yml` | root | repository | P0 | Maintainers | Configures or declares docker compose for repository. |
| 33 | `codecov.yml` | root | repository | P0 | Maintainers | Configures or declares codecov for repository. |
| 34 | `release-plz.toml` | root | repository | P0 | Maintainers | Configures or declares release plz for repository. |
| 35 | `.editorconfig` | root | repository | P0 | Maintainers | Provides  for repository. |
| 36 | `.env.example` | root | repository | P0 | Maintainers | Provides .env for repository. |
| 37 | `.gitattributes` | root | repository | P0 | Maintainers | Provides  for repository. |
| 38 | `.gitignore` | root | repository | P0 | Maintainers | Provides  for repository. |
| 39 | `.pre-commit-config.yaml` | root | repository | P0 | Maintainers | Configures or declares .pre commit config for repository. |
| 40 | `.python-version` | root | repository | P0 | Maintainers | Provides  for repository. |
| 41 | `.node-version` | root | repository | P0 | Maintainers | Provides  for repository. |
| 42 | `.cargo/config.toml` | root | repository | P0 | Maintainers | Configures or declares config for repository. |
| 43 | `.config/cargo-nextest.toml` | root | repository | P0 | Maintainers | Configures or declares cargo nextest for repository. |
| 44 | `.config/markdownlint.json` | root | repository | P0 | Maintainers | Configures or declares markdownlint for repository. |
| 45 | `.config/pyrightconfig.json` | root | repository | P0 | Maintainers | Configures or declares pyrightconfig for repository. |
| 46 | `.config/pytest.ini` | root | repository | P0 | Maintainers | Provides pytest for repository. |
| 47 | `.config/ruff.toml` | root | repository | P0 | Maintainers | Configures or declares ruff for repository. |
| 48 | `.config/taplo.toml` | root | repository | P0 | Maintainers | Configures or declares taplo for repository. |
| 49 | `.config/vitest.workspace.ts` | root | repository | P0 | Maintainers | Implements vitest.workspace for repository. |
| 50 | `.github/CODEOWNERS` | root | repository | P0 | Maintainers | Provides CODEOWNERS for repository. |
| 51 | `.github/FUNDING.yml` | root | repository | P0 | Maintainers | Configures or declares FUNDING for repository. |
| 52 | `.github/dependabot.yml` | root | repository | P0 | Maintainers | Configures or declares dependabot for repository. |
| 53 | `.github/labeler.yml` | root | repository | P0 | Maintainers | Configures or declares labeler for repository. |
| 54 | `.github/pull_request_template.md` | root | repository | P0 | Maintainers | Documents pull request template for repository. |
| 55 | `.github/ISSUE_TEMPLATE/bug.yml` | root | repository | P0 | Maintainers | Configures or declares bug for repository. |
| 56 | `.github/ISSUE_TEMPLATE/feature.yml` | root | repository | P0 | Maintainers | Configures or declares feature for repository. |
| 57 | `.github/ISSUE_TEMPLATE/research-method.yml` | root | repository | P0 | Maintainers | Configures or declares research method for repository. |
| 58 | `.github/workflows/pr.yml` | root | repository | P0 | Maintainers | Configures or declares pr for repository. |
| 59 | `.github/workflows/nightly-science.yml` | root | repository | P0 | Maintainers | Configures or declares nightly science for repository. |
| 60 | `.github/workflows/release.yml` | root | repository | P0 | Maintainers | Configures or declares release for repository. |
| 61 | `.vscode/extensions.json` | root | repository | P0 | Maintainers | Configures or declares extensions for repository. |
| 62 | `.vscode/settings.json` | root | repository | P0 | Maintainers | Configures or declares settings for repository. |
| 63 | `specs/README.md` | specs | specifications | P0 | Architecture | Indexes all normative specifications. |
| 64 | `specs/VERSION` | specs | specifications | P0 | Architecture | Pins the active specification set version. |
| 65 | `specs/world-ir/README.md` | specs | world-ir | P0 | Architecture | Introduces the world-ir specification family. |
| 66 | `specs/world-ir/changelog.md` | specs | world-ir | P0 | Architecture | Tracks normative changes to world-ir. |
| 67 | `specs/world-ir/identifiers.md` | specs | world-ir | P0 | Architecture | Defines identifiers semantics for world-ir. |
| 68 | `specs/world-ir/types.md` | specs | world-ir | P0 | Architecture | Defines types semantics for world-ir. |
| 69 | `specs/world-ir/units.md` | specs | world-ir | P0 | Architecture | Defines units semantics for world-ir. |
| 70 | `specs/world-ir/variables.md` | specs | world-ir | P0 | Architecture | Defines variables semantics for world-ir. |
| 71 | `specs/world-ir/laws.md` | specs | world-ir | P0 | Architecture | Defines laws semantics for world-ir. |
| 72 | `specs/world-ir/regimes.md` | specs | world-ir | P0 | Architecture | Defines regimes semantics for world-ir. |
| 73 | `specs/world-ir/events.md` | specs | world-ir | P0 | Architecture | Defines events semantics for world-ir. |
| 74 | `specs/world-ir/provenance.md` | specs | world-ir | P0 | Architecture | Defines provenance semantics for world-ir. |
| 75 | `specs/bundle/README.md` | specs | bundle | P0 | Architecture | Introduces the bundle specification family. |
| 76 | `specs/bundle/changelog.md` | specs | bundle | P0 | Architecture | Tracks normative changes to bundle. |
| 77 | `specs/bundle/layout.md` | specs | bundle | P0 | Architecture | Defines layout semantics for bundle. |
| 78 | `specs/bundle/manifest.md` | specs | bundle | P0 | Architecture | Defines manifest semantics for bundle. |
| 79 | `specs/bundle/content-types.md` | specs | bundle | P0 | Architecture | Defines content types semantics for bundle. |
| 80 | `specs/bundle/checksums.md` | specs | bundle | P0 | Architecture | Defines checksums semantics for bundle. |
| 81 | `specs/bundle/signatures.md` | specs | bundle | P0 | Architecture | Defines signatures semantics for bundle. |
| 82 | `specs/bundle/migrations.md` | specs | bundle | P0 | Architecture | Defines migrations semantics for bundle. |
| 83 | `specs/bundle/limits.md` | specs | bundle | P0 | Architecture | Defines limits semantics for bundle. |
| 84 | `specs/bundle/compatibility.md` | specs | bundle | P0 | Architecture | Defines compatibility semantics for bundle. |
| 85 | `specs/expression-language/README.md` | specs | expression-language | P0 | Architecture | Introduces the expression-language specification family. |
| 86 | `specs/expression-language/changelog.md` | specs | expression-language | P0 | Architecture | Tracks normative changes to expression-language. |
| 87 | `specs/expression-language/grammar.md` | specs | expression-language | P0 | Architecture | Defines grammar semantics for expression-language. |
| 88 | `specs/expression-language/operators.md` | specs | expression-language | P0 | Architecture | Defines operators semantics for expression-language. |
| 89 | `specs/expression-language/typing.md` | specs | expression-language | P0 | Architecture | Defines typing semantics for expression-language. |
| 90 | `specs/expression-language/domains.md` | specs | expression-language | P0 | Architecture | Defines domains semantics for expression-language. |
| 91 | `specs/expression-language/canonicalization.md` | specs | expression-language | P0 | Architecture | Defines canonicalization semantics for expression-language. |
| 92 | `specs/expression-language/evaluation.md` | specs | expression-language | P0 | Architecture | Defines evaluation semantics for expression-language. |
| 93 | `specs/expression-language/differentiation.md` | specs | expression-language | P0 | Architecture | Defines differentiation semantics for expression-language. |
| 94 | `specs/expression-language/serialization.md` | specs | expression-language | P0 | Architecture | Defines serialization semantics for expression-language. |
| 95 | `specs/discovery-run/README.md` | specs | discovery-run | P0 | Architecture | Introduces the discovery-run specification family. |
| 96 | `specs/discovery-run/changelog.md` | specs | discovery-run | P0 | Architecture | Tracks normative changes to discovery-run. |
| 97 | `specs/discovery-run/run-spec.md` | specs | discovery-run | P0 | Architecture | Defines run spec semantics for discovery-run. |
| 98 | `specs/discovery-run/stage-contract.md` | specs | discovery-run | P0 | Architecture | Defines stage contract semantics for discovery-run. |
| 99 | `specs/discovery-run/candidate-contract.md` | specs | discovery-run | P0 | Architecture | Defines candidate contract semantics for discovery-run. |
| 100 | `specs/discovery-run/score-contract.md` | specs | discovery-run | P0 | Architecture | Defines score contract semantics for discovery-run. |
| 101 | `specs/discovery-run/checkpoint-contract.md` | specs | discovery-run | P0 | Architecture | Defines checkpoint contract semantics for discovery-run. |
| 102 | `specs/discovery-run/event-contract.md` | specs | discovery-run | P0 | Architecture | Defines event contract semantics for discovery-run. |
| 103 | `specs/discovery-run/determinism.md` | specs | discovery-run | P0 | Architecture | Defines determinism semantics for discovery-run. |
| 104 | `specs/discovery-run/resources.md` | specs | discovery-run | P0 | Architecture | Defines resources semantics for discovery-run. |
| 105 | `specs/dataset-contract/README.md` | specs | dataset-contract | P0 | Architecture | Introduces the dataset-contract specification family. |
| 106 | `specs/dataset-contract/changelog.md` | specs | dataset-contract | P0 | Architecture | Tracks normative changes to dataset-contract. |
| 107 | `specs/dataset-contract/schema.md` | specs | dataset-contract | P0 | Architecture | Defines schema semantics for dataset-contract. |
| 108 | `specs/dataset-contract/variables.md` | specs | dataset-contract | P0 | Architecture | Defines variables semantics for dataset-contract. |
| 109 | `specs/dataset-contract/time-axis.md` | specs | dataset-contract | P0 | Architecture | Defines time axis semantics for dataset-contract. |
| 110 | `specs/dataset-contract/missingness.md` | specs | dataset-contract | P0 | Architecture | Defines missingness semantics for dataset-contract. |
| 111 | `specs/dataset-contract/units.md` | specs | dataset-contract | P0 | Architecture | Defines units semantics for dataset-contract. |
| 112 | `specs/dataset-contract/fingerprints.md` | specs | dataset-contract | P0 | Architecture | Defines fingerprints semantics for dataset-contract. |
| 113 | `specs/dataset-contract/partitions.md` | specs | dataset-contract | P0 | Architecture | Defines partitions semantics for dataset-contract. |
| 114 | `specs/dataset-contract/provenance.md` | specs | dataset-contract | P0 | Architecture | Defines provenance semantics for dataset-contract. |
| 115 | `specs/uncertainty-contract/README.md` | specs | uncertainty-contract | P0 | Architecture | Introduces the uncertainty-contract specification family. |
| 116 | `specs/uncertainty-contract/changelog.md` | specs | uncertainty-contract | P0 | Architecture | Tracks normative changes to uncertainty-contract. |
| 117 | `specs/uncertainty-contract/sources.md` | specs | uncertainty-contract | P0 | Architecture | Defines sources semantics for uncertainty-contract. |
| 118 | `specs/uncertainty-contract/intervals.md` | specs | uncertainty-contract | P0 | Architecture | Defines intervals semantics for uncertainty-contract. |
| 119 | `specs/uncertainty-contract/samples.md` | specs | uncertainty-contract | P0 | Architecture | Defines samples semantics for uncertainty-contract. |
| 120 | `specs/uncertainty-contract/structural.md` | specs | uncertainty-contract | P0 | Architecture | Defines structural semantics for uncertainty-contract. |
| 121 | `specs/uncertainty-contract/parameter.md` | specs | uncertainty-contract | P0 | Architecture | Defines parameter semantics for uncertainty-contract. |
| 122 | `specs/uncertainty-contract/trajectory.md` | specs | uncertainty-contract | P0 | Architecture | Defines trajectory semantics for uncertainty-contract. |
| 123 | `specs/uncertainty-contract/propagation.md` | specs | uncertainty-contract | P0 | Architecture | Defines propagation semantics for uncertainty-contract. |
| 124 | `specs/uncertainty-contract/summaries.md` | specs | uncertainty-contract | P0 | Architecture | Defines summaries semantics for uncertainty-contract. |
| 125 | `specs/causal-contract/README.md` | specs | causal-contract | P0 | Architecture | Introduces the causal-contract specification family. |
| 126 | `specs/causal-contract/changelog.md` | specs | causal-contract | P0 | Architecture | Tracks normative changes to causal-contract. |
| 127 | `specs/causal-contract/assumptions.md` | specs | causal-contract | P0 | Architecture | Defines assumptions semantics for causal-contract. |
| 128 | `specs/causal-contract/graphs.md` | specs | causal-contract | P0 | Architecture | Defines graphs semantics for causal-contract. |
| 129 | `specs/causal-contract/lags.md` | specs | causal-contract | P0 | Architecture | Defines lags semantics for causal-contract. |
| 130 | `specs/causal-contract/interventions.md` | specs | causal-contract | P0 | Architecture | Defines interventions semantics for causal-contract. |
| 131 | `specs/causal-contract/identification.md` | specs | causal-contract | P0 | Architecture | Defines identification semantics for causal-contract. |
| 132 | `specs/causal-contract/equivalence.md` | specs | causal-contract | P0 | Architecture | Defines equivalence semantics for causal-contract. |
| 133 | `specs/causal-contract/stability.md` | specs | causal-contract | P0 | Architecture | Defines stability semantics for causal-contract. |
| 134 | `specs/causal-contract/sensitivity.md` | specs | causal-contract | P0 | Architecture | Defines sensitivity semantics for causal-contract. |
| 135 | `specs/regime-contract/README.md` | specs | regime-contract | P0 | Architecture | Introduces the regime-contract specification family. |
| 136 | `specs/regime-contract/changelog.md` | specs | regime-contract | P0 | Architecture | Tracks normative changes to regime-contract. |
| 137 | `specs/regime-contract/segments.md` | specs | regime-contract | P0 | Architecture | Defines segments semantics for regime-contract. |
| 138 | `specs/regime-contract/change-points.md` | specs | regime-contract | P0 | Architecture | Defines change points semantics for regime-contract. |
| 139 | `specs/regime-contract/states.md` | specs | regime-contract | P0 | Architecture | Defines states semantics for regime-contract. |
| 140 | `specs/regime-contract/transitions.md` | specs | regime-contract | P0 | Architecture | Defines transitions semantics for regime-contract. |
| 141 | `specs/regime-contract/guards.md` | specs | regime-contract | P0 | Architecture | Defines guards semantics for regime-contract. |
| 142 | `specs/regime-contract/events.md` | specs | regime-contract | P0 | Architecture | Defines events semantics for regime-contract. |
| 143 | `specs/regime-contract/shared-laws.md` | specs | regime-contract | P0 | Architecture | Defines shared laws semantics for regime-contract. |
| 144 | `specs/regime-contract/regime-laws.md` | specs | regime-contract | P0 | Architecture | Defines regime laws semantics for regime-contract. |
| 145 | `specs/simulation-contract/README.md` | specs | simulation-contract | P0 | Architecture | Introduces the simulation-contract specification family. |
| 146 | `specs/simulation-contract/changelog.md` | specs | simulation-contract | P0 | Architecture | Tracks normative changes to simulation-contract. |
| 147 | `specs/simulation-contract/initial-state.md` | specs | simulation-contract | P0 | Architecture | Defines initial state semantics for simulation-contract. |
| 148 | `specs/simulation-contract/time-grid.md` | specs | simulation-contract | P0 | Architecture | Defines time grid semantics for simulation-contract. |
| 149 | `specs/simulation-contract/solvers.md` | specs | simulation-contract | P0 | Architecture | Defines solvers semantics for simulation-contract. |
| 150 | `specs/simulation-contract/noise.md` | specs | simulation-contract | P0 | Architecture | Defines noise semantics for simulation-contract. |
| 151 | `specs/simulation-contract/events.md` | specs | simulation-contract | P0 | Architecture | Defines events semantics for simulation-contract. |
| 152 | `specs/simulation-contract/interventions.md` | specs | simulation-contract | P0 | Architecture | Defines interventions semantics for simulation-contract. |
| 153 | `specs/simulation-contract/trajectories.md` | specs | simulation-contract | P0 | Architecture | Defines trajectories semantics for simulation-contract. |
| 154 | `specs/simulation-contract/diagnostics.md` | specs | simulation-contract | P0 | Architecture | Defines diagnostics semantics for simulation-contract. |
| 155 | `specs/plugin-protocol/README.md` | specs | plugin-protocol | P0 | Architecture | Introduces the plugin-protocol specification family. |
| 156 | `specs/plugin-protocol/changelog.md` | specs | plugin-protocol | P0 | Architecture | Tracks normative changes to plugin-protocol. |
| 157 | `specs/plugin-protocol/manifest.md` | specs | plugin-protocol | P0 | Architecture | Defines manifest semantics for plugin-protocol. |
| 158 | `specs/plugin-protocol/capabilities.md` | specs | plugin-protocol | P0 | Architecture | Defines capabilities semantics for plugin-protocol. |
| 159 | `specs/plugin-protocol/lifecycle.md` | specs | plugin-protocol | P0 | Architecture | Defines lifecycle semantics for plugin-protocol. |
| 160 | `specs/plugin-protocol/permissions.md` | specs | plugin-protocol | P0 | Architecture | Defines permissions semantics for plugin-protocol. |
| 161 | `specs/plugin-protocol/resources.md` | specs | plugin-protocol | P0 | Architecture | Defines resources semantics for plugin-protocol. |
| 162 | `specs/plugin-protocol/transport.md` | specs | plugin-protocol | P0 | Architecture | Defines transport semantics for plugin-protocol. |
| 163 | `specs/plugin-protocol/errors.md` | specs | plugin-protocol | P0 | Architecture | Defines errors semantics for plugin-protocol. |
| 164 | `specs/plugin-protocol/compatibility.md` | specs | plugin-protocol | P0 | Architecture | Defines compatibility semantics for plugin-protocol. |
| 165 | `specs/service-api/README.md` | specs | service-api | P0 | Architecture | Introduces the service-api specification family. |
| 166 | `specs/service-api/changelog.md` | specs | service-api | P0 | Architecture | Tracks normative changes to service-api. |
| 167 | `specs/service-api/resources.md` | specs | service-api | P0 | Architecture | Defines resources semantics for service-api. |
| 168 | `specs/service-api/errors.md` | specs | service-api | P0 | Architecture | Defines errors semantics for service-api. |
| 169 | `specs/service-api/pagination.md` | specs | service-api | P0 | Architecture | Defines pagination semantics for service-api. |
| 170 | `specs/service-api/idempotency.md` | specs | service-api | P0 | Architecture | Defines idempotency semantics for service-api. |
| 171 | `specs/service-api/authentication.md` | specs | service-api | P0 | Architecture | Defines authentication semantics for service-api. |
| 172 | `specs/service-api/authorization.md` | specs | service-api | P0 | Architecture | Defines authorization semantics for service-api. |
| 173 | `specs/service-api/streaming.md` | specs | service-api | P0 | Architecture | Defines streaming semantics for service-api. |
| 174 | `specs/service-api/versioning.md` | specs | service-api | P0 | Architecture | Defines versioning semantics for service-api. |
| 175 | `specs/event-protocol/README.md` | specs | event-protocol | P0 | Architecture | Introduces the event-protocol specification family. |
| 176 | `specs/event-protocol/changelog.md` | specs | event-protocol | P0 | Architecture | Tracks normative changes to event-protocol. |
| 177 | `specs/event-protocol/envelope.md` | specs | event-protocol | P0 | Architecture | Defines envelope semantics for event-protocol. |
| 178 | `specs/event-protocol/ordering.md` | specs | event-protocol | P0 | Architecture | Defines ordering semantics for event-protocol. |
| 179 | `specs/event-protocol/replay.md` | specs | event-protocol | P0 | Architecture | Defines replay semantics for event-protocol. |
| 180 | `specs/event-protocol/progress.md` | specs | event-protocol | P0 | Architecture | Defines progress semantics for event-protocol. |
| 181 | `specs/event-protocol/job-events.md` | specs | event-protocol | P0 | Architecture | Defines job events semantics for event-protocol. |
| 182 | `specs/event-protocol/artifact-events.md` | specs | event-protocol | P0 | Architecture | Defines artifact events semantics for event-protocol. |
| 183 | `specs/event-protocol/world-events.md` | specs | event-protocol | P0 | Architecture | Defines world events semantics for event-protocol. |
| 184 | `specs/event-protocol/audit-events.md` | specs | event-protocol | P0 | Architecture | Defines audit events semantics for event-protocol. |
| 185 | `specs/reproducibility/README.md` | specs | reproducibility | P0 | Architecture | Introduces the reproducibility specification family. |
| 186 | `specs/reproducibility/changelog.md` | specs | reproducibility | P0 | Architecture | Tracks normative changes to reproducibility. |
| 187 | `specs/reproducibility/seed-plan.md` | specs | reproducibility | P0 | Architecture | Defines seed plan semantics for reproducibility. |
| 188 | `specs/reproducibility/environment.md` | specs | reproducibility | P0 | Architecture | Defines environment semantics for reproducibility. |
| 189 | `specs/reproducibility/data-hash.md` | specs | reproducibility | P0 | Architecture | Defines data hash semantics for reproducibility. |
| 190 | `specs/reproducibility/plan-hash.md` | specs | reproducibility | P0 | Architecture | Defines plan hash semantics for reproducibility. |
| 191 | `specs/reproducibility/algorithm-version.md` | specs | reproducibility | P0 | Architecture | Defines algorithm version semantics for reproducibility. |
| 192 | `specs/reproducibility/hardware-class.md` | specs | reproducibility | P0 | Architecture | Defines hardware class semantics for reproducibility. |
| 193 | `specs/reproducibility/artifacts.md` | specs | reproducibility | P0 | Architecture | Defines artifacts semantics for reproducibility. |
| 194 | `specs/reproducibility/citations.md` | specs | reproducibility | P0 | Architecture | Defines citations semantics for reproducibility. |
| 195 | `specs/security-model/README.md` | specs | security-model | P0 | Architecture | Introduces the security-model specification family. |
| 196 | `specs/security-model/changelog.md` | specs | security-model | P0 | Architecture | Tracks normative changes to security-model. |
| 197 | `specs/security-model/trust-levels.md` | specs | security-model | P0 | Architecture | Defines trust levels semantics for security-model. |
| 198 | `specs/security-model/archives.md` | specs | security-model | P0 | Architecture | Defines archives semantics for security-model. |
| 199 | `specs/security-model/expressions.md` | specs | security-model | P0 | Architecture | Defines expressions semantics for security-model. |
| 200 | `specs/security-model/plugins.md` | specs | security-model | P0 | Architecture | Defines plugins semantics for security-model. |
| 201 | `specs/security-model/tenancy.md` | specs | security-model | P0 | Architecture | Defines tenancy semantics for security-model. |
| 202 | `specs/security-model/resource-limits.md` | specs | security-model | P0 | Architecture | Defines resource limits semantics for security-model. |
| 203 | `specs/security-model/signatures.md` | specs | security-model | P0 | Architecture | Defines signatures semantics for security-model. |
| 204 | `specs/security-model/telemetry.md` | specs | security-model | P0 | Architecture | Defines telemetry semantics for security-model. |
| 205 | `crates/lawsynth-core/Cargo.toml` | rust | lawsynth-core | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-core. |
| 206 | `crates/lawsynth-core/README.md` | rust | lawsynth-core | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-core. |
| 207 | `crates/lawsynth-core/src/lib.rs` | rust | lawsynth-core | P1 | Rust Core | Implements lib for lawsynth-core. |
| 208 | `crates/lawsynth-core/src/error.rs` | rust | lawsynth-core | P1 | Rust Core | Implements error for lawsynth-core. |
| 209 | `crates/lawsynth-core/src/config.rs` | rust | lawsynth-core | P1 | Rust Core | Implements config for lawsynth-core. |
| 210 | `crates/lawsynth-core/src/id.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the id module for lawsynth-core. |
| 211 | `crates/lawsynth-core/src/version.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the version module for lawsynth-core. |
| 212 | `crates/lawsynth-core/src/hash.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the hash module for lawsynth-core. |
| 213 | `crates/lawsynth-core/src/seed.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the seed module for lawsynth-core. |
| 214 | `crates/lawsynth-core/src/cancel.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the cancel module for lawsynth-core. |
| 215 | `crates/lawsynth-core/src/resource.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the resource module for lawsynth-core. |
| 216 | `crates/lawsynth-core/src/progress.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the progress module for lawsynth-core. |
| 217 | `crates/lawsynth-core/src/diagnostics.rs` | rust | lawsynth-core | P1 | Rust Core | Implements the diagnostics module for lawsynth-core. |
| 218 | `crates/lawsynth-core/tests/id_unit.rs` | rust | lawsynth-core | P1 | Rust Core | Verifies id through unit coverage. |
| 219 | `crates/lawsynth-core/tests/version_integration.rs` | rust | lawsynth-core | P1 | Rust Core | Verifies version through integration coverage. |
| 220 | `crates/lawsynth-core/tests/hash_property.rs` | rust | lawsynth-core | P1 | Rust Core | Verifies hash through property coverage. |
| 221 | `crates/lawsynth-core/tests/seed_roundtrip.rs` | rust | lawsynth-core | P1 | Rust Core | Verifies seed through roundtrip coverage. |
| 222 | `crates/lawsynth-core/benches/cancel_throughput.rs` | rust | lawsynth-core | P1 | Rust Core | Measures cancel throughput. |
| 223 | `crates/lawsynth-core/benches/resource_latency.rs` | rust | lawsynth-core | P1 | Rust Core | Measures resource latency. |
| 224 | `crates/lawsynth-core/examples/progress_basic.rs` | rust | lawsynth-core | P1 | Rust Core | Demonstrates basic progress usage. |
| 225 | `crates/lawsynth-core/fixtures/diagnostics/minimal.json` | rust | lawsynth-core | P1 | Rust Core | Provides the minimal fixture for diagnostics. |
| 226 | `crates/lawsynth-core/fixtures/diagnostics/typical.json` | rust | lawsynth-core | P1 | Rust Core | Provides the typical fixture for diagnostics. |
| 227 | `crates/lawsynth-core/fixtures/diagnostics/edge_case.json` | rust | lawsynth-core | P1 | Rust Core | Provides the edge case fixture for diagnostics. |
| 228 | `crates/lawsynth-expr/Cargo.toml` | rust | lawsynth-expr | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-expr. |
| 229 | `crates/lawsynth-expr/README.md` | rust | lawsynth-expr | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-expr. |
| 230 | `crates/lawsynth-expr/src/lib.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements lib for lawsynth-expr. |
| 231 | `crates/lawsynth-expr/src/error.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements error for lawsynth-expr. |
| 232 | `crates/lawsynth-expr/src/config.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements config for lawsynth-expr. |
| 233 | `crates/lawsynth-expr/src/ast.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the ast module for lawsynth-expr. |
| 234 | `crates/lawsynth-expr/src/node.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the node module for lawsynth-expr. |
| 235 | `crates/lawsynth-expr/src/operator.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the operator module for lawsynth-expr. |
| 236 | `crates/lawsynth-expr/src/literal.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the literal module for lawsynth-expr. |
| 237 | `crates/lawsynth-expr/src/symbol.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the symbol module for lawsynth-expr. |
| 238 | `crates/lawsynth-expr/src/parser.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the parser module for lawsynth-expr. |
| 239 | `crates/lawsynth-expr/src/printer.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the printer module for lawsynth-expr. |
| 240 | `crates/lawsynth-expr/src/evaluate.rs` | rust | lawsynth-expr | P1 | Rust Core | Implements the evaluate module for lawsynth-expr. |
| 241 | `crates/lawsynth-expr/tests/ast_unit.rs` | rust | lawsynth-expr | P1 | Rust Core | Verifies ast through unit coverage. |
| 242 | `crates/lawsynth-expr/tests/node_integration.rs` | rust | lawsynth-expr | P1 | Rust Core | Verifies node through integration coverage. |
| 243 | `crates/lawsynth-expr/tests/operator_property.rs` | rust | lawsynth-expr | P1 | Rust Core | Verifies operator through property coverage. |
| 244 | `crates/lawsynth-expr/tests/literal_roundtrip.rs` | rust | lawsynth-expr | P1 | Rust Core | Verifies literal through roundtrip coverage. |
| 245 | `crates/lawsynth-expr/benches/symbol_throughput.rs` | rust | lawsynth-expr | P1 | Rust Core | Measures symbol throughput. |
| 246 | `crates/lawsynth-expr/benches/parser_latency.rs` | rust | lawsynth-expr | P1 | Rust Core | Measures parser latency. |
| 247 | `crates/lawsynth-expr/examples/printer_basic.rs` | rust | lawsynth-expr | P1 | Rust Core | Demonstrates basic printer usage. |
| 248 | `crates/lawsynth-expr/fixtures/evaluate/minimal.json` | rust | lawsynth-expr | P1 | Rust Core | Provides the minimal fixture for evaluate. |
| 249 | `crates/lawsynth-expr/fixtures/evaluate/typical.json` | rust | lawsynth-expr | P1 | Rust Core | Provides the typical fixture for evaluate. |
| 250 | `crates/lawsynth-expr/fixtures/evaluate/edge_case.json` | rust | lawsynth-expr | P1 | Rust Core | Provides the edge case fixture for evaluate. |
| 251 | `crates/lawsynth-egraph/Cargo.toml` | rust | lawsynth-egraph | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-egraph. |
| 252 | `crates/lawsynth-egraph/README.md` | rust | lawsynth-egraph | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-egraph. |
| 253 | `crates/lawsynth-egraph/src/lib.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements lib for lawsynth-egraph. |
| 254 | `crates/lawsynth-egraph/src/error.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements error for lawsynth-egraph. |
| 255 | `crates/lawsynth-egraph/src/config.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements config for lawsynth-egraph. |
| 256 | `crates/lawsynth-egraph/src/language.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the language module for lawsynth-egraph. |
| 257 | `crates/lawsynth-egraph/src/analysis.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the analysis module for lawsynth-egraph. |
| 258 | `crates/lawsynth-egraph/src/rules.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the rules module for lawsynth-egraph. |
| 259 | `crates/lawsynth-egraph/src/schedule.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the schedule module for lawsynth-egraph. |
| 260 | `crates/lawsynth-egraph/src/extract.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the extract module for lawsynth-egraph. |
| 261 | `crates/lawsynth-egraph/src/cost.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the cost module for lawsynth-egraph. |
| 262 | `crates/lawsynth-egraph/src/proof.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the proof module for lawsynth-egraph. |
| 263 | `crates/lawsynth-egraph/src/limits.rs` | rust | lawsynth-egraph | P2 | Rust Core | Implements the limits module for lawsynth-egraph. |
| 264 | `crates/lawsynth-egraph/tests/language_unit.rs` | rust | lawsynth-egraph | P2 | Rust Core | Verifies language through unit coverage. |
| 265 | `crates/lawsynth-egraph/tests/analysis_integration.rs` | rust | lawsynth-egraph | P2 | Rust Core | Verifies analysis through integration coverage. |
| 266 | `crates/lawsynth-egraph/tests/rules_property.rs` | rust | lawsynth-egraph | P2 | Rust Core | Verifies rules through property coverage. |
| 267 | `crates/lawsynth-egraph/tests/schedule_roundtrip.rs` | rust | lawsynth-egraph | P2 | Rust Core | Verifies schedule through roundtrip coverage. |
| 268 | `crates/lawsynth-egraph/benches/extract_throughput.rs` | rust | lawsynth-egraph | P2 | Rust Core | Measures extract throughput. |
| 269 | `crates/lawsynth-egraph/benches/cost_latency.rs` | rust | lawsynth-egraph | P2 | Rust Core | Measures cost latency. |
| 270 | `crates/lawsynth-egraph/examples/proof_basic.rs` | rust | lawsynth-egraph | P2 | Rust Core | Demonstrates basic proof usage. |
| 271 | `crates/lawsynth-egraph/fixtures/limits/minimal.json` | rust | lawsynth-egraph | P2 | Rust Core | Provides the minimal fixture for limits. |
| 272 | `crates/lawsynth-egraph/fixtures/limits/typical.json` | rust | lawsynth-egraph | P2 | Rust Core | Provides the typical fixture for limits. |
| 273 | `crates/lawsynth-egraph/fixtures/limits/edge_case.json` | rust | lawsynth-egraph | P2 | Rust Core | Provides the edge case fixture for limits. |
| 274 | `crates/lawsynth-units/Cargo.toml` | rust | lawsynth-units | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-units. |
| 275 | `crates/lawsynth-units/README.md` | rust | lawsynth-units | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-units. |
| 276 | `crates/lawsynth-units/src/lib.rs` | rust | lawsynth-units | P1 | Rust Core | Implements lib for lawsynth-units. |
| 277 | `crates/lawsynth-units/src/error.rs` | rust | lawsynth-units | P1 | Rust Core | Implements error for lawsynth-units. |
| 278 | `crates/lawsynth-units/src/config.rs` | rust | lawsynth-units | P1 | Rust Core | Implements config for lawsynth-units. |
| 279 | `crates/lawsynth-units/src/dimension.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the dimension module for lawsynth-units. |
| 280 | `crates/lawsynth-units/src/unit.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the unit module for lawsynth-units. |
| 281 | `crates/lawsynth-units/src/registry.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the registry module for lawsynth-units. |
| 282 | `crates/lawsynth-units/src/parse.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the parse module for lawsynth-units. |
| 283 | `crates/lawsynth-units/src/convert.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the convert module for lawsynth-units. |
| 284 | `crates/lawsynth-units/src/infer.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the infer module for lawsynth-units. |
| 285 | `crates/lawsynth-units/src/check.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the check module for lawsynth-units. |
| 286 | `crates/lawsynth-units/src/builtins.rs` | rust | lawsynth-units | P1 | Rust Core | Implements the builtins module for lawsynth-units. |
| 287 | `crates/lawsynth-units/tests/dimension_unit.rs` | rust | lawsynth-units | P1 | Rust Core | Verifies dimension through unit coverage. |
| 288 | `crates/lawsynth-units/tests/unit_integration.rs` | rust | lawsynth-units | P1 | Rust Core | Verifies unit through integration coverage. |
| 289 | `crates/lawsynth-units/tests/registry_property.rs` | rust | lawsynth-units | P1 | Rust Core | Verifies registry through property coverage. |
| 290 | `crates/lawsynth-units/tests/parse_roundtrip.rs` | rust | lawsynth-units | P1 | Rust Core | Verifies parse through roundtrip coverage. |
| 291 | `crates/lawsynth-units/benches/convert_throughput.rs` | rust | lawsynth-units | P1 | Rust Core | Measures convert throughput. |
| 292 | `crates/lawsynth-units/benches/infer_latency.rs` | rust | lawsynth-units | P1 | Rust Core | Measures infer latency. |
| 293 | `crates/lawsynth-units/examples/check_basic.rs` | rust | lawsynth-units | P1 | Rust Core | Demonstrates basic check usage. |
| 294 | `crates/lawsynth-units/fixtures/builtins/minimal.json` | rust | lawsynth-units | P1 | Rust Core | Provides the minimal fixture for builtins. |
| 295 | `crates/lawsynth-units/fixtures/builtins/typical.json` | rust | lawsynth-units | P1 | Rust Core | Provides the typical fixture for builtins. |
| 296 | `crates/lawsynth-units/fixtures/builtins/edge_case.json` | rust | lawsynth-units | P1 | Rust Core | Provides the edge case fixture for builtins. |
| 297 | `crates/lawsynth-world/Cargo.toml` | rust | lawsynth-world | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-world. |
| 298 | `crates/lawsynth-world/README.md` | rust | lawsynth-world | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-world. |
| 299 | `crates/lawsynth-world/src/lib.rs` | rust | lawsynth-world | P1 | Rust Core | Implements lib for lawsynth-world. |
| 300 | `crates/lawsynth-world/src/error.rs` | rust | lawsynth-world | P1 | Rust Core | Implements error for lawsynth-world. |
| 301 | `crates/lawsynth-world/src/config.rs` | rust | lawsynth-world | P1 | Rust Core | Implements config for lawsynth-world. |
| 302 | `crates/lawsynth-world/src/world.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the world module for lawsynth-world. |
| 303 | `crates/lawsynth-world/src/variable.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the variable module for lawsynth-world. |
| 304 | `crates/lawsynth-world/src/parameter.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the parameter module for lawsynth-world. |
| 305 | `crates/lawsynth-world/src/law.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the law module for lawsynth-world. |
| 306 | `crates/lawsynth-world/src/graph.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the graph module for lawsynth-world. |
| 307 | `crates/lawsynth-world/src/regime.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the regime module for lawsynth-world. |
| 308 | `crates/lawsynth-world/src/event.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the event module for lawsynth-world. |
| 309 | `crates/lawsynth-world/src/intervention.rs` | rust | lawsynth-world | P1 | Rust Core | Implements the intervention module for lawsynth-world. |
| 310 | `crates/lawsynth-world/tests/world_unit.rs` | rust | lawsynth-world | P1 | Rust Core | Verifies world through unit coverage. |
| 311 | `crates/lawsynth-world/tests/variable_integration.rs` | rust | lawsynth-world | P1 | Rust Core | Verifies variable through integration coverage. |
| 312 | `crates/lawsynth-world/tests/parameter_property.rs` | rust | lawsynth-world | P1 | Rust Core | Verifies parameter through property coverage. |
| 313 | `crates/lawsynth-world/tests/law_roundtrip.rs` | rust | lawsynth-world | P1 | Rust Core | Verifies law through roundtrip coverage. |
| 314 | `crates/lawsynth-world/benches/graph_throughput.rs` | rust | lawsynth-world | P1 | Rust Core | Measures graph throughput. |
| 315 | `crates/lawsynth-world/benches/regime_latency.rs` | rust | lawsynth-world | P1 | Rust Core | Measures regime latency. |
| 316 | `crates/lawsynth-world/examples/event_basic.rs` | rust | lawsynth-world | P1 | Rust Core | Demonstrates basic event usage. |
| 317 | `crates/lawsynth-world/fixtures/intervention/minimal.json` | rust | lawsynth-world | P1 | Rust Core | Provides the minimal fixture for intervention. |
| 318 | `crates/lawsynth-world/fixtures/intervention/typical.json` | rust | lawsynth-world | P1 | Rust Core | Provides the typical fixture for intervention. |
| 319 | `crates/lawsynth-world/fixtures/intervention/edge_case.json` | rust | lawsynth-world | P1 | Rust Core | Provides the edge case fixture for intervention. |
| 320 | `crates/lawsynth-data/Cargo.toml` | rust | lawsynth-data | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-data. |
| 321 | `crates/lawsynth-data/README.md` | rust | lawsynth-data | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-data. |
| 322 | `crates/lawsynth-data/src/lib.rs` | rust | lawsynth-data | P1 | Rust Core | Implements lib for lawsynth-data. |
| 323 | `crates/lawsynth-data/src/error.rs` | rust | lawsynth-data | P1 | Rust Core | Implements error for lawsynth-data. |
| 324 | `crates/lawsynth-data/src/config.rs` | rust | lawsynth-data | P1 | Rust Core | Implements config for lawsynth-data. |
| 325 | `crates/lawsynth-data/src/dataset.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the dataset module for lawsynth-data. |
| 326 | `crates/lawsynth-data/src/schema.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the schema module for lawsynth-data. |
| 327 | `crates/lawsynth-data/src/column.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the column module for lawsynth-data. |
| 328 | `crates/lawsynth-data/src/time_axis.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the time axis module for lawsynth-data. |
| 329 | `crates/lawsynth-data/src/window.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the window module for lawsynth-data. |
| 330 | `crates/lawsynth-data/src/batch.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the batch module for lawsynth-data. |
| 331 | `crates/lawsynth-data/src/parquet.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the parquet module for lawsynth-data. |
| 332 | `crates/lawsynth-data/src/fingerprint.rs` | rust | lawsynth-data | P1 | Rust Core | Implements the fingerprint module for lawsynth-data. |
| 333 | `crates/lawsynth-data/tests/dataset_unit.rs` | rust | lawsynth-data | P1 | Rust Core | Verifies dataset through unit coverage. |
| 334 | `crates/lawsynth-data/tests/schema_integration.rs` | rust | lawsynth-data | P1 | Rust Core | Verifies schema through integration coverage. |
| 335 | `crates/lawsynth-data/tests/column_property.rs` | rust | lawsynth-data | P1 | Rust Core | Verifies column through property coverage. |
| 336 | `crates/lawsynth-data/tests/time_axis_roundtrip.rs` | rust | lawsynth-data | P1 | Rust Core | Verifies time axis through roundtrip coverage. |
| 337 | `crates/lawsynth-data/benches/window_throughput.rs` | rust | lawsynth-data | P1 | Rust Core | Measures window throughput. |
| 338 | `crates/lawsynth-data/benches/batch_latency.rs` | rust | lawsynth-data | P1 | Rust Core | Measures batch latency. |
| 339 | `crates/lawsynth-data/examples/parquet_basic.rs` | rust | lawsynth-data | P1 | Rust Core | Demonstrates basic parquet usage. |
| 340 | `crates/lawsynth-data/fixtures/fingerprint/minimal.json` | rust | lawsynth-data | P1 | Rust Core | Provides the minimal fixture for fingerprint. |
| 341 | `crates/lawsynth-data/fixtures/fingerprint/typical.json` | rust | lawsynth-data | P1 | Rust Core | Provides the typical fixture for fingerprint. |
| 342 | `crates/lawsynth-data/fixtures/fingerprint/edge_case.json` | rust | lawsynth-data | P1 | Rust Core | Provides the edge case fixture for fingerprint. |
| 343 | `crates/lawsynth-profile/Cargo.toml` | rust | lawsynth-profile | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-profile. |
| 344 | `crates/lawsynth-profile/README.md` | rust | lawsynth-profile | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-profile. |
| 345 | `crates/lawsynth-profile/src/lib.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements lib for lawsynth-profile. |
| 346 | `crates/lawsynth-profile/src/error.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements error for lawsynth-profile. |
| 347 | `crates/lawsynth-profile/src/config.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements config for lawsynth-profile. |
| 348 | `crates/lawsynth-profile/src/profiler.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the profiler module for lawsynth-profile. |
| 349 | `crates/lawsynth-profile/src/column_profile.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the column profile module for lawsynth-profile. |
| 350 | `crates/lawsynth-profile/src/time_profile.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the time profile module for lawsynth-profile. |
| 351 | `crates/lawsynth-profile/src/missingness.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the missingness module for lawsynth-profile. |
| 352 | `crates/lawsynth-profile/src/distribution.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the distribution module for lawsynth-profile. |
| 353 | `crates/lawsynth-profile/src/dependence.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the dependence module for lawsynth-profile. |
| 354 | `crates/lawsynth-profile/src/delays.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the delays module for lawsynth-profile. |
| 355 | `crates/lawsynth-profile/src/quality_flags.rs` | rust | lawsynth-profile | P2 | Rust Core | Implements the quality flags module for lawsynth-profile. |
| 356 | `crates/lawsynth-profile/tests/profiler_unit.rs` | rust | lawsynth-profile | P2 | Rust Core | Verifies profiler through unit coverage. |
| 357 | `crates/lawsynth-profile/tests/column_profile_integration.rs` | rust | lawsynth-profile | P2 | Rust Core | Verifies column profile through integration coverage. |
| 358 | `crates/lawsynth-profile/tests/time_profile_property.rs` | rust | lawsynth-profile | P2 | Rust Core | Verifies time profile through property coverage. |
| 359 | `crates/lawsynth-profile/tests/missingness_roundtrip.rs` | rust | lawsynth-profile | P2 | Rust Core | Verifies missingness through roundtrip coverage. |
| 360 | `crates/lawsynth-profile/benches/distribution_throughput.rs` | rust | lawsynth-profile | P2 | Rust Core | Measures distribution throughput. |
| 361 | `crates/lawsynth-profile/benches/dependence_latency.rs` | rust | lawsynth-profile | P2 | Rust Core | Measures dependence latency. |
| 362 | `crates/lawsynth-profile/examples/delays_basic.rs` | rust | lawsynth-profile | P2 | Rust Core | Demonstrates basic delays usage. |
| 363 | `crates/lawsynth-profile/fixtures/quality_flags/minimal.json` | rust | lawsynth-profile | P2 | Rust Core | Provides the minimal fixture for quality flags. |
| 364 | `crates/lawsynth-profile/fixtures/quality_flags/typical.json` | rust | lawsynth-profile | P2 | Rust Core | Provides the typical fixture for quality flags. |
| 365 | `crates/lawsynth-profile/fixtures/quality_flags/edge_case.json` | rust | lawsynth-profile | P2 | Rust Core | Provides the edge case fixture for quality flags. |
| 366 | `crates/lawsynth-preprocess/Cargo.toml` | rust | lawsynth-preprocess | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-preprocess. |
| 367 | `crates/lawsynth-preprocess/README.md` | rust | lawsynth-preprocess | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-preprocess. |
| 368 | `crates/lawsynth-preprocess/src/lib.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements lib for lawsynth-preprocess. |
| 369 | `crates/lawsynth-preprocess/src/error.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements error for lawsynth-preprocess. |
| 370 | `crates/lawsynth-preprocess/src/config.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements config for lawsynth-preprocess. |
| 371 | `crates/lawsynth-preprocess/src/pipeline.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the pipeline module for lawsynth-preprocess. |
| 372 | `crates/lawsynth-preprocess/src/transform.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the transform module for lawsynth-preprocess. |
| 373 | `crates/lawsynth-preprocess/src/align.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the align module for lawsynth-preprocess. |
| 374 | `crates/lawsynth-preprocess/src/resample.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the resample module for lawsynth-preprocess. |
| 375 | `crates/lawsynth-preprocess/src/impute.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the impute module for lawsynth-preprocess. |
| 376 | `crates/lawsynth-preprocess/src/scale.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the scale module for lawsynth-preprocess. |
| 377 | `crates/lawsynth-preprocess/src/detrend.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the detrend module for lawsynth-preprocess. |
| 378 | `crates/lawsynth-preprocess/src/smooth.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Implements the smooth module for lawsynth-preprocess. |
| 379 | `crates/lawsynth-preprocess/tests/pipeline_unit.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Verifies pipeline through unit coverage. |
| 380 | `crates/lawsynth-preprocess/tests/transform_integration.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Verifies transform through integration coverage. |
| 381 | `crates/lawsynth-preprocess/tests/align_property.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Verifies align through property coverage. |
| 382 | `crates/lawsynth-preprocess/tests/resample_roundtrip.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Verifies resample through roundtrip coverage. |
| 383 | `crates/lawsynth-preprocess/benches/impute_throughput.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Measures impute throughput. |
| 384 | `crates/lawsynth-preprocess/benches/scale_latency.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Measures scale latency. |
| 385 | `crates/lawsynth-preprocess/examples/detrend_basic.rs` | rust | lawsynth-preprocess | P2 | Rust Core | Demonstrates basic detrend usage. |
| 386 | `crates/lawsynth-preprocess/fixtures/smooth/minimal.json` | rust | lawsynth-preprocess | P2 | Rust Core | Provides the minimal fixture for smooth. |
| 387 | `crates/lawsynth-preprocess/fixtures/smooth/typical.json` | rust | lawsynth-preprocess | P2 | Rust Core | Provides the typical fixture for smooth. |
| 388 | `crates/lawsynth-preprocess/fixtures/smooth/edge_case.json` | rust | lawsynth-preprocess | P2 | Rust Core | Provides the edge case fixture for smooth. |
| 389 | `crates/lawsynth-stats/Cargo.toml` | rust | lawsynth-stats | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-stats. |
| 390 | `crates/lawsynth-stats/README.md` | rust | lawsynth-stats | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-stats. |
| 391 | `crates/lawsynth-stats/src/lib.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements lib for lawsynth-stats. |
| 392 | `crates/lawsynth-stats/src/error.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements error for lawsynth-stats. |
| 393 | `crates/lawsynth-stats/src/config.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements config for lawsynth-stats. |
| 394 | `crates/lawsynth-stats/src/moments.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the moments module for lawsynth-stats. |
| 395 | `crates/lawsynth-stats/src/quantile.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the quantile module for lawsynth-stats. |
| 396 | `crates/lawsynth-stats/src/covariance.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the covariance module for lawsynth-stats. |
| 397 | `crates/lawsynth-stats/src/robust.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the robust module for lawsynth-stats. |
| 398 | `crates/lawsynth-stats/src/distributions.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the distributions module for lawsynth-stats. |
| 399 | `crates/lawsynth-stats/src/bootstrap.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the bootstrap module for lawsynth-stats. |
| 400 | `crates/lawsynth-stats/src/information.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the information module for lawsynth-stats. |
| 401 | `crates/lawsynth-stats/src/sampling.rs` | rust | lawsynth-stats | P2 | Rust Core | Implements the sampling module for lawsynth-stats. |
| 402 | `crates/lawsynth-stats/tests/moments_unit.rs` | rust | lawsynth-stats | P2 | Rust Core | Verifies moments through unit coverage. |
| 403 | `crates/lawsynth-stats/tests/quantile_integration.rs` | rust | lawsynth-stats | P2 | Rust Core | Verifies quantile through integration coverage. |
| 404 | `crates/lawsynth-stats/tests/covariance_property.rs` | rust | lawsynth-stats | P2 | Rust Core | Verifies covariance through property coverage. |
| 405 | `crates/lawsynth-stats/tests/robust_roundtrip.rs` | rust | lawsynth-stats | P2 | Rust Core | Verifies robust through roundtrip coverage. |
| 406 | `crates/lawsynth-stats/benches/distributions_throughput.rs` | rust | lawsynth-stats | P2 | Rust Core | Measures distributions throughput. |
| 407 | `crates/lawsynth-stats/benches/bootstrap_latency.rs` | rust | lawsynth-stats | P2 | Rust Core | Measures bootstrap latency. |
| 408 | `crates/lawsynth-stats/examples/information_basic.rs` | rust | lawsynth-stats | P2 | Rust Core | Demonstrates basic information usage. |
| 409 | `crates/lawsynth-stats/fixtures/sampling/minimal.json` | rust | lawsynth-stats | P2 | Rust Core | Provides the minimal fixture for sampling. |
| 410 | `crates/lawsynth-stats/fixtures/sampling/typical.json` | rust | lawsynth-stats | P2 | Rust Core | Provides the typical fixture for sampling. |
| 411 | `crates/lawsynth-stats/fixtures/sampling/edge_case.json` | rust | lawsynth-stats | P2 | Rust Core | Provides the edge case fixture for sampling. |
| 412 | `crates/lawsynth-differentiate/Cargo.toml` | rust | lawsynth-differentiate | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-differentiate. |
| 413 | `crates/lawsynth-differentiate/README.md` | rust | lawsynth-differentiate | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-differentiate. |
| 414 | `crates/lawsynth-differentiate/src/lib.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements lib for lawsynth-differentiate. |
| 415 | `crates/lawsynth-differentiate/src/error.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements error for lawsynth-differentiate. |
| 416 | `crates/lawsynth-differentiate/src/config.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements config for lawsynth-differentiate. |
| 417 | `crates/lawsynth-differentiate/src/method.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the method module for lawsynth-differentiate. |
| 418 | `crates/lawsynth-differentiate/src/finite.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the finite module for lawsynth-differentiate. |
| 419 | `crates/lawsynth-differentiate/src/savgol.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the savgol module for lawsynth-differentiate. |
| 420 | `crates/lawsynth-differentiate/src/spline.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the spline module for lawsynth-differentiate. |
| 421 | `crates/lawsynth-differentiate/src/tvreg.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the tvreg module for lawsynth-differentiate. |
| 422 | `crates/lawsynth-differentiate/src/spectral.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the spectral module for lawsynth-differentiate. |
| 423 | `crates/lawsynth-differentiate/src/weak_form.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the weak form module for lawsynth-differentiate. |
| 424 | `crates/lawsynth-differentiate/src/irregular.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Implements the irregular module for lawsynth-differentiate. |
| 425 | `crates/lawsynth-differentiate/tests/method_unit.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Verifies method through unit coverage. |
| 426 | `crates/lawsynth-differentiate/tests/finite_integration.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Verifies finite through integration coverage. |
| 427 | `crates/lawsynth-differentiate/tests/savgol_property.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Verifies savgol through property coverage. |
| 428 | `crates/lawsynth-differentiate/tests/spline_roundtrip.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Verifies spline through roundtrip coverage. |
| 429 | `crates/lawsynth-differentiate/benches/tvreg_throughput.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Measures tvreg throughput. |
| 430 | `crates/lawsynth-differentiate/benches/spectral_latency.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Measures spectral latency. |
| 431 | `crates/lawsynth-differentiate/examples/weak_form_basic.rs` | rust | lawsynth-differentiate | P2 | Rust Core | Demonstrates basic weak form usage. |
| 432 | `crates/lawsynth-differentiate/fixtures/irregular/minimal.json` | rust | lawsynth-differentiate | P2 | Rust Core | Provides the minimal fixture for irregular. |
| 433 | `crates/lawsynth-differentiate/fixtures/irregular/typical.json` | rust | lawsynth-differentiate | P2 | Rust Core | Provides the typical fixture for irregular. |
| 434 | `crates/lawsynth-differentiate/fixtures/irregular/edge_case.json` | rust | lawsynth-differentiate | P2 | Rust Core | Provides the edge case fixture for irregular. |
| 435 | `crates/lawsynth-features/Cargo.toml` | rust | lawsynth-features | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-features. |
| 436 | `crates/lawsynth-features/README.md` | rust | lawsynth-features | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-features. |
| 437 | `crates/lawsynth-features/src/lib.rs` | rust | lawsynth-features | P2 | Rust Core | Implements lib for lawsynth-features. |
| 438 | `crates/lawsynth-features/src/error.rs` | rust | lawsynth-features | P2 | Rust Core | Implements error for lawsynth-features. |
| 439 | `crates/lawsynth-features/src/config.rs` | rust | lawsynth-features | P2 | Rust Core | Implements config for lawsynth-features. |
| 440 | `crates/lawsynth-features/src/library.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the library module for lawsynth-features. |
| 441 | `crates/lawsynth-features/src/term.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the term module for lawsynth-features. |
| 442 | `crates/lawsynth-features/src/polynomial.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the polynomial module for lawsynth-features. |
| 443 | `crates/lawsynth-features/src/trigonometric.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the trigonometric module for lawsynth-features. |
| 444 | `crates/lawsynth-features/src/rational.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the rational module for lawsynth-features. |
| 445 | `crates/lawsynth-features/src/delay.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the delay module for lawsynth-features. |
| 446 | `crates/lawsynth-features/src/interaction.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the interaction module for lawsynth-features. |
| 447 | `crates/lawsynth-features/src/constraints.rs` | rust | lawsynth-features | P2 | Rust Core | Implements the constraints module for lawsynth-features. |
| 448 | `crates/lawsynth-features/tests/library_unit.rs` | rust | lawsynth-features | P2 | Rust Core | Verifies library through unit coverage. |
| 449 | `crates/lawsynth-features/tests/term_integration.rs` | rust | lawsynth-features | P2 | Rust Core | Verifies term through integration coverage. |
| 450 | `crates/lawsynth-features/tests/polynomial_property.rs` | rust | lawsynth-features | P2 | Rust Core | Verifies polynomial through property coverage. |
| 451 | `crates/lawsynth-features/tests/trigonometric_roundtrip.rs` | rust | lawsynth-features | P2 | Rust Core | Verifies trigonometric through roundtrip coverage. |
| 452 | `crates/lawsynth-features/benches/rational_throughput.rs` | rust | lawsynth-features | P2 | Rust Core | Measures rational throughput. |
| 453 | `crates/lawsynth-features/benches/delay_latency.rs` | rust | lawsynth-features | P2 | Rust Core | Measures delay latency. |
| 454 | `crates/lawsynth-features/examples/interaction_basic.rs` | rust | lawsynth-features | P2 | Rust Core | Demonstrates basic interaction usage. |
| 455 | `crates/lawsynth-features/fixtures/constraints/minimal.json` | rust | lawsynth-features | P2 | Rust Core | Provides the minimal fixture for constraints. |
| 456 | `crates/lawsynth-features/fixtures/constraints/typical.json` | rust | lawsynth-features | P2 | Rust Core | Provides the typical fixture for constraints. |
| 457 | `crates/lawsynth-features/fixtures/constraints/edge_case.json` | rust | lawsynth-features | P2 | Rust Core | Provides the edge case fixture for constraints. |
| 458 | `crates/lawsynth-opt/Cargo.toml` | rust | lawsynth-opt | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-opt. |
| 459 | `crates/lawsynth-opt/README.md` | rust | lawsynth-opt | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-opt. |
| 460 | `crates/lawsynth-opt/src/lib.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements lib for lawsynth-opt. |
| 461 | `crates/lawsynth-opt/src/error.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements error for lawsynth-opt. |
| 462 | `crates/lawsynth-opt/src/config.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements config for lawsynth-opt. |
| 463 | `crates/lawsynth-opt/src/objective.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the objective module for lawsynth-opt. |
| 464 | `crates/lawsynth-opt/src/bounds.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the bounds module for lawsynth-opt. |
| 465 | `crates/lawsynth-opt/src/least_squares.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the least squares module for lawsynth-opt. |
| 466 | `crates/lawsynth-opt/src/lbfgs.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the lbfgs module for lawsynth-opt. |
| 467 | `crates/lawsynth-opt/src/nelder_mead.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the nelder mead module for lawsynth-opt. |
| 468 | `crates/lawsynth-opt/src/coordinate.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the coordinate module for lawsynth-opt. |
| 469 | `crates/lawsynth-opt/src/mixed.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the mixed module for lawsynth-opt. |
| 470 | `crates/lawsynth-opt/src/termination.rs` | rust | lawsynth-opt | P2 | Rust Core | Implements the termination module for lawsynth-opt. |
| 471 | `crates/lawsynth-opt/tests/objective_unit.rs` | rust | lawsynth-opt | P2 | Rust Core | Verifies objective through unit coverage. |
| 472 | `crates/lawsynth-opt/tests/bounds_integration.rs` | rust | lawsynth-opt | P2 | Rust Core | Verifies bounds through integration coverage. |
| 473 | `crates/lawsynth-opt/tests/least_squares_property.rs` | rust | lawsynth-opt | P2 | Rust Core | Verifies least squares through property coverage. |
| 474 | `crates/lawsynth-opt/tests/lbfgs_roundtrip.rs` | rust | lawsynth-opt | P2 | Rust Core | Verifies lbfgs through roundtrip coverage. |
| 475 | `crates/lawsynth-opt/benches/nelder_mead_throughput.rs` | rust | lawsynth-opt | P2 | Rust Core | Measures nelder mead throughput. |
| 476 | `crates/lawsynth-opt/benches/coordinate_latency.rs` | rust | lawsynth-opt | P2 | Rust Core | Measures coordinate latency. |
| 477 | `crates/lawsynth-opt/examples/mixed_basic.rs` | rust | lawsynth-opt | P2 | Rust Core | Demonstrates basic mixed usage. |
| 478 | `crates/lawsynth-opt/fixtures/termination/minimal.json` | rust | lawsynth-opt | P2 | Rust Core | Provides the minimal fixture for termination. |
| 479 | `crates/lawsynth-opt/fixtures/termination/typical.json` | rust | lawsynth-opt | P2 | Rust Core | Provides the typical fixture for termination. |
| 480 | `crates/lawsynth-opt/fixtures/termination/edge_case.json` | rust | lawsynth-opt | P2 | Rust Core | Provides the edge case fixture for termination. |
| 481 | `crates/lawsynth-sparse/Cargo.toml` | rust | lawsynth-sparse | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-sparse. |
| 482 | `crates/lawsynth-sparse/README.md` | rust | lawsynth-sparse | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-sparse. |
| 483 | `crates/lawsynth-sparse/src/lib.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements lib for lawsynth-sparse. |
| 484 | `crates/lawsynth-sparse/src/error.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements error for lawsynth-sparse. |
| 485 | `crates/lawsynth-sparse/src/config.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements config for lawsynth-sparse. |
| 486 | `crates/lawsynth-sparse/src/problem.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the problem module for lawsynth-sparse. |
| 487 | `crates/lawsynth-sparse/src/standardize.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the standardize module for lawsynth-sparse. |
| 488 | `crates/lawsynth-sparse/src/stlsq.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the stlsq module for lawsynth-sparse. |
| 489 | `crates/lawsynth-sparse/src/sr3.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the sr3 module for lawsynth-sparse. |
| 490 | `crates/lawsynth-sparse/src/lasso.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the lasso module for lawsynth-sparse. |
| 491 | `crates/lawsynth-sparse/src/group.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the group module for lawsynth-sparse. |
| 492 | `crates/lawsynth-sparse/src/constrained.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the constrained module for lawsynth-sparse. |
| 493 | `crates/lawsynth-sparse/src/stability.rs` | rust | lawsynth-sparse | P2 | Rust Core | Implements the stability module for lawsynth-sparse. |
| 494 | `crates/lawsynth-sparse/tests/problem_unit.rs` | rust | lawsynth-sparse | P2 | Rust Core | Verifies problem through unit coverage. |
| 495 | `crates/lawsynth-sparse/tests/standardize_integration.rs` | rust | lawsynth-sparse | P2 | Rust Core | Verifies standardize through integration coverage. |
| 496 | `crates/lawsynth-sparse/tests/stlsq_property.rs` | rust | lawsynth-sparse | P2 | Rust Core | Verifies stlsq through property coverage. |
| 497 | `crates/lawsynth-sparse/tests/sr3_roundtrip.rs` | rust | lawsynth-sparse | P2 | Rust Core | Verifies sr3 through roundtrip coverage. |
| 498 | `crates/lawsynth-sparse/benches/lasso_throughput.rs` | rust | lawsynth-sparse | P2 | Rust Core | Measures lasso throughput. |
| 499 | `crates/lawsynth-sparse/benches/group_latency.rs` | rust | lawsynth-sparse | P2 | Rust Core | Measures group latency. |
| 500 | `crates/lawsynth-sparse/examples/constrained_basic.rs` | rust | lawsynth-sparse | P2 | Rust Core | Demonstrates basic constrained usage. |
| 501 | `crates/lawsynth-sparse/fixtures/stability/minimal.json` | rust | lawsynth-sparse | P2 | Rust Core | Provides the minimal fixture for stability. |
| 502 | `crates/lawsynth-sparse/fixtures/stability/typical.json` | rust | lawsynth-sparse | P2 | Rust Core | Provides the typical fixture for stability. |
| 503 | `crates/lawsynth-sparse/fixtures/stability/edge_case.json` | rust | lawsynth-sparse | P2 | Rust Core | Provides the edge case fixture for stability. |
| 504 | `crates/lawsynth-symbolic/Cargo.toml` | rust | lawsynth-symbolic | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-symbolic. |
| 505 | `crates/lawsynth-symbolic/README.md` | rust | lawsynth-symbolic | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-symbolic. |
| 506 | `crates/lawsynth-symbolic/src/lib.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements lib for lawsynth-symbolic. |
| 507 | `crates/lawsynth-symbolic/src/error.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements error for lawsynth-symbolic. |
| 508 | `crates/lawsynth-symbolic/src/config.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements config for lawsynth-symbolic. |
| 509 | `crates/lawsynth-symbolic/src/grammar.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the grammar module for lawsynth-symbolic. |
| 510 | `crates/lawsynth-symbolic/src/population.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the population module for lawsynth-symbolic. |
| 511 | `crates/lawsynth-symbolic/src/initialize.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the initialize module for lawsynth-symbolic. |
| 512 | `crates/lawsynth-symbolic/src/mutate.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the mutate module for lawsynth-symbolic. |
| 513 | `crates/lawsynth-symbolic/src/crossover.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the crossover module for lawsynth-symbolic. |
| 514 | `crates/lawsynth-symbolic/src/constants.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the constants module for lawsynth-symbolic. |
| 515 | `crates/lawsynth-symbolic/src/simplify.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the simplify module for lawsynth-symbolic. |
| 516 | `crates/lawsynth-symbolic/src/frontier.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Implements the frontier module for lawsynth-symbolic. |
| 517 | `crates/lawsynth-symbolic/tests/grammar_unit.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Verifies grammar through unit coverage. |
| 518 | `crates/lawsynth-symbolic/tests/population_integration.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Verifies population through integration coverage. |
| 519 | `crates/lawsynth-symbolic/tests/initialize_property.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Verifies initialize through property coverage. |
| 520 | `crates/lawsynth-symbolic/tests/mutate_roundtrip.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Verifies mutate through roundtrip coverage. |
| 521 | `crates/lawsynth-symbolic/benches/crossover_throughput.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Measures crossover throughput. |
| 522 | `crates/lawsynth-symbolic/benches/constants_latency.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Measures constants latency. |
| 523 | `crates/lawsynth-symbolic/examples/simplify_basic.rs` | rust | lawsynth-symbolic | P2 | Rust Core | Demonstrates basic simplify usage. |
| 524 | `crates/lawsynth-symbolic/fixtures/frontier/minimal.json` | rust | lawsynth-symbolic | P2 | Rust Core | Provides the minimal fixture for frontier. |
| 525 | `crates/lawsynth-symbolic/fixtures/frontier/typical.json` | rust | lawsynth-symbolic | P2 | Rust Core | Provides the typical fixture for frontier. |
| 526 | `crates/lawsynth-symbolic/fixtures/frontier/edge_case.json` | rust | lawsynth-symbolic | P2 | Rust Core | Provides the edge case fixture for frontier. |
| 527 | `crates/lawsynth-dynamics/Cargo.toml` | rust | lawsynth-dynamics | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-dynamics. |
| 528 | `crates/lawsynth-dynamics/README.md` | rust | lawsynth-dynamics | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-dynamics. |
| 529 | `crates/lawsynth-dynamics/src/lib.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements lib for lawsynth-dynamics. |
| 530 | `crates/lawsynth-dynamics/src/error.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements error for lawsynth-dynamics. |
| 531 | `crates/lawsynth-dynamics/src/config.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements config for lawsynth-dynamics. |
| 532 | `crates/lawsynth-dynamics/src/problem.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the problem module for lawsynth-dynamics. |
| 533 | `crates/lawsynth-dynamics/src/continuous.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the continuous module for lawsynth-dynamics. |
| 534 | `crates/lawsynth-dynamics/src/discrete.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the discrete module for lawsynth-dynamics. |
| 535 | `crates/lawsynth-dynamics/src/delay.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the delay module for lawsynth-dynamics. |
| 536 | `crates/lawsynth-dynamics/src/implicit.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the implicit module for lawsynth-dynamics. |
| 537 | `crates/lawsynth-dynamics/src/control.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the control module for lawsynth-dynamics. |
| 538 | `crates/lawsynth-dynamics/src/refine.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the refine module for lawsynth-dynamics. |
| 539 | `crates/lawsynth-dynamics/src/result.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Implements the result module for lawsynth-dynamics. |
| 540 | `crates/lawsynth-dynamics/tests/problem_unit.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Verifies problem through unit coverage. |
| 541 | `crates/lawsynth-dynamics/tests/continuous_integration.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Verifies continuous through integration coverage. |
| 542 | `crates/lawsynth-dynamics/tests/discrete_property.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Verifies discrete through property coverage. |
| 543 | `crates/lawsynth-dynamics/tests/delay_roundtrip.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Verifies delay through roundtrip coverage. |
| 544 | `crates/lawsynth-dynamics/benches/implicit_throughput.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Measures implicit throughput. |
| 545 | `crates/lawsynth-dynamics/benches/control_latency.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Measures control latency. |
| 546 | `crates/lawsynth-dynamics/examples/refine_basic.rs` | rust | lawsynth-dynamics | P2 | Rust Core | Demonstrates basic refine usage. |
| 547 | `crates/lawsynth-dynamics/fixtures/result/minimal.json` | rust | lawsynth-dynamics | P2 | Rust Core | Provides the minimal fixture for result. |
| 548 | `crates/lawsynth-dynamics/fixtures/result/typical.json` | rust | lawsynth-dynamics | P2 | Rust Core | Provides the typical fixture for result. |
| 549 | `crates/lawsynth-dynamics/fixtures/result/edge_case.json` | rust | lawsynth-dynamics | P2 | Rust Core | Provides the edge case fixture for result. |
| 550 | `crates/lawsynth-causal/Cargo.toml` | rust | lawsynth-causal | P3 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-causal. |
| 551 | `crates/lawsynth-causal/README.md` | rust | lawsynth-causal | P3 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-causal. |
| 552 | `crates/lawsynth-causal/src/lib.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements lib for lawsynth-causal. |
| 553 | `crates/lawsynth-causal/src/error.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements error for lawsynth-causal. |
| 554 | `crates/lawsynth-causal/src/config.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements config for lawsynth-causal. |
| 555 | `crates/lawsynth-causal/src/graph.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the graph module for lawsynth-causal. |
| 556 | `crates/lawsynth-causal/src/assumptions.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the assumptions module for lawsynth-causal. |
| 557 | `crates/lawsynth-causal/src/time_order.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the time order module for lawsynth-causal. |
| 558 | `crates/lawsynth-causal/src/lagged.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the lagged module for lawsynth-causal. |
| 559 | `crates/lawsynth-causal/src/granger.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the granger module for lawsynth-causal. |
| 560 | `crates/lawsynth-causal/src/independence.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the independence module for lawsynth-causal. |
| 561 | `crates/lawsynth-causal/src/equivalence.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the equivalence module for lawsynth-causal. |
| 562 | `crates/lawsynth-causal/src/sensitivity.rs` | rust | lawsynth-causal | P3 | Rust Core | Implements the sensitivity module for lawsynth-causal. |
| 563 | `crates/lawsynth-causal/tests/graph_unit.rs` | rust | lawsynth-causal | P3 | Rust Core | Verifies graph through unit coverage. |
| 564 | `crates/lawsynth-causal/tests/assumptions_integration.rs` | rust | lawsynth-causal | P3 | Rust Core | Verifies assumptions through integration coverage. |
| 565 | `crates/lawsynth-causal/tests/time_order_property.rs` | rust | lawsynth-causal | P3 | Rust Core | Verifies time order through property coverage. |
| 566 | `crates/lawsynth-causal/tests/lagged_roundtrip.rs` | rust | lawsynth-causal | P3 | Rust Core | Verifies lagged through roundtrip coverage. |
| 567 | `crates/lawsynth-causal/benches/granger_throughput.rs` | rust | lawsynth-causal | P3 | Rust Core | Measures granger throughput. |
| 568 | `crates/lawsynth-causal/benches/independence_latency.rs` | rust | lawsynth-causal | P3 | Rust Core | Measures independence latency. |
| 569 | `crates/lawsynth-causal/examples/equivalence_basic.rs` | rust | lawsynth-causal | P3 | Rust Core | Demonstrates basic equivalence usage. |
| 570 | `crates/lawsynth-causal/fixtures/sensitivity/minimal.json` | rust | lawsynth-causal | P3 | Rust Core | Provides the minimal fixture for sensitivity. |
| 571 | `crates/lawsynth-causal/fixtures/sensitivity/typical.json` | rust | lawsynth-causal | P3 | Rust Core | Provides the typical fixture for sensitivity. |
| 572 | `crates/lawsynth-causal/fixtures/sensitivity/edge_case.json` | rust | lawsynth-causal | P3 | Rust Core | Provides the edge case fixture for sensitivity. |
| 573 | `crates/lawsynth-regime/Cargo.toml` | rust | lawsynth-regime | P3 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-regime. |
| 574 | `crates/lawsynth-regime/README.md` | rust | lawsynth-regime | P3 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-regime. |
| 575 | `crates/lawsynth-regime/src/lib.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements lib for lawsynth-regime. |
| 576 | `crates/lawsynth-regime/src/error.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements error for lawsynth-regime. |
| 577 | `crates/lawsynth-regime/src/config.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements config for lawsynth-regime. |
| 578 | `crates/lawsynth-regime/src/segmentation.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the segmentation module for lawsynth-regime. |
| 579 | `crates/lawsynth-regime/src/cost.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the cost module for lawsynth-regime. |
| 580 | `crates/lawsynth-regime/src/pelt.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the pelt module for lawsynth-regime. |
| 581 | `crates/lawsynth-regime/src/binary.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the binary module for lawsynth-regime. |
| 582 | `crates/lawsynth-regime/src/bocpd.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the bocpd module for lawsynth-regime. |
| 583 | `crates/lawsynth-regime/src/hmm.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the hmm module for lawsynth-regime. |
| 584 | `crates/lawsynth-regime/src/transitions.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the transitions module for lawsynth-regime. |
| 585 | `crates/lawsynth-regime/src/regime_laws.rs` | rust | lawsynth-regime | P3 | Rust Core | Implements the regime laws module for lawsynth-regime. |
| 586 | `crates/lawsynth-regime/tests/segmentation_unit.rs` | rust | lawsynth-regime | P3 | Rust Core | Verifies segmentation through unit coverage. |
| 587 | `crates/lawsynth-regime/tests/cost_integration.rs` | rust | lawsynth-regime | P3 | Rust Core | Verifies cost through integration coverage. |
| 588 | `crates/lawsynth-regime/tests/pelt_property.rs` | rust | lawsynth-regime | P3 | Rust Core | Verifies pelt through property coverage. |
| 589 | `crates/lawsynth-regime/tests/binary_roundtrip.rs` | rust | lawsynth-regime | P3 | Rust Core | Verifies binary through roundtrip coverage. |
| 590 | `crates/lawsynth-regime/benches/bocpd_throughput.rs` | rust | lawsynth-regime | P3 | Rust Core | Measures bocpd throughput. |
| 591 | `crates/lawsynth-regime/benches/hmm_latency.rs` | rust | lawsynth-regime | P3 | Rust Core | Measures hmm latency. |
| 592 | `crates/lawsynth-regime/examples/transitions_basic.rs` | rust | lawsynth-regime | P3 | Rust Core | Demonstrates basic transitions usage. |
| 593 | `crates/lawsynth-regime/fixtures/regime_laws/minimal.json` | rust | lawsynth-regime | P3 | Rust Core | Provides the minimal fixture for regime laws. |
| 594 | `crates/lawsynth-regime/fixtures/regime_laws/typical.json` | rust | lawsynth-regime | P3 | Rust Core | Provides the typical fixture for regime laws. |
| 595 | `crates/lawsynth-regime/fixtures/regime_laws/edge_case.json` | rust | lawsynth-regime | P3 | Rust Core | Provides the edge case fixture for regime laws. |
| 596 | `crates/lawsynth-uncertainty/Cargo.toml` | rust | lawsynth-uncertainty | P3 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-uncertainty. |
| 597 | `crates/lawsynth-uncertainty/README.md` | rust | lawsynth-uncertainty | P3 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-uncertainty. |
| 598 | `crates/lawsynth-uncertainty/src/lib.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements lib for lawsynth-uncertainty. |
| 599 | `crates/lawsynth-uncertainty/src/error.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements error for lawsynth-uncertainty. |
| 600 | `crates/lawsynth-uncertainty/src/config.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements config for lawsynth-uncertainty. |
| 601 | `crates/lawsynth-uncertainty/src/source.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the source module for lawsynth-uncertainty. |
| 602 | `crates/lawsynth-uncertainty/src/interval.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the interval module for lawsynth-uncertainty. |
| 603 | `crates/lawsynth-uncertainty/src/samples.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the samples module for lawsynth-uncertainty. |
| 604 | `crates/lawsynth-uncertainty/src/covariance.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the covariance module for lawsynth-uncertainty. |
| 605 | `crates/lawsynth-uncertainty/src/profile.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the profile module for lawsynth-uncertainty. |
| 606 | `crates/lawsynth-uncertainty/src/bootstrap.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the bootstrap module for lawsynth-uncertainty. |
| 607 | `crates/lawsynth-uncertainty/src/structural.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the structural module for lawsynth-uncertainty. |
| 608 | `crates/lawsynth-uncertainty/src/propagate.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Implements the propagate module for lawsynth-uncertainty. |
| 609 | `crates/lawsynth-uncertainty/tests/source_unit.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Verifies source through unit coverage. |
| 610 | `crates/lawsynth-uncertainty/tests/interval_integration.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Verifies interval through integration coverage. |
| 611 | `crates/lawsynth-uncertainty/tests/samples_property.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Verifies samples through property coverage. |
| 612 | `crates/lawsynth-uncertainty/tests/covariance_roundtrip.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Verifies covariance through roundtrip coverage. |
| 613 | `crates/lawsynth-uncertainty/benches/profile_throughput.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Measures profile throughput. |
| 614 | `crates/lawsynth-uncertainty/benches/bootstrap_latency.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Measures bootstrap latency. |
| 615 | `crates/lawsynth-uncertainty/examples/structural_basic.rs` | rust | lawsynth-uncertainty | P3 | Rust Core | Demonstrates basic structural usage. |
| 616 | `crates/lawsynth-uncertainty/fixtures/propagate/minimal.json` | rust | lawsynth-uncertainty | P3 | Rust Core | Provides the minimal fixture for propagate. |
| 617 | `crates/lawsynth-uncertainty/fixtures/propagate/typical.json` | rust | lawsynth-uncertainty | P3 | Rust Core | Provides the typical fixture for propagate. |
| 618 | `crates/lawsynth-uncertainty/fixtures/propagate/edge_case.json` | rust | lawsynth-uncertainty | P3 | Rust Core | Provides the edge case fixture for propagate. |
| 619 | `crates/lawsynth-sim/Cargo.toml` | rust | lawsynth-sim | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-sim. |
| 620 | `crates/lawsynth-sim/README.md` | rust | lawsynth-sim | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-sim. |
| 621 | `crates/lawsynth-sim/src/lib.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements lib for lawsynth-sim. |
| 622 | `crates/lawsynth-sim/src/error.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements error for lawsynth-sim. |
| 623 | `crates/lawsynth-sim/src/config.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements config for lawsynth-sim. |
| 624 | `crates/lawsynth-sim/src/state.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the state module for lawsynth-sim. |
| 625 | `crates/lawsynth-sim/src/context.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the context module for lawsynth-sim. |
| 626 | `crates/lawsynth-sim/src/compile.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the compile module for lawsynth-sim. |
| 627 | `crates/lawsynth-sim/src/interpreter.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the interpreter module for lawsynth-sim. |
| 628 | `crates/lawsynth-sim/src/discrete.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the discrete module for lawsynth-sim. |
| 629 | `crates/lawsynth-sim/src/ode.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the ode module for lawsynth-sim. |
| 630 | `crates/lawsynth-sim/src/sde.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the sde module for lawsynth-sim. |
| 631 | `crates/lawsynth-sim/src/hybrid.rs` | rust | lawsynth-sim | P1 | Rust Core | Implements the hybrid module for lawsynth-sim. |
| 632 | `crates/lawsynth-sim/tests/state_unit.rs` | rust | lawsynth-sim | P1 | Rust Core | Verifies state through unit coverage. |
| 633 | `crates/lawsynth-sim/tests/context_integration.rs` | rust | lawsynth-sim | P1 | Rust Core | Verifies context through integration coverage. |
| 634 | `crates/lawsynth-sim/tests/compile_property.rs` | rust | lawsynth-sim | P1 | Rust Core | Verifies compile through property coverage. |
| 635 | `crates/lawsynth-sim/tests/interpreter_roundtrip.rs` | rust | lawsynth-sim | P1 | Rust Core | Verifies interpreter through roundtrip coverage. |
| 636 | `crates/lawsynth-sim/benches/discrete_throughput.rs` | rust | lawsynth-sim | P1 | Rust Core | Measures discrete throughput. |
| 637 | `crates/lawsynth-sim/benches/ode_latency.rs` | rust | lawsynth-sim | P1 | Rust Core | Measures ode latency. |
| 638 | `crates/lawsynth-sim/examples/sde_basic.rs` | rust | lawsynth-sim | P1 | Rust Core | Demonstrates basic sde usage. |
| 639 | `crates/lawsynth-sim/fixtures/hybrid/minimal.json` | rust | lawsynth-sim | P1 | Rust Core | Provides the minimal fixture for hybrid. |
| 640 | `crates/lawsynth-sim/fixtures/hybrid/typical.json` | rust | lawsynth-sim | P1 | Rust Core | Provides the typical fixture for hybrid. |
| 641 | `crates/lawsynth-sim/fixtures/hybrid/edge_case.json` | rust | lawsynth-sim | P1 | Rust Core | Provides the edge case fixture for hybrid. |
| 642 | `crates/lawsynth-score/Cargo.toml` | rust | lawsynth-score | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-score. |
| 643 | `crates/lawsynth-score/README.md` | rust | lawsynth-score | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-score. |
| 644 | `crates/lawsynth-score/src/lib.rs` | rust | lawsynth-score | P2 | Rust Core | Implements lib for lawsynth-score. |
| 645 | `crates/lawsynth-score/src/error.rs` | rust | lawsynth-score | P2 | Rust Core | Implements error for lawsynth-score. |
| 646 | `crates/lawsynth-score/src/config.rs` | rust | lawsynth-score | P2 | Rust Core | Implements config for lawsynth-score. |
| 647 | `crates/lawsynth-score/src/metric.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the metric module for lawsynth-score. |
| 648 | `crates/lawsynth-score/src/fit.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the fit module for lawsynth-score. |
| 649 | `crates/lawsynth-score/src/complexity.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the complexity module for lawsynth-score. |
| 650 | `crates/lawsynth-score/src/stability.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the stability module for lawsynth-score. |
| 651 | `crates/lawsynth-score/src/dimensionality.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the dimensionality module for lawsynth-score. |
| 652 | `crates/lawsynth-score/src/residual.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the residual module for lawsynth-score. |
| 653 | `crates/lawsynth-score/src/pareto.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the pareto module for lawsynth-score. |
| 654 | `crates/lawsynth-score/src/rank.rs` | rust | lawsynth-score | P2 | Rust Core | Implements the rank module for lawsynth-score. |
| 655 | `crates/lawsynth-score/tests/metric_unit.rs` | rust | lawsynth-score | P2 | Rust Core | Verifies metric through unit coverage. |
| 656 | `crates/lawsynth-score/tests/fit_integration.rs` | rust | lawsynth-score | P2 | Rust Core | Verifies fit through integration coverage. |
| 657 | `crates/lawsynth-score/tests/complexity_property.rs` | rust | lawsynth-score | P2 | Rust Core | Verifies complexity through property coverage. |
| 658 | `crates/lawsynth-score/tests/stability_roundtrip.rs` | rust | lawsynth-score | P2 | Rust Core | Verifies stability through roundtrip coverage. |
| 659 | `crates/lawsynth-score/benches/dimensionality_throughput.rs` | rust | lawsynth-score | P2 | Rust Core | Measures dimensionality throughput. |
| 660 | `crates/lawsynth-score/benches/residual_latency.rs` | rust | lawsynth-score | P2 | Rust Core | Measures residual latency. |
| 661 | `crates/lawsynth-score/examples/pareto_basic.rs` | rust | lawsynth-score | P2 | Rust Core | Demonstrates basic pareto usage. |
| 662 | `crates/lawsynth-score/fixtures/rank/minimal.json` | rust | lawsynth-score | P2 | Rust Core | Provides the minimal fixture for rank. |
| 663 | `crates/lawsynth-score/fixtures/rank/typical.json` | rust | lawsynth-score | P2 | Rust Core | Provides the typical fixture for rank. |
| 664 | `crates/lawsynth-score/fixtures/rank/edge_case.json` | rust | lawsynth-score | P2 | Rust Core | Provides the edge case fixture for rank. |
| 665 | `crates/lawsynth-discovery/Cargo.toml` | rust | lawsynth-discovery | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-discovery. |
| 666 | `crates/lawsynth-discovery/README.md` | rust | lawsynth-discovery | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-discovery. |
| 667 | `crates/lawsynth-discovery/src/lib.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements lib for lawsynth-discovery. |
| 668 | `crates/lawsynth-discovery/src/error.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements error for lawsynth-discovery. |
| 669 | `crates/lawsynth-discovery/src/config.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements config for lawsynth-discovery. |
| 670 | `crates/lawsynth-discovery/src/plan.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the plan module for lawsynth-discovery. |
| 671 | `crates/lawsynth-discovery/src/assumptions.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the assumptions module for lawsynth-discovery. |
| 672 | `crates/lawsynth-discovery/src/stage.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the stage module for lawsynth-discovery. |
| 673 | `crates/lawsynth-discovery/src/graph.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the graph module for lawsynth-discovery. |
| 674 | `crates/lawsynth-discovery/src/candidate.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the candidate module for lawsynth-discovery. |
| 675 | `crates/lawsynth-discovery/src/branch.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the branch module for lawsynth-discovery. |
| 676 | `crates/lawsynth-discovery/src/checkpoint.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the checkpoint module for lawsynth-discovery. |
| 677 | `crates/lawsynth-discovery/src/execute.rs` | rust | lawsynth-discovery | P2 | Rust Core | Implements the execute module for lawsynth-discovery. |
| 678 | `crates/lawsynth-discovery/tests/plan_unit.rs` | rust | lawsynth-discovery | P2 | Rust Core | Verifies plan through unit coverage. |
| 679 | `crates/lawsynth-discovery/tests/assumptions_integration.rs` | rust | lawsynth-discovery | P2 | Rust Core | Verifies assumptions through integration coverage. |
| 680 | `crates/lawsynth-discovery/tests/stage_property.rs` | rust | lawsynth-discovery | P2 | Rust Core | Verifies stage through property coverage. |
| 681 | `crates/lawsynth-discovery/tests/graph_roundtrip.rs` | rust | lawsynth-discovery | P2 | Rust Core | Verifies graph through roundtrip coverage. |
| 682 | `crates/lawsynth-discovery/benches/candidate_throughput.rs` | rust | lawsynth-discovery | P2 | Rust Core | Measures candidate throughput. |
| 683 | `crates/lawsynth-discovery/benches/branch_latency.rs` | rust | lawsynth-discovery | P2 | Rust Core | Measures branch latency. |
| 684 | `crates/lawsynth-discovery/examples/checkpoint_basic.rs` | rust | lawsynth-discovery | P2 | Rust Core | Demonstrates basic checkpoint usage. |
| 685 | `crates/lawsynth-discovery/fixtures/execute/minimal.json` | rust | lawsynth-discovery | P2 | Rust Core | Provides the minimal fixture for execute. |
| 686 | `crates/lawsynth-discovery/fixtures/execute/typical.json` | rust | lawsynth-discovery | P2 | Rust Core | Provides the typical fixture for execute. |
| 687 | `crates/lawsynth-discovery/fixtures/execute/edge_case.json` | rust | lawsynth-discovery | P2 | Rust Core | Provides the edge case fixture for execute. |
| 688 | `crates/lawsynth-bundle/Cargo.toml` | rust | lawsynth-bundle | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-bundle. |
| 689 | `crates/lawsynth-bundle/README.md` | rust | lawsynth-bundle | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-bundle. |
| 690 | `crates/lawsynth-bundle/src/lib.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements lib for lawsynth-bundle. |
| 691 | `crates/lawsynth-bundle/src/error.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements error for lawsynth-bundle. |
| 692 | `crates/lawsynth-bundle/src/config.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements config for lawsynth-bundle. |
| 693 | `crates/lawsynth-bundle/src/manifest.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the manifest module for lawsynth-bundle. |
| 694 | `crates/lawsynth-bundle/src/layout.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the layout module for lawsynth-bundle. |
| 695 | `crates/lawsynth-bundle/src/reader.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the reader module for lawsynth-bundle. |
| 696 | `crates/lawsynth-bundle/src/writer.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the writer module for lawsynth-bundle. |
| 697 | `crates/lawsynth-bundle/src/canonical.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the canonical module for lawsynth-bundle. |
| 698 | `crates/lawsynth-bundle/src/checksum.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the checksum module for lawsynth-bundle. |
| 699 | `crates/lawsynth-bundle/src/signature.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the signature module for lawsynth-bundle. |
| 700 | `crates/lawsynth-bundle/src/migration.rs` | rust | lawsynth-bundle | P1 | Rust Core | Implements the migration module for lawsynth-bundle. |
| 701 | `crates/lawsynth-bundle/tests/manifest_unit.rs` | rust | lawsynth-bundle | P1 | Rust Core | Verifies manifest through unit coverage. |
| 702 | `crates/lawsynth-bundle/tests/layout_integration.rs` | rust | lawsynth-bundle | P1 | Rust Core | Verifies layout through integration coverage. |
| 703 | `crates/lawsynth-bundle/tests/reader_property.rs` | rust | lawsynth-bundle | P1 | Rust Core | Verifies reader through property coverage. |
| 704 | `crates/lawsynth-bundle/tests/writer_roundtrip.rs` | rust | lawsynth-bundle | P1 | Rust Core | Verifies writer through roundtrip coverage. |
| 705 | `crates/lawsynth-bundle/benches/canonical_throughput.rs` | rust | lawsynth-bundle | P1 | Rust Core | Measures canonical throughput. |
| 706 | `crates/lawsynth-bundle/benches/checksum_latency.rs` | rust | lawsynth-bundle | P1 | Rust Core | Measures checksum latency. |
| 707 | `crates/lawsynth-bundle/examples/signature_basic.rs` | rust | lawsynth-bundle | P1 | Rust Core | Demonstrates basic signature usage. |
| 708 | `crates/lawsynth-bundle/fixtures/migration/minimal.json` | rust | lawsynth-bundle | P1 | Rust Core | Provides the minimal fixture for migration. |
| 709 | `crates/lawsynth-bundle/fixtures/migration/typical.json` | rust | lawsynth-bundle | P1 | Rust Core | Provides the typical fixture for migration. |
| 710 | `crates/lawsynth-bundle/fixtures/migration/edge_case.json` | rust | lawsynth-bundle | P1 | Rust Core | Provides the edge case fixture for migration. |
| 711 | `crates/lawsynth-store/Cargo.toml` | rust | lawsynth-store | P4 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-store. |
| 712 | `crates/lawsynth-store/README.md` | rust | lawsynth-store | P4 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-store. |
| 713 | `crates/lawsynth-store/src/lib.rs` | rust | lawsynth-store | P4 | Rust Core | Implements lib for lawsynth-store. |
| 714 | `crates/lawsynth-store/src/error.rs` | rust | lawsynth-store | P4 | Rust Core | Implements error for lawsynth-store. |
| 715 | `crates/lawsynth-store/src/config.rs` | rust | lawsynth-store | P4 | Rust Core | Implements config for lawsynth-store. |
| 716 | `crates/lawsynth-store/src/store.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the store module for lawsynth-store. |
| 717 | `crates/lawsynth-store/src/object.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the object module for lawsynth-store. |
| 718 | `crates/lawsynth-store/src/local.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the local module for lawsynth-store. |
| 719 | `crates/lawsynth-store/src/memory.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the memory module for lawsynth-store. |
| 720 | `crates/lawsynth-store/src/s3.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the s3 module for lawsynth-store. |
| 721 | `crates/lawsynth-store/src/multipart.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the multipart module for lawsynth-store. |
| 722 | `crates/lawsynth-store/src/cache.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the cache module for lawsynth-store. |
| 723 | `crates/lawsynth-store/src/gc.rs` | rust | lawsynth-store | P4 | Rust Core | Implements the gc module for lawsynth-store. |
| 724 | `crates/lawsynth-store/tests/store_unit.rs` | rust | lawsynth-store | P4 | Rust Core | Verifies store through unit coverage. |
| 725 | `crates/lawsynth-store/tests/object_integration.rs` | rust | lawsynth-store | P4 | Rust Core | Verifies object through integration coverage. |
| 726 | `crates/lawsynth-store/tests/local_property.rs` | rust | lawsynth-store | P4 | Rust Core | Verifies local through property coverage. |
| 727 | `crates/lawsynth-store/tests/memory_roundtrip.rs` | rust | lawsynth-store | P4 | Rust Core | Verifies memory through roundtrip coverage. |
| 728 | `crates/lawsynth-store/benches/s3_throughput.rs` | rust | lawsynth-store | P4 | Rust Core | Measures s3 throughput. |
| 729 | `crates/lawsynth-store/benches/multipart_latency.rs` | rust | lawsynth-store | P4 | Rust Core | Measures multipart latency. |
| 730 | `crates/lawsynth-store/examples/cache_basic.rs` | rust | lawsynth-store | P4 | Rust Core | Demonstrates basic cache usage. |
| 731 | `crates/lawsynth-store/fixtures/gc/minimal.json` | rust | lawsynth-store | P4 | Rust Core | Provides the minimal fixture for gc. |
| 732 | `crates/lawsynth-store/fixtures/gc/typical.json` | rust | lawsynth-store | P4 | Rust Core | Provides the typical fixture for gc. |
| 733 | `crates/lawsynth-store/fixtures/gc/edge_case.json` | rust | lawsynth-store | P4 | Rust Core | Provides the edge case fixture for gc. |
| 734 | `crates/lawsynth-plugin-api/Cargo.toml` | rust | lawsynth-plugin-api | P5 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-plugin-api. |
| 735 | `crates/lawsynth-plugin-api/README.md` | rust | lawsynth-plugin-api | P5 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-plugin-api. |
| 736 | `crates/lawsynth-plugin-api/src/lib.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements lib for lawsynth-plugin-api. |
| 737 | `crates/lawsynth-plugin-api/src/error.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements error for lawsynth-plugin-api. |
| 738 | `crates/lawsynth-plugin-api/src/config.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements config for lawsynth-plugin-api. |
| 739 | `crates/lawsynth-plugin-api/src/manifest.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the manifest module for lawsynth-plugin-api. |
| 740 | `crates/lawsynth-plugin-api/src/capability.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the capability module for lawsynth-plugin-api. |
| 741 | `crates/lawsynth-plugin-api/src/algorithm.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the algorithm module for lawsynth-plugin-api. |
| 742 | `crates/lawsynth-plugin-api/src/data_adapter.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the data adapter module for lawsynth-plugin-api. |
| 743 | `crates/lawsynth-plugin-api/src/simulator.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the simulator module for lawsynth-plugin-api. |
| 744 | `crates/lawsynth-plugin-api/src/protocol.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the protocol module for lawsynth-plugin-api. |
| 745 | `crates/lawsynth-plugin-api/src/limits.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the limits module for lawsynth-plugin-api. |
| 746 | `crates/lawsynth-plugin-api/src/lifecycle.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Implements the lifecycle module for lawsynth-plugin-api. |
| 747 | `crates/lawsynth-plugin-api/tests/manifest_unit.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Verifies manifest through unit coverage. |
| 748 | `crates/lawsynth-plugin-api/tests/capability_integration.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Verifies capability through integration coverage. |
| 749 | `crates/lawsynth-plugin-api/tests/algorithm_property.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Verifies algorithm through property coverage. |
| 750 | `crates/lawsynth-plugin-api/tests/data_adapter_roundtrip.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Verifies data adapter through roundtrip coverage. |
| 751 | `crates/lawsynth-plugin-api/benches/simulator_throughput.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Measures simulator throughput. |
| 752 | `crates/lawsynth-plugin-api/benches/protocol_latency.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Measures protocol latency. |
| 753 | `crates/lawsynth-plugin-api/examples/limits_basic.rs` | rust | lawsynth-plugin-api | P5 | Rust Core | Demonstrates basic limits usage. |
| 754 | `crates/lawsynth-plugin-api/fixtures/lifecycle/minimal.json` | rust | lawsynth-plugin-api | P5 | Rust Core | Provides the minimal fixture for lifecycle. |
| 755 | `crates/lawsynth-plugin-api/fixtures/lifecycle/typical.json` | rust | lawsynth-plugin-api | P5 | Rust Core | Provides the typical fixture for lifecycle. |
| 756 | `crates/lawsynth-plugin-api/fixtures/lifecycle/edge_case.json` | rust | lawsynth-plugin-api | P5 | Rust Core | Provides the edge case fixture for lifecycle. |
| 757 | `crates/lawsynth-plugin-host/Cargo.toml` | rust | lawsynth-plugin-host | P5 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-plugin-host. |
| 758 | `crates/lawsynth-plugin-host/README.md` | rust | lawsynth-plugin-host | P5 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-plugin-host. |
| 759 | `crates/lawsynth-plugin-host/src/lib.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements lib for lawsynth-plugin-host. |
| 760 | `crates/lawsynth-plugin-host/src/error.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements error for lawsynth-plugin-host. |
| 761 | `crates/lawsynth-plugin-host/src/config.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements config for lawsynth-plugin-host. |
| 762 | `crates/lawsynth-plugin-host/src/discover.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the discover module for lawsynth-plugin-host. |
| 763 | `crates/lawsynth-plugin-host/src/registry.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the registry module for lawsynth-plugin-host. |
| 764 | `crates/lawsynth-plugin-host/src/process.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the process module for lawsynth-plugin-host. |
| 765 | `crates/lawsynth-plugin-host/src/wasi.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the wasi module for lawsynth-plugin-host. |
| 766 | `crates/lawsynth-plugin-host/src/rpc.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the rpc module for lawsynth-plugin-host. |
| 767 | `crates/lawsynth-plugin-host/src/permissions.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the permissions module for lawsynth-plugin-host. |
| 768 | `crates/lawsynth-plugin-host/src/resources.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the resources module for lawsynth-plugin-host. |
| 769 | `crates/lawsynth-plugin-host/src/lifecycle.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Implements the lifecycle module for lawsynth-plugin-host. |
| 770 | `crates/lawsynth-plugin-host/tests/discover_unit.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Verifies discover through unit coverage. |
| 771 | `crates/lawsynth-plugin-host/tests/registry_integration.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Verifies registry through integration coverage. |
| 772 | `crates/lawsynth-plugin-host/tests/process_property.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Verifies process through property coverage. |
| 773 | `crates/lawsynth-plugin-host/tests/wasi_roundtrip.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Verifies wasi through roundtrip coverage. |
| 774 | `crates/lawsynth-plugin-host/benches/rpc_throughput.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Measures rpc throughput. |
| 775 | `crates/lawsynth-plugin-host/benches/permissions_latency.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Measures permissions latency. |
| 776 | `crates/lawsynth-plugin-host/examples/resources_basic.rs` | rust | lawsynth-plugin-host | P5 | Rust Core | Demonstrates basic resources usage. |
| 777 | `crates/lawsynth-plugin-host/fixtures/lifecycle/minimal.json` | rust | lawsynth-plugin-host | P5 | Rust Core | Provides the minimal fixture for lifecycle. |
| 778 | `crates/lawsynth-plugin-host/fixtures/lifecycle/typical.json` | rust | lawsynth-plugin-host | P5 | Rust Core | Provides the typical fixture for lifecycle. |
| 779 | `crates/lawsynth-plugin-host/fixtures/lifecycle/edge_case.json` | rust | lawsynth-plugin-host | P5 | Rust Core | Provides the edge case fixture for lifecycle. |
| 780 | `crates/lawsynth-runner/Cargo.toml` | rust | lawsynth-runner | P4 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-runner. |
| 781 | `crates/lawsynth-runner/README.md` | rust | lawsynth-runner | P4 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-runner. |
| 782 | `crates/lawsynth-runner/src/lib.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements lib for lawsynth-runner. |
| 783 | `crates/lawsynth-runner/src/error.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements error for lawsynth-runner. |
| 784 | `crates/lawsynth-runner/src/config.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements config for lawsynth-runner. |
| 785 | `crates/lawsynth-runner/src/run.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the run module for lawsynth-runner. |
| 786 | `crates/lawsynth-runner/src/process.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the process module for lawsynth-runner. |
| 787 | `crates/lawsynth-runner/src/envelope.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the envelope module for lawsynth-runner. |
| 788 | `crates/lawsynth-runner/src/resources.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the resources module for lawsynth-runner. |
| 789 | `crates/lawsynth-runner/src/limits.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the limits module for lawsynth-runner. |
| 790 | `crates/lawsynth-runner/src/heartbeat.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the heartbeat module for lawsynth-runner. |
| 791 | `crates/lawsynth-runner/src/checkpoint.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the checkpoint module for lawsynth-runner. |
| 792 | `crates/lawsynth-runner/src/cancellation.rs` | rust | lawsynth-runner | P4 | Rust Core | Implements the cancellation module for lawsynth-runner. |
| 793 | `crates/lawsynth-runner/tests/run_unit.rs` | rust | lawsynth-runner | P4 | Rust Core | Verifies run through unit coverage. |
| 794 | `crates/lawsynth-runner/tests/process_integration.rs` | rust | lawsynth-runner | P4 | Rust Core | Verifies process through integration coverage. |
| 795 | `crates/lawsynth-runner/tests/envelope_property.rs` | rust | lawsynth-runner | P4 | Rust Core | Verifies envelope through property coverage. |
| 796 | `crates/lawsynth-runner/tests/resources_roundtrip.rs` | rust | lawsynth-runner | P4 | Rust Core | Verifies resources through roundtrip coverage. |
| 797 | `crates/lawsynth-runner/benches/limits_throughput.rs` | rust | lawsynth-runner | P4 | Rust Core | Measures limits throughput. |
| 798 | `crates/lawsynth-runner/benches/heartbeat_latency.rs` | rust | lawsynth-runner | P4 | Rust Core | Measures heartbeat latency. |
| 799 | `crates/lawsynth-runner/examples/checkpoint_basic.rs` | rust | lawsynth-runner | P4 | Rust Core | Demonstrates basic checkpoint usage. |
| 800 | `crates/lawsynth-runner/fixtures/cancellation/minimal.json` | rust | lawsynth-runner | P4 | Rust Core | Provides the minimal fixture for cancellation. |
| 801 | `crates/lawsynth-runner/fixtures/cancellation/typical.json` | rust | lawsynth-runner | P4 | Rust Core | Provides the typical fixture for cancellation. |
| 802 | `crates/lawsynth-runner/fixtures/cancellation/edge_case.json` | rust | lawsynth-runner | P4 | Rust Core | Provides the edge case fixture for cancellation. |
| 803 | `crates/lawsynth-api-types/Cargo.toml` | rust | lawsynth-api-types | P4 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-api-types. |
| 804 | `crates/lawsynth-api-types/README.md` | rust | lawsynth-api-types | P4 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-api-types. |
| 805 | `crates/lawsynth-api-types/src/lib.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements lib for lawsynth-api-types. |
| 806 | `crates/lawsynth-api-types/src/error.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements error for lawsynth-api-types. |
| 807 | `crates/lawsynth-api-types/src/config.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements config for lawsynth-api-types. |
| 808 | `crates/lawsynth-api-types/src/project.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the project module for lawsynth-api-types. |
| 809 | `crates/lawsynth-api-types/src/dataset.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the dataset module for lawsynth-api-types. |
| 810 | `crates/lawsynth-api-types/src/run.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the run module for lawsynth-api-types. |
| 811 | `crates/lawsynth-api-types/src/world.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the world module for lawsynth-api-types. |
| 812 | `crates/lawsynth-api-types/src/simulation.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the simulation module for lawsynth-api-types. |
| 813 | `crates/lawsynth-api-types/src/artifact.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the artifact module for lawsynth-api-types. |
| 814 | `crates/lawsynth-api-types/src/pagination.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the pagination module for lawsynth-api-types. |
| 815 | `crates/lawsynth-api-types/src/events.rs` | rust | lawsynth-api-types | P4 | Rust Core | Implements the events module for lawsynth-api-types. |
| 816 | `crates/lawsynth-api-types/tests/project_unit.rs` | rust | lawsynth-api-types | P4 | Rust Core | Verifies project through unit coverage. |
| 817 | `crates/lawsynth-api-types/tests/dataset_integration.rs` | rust | lawsynth-api-types | P4 | Rust Core | Verifies dataset through integration coverage. |
| 818 | `crates/lawsynth-api-types/tests/run_property.rs` | rust | lawsynth-api-types | P4 | Rust Core | Verifies run through property coverage. |
| 819 | `crates/lawsynth-api-types/tests/world_roundtrip.rs` | rust | lawsynth-api-types | P4 | Rust Core | Verifies world through roundtrip coverage. |
| 820 | `crates/lawsynth-api-types/benches/simulation_throughput.rs` | rust | lawsynth-api-types | P4 | Rust Core | Measures simulation throughput. |
| 821 | `crates/lawsynth-api-types/benches/artifact_latency.rs` | rust | lawsynth-api-types | P4 | Rust Core | Measures artifact latency. |
| 822 | `crates/lawsynth-api-types/examples/pagination_basic.rs` | rust | lawsynth-api-types | P4 | Rust Core | Demonstrates basic pagination usage. |
| 823 | `crates/lawsynth-api-types/fixtures/events/minimal.json` | rust | lawsynth-api-types | P4 | Rust Core | Provides the minimal fixture for events. |
| 824 | `crates/lawsynth-api-types/fixtures/events/typical.json` | rust | lawsynth-api-types | P4 | Rust Core | Provides the typical fixture for events. |
| 825 | `crates/lawsynth-api-types/fixtures/events/edge_case.json` | rust | lawsynth-api-types | P4 | Rust Core | Provides the edge case fixture for events. |
| 826 | `crates/lawsynth-cli/Cargo.toml` | rust | lawsynth-cli | P2 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-cli. |
| 827 | `crates/lawsynth-cli/README.md` | rust | lawsynth-cli | P2 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-cli. |
| 828 | `crates/lawsynth-cli/src/lib.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements lib for lawsynth-cli. |
| 829 | `crates/lawsynth-cli/src/error.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements error for lawsynth-cli. |
| 830 | `crates/lawsynth-cli/src/config.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements config for lawsynth-cli. |
| 831 | `crates/lawsynth-cli/src/args.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the args module for lawsynth-cli. |
| 832 | `crates/lawsynth-cli/src/output.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the output module for lawsynth-cli. |
| 833 | `crates/lawsynth-cli/src/discover.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the discover module for lawsynth-cli. |
| 834 | `crates/lawsynth-cli/src/inspect.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the inspect module for lawsynth-cli. |
| 835 | `crates/lawsynth-cli/src/profile.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the profile module for lawsynth-cli. |
| 836 | `crates/lawsynth-cli/src/simulate.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the simulate module for lawsynth-cli. |
| 837 | `crates/lawsynth-cli/src/intervene.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the intervene module for lawsynth-cli. |
| 838 | `crates/lawsynth-cli/src/serve.rs` | rust | lawsynth-cli | P2 | Rust Core | Implements the serve module for lawsynth-cli. |
| 839 | `crates/lawsynth-cli/tests/args_unit.rs` | rust | lawsynth-cli | P2 | Rust Core | Verifies args through unit coverage. |
| 840 | `crates/lawsynth-cli/tests/output_integration.rs` | rust | lawsynth-cli | P2 | Rust Core | Verifies output through integration coverage. |
| 841 | `crates/lawsynth-cli/tests/discover_property.rs` | rust | lawsynth-cli | P2 | Rust Core | Verifies discover through property coverage. |
| 842 | `crates/lawsynth-cli/tests/inspect_roundtrip.rs` | rust | lawsynth-cli | P2 | Rust Core | Verifies inspect through roundtrip coverage. |
| 843 | `crates/lawsynth-cli/benches/profile_throughput.rs` | rust | lawsynth-cli | P2 | Rust Core | Measures profile throughput. |
| 844 | `crates/lawsynth-cli/benches/simulate_latency.rs` | rust | lawsynth-cli | P2 | Rust Core | Measures simulate latency. |
| 845 | `crates/lawsynth-cli/examples/intervene_basic.rs` | rust | lawsynth-cli | P2 | Rust Core | Demonstrates basic intervene usage. |
| 846 | `crates/lawsynth-cli/fixtures/serve/minimal.json` | rust | lawsynth-cli | P2 | Rust Core | Provides the minimal fixture for serve. |
| 847 | `crates/lawsynth-cli/fixtures/serve/typical.json` | rust | lawsynth-cli | P2 | Rust Core | Provides the typical fixture for serve. |
| 848 | `crates/lawsynth-cli/fixtures/serve/edge_case.json` | rust | lawsynth-cli | P2 | Rust Core | Provides the edge case fixture for serve. |
| 849 | `crates/lawsynth-python/Cargo.toml` | rust | lawsynth-python | P1 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-python. |
| 850 | `crates/lawsynth-python/README.md` | rust | lawsynth-python | P1 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-python. |
| 851 | `crates/lawsynth-python/src/lib.rs` | rust | lawsynth-python | P1 | Rust Core | Implements lib for lawsynth-python. |
| 852 | `crates/lawsynth-python/src/error.rs` | rust | lawsynth-python | P1 | Rust Core | Implements error for lawsynth-python. |
| 853 | `crates/lawsynth-python/src/config.rs` | rust | lawsynth-python | P1 | Rust Core | Implements config for lawsynth-python. |
| 854 | `crates/lawsynth-python/src/convert.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the convert module for lawsynth-python. |
| 855 | `crates/lawsynth-python/src/py_dataset.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py dataset module for lawsynth-python. |
| 856 | `crates/lawsynth-python/src/py_plan.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py plan module for lawsynth-python. |
| 857 | `crates/lawsynth-python/src/py_run.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py run module for lawsynth-python. |
| 858 | `crates/lawsynth-python/src/py_world.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py world module for lawsynth-python. |
| 859 | `crates/lawsynth-python/src/py_simulation.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py simulation module for lawsynth-python. |
| 860 | `crates/lawsynth-python/src/py_bundle.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py bundle module for lawsynth-python. |
| 861 | `crates/lawsynth-python/src/py_events.rs` | rust | lawsynth-python | P1 | Rust Core | Implements the py events module for lawsynth-python. |
| 862 | `crates/lawsynth-python/tests/convert_unit.rs` | rust | lawsynth-python | P1 | Rust Core | Verifies convert through unit coverage. |
| 863 | `crates/lawsynth-python/tests/py_dataset_integration.rs` | rust | lawsynth-python | P1 | Rust Core | Verifies py dataset through integration coverage. |
| 864 | `crates/lawsynth-python/tests/py_plan_property.rs` | rust | lawsynth-python | P1 | Rust Core | Verifies py plan through property coverage. |
| 865 | `crates/lawsynth-python/tests/py_run_roundtrip.rs` | rust | lawsynth-python | P1 | Rust Core | Verifies py run through roundtrip coverage. |
| 866 | `crates/lawsynth-python/benches/py_world_throughput.rs` | rust | lawsynth-python | P1 | Rust Core | Measures py world throughput. |
| 867 | `crates/lawsynth-python/benches/py_simulation_latency.rs` | rust | lawsynth-python | P1 | Rust Core | Measures py simulation latency. |
| 868 | `crates/lawsynth-python/examples/py_bundle_basic.rs` | rust | lawsynth-python | P1 | Rust Core | Demonstrates basic py bundle usage. |
| 869 | `crates/lawsynth-python/fixtures/py_events/minimal.json` | rust | lawsynth-python | P1 | Rust Core | Provides the minimal fixture for py events. |
| 870 | `crates/lawsynth-python/fixtures/py_events/typical.json` | rust | lawsynth-python | P1 | Rust Core | Provides the typical fixture for py events. |
| 871 | `crates/lawsynth-python/fixtures/py_events/edge_case.json` | rust | lawsynth-python | P1 | Rust Core | Provides the edge case fixture for py events. |
| 872 | `crates/lawsynth-wasm/Cargo.toml` | rust | lawsynth-wasm | P3 | Rust Core | Declares the build, dependencies, and package metadata for lawsynth-wasm. |
| 873 | `crates/lawsynth-wasm/README.md` | rust | lawsynth-wasm | P3 | Rust Core | Documents the purpose, boundaries, and usage of lawsynth-wasm. |
| 874 | `crates/lawsynth-wasm/src/lib.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements lib for lawsynth-wasm. |
| 875 | `crates/lawsynth-wasm/src/error.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements error for lawsynth-wasm. |
| 876 | `crates/lawsynth-wasm/src/config.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements config for lawsynth-wasm. |
| 877 | `crates/lawsynth-wasm/src/world.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the world module for lawsynth-wasm. |
| 878 | `crates/lawsynth-wasm/src/expression.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the expression module for lawsynth-wasm. |
| 879 | `crates/lawsynth-wasm/src/bundle.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the bundle module for lawsynth-wasm. |
| 880 | `crates/lawsynth-wasm/src/simulate.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the simulate module for lawsynth-wasm. |
| 881 | `crates/lawsynth-wasm/src/trajectory.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the trajectory module for lawsynth-wasm. |
| 882 | `crates/lawsynth-wasm/src/events.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the events module for lawsynth-wasm. |
| 883 | `crates/lawsynth-wasm/src/memory.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the memory module for lawsynth-wasm. |
| 884 | `crates/lawsynth-wasm/src/errors.rs` | rust | lawsynth-wasm | P3 | Rust Core | Implements the errors module for lawsynth-wasm. |
| 885 | `crates/lawsynth-wasm/tests/world_unit.rs` | rust | lawsynth-wasm | P3 | Rust Core | Verifies world through unit coverage. |
| 886 | `crates/lawsynth-wasm/tests/expression_integration.rs` | rust | lawsynth-wasm | P3 | Rust Core | Verifies expression through integration coverage. |
| 887 | `crates/lawsynth-wasm/tests/bundle_property.rs` | rust | lawsynth-wasm | P3 | Rust Core | Verifies bundle through property coverage. |
| 888 | `crates/lawsynth-wasm/tests/simulate_roundtrip.rs` | rust | lawsynth-wasm | P3 | Rust Core | Verifies simulate through roundtrip coverage. |
| 889 | `crates/lawsynth-wasm/benches/trajectory_throughput.rs` | rust | lawsynth-wasm | P3 | Rust Core | Measures trajectory throughput. |
| 890 | `crates/lawsynth-wasm/benches/events_latency.rs` | rust | lawsynth-wasm | P3 | Rust Core | Measures events latency. |
| 891 | `crates/lawsynth-wasm/examples/memory_basic.rs` | rust | lawsynth-wasm | P3 | Rust Core | Demonstrates basic memory usage. |
| 892 | `crates/lawsynth-wasm/fixtures/errors/minimal.json` | rust | lawsynth-wasm | P3 | Rust Core | Provides the minimal fixture for errors. |
| 893 | `crates/lawsynth-wasm/fixtures/errors/typical.json` | rust | lawsynth-wasm | P3 | Rust Core | Provides the typical fixture for errors. |
| 894 | `crates/lawsynth-wasm/fixtures/errors/edge_case.json` | rust | lawsynth-wasm | P3 | Rust Core | Provides the edge case fixture for errors. |
| 895 | `python/lawsynth/pyproject.toml` | python | lawsynth | P2 | Python SDK | Declares the build, dependencies, and package metadata for lawsynth. |
| 896 | `python/lawsynth/README.md` | python | lawsynth | P2 | Python SDK | Documents the purpose, boundaries, and usage of lawsynth. |
| 897 | `python/lawsynth/LICENSE` | python | lawsynth | P2 | Python SDK | Declares legal terms and notices for lawsynth. |
| 898 | `python/lawsynth/src/lawsynth/__init__.py` | python | lawsynth | P2 | Python SDK | Implements   init   for lawsynth. |
| 899 | `python/lawsynth/src/lawsynth/py.typed` | python | lawsynth | P2 | Python SDK | Provides py for lawsynth. |
| 900 | `python/lawsynth/src/lawsynth/_version.py` | python | lawsynth | P2 | Python SDK | Implements  version for lawsynth. |
| 901 | `python/lawsynth/src/lawsynth/errors.py` | python | lawsynth | P2 | Python SDK | Implements errors for lawsynth. |
| 902 | `python/lawsynth/src/lawsynth/config.py` | python | lawsynth | P2 | Python SDK | Implements config for lawsynth. |
| 903 | `python/lawsynth/src/lawsynth/dataset.py` | python | lawsynth | P2 | Python SDK | Implements dataset for lawsynth. |
| 904 | `python/lawsynth/src/lawsynth/variable.py` | python | lawsynth | P2 | Python SDK | Implements variable for lawsynth. |
| 905 | `python/lawsynth/src/lawsynth/units.py` | python | lawsynth | P2 | Python SDK | Implements units for lawsynth. |
| 906 | `python/lawsynth/src/lawsynth/assumptions.py` | python | lawsynth | P2 | Python SDK | Implements assumptions for lawsynth. |
| 907 | `python/lawsynth/src/lawsynth/plan.py` | python | lawsynth | P2 | Python SDK | Implements plan for lawsynth. |
| 908 | `python/lawsynth/src/lawsynth/run.py` | python | lawsynth | P2 | Python SDK | Implements run for lawsynth. |
| 909 | `python/lawsynth/src/lawsynth/candidate.py` | python | lawsynth | P2 | Python SDK | Implements candidate for lawsynth. |
| 910 | `python/lawsynth/src/lawsynth/frontier.py` | python | lawsynth | P2 | Python SDK | Implements frontier for lawsynth. |
| 911 | `python/lawsynth/src/lawsynth/equation.py` | python | lawsynth | P2 | Python SDK | Implements equation for lawsynth. |
| 912 | `python/lawsynth/src/lawsynth/graph.py` | python | lawsynth | P2 | Python SDK | Implements graph for lawsynth. |
| 913 | `python/lawsynth/src/lawsynth/regime.py` | python | lawsynth | P2 | Python SDK | Implements regime for lawsynth. |
| 914 | `python/lawsynth/src/lawsynth/event.py` | python | lawsynth | P2 | Python SDK | Implements event for lawsynth. |
| 915 | `python/lawsynth/src/lawsynth/uncertainty.py` | python | lawsynth | P2 | Python SDK | Implements uncertainty for lawsynth. |
| 916 | `python/lawsynth/src/lawsynth/world.py` | python | lawsynth | P2 | Python SDK | Implements world for lawsynth. |
| 917 | `python/lawsynth/src/lawsynth/intervention.py` | python | lawsynth | P2 | Python SDK | Implements intervention for lawsynth. |
| 918 | `python/lawsynth/src/lawsynth/scenario.py` | python | lawsynth | P2 | Python SDK | Implements scenario for lawsynth. |
| 919 | `python/lawsynth/src/lawsynth/trajectory.py` | python | lawsynth | P2 | Python SDK | Implements trajectory for lawsynth. |
| 920 | `python/lawsynth/src/lawsynth/bundle.py` | python | lawsynth | P2 | Python SDK | Implements bundle for lawsynth. |
| 921 | `python/lawsynth/src/lawsynth/discover.py` | python | lawsynth | P2 | Python SDK | Implements discover for lawsynth. |
| 922 | `python/lawsynth/src/lawsynth/simulate.py` | python | lawsynth | P2 | Python SDK | Implements simulate for lawsynth. |
| 923 | `python/lawsynth/src/lawsynth/inspect.py` | python | lawsynth | P2 | Python SDK | Implements inspect for lawsynth. |
| 924 | `python/lawsynth/tests/conftest.py` | python | lawsynth | P2 | Python SDK | Defines shared test fixtures for lawsynth. |
| 925 | `python/lawsynth/tests/test_dataset.py` | python | lawsynth | P2 | Python SDK | Verifies dataset behavior in lawsynth. |
| 926 | `python/lawsynth/tests/test_variable.py` | python | lawsynth | P2 | Python SDK | Verifies variable behavior in lawsynth. |
| 927 | `python/lawsynth/tests/test_units.py` | python | lawsynth | P2 | Python SDK | Verifies units behavior in lawsynth. |
| 928 | `python/lawsynth/tests/test_assumptions.py` | python | lawsynth | P2 | Python SDK | Verifies assumptions behavior in lawsynth. |
| 929 | `python/lawsynth/tests/test_plan.py` | python | lawsynth | P2 | Python SDK | Verifies plan behavior in lawsynth. |
| 930 | `python/lawsynth/tests/test_run.py` | python | lawsynth | P2 | Python SDK | Verifies run behavior in lawsynth. |
| 931 | `python/lawsynth/tests/test_candidate.py` | python | lawsynth | P2 | Python SDK | Verifies candidate behavior in lawsynth. |
| 932 | `python/lawsynth/tests/test_frontier.py` | python | lawsynth | P2 | Python SDK | Verifies frontier behavior in lawsynth. |
| 933 | `python/lawsynth/tests/test_equation.py` | python | lawsynth | P2 | Python SDK | Verifies equation behavior in lawsynth. |
| 934 | `python/lawsynth/tests/test_graph.py` | python | lawsynth | P2 | Python SDK | Verifies graph behavior in lawsynth. |
| 935 | `python/lawsynth/tests/test_regime.py` | python | lawsynth | P2 | Python SDK | Verifies regime behavior in lawsynth. |
| 936 | `python/lawsynth/tests/test_event.py` | python | lawsynth | P2 | Python SDK | Verifies event behavior in lawsynth. |
| 937 | `python/lawsynth/tests/test_uncertainty.py` | python | lawsynth | P2 | Python SDK | Verifies uncertainty behavior in lawsynth. |
| 938 | `python/lawsynth/tests/test_world.py` | python | lawsynth | P2 | Python SDK | Verifies world behavior in lawsynth. |
| 939 | `python/lawsynth/tests/test_intervention.py` | python | lawsynth | P2 | Python SDK | Verifies intervention behavior in lawsynth. |
| 940 | `python/lawsynth/tests/test_scenario.py` | python | lawsynth | P2 | Python SDK | Verifies scenario behavior in lawsynth. |
| 941 | `python/lawsynth/tests/test_trajectory.py` | python | lawsynth | P2 | Python SDK | Verifies trajectory behavior in lawsynth. |
| 942 | `python/lawsynth/tests/test_bundle.py` | python | lawsynth | P2 | Python SDK | Verifies bundle behavior in lawsynth. |
| 943 | `python/lawsynth/tests/test_discover.py` | python | lawsynth | P2 | Python SDK | Verifies discover behavior in lawsynth. |
| 944 | `python/lawsynth/tests/test_simulate.py` | python | lawsynth | P2 | Python SDK | Verifies simulate behavior in lawsynth. |
| 945 | `python/lawsynth/tests/test_inspect.py` | python | lawsynth | P2 | Python SDK | Verifies inspect behavior in lawsynth. |
| 946 | `python/lawsynth/fixtures/dataset/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample dataset fixture. |
| 947 | `python/lawsynth/fixtures/variable/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample variable fixture. |
| 948 | `python/lawsynth/fixtures/units/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample units fixture. |
| 949 | `python/lawsynth/fixtures/assumptions/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample assumptions fixture. |
| 950 | `python/lawsynth/fixtures/plan/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample plan fixture. |
| 951 | `python/lawsynth/fixtures/run/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample run fixture. |
| 952 | `python/lawsynth/fixtures/candidate/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample candidate fixture. |
| 953 | `python/lawsynth/fixtures/frontier/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample frontier fixture. |
| 954 | `python/lawsynth/fixtures/equation/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample equation fixture. |
| 955 | `python/lawsynth/fixtures/graph/sample.json` | python | lawsynth | P2 | Python SDK | Provides a sample graph fixture. |
| 956 | `python/lawsynth/docs/event.md` | python | lawsynth | P2 | Python SDK | Documents event in lawsynth. |
| 957 | `python/lawsynth/docs/uncertainty.md` | python | lawsynth | P2 | Python SDK | Documents uncertainty in lawsynth. |
| 958 | `python/lawsynth/docs/world.md` | python | lawsynth | P2 | Python SDK | Documents world in lawsynth. |
| 959 | `python/lawsynth/docs/intervention.md` | python | lawsynth | P2 | Python SDK | Documents intervention in lawsynth. |
| 960 | `python/lawsynth/docs/scenario.md` | python | lawsynth | P2 | Python SDK | Documents scenario in lawsynth. |
| 961 | `python/lawsynth/docs/trajectory.md` | python | lawsynth | P2 | Python SDK | Documents trajectory in lawsynth. |
| 962 | `python/lawsynth/docs/bundle.md` | python | lawsynth | P2 | Python SDK | Documents bundle in lawsynth. |
| 963 | `python/lawsynth/docs/discover.md` | python | lawsynth | P2 | Python SDK | Documents discover in lawsynth. |
| 964 | `python/lawsynth/docs/simulate.md` | python | lawsynth | P2 | Python SDK | Documents simulate in lawsynth. |
| 965 | `python/lawsynth/docs/inspect.md` | python | lawsynth | P2 | Python SDK | Documents inspect in lawsynth. |
| 966 | `python/lawsynth-server/pyproject.toml` | python | lawsynth-server | P4 | Python SDK | Declares the build, dependencies, and package metadata for lawsynth-server. |
| 967 | `python/lawsynth-server/README.md` | python | lawsynth-server | P4 | Python SDK | Documents the purpose, boundaries, and usage of lawsynth-server. |
| 968 | `python/lawsynth-server/LICENSE` | python | lawsynth-server | P4 | Python SDK | Declares legal terms and notices for lawsynth-server. |
| 969 | `python/lawsynth-server/src/lawsynth_server/__init__.py` | python | lawsynth-server | P4 | Python SDK | Implements   init   for lawsynth-server. |
| 970 | `python/lawsynth-server/src/lawsynth_server/py.typed` | python | lawsynth-server | P4 | Python SDK | Provides py for lawsynth-server. |
| 971 | `python/lawsynth-server/src/lawsynth_server/_version.py` | python | lawsynth-server | P4 | Python SDK | Implements  version for lawsynth-server. |
| 972 | `python/lawsynth-server/src/lawsynth_server/errors.py` | python | lawsynth-server | P4 | Python SDK | Implements errors for lawsynth-server. |
| 973 | `python/lawsynth-server/src/lawsynth_server/config.py` | python | lawsynth-server | P4 | Python SDK | Implements config for lawsynth-server. |
| 974 | `python/lawsynth-server/src/lawsynth_server/app.py` | python | lawsynth-server | P4 | Python SDK | Implements app for lawsynth-server. |
| 975 | `python/lawsynth-server/src/lawsynth_server/lifespan.py` | python | lawsynth-server | P4 | Python SDK | Implements lifespan for lawsynth-server. |
| 976 | `python/lawsynth-server/src/lawsynth_server/settings.py` | python | lawsynth-server | P4 | Python SDK | Implements settings for lawsynth-server. |
| 977 | `python/lawsynth-server/src/lawsynth_server/dependencies.py` | python | lawsynth-server | P4 | Python SDK | Implements dependencies for lawsynth-server. |
| 978 | `python/lawsynth-server/src/lawsynth_server/auth.py` | python | lawsynth-server | P4 | Python SDK | Implements auth for lawsynth-server. |
| 979 | `python/lawsynth-server/src/lawsynth_server/pagination.py` | python | lawsynth-server | P4 | Python SDK | Implements pagination for lawsynth-server. |
| 980 | `python/lawsynth-server/src/lawsynth_server/idempotency.py` | python | lawsynth-server | P4 | Python SDK | Implements idempotency for lawsynth-server. |
| 981 | `python/lawsynth-server/src/lawsynth_server/events.py` | python | lawsynth-server | P4 | Python SDK | Implements events for lawsynth-server. |
| 982 | `python/lawsynth-server/src/lawsynth_server/projects.py` | python | lawsynth-server | P4 | Python SDK | Implements projects for lawsynth-server. |
| 983 | `python/lawsynth-server/src/lawsynth_server/datasets.py` | python | lawsynth-server | P4 | Python SDK | Implements datasets for lawsynth-server. |
| 984 | `python/lawsynth-server/src/lawsynth_server/runs.py` | python | lawsynth-server | P4 | Python SDK | Implements runs for lawsynth-server. |
| 985 | `python/lawsynth-server/src/lawsynth_server/worlds.py` | python | lawsynth-server | P4 | Python SDK | Implements worlds for lawsynth-server. |
| 986 | `python/lawsynth-server/src/lawsynth_server/simulations.py` | python | lawsynth-server | P4 | Python SDK | Implements simulations for lawsynth-server. |
| 987 | `python/lawsynth-server/src/lawsynth_server/artifacts.py` | python | lawsynth-server | P4 | Python SDK | Implements artifacts for lawsynth-server. |
| 988 | `python/lawsynth-server/src/lawsynth_server/repositories.py` | python | lawsynth-server | P4 | Python SDK | Implements repositories for lawsynth-server. |
| 989 | `python/lawsynth-server/src/lawsynth_server/database.py` | python | lawsynth-server | P4 | Python SDK | Implements database for lawsynth-server. |
| 990 | `python/lawsynth-server/src/lawsynth_server/storage.py` | python | lawsynth-server | P4 | Python SDK | Implements storage for lawsynth-server. |
| 991 | `python/lawsynth-server/src/lawsynth_server/middleware.py` | python | lawsynth-server | P4 | Python SDK | Implements middleware for lawsynth-server. |
| 992 | `python/lawsynth-server/src/lawsynth_server/telemetry.py` | python | lawsynth-server | P4 | Python SDK | Implements telemetry for lawsynth-server. |
| 993 | `python/lawsynth-server/src/lawsynth_server/health.py` | python | lawsynth-server | P4 | Python SDK | Implements health for lawsynth-server. |
| 994 | `python/lawsynth-server/src/lawsynth_server/errors_api.py` | python | lawsynth-server | P4 | Python SDK | Implements errors api for lawsynth-server. |
| 995 | `python/lawsynth-server/tests/conftest.py` | python | lawsynth-server | P4 | Python SDK | Defines shared test fixtures for lawsynth-server. |
| 996 | `python/lawsynth-server/tests/test_app.py` | python | lawsynth-server | P4 | Python SDK | Verifies app behavior in lawsynth-server. |
| 997 | `python/lawsynth-server/tests/test_lifespan.py` | python | lawsynth-server | P4 | Python SDK | Verifies lifespan behavior in lawsynth-server. |
| 998 | `python/lawsynth-server/tests/test_settings.py` | python | lawsynth-server | P4 | Python SDK | Verifies settings behavior in lawsynth-server. |
| 999 | `python/lawsynth-server/tests/test_dependencies.py` | python | lawsynth-server | P4 | Python SDK | Verifies dependencies behavior in lawsynth-server. |
| 1,000 | `python/lawsynth-server/tests/test_auth.py` | python | lawsynth-server | P4 | Python SDK | Verifies auth behavior in lawsynth-server. |
| 1,001 | `python/lawsynth-server/tests/test_pagination.py` | python | lawsynth-server | P4 | Python SDK | Verifies pagination behavior in lawsynth-server. |
| 1,002 | `python/lawsynth-server/tests/test_idempotency.py` | python | lawsynth-server | P4 | Python SDK | Verifies idempotency behavior in lawsynth-server. |
| 1,003 | `python/lawsynth-server/tests/test_events.py` | python | lawsynth-server | P4 | Python SDK | Verifies events behavior in lawsynth-server. |
| 1,004 | `python/lawsynth-server/tests/test_projects.py` | python | lawsynth-server | P4 | Python SDK | Verifies projects behavior in lawsynth-server. |
| 1,005 | `python/lawsynth-server/tests/test_datasets.py` | python | lawsynth-server | P4 | Python SDK | Verifies datasets behavior in lawsynth-server. |
| 1,006 | `python/lawsynth-server/tests/test_runs.py` | python | lawsynth-server | P4 | Python SDK | Verifies runs behavior in lawsynth-server. |
| 1,007 | `python/lawsynth-server/tests/test_worlds.py` | python | lawsynth-server | P4 | Python SDK | Verifies worlds behavior in lawsynth-server. |
| 1,008 | `python/lawsynth-server/tests/test_simulations.py` | python | lawsynth-server | P4 | Python SDK | Verifies simulations behavior in lawsynth-server. |
| 1,009 | `python/lawsynth-server/tests/test_artifacts.py` | python | lawsynth-server | P4 | Python SDK | Verifies artifacts behavior in lawsynth-server. |
| 1,010 | `python/lawsynth-server/tests/test_repositories.py` | python | lawsynth-server | P4 | Python SDK | Verifies repositories behavior in lawsynth-server. |
| 1,011 | `python/lawsynth-server/tests/test_database.py` | python | lawsynth-server | P4 | Python SDK | Verifies database behavior in lawsynth-server. |
| 1,012 | `python/lawsynth-server/tests/test_storage.py` | python | lawsynth-server | P4 | Python SDK | Verifies storage behavior in lawsynth-server. |
| 1,013 | `python/lawsynth-server/tests/test_middleware.py` | python | lawsynth-server | P4 | Python SDK | Verifies middleware behavior in lawsynth-server. |
| 1,014 | `python/lawsynth-server/tests/test_telemetry.py` | python | lawsynth-server | P4 | Python SDK | Verifies telemetry behavior in lawsynth-server. |
| 1,015 | `python/lawsynth-server/tests/test_health.py` | python | lawsynth-server | P4 | Python SDK | Verifies health behavior in lawsynth-server. |
| 1,016 | `python/lawsynth-server/tests/test_errors_api.py` | python | lawsynth-server | P4 | Python SDK | Verifies errors api behavior in lawsynth-server. |
| 1,017 | `python/lawsynth-server/fixtures/app/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample app fixture. |
| 1,018 | `python/lawsynth-server/fixtures/lifespan/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample lifespan fixture. |
| 1,019 | `python/lawsynth-server/fixtures/settings/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample settings fixture. |
| 1,020 | `python/lawsynth-server/fixtures/dependencies/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample dependencies fixture. |
| 1,021 | `python/lawsynth-server/fixtures/auth/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample auth fixture. |
| 1,022 | `python/lawsynth-server/fixtures/pagination/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample pagination fixture. |
| 1,023 | `python/lawsynth-server/fixtures/idempotency/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample idempotency fixture. |
| 1,024 | `python/lawsynth-server/fixtures/events/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample events fixture. |
| 1,025 | `python/lawsynth-server/fixtures/projects/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample projects fixture. |
| 1,026 | `python/lawsynth-server/fixtures/datasets/sample.json` | python | lawsynth-server | P4 | Python SDK | Provides a sample datasets fixture. |
| 1,027 | `python/lawsynth-server/docs/worlds.md` | python | lawsynth-server | P4 | Python SDK | Documents worlds in lawsynth-server. |
| 1,028 | `python/lawsynth-server/docs/simulations.md` | python | lawsynth-server | P4 | Python SDK | Documents simulations in lawsynth-server. |
| 1,029 | `python/lawsynth-server/docs/artifacts.md` | python | lawsynth-server | P4 | Python SDK | Documents artifacts in lawsynth-server. |
| 1,030 | `python/lawsynth-server/docs/repositories.md` | python | lawsynth-server | P4 | Python SDK | Documents repositories in lawsynth-server. |
| 1,031 | `python/lawsynth-server/docs/database.md` | python | lawsynth-server | P4 | Python SDK | Documents database in lawsynth-server. |
| 1,032 | `python/lawsynth-server/docs/storage.md` | python | lawsynth-server | P4 | Python SDK | Documents storage in lawsynth-server. |
| 1,033 | `python/lawsynth-server/docs/middleware.md` | python | lawsynth-server | P4 | Python SDK | Documents middleware in lawsynth-server. |
| 1,034 | `python/lawsynth-server/docs/telemetry.md` | python | lawsynth-server | P4 | Python SDK | Documents telemetry in lawsynth-server. |
| 1,035 | `python/lawsynth-server/docs/health.md` | python | lawsynth-server | P4 | Python SDK | Documents health in lawsynth-server. |
| 1,036 | `python/lawsynth-server/docs/errors_api.md` | python | lawsynth-server | P4 | Python SDK | Documents errors api in lawsynth-server. |
| 1,037 | `python/lawsynth-connectors/pyproject.toml` | python | lawsynth-connectors | P4 | Python SDK | Declares the build, dependencies, and package metadata for lawsynth-connectors. |
| 1,038 | `python/lawsynth-connectors/README.md` | python | lawsynth-connectors | P4 | Python SDK | Documents the purpose, boundaries, and usage of lawsynth-connectors. |
| 1,039 | `python/lawsynth-connectors/LICENSE` | python | lawsynth-connectors | P4 | Python SDK | Declares legal terms and notices for lawsynth-connectors. |
| 1,040 | `python/lawsynth-connectors/src/lawsynth_connectors/__init__.py` | python | lawsynth-connectors | P4 | Python SDK | Implements   init   for lawsynth-connectors. |
| 1,041 | `python/lawsynth-connectors/src/lawsynth_connectors/py.typed` | python | lawsynth-connectors | P4 | Python SDK | Provides py for lawsynth-connectors. |
| 1,042 | `python/lawsynth-connectors/src/lawsynth_connectors/_version.py` | python | lawsynth-connectors | P4 | Python SDK | Implements  version for lawsynth-connectors. |
| 1,043 | `python/lawsynth-connectors/src/lawsynth_connectors/errors.py` | python | lawsynth-connectors | P4 | Python SDK | Implements errors for lawsynth-connectors. |
| 1,044 | `python/lawsynth-connectors/src/lawsynth_connectors/config.py` | python | lawsynth-connectors | P4 | Python SDK | Implements config for lawsynth-connectors. |
| 1,045 | `python/lawsynth-connectors/src/lawsynth_connectors/base.py` | python | lawsynth-connectors | P4 | Python SDK | Implements base for lawsynth-connectors. |
| 1,046 | `python/lawsynth-connectors/src/lawsynth_connectors/registry.py` | python | lawsynth-connectors | P4 | Python SDK | Implements registry for lawsynth-connectors. |
| 1,047 | `python/lawsynth-connectors/src/lawsynth_connectors/filesystem.py` | python | lawsynth-connectors | P4 | Python SDK | Implements filesystem for lawsynth-connectors. |
| 1,048 | `python/lawsynth-connectors/src/lawsynth_connectors/http.py` | python | lawsynth-connectors | P4 | Python SDK | Implements http for lawsynth-connectors. |
| 1,049 | `python/lawsynth-connectors/src/lawsynth_connectors/sql.py` | python | lawsynth-connectors | P4 | Python SDK | Implements sql for lawsynth-connectors. |
| 1,050 | `python/lawsynth-connectors/src/lawsynth_connectors/duckdb.py` | python | lawsynth-connectors | P4 | Python SDK | Implements duckdb for lawsynth-connectors. |
| 1,051 | `python/lawsynth-connectors/src/lawsynth_connectors/postgres.py` | python | lawsynth-connectors | P4 | Python SDK | Implements postgres for lawsynth-connectors. |
| 1,052 | `python/lawsynth-connectors/src/lawsynth_connectors/s3.py` | python | lawsynth-connectors | P4 | Python SDK | Implements s3 for lawsynth-connectors. |
| 1,053 | `python/lawsynth-connectors/src/lawsynth_connectors/delta.py` | python | lawsynth-connectors | P4 | Python SDK | Implements delta for lawsynth-connectors. |
| 1,054 | `python/lawsynth-connectors/src/lawsynth_connectors/iceberg.py` | python | lawsynth-connectors | P4 | Python SDK | Implements iceberg for lawsynth-connectors. |
| 1,055 | `python/lawsynth-connectors/src/lawsynth_connectors/pandas.py` | python | lawsynth-connectors | P4 | Python SDK | Implements pandas for lawsynth-connectors. |
| 1,056 | `python/lawsynth-connectors/src/lawsynth_connectors/polars.py` | python | lawsynth-connectors | P4 | Python SDK | Implements polars for lawsynth-connectors. |
| 1,057 | `python/lawsynth-connectors/src/lawsynth_connectors/arrow.py` | python | lawsynth-connectors | P4 | Python SDK | Implements arrow for lawsynth-connectors. |
| 1,058 | `python/lawsynth-connectors/src/lawsynth_connectors/xarray.py` | python | lawsynth-connectors | P4 | Python SDK | Implements xarray for lawsynth-connectors. |
| 1,059 | `python/lawsynth-connectors/src/lawsynth_connectors/kafka.py` | python | lawsynth-connectors | P4 | Python SDK | Implements kafka for lawsynth-connectors. |
| 1,060 | `python/lawsynth-connectors/src/lawsynth_connectors/validation.py` | python | lawsynth-connectors | P4 | Python SDK | Implements validation for lawsynth-connectors. |
| 1,061 | `python/lawsynth-connectors/src/lawsynth_connectors/credentials.py` | python | lawsynth-connectors | P4 | Python SDK | Implements credentials for lawsynth-connectors. |
| 1,062 | `python/lawsynth-connectors/src/lawsynth_connectors/pagination.py` | python | lawsynth-connectors | P4 | Python SDK | Implements pagination for lawsynth-connectors. |
| 1,063 | `python/lawsynth-connectors/src/lawsynth_connectors/partitioning.py` | python | lawsynth-connectors | P4 | Python SDK | Implements partitioning for lawsynth-connectors. |
| 1,064 | `python/lawsynth-connectors/src/lawsynth_connectors/fingerprints.py` | python | lawsynth-connectors | P4 | Python SDK | Implements fingerprints for lawsynth-connectors. |
| 1,065 | `python/lawsynth-connectors/src/lawsynth_connectors/errors_connector.py` | python | lawsynth-connectors | P4 | Python SDK | Implements errors connector for lawsynth-connectors. |
| 1,066 | `python/lawsynth-connectors/tests/conftest.py` | python | lawsynth-connectors | P4 | Python SDK | Defines shared test fixtures for lawsynth-connectors. |
| 1,067 | `python/lawsynth-connectors/tests/test_base.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies base behavior in lawsynth-connectors. |
| 1,068 | `python/lawsynth-connectors/tests/test_registry.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies registry behavior in lawsynth-connectors. |
| 1,069 | `python/lawsynth-connectors/tests/test_filesystem.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies filesystem behavior in lawsynth-connectors. |
| 1,070 | `python/lawsynth-connectors/tests/test_http.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies http behavior in lawsynth-connectors. |
| 1,071 | `python/lawsynth-connectors/tests/test_sql.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies sql behavior in lawsynth-connectors. |
| 1,072 | `python/lawsynth-connectors/tests/test_duckdb.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies duckdb behavior in lawsynth-connectors. |
| 1,073 | `python/lawsynth-connectors/tests/test_postgres.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies postgres behavior in lawsynth-connectors. |
| 1,074 | `python/lawsynth-connectors/tests/test_s3.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies s3 behavior in lawsynth-connectors. |
| 1,075 | `python/lawsynth-connectors/tests/test_delta.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies delta behavior in lawsynth-connectors. |
| 1,076 | `python/lawsynth-connectors/tests/test_iceberg.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies iceberg behavior in lawsynth-connectors. |
| 1,077 | `python/lawsynth-connectors/tests/test_pandas.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies pandas behavior in lawsynth-connectors. |
| 1,078 | `python/lawsynth-connectors/tests/test_polars.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies polars behavior in lawsynth-connectors. |
| 1,079 | `python/lawsynth-connectors/tests/test_arrow.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies arrow behavior in lawsynth-connectors. |
| 1,080 | `python/lawsynth-connectors/tests/test_xarray.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies xarray behavior in lawsynth-connectors. |
| 1,081 | `python/lawsynth-connectors/tests/test_kafka.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies kafka behavior in lawsynth-connectors. |
| 1,082 | `python/lawsynth-connectors/tests/test_validation.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies validation behavior in lawsynth-connectors. |
| 1,083 | `python/lawsynth-connectors/tests/test_credentials.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies credentials behavior in lawsynth-connectors. |
| 1,084 | `python/lawsynth-connectors/tests/test_pagination.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies pagination behavior in lawsynth-connectors. |
| 1,085 | `python/lawsynth-connectors/tests/test_partitioning.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies partitioning behavior in lawsynth-connectors. |
| 1,086 | `python/lawsynth-connectors/tests/test_fingerprints.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies fingerprints behavior in lawsynth-connectors. |
| 1,087 | `python/lawsynth-connectors/tests/test_errors_connector.py` | python | lawsynth-connectors | P4 | Python SDK | Verifies errors connector behavior in lawsynth-connectors. |
| 1,088 | `python/lawsynth-connectors/fixtures/base/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample base fixture. |
| 1,089 | `python/lawsynth-connectors/fixtures/registry/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample registry fixture. |
| 1,090 | `python/lawsynth-connectors/fixtures/filesystem/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample filesystem fixture. |
| 1,091 | `python/lawsynth-connectors/fixtures/http/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample http fixture. |
| 1,092 | `python/lawsynth-connectors/fixtures/sql/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample sql fixture. |
| 1,093 | `python/lawsynth-connectors/fixtures/duckdb/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample duckdb fixture. |
| 1,094 | `python/lawsynth-connectors/fixtures/postgres/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample postgres fixture. |
| 1,095 | `python/lawsynth-connectors/fixtures/s3/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample s3 fixture. |
| 1,096 | `python/lawsynth-connectors/fixtures/delta/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample delta fixture. |
| 1,097 | `python/lawsynth-connectors/fixtures/iceberg/sample.json` | python | lawsynth-connectors | P4 | Python SDK | Provides a sample iceberg fixture. |
| 1,098 | `python/lawsynth-connectors/docs/polars.md` | python | lawsynth-connectors | P4 | Python SDK | Documents polars in lawsynth-connectors. |
| 1,099 | `python/lawsynth-connectors/docs/arrow.md` | python | lawsynth-connectors | P4 | Python SDK | Documents arrow in lawsynth-connectors. |
| 1,100 | `python/lawsynth-connectors/docs/xarray.md` | python | lawsynth-connectors | P4 | Python SDK | Documents xarray in lawsynth-connectors. |
| 1,101 | `python/lawsynth-connectors/docs/kafka.md` | python | lawsynth-connectors | P4 | Python SDK | Documents kafka in lawsynth-connectors. |
| 1,102 | `python/lawsynth-connectors/docs/validation.md` | python | lawsynth-connectors | P4 | Python SDK | Documents validation in lawsynth-connectors. |
| 1,103 | `python/lawsynth-connectors/docs/credentials.md` | python | lawsynth-connectors | P4 | Python SDK | Documents credentials in lawsynth-connectors. |
| 1,104 | `python/lawsynth-connectors/docs/pagination.md` | python | lawsynth-connectors | P4 | Python SDK | Documents pagination in lawsynth-connectors. |
| 1,105 | `python/lawsynth-connectors/docs/partitioning.md` | python | lawsynth-connectors | P4 | Python SDK | Documents partitioning in lawsynth-connectors. |
| 1,106 | `python/lawsynth-connectors/docs/fingerprints.md` | python | lawsynth-connectors | P4 | Python SDK | Documents fingerprints in lawsynth-connectors. |
| 1,107 | `python/lawsynth-connectors/docs/errors_connector.md` | python | lawsynth-connectors | P4 | Python SDK | Documents errors connector in lawsynth-connectors. |
| 1,108 | `python/lawsynth-bench/pyproject.toml` | python | lawsynth-bench | P3 | Python SDK | Declares the build, dependencies, and package metadata for lawsynth-bench. |
| 1,109 | `python/lawsynth-bench/README.md` | python | lawsynth-bench | P3 | Python SDK | Documents the purpose, boundaries, and usage of lawsynth-bench. |
| 1,110 | `python/lawsynth-bench/LICENSE` | python | lawsynth-bench | P3 | Python SDK | Declares legal terms and notices for lawsynth-bench. |
| 1,111 | `python/lawsynth-bench/src/lawsynth_bench/__init__.py` | python | lawsynth-bench | P3 | Python SDK | Implements   init   for lawsynth-bench. |
| 1,112 | `python/lawsynth-bench/src/lawsynth_bench/py.typed` | python | lawsynth-bench | P3 | Python SDK | Provides py for lawsynth-bench. |
| 1,113 | `python/lawsynth-bench/src/lawsynth_bench/_version.py` | python | lawsynth-bench | P3 | Python SDK | Implements  version for lawsynth-bench. |
| 1,114 | `python/lawsynth-bench/src/lawsynth_bench/errors.py` | python | lawsynth-bench | P3 | Python SDK | Implements errors for lawsynth-bench. |
| 1,115 | `python/lawsynth-bench/src/lawsynth_bench/config.py` | python | lawsynth-bench | P3 | Python SDK | Implements config for lawsynth-bench. |
| 1,116 | `python/lawsynth-bench/src/lawsynth_bench/registry.py` | python | lawsynth-bench | P3 | Python SDK | Implements registry for lawsynth-bench. |
| 1,117 | `python/lawsynth-bench/src/lawsynth_bench/problem.py` | python | lawsynth-bench | P3 | Python SDK | Implements problem for lawsynth-bench. |
| 1,118 | `python/lawsynth-bench/src/lawsynth_bench/dataset.py` | python | lawsynth-bench | P3 | Python SDK | Implements dataset for lawsynth-bench. |
| 1,119 | `python/lawsynth-bench/src/lawsynth_bench/runner.py` | python | lawsynth-bench | P3 | Python SDK | Implements runner for lawsynth-bench. |
| 1,120 | `python/lawsynth-bench/src/lawsynth_bench/metrics.py` | python | lawsynth-bench | P3 | Python SDK | Implements metrics for lawsynth-bench. |
| 1,121 | `python/lawsynth-bench/src/lawsynth_bench/leaderboard.py` | python | lawsynth-bench | P3 | Python SDK | Implements leaderboard for lawsynth-bench. |
| 1,122 | `python/lawsynth-bench/src/lawsynth_bench/report.py` | python | lawsynth-bench | P3 | Python SDK | Implements report for lawsynth-bench. |
| 1,123 | `python/lawsynth-bench/src/lawsynth_bench/cli.py` | python | lawsynth-bench | P3 | Python SDK | Implements cli for lawsynth-bench. |
| 1,124 | `python/lawsynth-bench/src/lawsynth_bench/baseline.py` | python | lawsynth-bench | P3 | Python SDK | Implements baseline for lawsynth-bench. |
| 1,125 | `python/lawsynth-bench/src/lawsynth_bench/environment.py` | python | lawsynth-bench | P3 | Python SDK | Implements environment for lawsynth-bench. |
| 1,126 | `python/lawsynth-bench/src/lawsynth_bench/reproduce.py` | python | lawsynth-bench | P3 | Python SDK | Implements reproduce for lawsynth-bench. |
| 1,127 | `python/lawsynth-bench/src/lawsynth_bench/equation_recovery.py` | python | lawsynth-bench | P3 | Python SDK | Implements equation recovery for lawsynth-bench. |
| 1,128 | `python/lawsynth-bench/src/lawsynth_bench/trajectory_accuracy.py` | python | lawsynth-bench | P3 | Python SDK | Implements trajectory accuracy for lawsynth-bench. |
| 1,129 | `python/lawsynth-bench/src/lawsynth_bench/graph_recovery.py` | python | lawsynth-bench | P3 | Python SDK | Implements graph recovery for lawsynth-bench. |
| 1,130 | `python/lawsynth-bench/src/lawsynth_bench/regime_recovery.py` | python | lawsynth-bench | P3 | Python SDK | Implements regime recovery for lawsynth-bench. |
| 1,131 | `python/lawsynth-bench/src/lawsynth_bench/uncertainty_coverage.py` | python | lawsynth-bench | P3 | Python SDK | Implements uncertainty coverage for lawsynth-bench. |
| 1,132 | `python/lawsynth-bench/src/lawsynth_bench/performance.py` | python | lawsynth-bench | P3 | Python SDK | Implements performance for lawsynth-bench. |
| 1,133 | `python/lawsynth-bench/src/lawsynth_bench/aggregation.py` | python | lawsynth-bench | P3 | Python SDK | Implements aggregation for lawsynth-bench. |
| 1,134 | `python/lawsynth-bench/src/lawsynth_bench/render.py` | python | lawsynth-bench | P3 | Python SDK | Implements render for lawsynth-bench. |
| 1,135 | `python/lawsynth-bench/src/lawsynth_bench/publish.py` | python | lawsynth-bench | P3 | Python SDK | Implements publish for lawsynth-bench. |
| 1,136 | `python/lawsynth-bench/src/lawsynth_bench/errors_bench.py` | python | lawsynth-bench | P3 | Python SDK | Implements errors bench for lawsynth-bench. |
| 1,137 | `python/lawsynth-bench/tests/conftest.py` | python | lawsynth-bench | P3 | Python SDK | Defines shared test fixtures for lawsynth-bench. |
| 1,138 | `python/lawsynth-bench/tests/test_registry.py` | python | lawsynth-bench | P3 | Python SDK | Verifies registry behavior in lawsynth-bench. |
| 1,139 | `python/lawsynth-bench/tests/test_problem.py` | python | lawsynth-bench | P3 | Python SDK | Verifies problem behavior in lawsynth-bench. |
| 1,140 | `python/lawsynth-bench/tests/test_dataset.py` | python | lawsynth-bench | P3 | Python SDK | Verifies dataset behavior in lawsynth-bench. |
| 1,141 | `python/lawsynth-bench/tests/test_runner.py` | python | lawsynth-bench | P3 | Python SDK | Verifies runner behavior in lawsynth-bench. |
| 1,142 | `python/lawsynth-bench/tests/test_metrics.py` | python | lawsynth-bench | P3 | Python SDK | Verifies metrics behavior in lawsynth-bench. |
| 1,143 | `python/lawsynth-bench/tests/test_leaderboard.py` | python | lawsynth-bench | P3 | Python SDK | Verifies leaderboard behavior in lawsynth-bench. |
| 1,144 | `python/lawsynth-bench/tests/test_report.py` | python | lawsynth-bench | P3 | Python SDK | Verifies report behavior in lawsynth-bench. |
| 1,145 | `python/lawsynth-bench/tests/test_cli.py` | python | lawsynth-bench | P3 | Python SDK | Verifies cli behavior in lawsynth-bench. |
| 1,146 | `python/lawsynth-bench/tests/test_baseline.py` | python | lawsynth-bench | P3 | Python SDK | Verifies baseline behavior in lawsynth-bench. |
| 1,147 | `python/lawsynth-bench/tests/test_environment.py` | python | lawsynth-bench | P3 | Python SDK | Verifies environment behavior in lawsynth-bench. |
| 1,148 | `python/lawsynth-bench/tests/test_reproduce.py` | python | lawsynth-bench | P3 | Python SDK | Verifies reproduce behavior in lawsynth-bench. |
| 1,149 | `python/lawsynth-bench/tests/test_equation_recovery.py` | python | lawsynth-bench | P3 | Python SDK | Verifies equation recovery behavior in lawsynth-bench. |
| 1,150 | `python/lawsynth-bench/tests/test_trajectory_accuracy.py` | python | lawsynth-bench | P3 | Python SDK | Verifies trajectory accuracy behavior in lawsynth-bench. |
| 1,151 | `python/lawsynth-bench/tests/test_graph_recovery.py` | python | lawsynth-bench | P3 | Python SDK | Verifies graph recovery behavior in lawsynth-bench. |
| 1,152 | `python/lawsynth-bench/tests/test_regime_recovery.py` | python | lawsynth-bench | P3 | Python SDK | Verifies regime recovery behavior in lawsynth-bench. |
| 1,153 | `python/lawsynth-bench/tests/test_uncertainty_coverage.py` | python | lawsynth-bench | P3 | Python SDK | Verifies uncertainty coverage behavior in lawsynth-bench. |
| 1,154 | `python/lawsynth-bench/tests/test_performance.py` | python | lawsynth-bench | P3 | Python SDK | Verifies performance behavior in lawsynth-bench. |
| 1,155 | `python/lawsynth-bench/tests/test_aggregation.py` | python | lawsynth-bench | P3 | Python SDK | Verifies aggregation behavior in lawsynth-bench. |
| 1,156 | `python/lawsynth-bench/tests/test_render.py` | python | lawsynth-bench | P3 | Python SDK | Verifies render behavior in lawsynth-bench. |
| 1,157 | `python/lawsynth-bench/tests/test_publish.py` | python | lawsynth-bench | P3 | Python SDK | Verifies publish behavior in lawsynth-bench. |
| 1,158 | `python/lawsynth-bench/tests/test_errors_bench.py` | python | lawsynth-bench | P3 | Python SDK | Verifies errors bench behavior in lawsynth-bench. |
| 1,159 | `python/lawsynth-bench/fixtures/registry/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample registry fixture. |
| 1,160 | `python/lawsynth-bench/fixtures/problem/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample problem fixture. |
| 1,161 | `python/lawsynth-bench/fixtures/dataset/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample dataset fixture. |
| 1,162 | `python/lawsynth-bench/fixtures/runner/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample runner fixture. |
| 1,163 | `python/lawsynth-bench/fixtures/metrics/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample metrics fixture. |
| 1,164 | `python/lawsynth-bench/fixtures/leaderboard/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample leaderboard fixture. |
| 1,165 | `python/lawsynth-bench/fixtures/report/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample report fixture. |
| 1,166 | `python/lawsynth-bench/fixtures/cli/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample cli fixture. |
| 1,167 | `python/lawsynth-bench/fixtures/baseline/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample baseline fixture. |
| 1,168 | `python/lawsynth-bench/fixtures/environment/sample.json` | python | lawsynth-bench | P3 | Python SDK | Provides a sample environment fixture. |
| 1,169 | `python/lawsynth-bench/docs/equation_recovery.md` | python | lawsynth-bench | P3 | Python SDK | Documents equation recovery in lawsynth-bench. |
| 1,170 | `python/lawsynth-bench/docs/trajectory_accuracy.md` | python | lawsynth-bench | P3 | Python SDK | Documents trajectory accuracy in lawsynth-bench. |
| 1,171 | `python/lawsynth-bench/docs/graph_recovery.md` | python | lawsynth-bench | P3 | Python SDK | Documents graph recovery in lawsynth-bench. |
| 1,172 | `python/lawsynth-bench/docs/regime_recovery.md` | python | lawsynth-bench | P3 | Python SDK | Documents regime recovery in lawsynth-bench. |
| 1,173 | `python/lawsynth-bench/docs/uncertainty_coverage.md` | python | lawsynth-bench | P3 | Python SDK | Documents uncertainty coverage in lawsynth-bench. |
| 1,174 | `python/lawsynth-bench/docs/performance.md` | python | lawsynth-bench | P3 | Python SDK | Documents performance in lawsynth-bench. |
| 1,175 | `python/lawsynth-bench/docs/aggregation.md` | python | lawsynth-bench | P3 | Python SDK | Documents aggregation in lawsynth-bench. |
| 1,176 | `python/lawsynth-bench/docs/render.md` | python | lawsynth-bench | P3 | Python SDK | Documents render in lawsynth-bench. |
| 1,177 | `python/lawsynth-bench/docs/publish.md` | python | lawsynth-bench | P3 | Python SDK | Documents publish in lawsynth-bench. |
| 1,178 | `python/lawsynth-bench/docs/errors_bench.md` | python | lawsynth-bench | P3 | Python SDK | Documents errors bench in lawsynth-bench. |
| 1,179 | `python/lawsynth-notebook/pyproject.toml` | python | lawsynth-notebook | P3 | Python SDK | Declares the build, dependencies, and package metadata for lawsynth-notebook. |
| 1,180 | `python/lawsynth-notebook/README.md` | python | lawsynth-notebook | P3 | Python SDK | Documents the purpose, boundaries, and usage of lawsynth-notebook. |
| 1,181 | `python/lawsynth-notebook/LICENSE` | python | lawsynth-notebook | P3 | Python SDK | Declares legal terms and notices for lawsynth-notebook. |
| 1,182 | `python/lawsynth-notebook/src/lawsynth_notebook/__init__.py` | python | lawsynth-notebook | P3 | Python SDK | Implements   init   for lawsynth-notebook. |
| 1,183 | `python/lawsynth-notebook/src/lawsynth_notebook/py.typed` | python | lawsynth-notebook | P3 | Python SDK | Provides py for lawsynth-notebook. |
| 1,184 | `python/lawsynth-notebook/src/lawsynth_notebook/_version.py` | python | lawsynth-notebook | P3 | Python SDK | Implements  version for lawsynth-notebook. |
| 1,185 | `python/lawsynth-notebook/src/lawsynth_notebook/errors.py` | python | lawsynth-notebook | P3 | Python SDK | Implements errors for lawsynth-notebook. |
| 1,186 | `python/lawsynth-notebook/src/lawsynth_notebook/config.py` | python | lawsynth-notebook | P3 | Python SDK | Implements config for lawsynth-notebook. |
| 1,187 | `python/lawsynth-notebook/src/lawsynth_notebook/display.py` | python | lawsynth-notebook | P3 | Python SDK | Implements display for lawsynth-notebook. |
| 1,188 | `python/lawsynth-notebook/src/lawsynth_notebook/widget.py` | python | lawsynth-notebook | P3 | Python SDK | Implements widget for lawsynth-notebook. |
| 1,189 | `python/lawsynth-notebook/src/lawsynth_notebook/events.py` | python | lawsynth-notebook | P3 | Python SDK | Implements events for lawsynth-notebook. |
| 1,190 | `python/lawsynth-notebook/src/lawsynth_notebook/assets.py` | python | lawsynth-notebook | P3 | Python SDK | Implements assets for lawsynth-notebook. |
| 1,191 | `python/lawsynth-notebook/src/lawsynth_notebook/equation_view.py` | python | lawsynth-notebook | P3 | Python SDK | Implements equation view for lawsynth-notebook. |
| 1,192 | `python/lawsynth-notebook/src/lawsynth_notebook/graph_view.py` | python | lawsynth-notebook | P3 | Python SDK | Implements graph view for lawsynth-notebook. |
| 1,193 | `python/lawsynth-notebook/src/lawsynth_notebook/trajectory_view.py` | python | lawsynth-notebook | P3 | Python SDK | Implements trajectory view for lawsynth-notebook. |
| 1,194 | `python/lawsynth-notebook/src/lawsynth_notebook/frontier_view.py` | python | lawsynth-notebook | P3 | Python SDK | Implements frontier view for lawsynth-notebook. |
| 1,195 | `python/lawsynth-notebook/src/lawsynth_notebook/regime_view.py` | python | lawsynth-notebook | P3 | Python SDK | Implements regime view for lawsynth-notebook. |
| 1,196 | `python/lawsynth-notebook/src/lawsynth_notebook/uncertainty_view.py` | python | lawsynth-notebook | P3 | Python SDK | Implements uncertainty view for lawsynth-notebook. |
| 1,197 | `python/lawsynth-notebook/src/lawsynth_notebook/progress.py` | python | lawsynth-notebook | P3 | Python SDK | Implements progress for lawsynth-notebook. |
| 1,198 | `python/lawsynth-notebook/src/lawsynth_notebook/controls.py` | python | lawsynth-notebook | P3 | Python SDK | Implements controls for lawsynth-notebook. |
| 1,199 | `python/lawsynth-notebook/src/lawsynth_notebook/comm.py` | python | lawsynth-notebook | P3 | Python SDK | Implements comm for lawsynth-notebook. |
| 1,200 | `python/lawsynth-notebook/src/lawsynth_notebook/serialization.py` | python | lawsynth-notebook | P3 | Python SDK | Implements serialization for lawsynth-notebook. |
| 1,201 | `python/lawsynth-notebook/src/lawsynth_notebook/themes.py` | python | lawsynth-notebook | P3 | Python SDK | Implements themes for lawsynth-notebook. |
| 1,202 | `python/lawsynth-notebook/src/lawsynth_notebook/templates.py` | python | lawsynth-notebook | P3 | Python SDK | Implements templates for lawsynth-notebook. |
| 1,203 | `python/lawsynth-notebook/src/lawsynth_notebook/extension.py` | python | lawsynth-notebook | P3 | Python SDK | Implements extension for lawsynth-notebook. |
| 1,204 | `python/lawsynth-notebook/src/lawsynth_notebook/server_proxy.py` | python | lawsynth-notebook | P3 | Python SDK | Implements server proxy for lawsynth-notebook. |
| 1,205 | `python/lawsynth-notebook/src/lawsynth_notebook/export.py` | python | lawsynth-notebook | P3 | Python SDK | Implements export for lawsynth-notebook. |
| 1,206 | `python/lawsynth-notebook/src/lawsynth_notebook/compatibility.py` | python | lawsynth-notebook | P3 | Python SDK | Implements compatibility for lawsynth-notebook. |
| 1,207 | `python/lawsynth-notebook/src/lawsynth_notebook/errors_notebook.py` | python | lawsynth-notebook | P3 | Python SDK | Implements errors notebook for lawsynth-notebook. |
| 1,208 | `python/lawsynth-notebook/tests/conftest.py` | python | lawsynth-notebook | P3 | Python SDK | Defines shared test fixtures for lawsynth-notebook. |
| 1,209 | `python/lawsynth-notebook/tests/test_display.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies display behavior in lawsynth-notebook. |
| 1,210 | `python/lawsynth-notebook/tests/test_widget.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies widget behavior in lawsynth-notebook. |
| 1,211 | `python/lawsynth-notebook/tests/test_events.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies events behavior in lawsynth-notebook. |
| 1,212 | `python/lawsynth-notebook/tests/test_assets.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies assets behavior in lawsynth-notebook. |
| 1,213 | `python/lawsynth-notebook/tests/test_equation_view.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies equation view behavior in lawsynth-notebook. |
| 1,214 | `python/lawsynth-notebook/tests/test_graph_view.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies graph view behavior in lawsynth-notebook. |
| 1,215 | `python/lawsynth-notebook/tests/test_trajectory_view.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies trajectory view behavior in lawsynth-notebook. |
| 1,216 | `python/lawsynth-notebook/tests/test_frontier_view.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies frontier view behavior in lawsynth-notebook. |
| 1,217 | `python/lawsynth-notebook/tests/test_regime_view.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies regime view behavior in lawsynth-notebook. |
| 1,218 | `python/lawsynth-notebook/tests/test_uncertainty_view.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies uncertainty view behavior in lawsynth-notebook. |
| 1,219 | `python/lawsynth-notebook/tests/test_progress.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies progress behavior in lawsynth-notebook. |
| 1,220 | `python/lawsynth-notebook/tests/test_controls.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies controls behavior in lawsynth-notebook. |
| 1,221 | `python/lawsynth-notebook/tests/test_comm.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies comm behavior in lawsynth-notebook. |
| 1,222 | `python/lawsynth-notebook/tests/test_serialization.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies serialization behavior in lawsynth-notebook. |
| 1,223 | `python/lawsynth-notebook/tests/test_themes.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies themes behavior in lawsynth-notebook. |
| 1,224 | `python/lawsynth-notebook/tests/test_templates.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies templates behavior in lawsynth-notebook. |
| 1,225 | `python/lawsynth-notebook/tests/test_extension.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies extension behavior in lawsynth-notebook. |
| 1,226 | `python/lawsynth-notebook/tests/test_server_proxy.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies server proxy behavior in lawsynth-notebook. |
| 1,227 | `python/lawsynth-notebook/tests/test_export.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies export behavior in lawsynth-notebook. |
| 1,228 | `python/lawsynth-notebook/tests/test_compatibility.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies compatibility behavior in lawsynth-notebook. |
| 1,229 | `python/lawsynth-notebook/tests/test_errors_notebook.py` | python | lawsynth-notebook | P3 | Python SDK | Verifies errors notebook behavior in lawsynth-notebook. |
| 1,230 | `python/lawsynth-notebook/fixtures/display/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample display fixture. |
| 1,231 | `python/lawsynth-notebook/fixtures/widget/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample widget fixture. |
| 1,232 | `python/lawsynth-notebook/fixtures/events/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample events fixture. |
| 1,233 | `python/lawsynth-notebook/fixtures/assets/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample assets fixture. |
| 1,234 | `python/lawsynth-notebook/fixtures/equation_view/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample equation view fixture. |
| 1,235 | `python/lawsynth-notebook/fixtures/graph_view/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample graph view fixture. |
| 1,236 | `python/lawsynth-notebook/fixtures/trajectory_view/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample trajectory view fixture. |
| 1,237 | `python/lawsynth-notebook/fixtures/frontier_view/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample frontier view fixture. |
| 1,238 | `python/lawsynth-notebook/fixtures/regime_view/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample regime view fixture. |
| 1,239 | `python/lawsynth-notebook/fixtures/uncertainty_view/sample.json` | python | lawsynth-notebook | P3 | Python SDK | Provides a sample uncertainty view fixture. |
| 1,240 | `python/lawsynth-notebook/docs/controls.md` | python | lawsynth-notebook | P3 | Python SDK | Documents controls in lawsynth-notebook. |
| 1,241 | `python/lawsynth-notebook/docs/comm.md` | python | lawsynth-notebook | P3 | Python SDK | Documents comm in lawsynth-notebook. |
| 1,242 | `python/lawsynth-notebook/docs/serialization.md` | python | lawsynth-notebook | P3 | Python SDK | Documents serialization in lawsynth-notebook. |
| 1,243 | `python/lawsynth-notebook/docs/themes.md` | python | lawsynth-notebook | P3 | Python SDK | Documents themes in lawsynth-notebook. |
| 1,244 | `python/lawsynth-notebook/docs/templates.md` | python | lawsynth-notebook | P3 | Python SDK | Documents templates in lawsynth-notebook. |
| 1,245 | `python/lawsynth-notebook/docs/extension.md` | python | lawsynth-notebook | P3 | Python SDK | Documents extension in lawsynth-notebook. |
| 1,246 | `python/lawsynth-notebook/docs/server_proxy.md` | python | lawsynth-notebook | P3 | Python SDK | Documents server proxy in lawsynth-notebook. |
| 1,247 | `python/lawsynth-notebook/docs/export.md` | python | lawsynth-notebook | P3 | Python SDK | Documents export in lawsynth-notebook. |
| 1,248 | `python/lawsynth-notebook/docs/compatibility.md` | python | lawsynth-notebook | P3 | Python SDK | Documents compatibility in lawsynth-notebook. |
| 1,249 | `python/lawsynth-notebook/docs/errors_notebook.md` | python | lawsynth-notebook | P3 | Python SDK | Documents errors notebook in lawsynth-notebook. |
| 1,250 | `apps/studio/package.json` | studio | apps/studio | P3 | Web Studio | Declares the build, dependencies, and package metadata for apps/studio. |
| 1,251 | `apps/studio/README.md` | studio | apps/studio | P3 | Web Studio | Documents the purpose, boundaries, and usage of apps/studio. |
| 1,252 | `apps/studio/tsconfig.json` | studio | apps/studio | P3 | Web Studio | Configures or declares tsconfig for apps/studio. |
| 1,253 | `apps/studio/src/index.ts` | studio | apps/studio | P3 | Web Studio | Implements index for apps/studio. |
| 1,254 | `apps/studio/src/app.ts` | studio | apps/studio | P3 | Web Studio | Implements app for apps/studio. |
| 1,255 | `apps/studio/src/routes.ts` | studio | apps/studio | P3 | Web Studio | Implements routes for apps/studio. |
| 1,256 | `apps/studio/src/providers.ts` | studio | apps/studio | P3 | Web Studio | Implements providers for apps/studio. |
| 1,257 | `apps/studio/src/workspace.ts` | studio | apps/studio | P3 | Web Studio | Implements workspace for apps/studio. |
| 1,258 | `apps/studio/src/dataset.ts` | studio | apps/studio | P3 | Web Studio | Implements dataset for apps/studio. |
| 1,259 | `apps/studio/src/discovery.ts` | studio | apps/studio | P3 | Web Studio | Implements discovery for apps/studio. |
| 1,260 | `apps/studio/src/equations.ts` | studio | apps/studio | P3 | Web Studio | Implements equations for apps/studio. |
| 1,261 | `apps/studio/src/structure.ts` | studio | apps/studio | P3 | Web Studio | Implements structure for apps/studio. |
| 1,262 | `apps/studio/src/regimes.ts` | studio | apps/studio | P3 | Web Studio | Implements regimes for apps/studio. |
| 1,263 | `apps/studio/src/simulation.ts` | studio | apps/studio | P3 | Web Studio | Implements simulation for apps/studio. |
| 1,264 | `apps/studio/src/uncertainty.ts` | studio | apps/studio | P3 | Web Studio | Implements uncertainty for apps/studio. |
| 1,265 | `apps/studio/src/provenance.ts` | studio | apps/studio | P3 | Web Studio | Implements provenance for apps/studio. |
| 1,266 | `apps/studio/src/export.ts` | studio | apps/studio | P3 | Web Studio | Implements export for apps/studio. |
| 1,267 | `apps/studio/src/settings.ts` | studio | apps/studio | P3 | Web Studio | Implements settings for apps/studio. |
| 1,268 | `apps/studio/src/shortcuts.ts` | studio | apps/studio | P3 | Web Studio | Implements shortcuts for apps/studio. |
| 1,269 | `apps/studio/tests/app.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies app in apps/studio. |
| 1,270 | `apps/studio/tests/routes.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies routes in apps/studio. |
| 1,271 | `apps/studio/tests/providers.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies providers in apps/studio. |
| 1,272 | `apps/studio/tests/workspace.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies workspace in apps/studio. |
| 1,273 | `apps/studio/tests/dataset.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies dataset in apps/studio. |
| 1,274 | `apps/studio/tests/discovery.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies discovery in apps/studio. |
| 1,275 | `apps/studio/tests/equations.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies equations in apps/studio. |
| 1,276 | `apps/studio/tests/structure.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies structure in apps/studio. |
| 1,277 | `apps/studio/tests/regimes.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies regimes in apps/studio. |
| 1,278 | `apps/studio/tests/simulation.test.ts` | studio | apps/studio | P3 | Web Studio | Verifies simulation in apps/studio. |
| 1,279 | `apps/studio/examples/uncertainty.example.ts` | studio | apps/studio | P3 | Web Studio | Demonstrates uncertainty in apps/studio. |
| 1,280 | `apps/studio/examples/provenance.example.ts` | studio | apps/studio | P3 | Web Studio | Demonstrates provenance in apps/studio. |
| 1,281 | `apps/studio/examples/export.example.ts` | studio | apps/studio | P3 | Web Studio | Demonstrates export in apps/studio. |
| 1,282 | `apps/studio/examples/settings.example.ts` | studio | apps/studio | P3 | Web Studio | Demonstrates settings in apps/studio. |
| 1,283 | `apps/studio/examples/shortcuts.example.ts` | studio | apps/studio | P3 | Web Studio | Demonstrates shortcuts in apps/studio. |
| 1,284 | `apps/studio/fixtures/app.json` | studio | apps/studio | P3 | Web Studio | Supplies app fixture data for apps/studio. |
| 1,285 | `apps/studio/fixtures/routes.json` | studio | apps/studio | P3 | Web Studio | Supplies routes fixture data for apps/studio. |
| 1,286 | `apps/studio/fixtures/providers.json` | studio | apps/studio | P3 | Web Studio | Supplies providers fixture data for apps/studio. |
| 1,287 | `apps/studio/fixtures/workspace.json` | studio | apps/studio | P3 | Web Studio | Supplies workspace fixture data for apps/studio. |
| 1,288 | `apps/studio/fixtures/dataset.json` | studio | apps/studio | P3 | Web Studio | Supplies dataset fixture data for apps/studio. |
| 1,289 | `apps/docs-site/package.json` | studio | apps/docs-site | P3 | Web Studio | Declares the build, dependencies, and package metadata for apps/docs-site. |
| 1,290 | `apps/docs-site/README.md` | studio | apps/docs-site | P3 | Web Studio | Documents the purpose, boundaries, and usage of apps/docs-site. |
| 1,291 | `apps/docs-site/tsconfig.json` | studio | apps/docs-site | P3 | Web Studio | Configures or declares tsconfig for apps/docs-site. |
| 1,292 | `apps/docs-site/src/index.ts` | studio | apps/docs-site | P3 | Web Studio | Implements index for apps/docs-site. |
| 1,293 | `apps/docs-site/src/site.ts` | studio | apps/docs-site | P3 | Web Studio | Implements site for apps/docs-site. |
| 1,294 | `apps/docs-site/src/navigation.ts` | studio | apps/docs-site | P3 | Web Studio | Implements navigation for apps/docs-site. |
| 1,295 | `apps/docs-site/src/search.ts` | studio | apps/docs-site | P3 | Web Studio | Implements search for apps/docs-site. |
| 1,296 | `apps/docs-site/src/markdown.ts` | studio | apps/docs-site | P3 | Web Studio | Implements markdown for apps/docs-site. |
| 1,297 | `apps/docs-site/src/code.ts` | studio | apps/docs-site | P3 | Web Studio | Implements code for apps/docs-site. |
| 1,298 | `apps/docs-site/src/equations.ts` | studio | apps/docs-site | P3 | Web Studio | Implements equations for apps/docs-site. |
| 1,299 | `apps/docs-site/src/api_reference.ts` | studio | apps/docs-site | P3 | Web Studio | Implements api reference for apps/docs-site. |
| 1,300 | `apps/docs-site/src/blog.ts` | studio | apps/docs-site | P3 | Web Studio | Implements blog for apps/docs-site. |
| 1,301 | `apps/docs-site/src/examples.ts` | studio | apps/docs-site | P3 | Web Studio | Implements examples for apps/docs-site. |
| 1,302 | `apps/docs-site/src/benchmarks.ts` | studio | apps/docs-site | P3 | Web Studio | Implements benchmarks for apps/docs-site. |
| 1,303 | `apps/docs-site/src/versions.ts` | studio | apps/docs-site | P3 | Web Studio | Implements versions for apps/docs-site. |
| 1,304 | `apps/docs-site/src/seo.ts` | studio | apps/docs-site | P3 | Web Studio | Implements seo for apps/docs-site. |
| 1,305 | `apps/docs-site/src/analytics.ts` | studio | apps/docs-site | P3 | Web Studio | Implements analytics for apps/docs-site. |
| 1,306 | `apps/docs-site/src/theme.ts` | studio | apps/docs-site | P3 | Web Studio | Implements theme for apps/docs-site. |
| 1,307 | `apps/docs-site/src/redirects.ts` | studio | apps/docs-site | P3 | Web Studio | Implements redirects for apps/docs-site. |
| 1,308 | `apps/docs-site/tests/site.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies site in apps/docs-site. |
| 1,309 | `apps/docs-site/tests/navigation.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies navigation in apps/docs-site. |
| 1,310 | `apps/docs-site/tests/search.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies search in apps/docs-site. |
| 1,311 | `apps/docs-site/tests/markdown.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies markdown in apps/docs-site. |
| 1,312 | `apps/docs-site/tests/code.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies code in apps/docs-site. |
| 1,313 | `apps/docs-site/tests/equations.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies equations in apps/docs-site. |
| 1,314 | `apps/docs-site/tests/api_reference.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies api reference in apps/docs-site. |
| 1,315 | `apps/docs-site/tests/blog.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies blog in apps/docs-site. |
| 1,316 | `apps/docs-site/tests/examples.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies examples in apps/docs-site. |
| 1,317 | `apps/docs-site/tests/benchmarks.test.ts` | studio | apps/docs-site | P3 | Web Studio | Verifies benchmarks in apps/docs-site. |
| 1,318 | `apps/docs-site/examples/versions.example.ts` | studio | apps/docs-site | P3 | Web Studio | Demonstrates versions in apps/docs-site. |
| 1,319 | `apps/docs-site/examples/seo.example.ts` | studio | apps/docs-site | P3 | Web Studio | Demonstrates seo in apps/docs-site. |
| 1,320 | `apps/docs-site/examples/analytics.example.ts` | studio | apps/docs-site | P3 | Web Studio | Demonstrates analytics in apps/docs-site. |
| 1,321 | `apps/docs-site/examples/theme.example.ts` | studio | apps/docs-site | P3 | Web Studio | Demonstrates theme in apps/docs-site. |
| 1,322 | `apps/docs-site/examples/redirects.example.ts` | studio | apps/docs-site | P3 | Web Studio | Demonstrates redirects in apps/docs-site. |
| 1,323 | `apps/docs-site/fixtures/site.json` | studio | apps/docs-site | P3 | Web Studio | Supplies site fixture data for apps/docs-site. |
| 1,324 | `apps/docs-site/fixtures/navigation.json` | studio | apps/docs-site | P3 | Web Studio | Supplies navigation fixture data for apps/docs-site. |
| 1,325 | `apps/docs-site/fixtures/search.json` | studio | apps/docs-site | P3 | Web Studio | Supplies search fixture data for apps/docs-site. |
| 1,326 | `apps/docs-site/fixtures/markdown.json` | studio | apps/docs-site | P3 | Web Studio | Supplies markdown fixture data for apps/docs-site. |
| 1,327 | `apps/docs-site/fixtures/code.json` | studio | apps/docs-site | P3 | Web Studio | Supplies code fixture data for apps/docs-site. |
| 1,328 | `apps/playground/package.json` | studio | apps/playground | P3 | Web Studio | Declares the build, dependencies, and package metadata for apps/playground. |
| 1,329 | `apps/playground/README.md` | studio | apps/playground | P3 | Web Studio | Documents the purpose, boundaries, and usage of apps/playground. |
| 1,330 | `apps/playground/tsconfig.json` | studio | apps/playground | P3 | Web Studio | Configures or declares tsconfig for apps/playground. |
| 1,331 | `apps/playground/src/index.ts` | studio | apps/playground | P3 | Web Studio | Implements index for apps/playground. |
| 1,332 | `apps/playground/src/playground.ts` | studio | apps/playground | P3 | Web Studio | Implements playground for apps/playground. |
| 1,333 | `apps/playground/src/editor.ts` | studio | apps/playground | P3 | Web Studio | Implements editor for apps/playground. |
| 1,334 | `apps/playground/src/dataset_picker.ts` | studio | apps/playground | P3 | Web Studio | Implements dataset picker for apps/playground. |
| 1,335 | `apps/playground/src/world_picker.ts` | studio | apps/playground | P3 | Web Studio | Implements world picker for apps/playground. |
| 1,336 | `apps/playground/src/parameter_panel.ts` | studio | apps/playground | P3 | Web Studio | Implements parameter panel for apps/playground. |
| 1,337 | `apps/playground/src/simulation.ts` | studio | apps/playground | P3 | Web Studio | Implements simulation for apps/playground. |
| 1,338 | `apps/playground/src/charts.ts` | studio | apps/playground | P3 | Web Studio | Implements charts for apps/playground. |
| 1,339 | `apps/playground/src/share.ts` | studio | apps/playground | P3 | Web Studio | Implements share for apps/playground. |
| 1,340 | `apps/playground/src/examples.ts` | studio | apps/playground | P3 | Web Studio | Implements examples for apps/playground. |
| 1,341 | `apps/playground/src/wasm.ts` | studio | apps/playground | P3 | Web Studio | Implements wasm for apps/playground. |
| 1,342 | `apps/playground/src/worker.ts` | studio | apps/playground | P3 | Web Studio | Implements worker for apps/playground. |
| 1,343 | `apps/playground/src/storage.ts` | studio | apps/playground | P3 | Web Studio | Implements storage for apps/playground. |
| 1,344 | `apps/playground/src/errors.ts` | studio | apps/playground | P3 | Web Studio | Implements errors for apps/playground. |
| 1,345 | `apps/playground/src/theme.ts` | studio | apps/playground | P3 | Web Studio | Implements theme for apps/playground. |
| 1,346 | `apps/playground/src/embed.ts` | studio | apps/playground | P3 | Web Studio | Implements embed for apps/playground. |
| 1,347 | `apps/playground/tests/playground.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies playground in apps/playground. |
| 1,348 | `apps/playground/tests/editor.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies editor in apps/playground. |
| 1,349 | `apps/playground/tests/dataset_picker.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies dataset picker in apps/playground. |
| 1,350 | `apps/playground/tests/world_picker.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies world picker in apps/playground. |
| 1,351 | `apps/playground/tests/parameter_panel.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies parameter panel in apps/playground. |
| 1,352 | `apps/playground/tests/simulation.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies simulation in apps/playground. |
| 1,353 | `apps/playground/tests/charts.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies charts in apps/playground. |
| 1,354 | `apps/playground/tests/share.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies share in apps/playground. |
| 1,355 | `apps/playground/tests/examples.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies examples in apps/playground. |
| 1,356 | `apps/playground/tests/wasm.test.ts` | studio | apps/playground | P3 | Web Studio | Verifies wasm in apps/playground. |
| 1,357 | `apps/playground/examples/worker.example.ts` | studio | apps/playground | P3 | Web Studio | Demonstrates worker in apps/playground. |
| 1,358 | `apps/playground/examples/storage.example.ts` | studio | apps/playground | P3 | Web Studio | Demonstrates storage in apps/playground. |
| 1,359 | `apps/playground/examples/errors.example.ts` | studio | apps/playground | P3 | Web Studio | Demonstrates errors in apps/playground. |
| 1,360 | `apps/playground/examples/theme.example.ts` | studio | apps/playground | P3 | Web Studio | Demonstrates theme in apps/playground. |
| 1,361 | `apps/playground/examples/embed.example.ts` | studio | apps/playground | P3 | Web Studio | Demonstrates embed in apps/playground. |
| 1,362 | `apps/playground/fixtures/playground.json` | studio | apps/playground | P3 | Web Studio | Supplies playground fixture data for apps/playground. |
| 1,363 | `apps/playground/fixtures/editor.json` | studio | apps/playground | P3 | Web Studio | Supplies editor fixture data for apps/playground. |
| 1,364 | `apps/playground/fixtures/dataset_picker.json` | studio | apps/playground | P3 | Web Studio | Supplies dataset picker fixture data for apps/playground. |
| 1,365 | `apps/playground/fixtures/world_picker.json` | studio | apps/playground | P3 | Web Studio | Supplies world picker fixture data for apps/playground. |
| 1,366 | `apps/playground/fixtures/parameter_panel.json` | studio | apps/playground | P3 | Web Studio | Supplies parameter panel fixture data for apps/playground. |
| 1,367 | `packages/api-client/package.json` | studio | packages/api-client | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/api-client. |
| 1,368 | `packages/api-client/README.md` | studio | packages/api-client | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/api-client. |
| 1,369 | `packages/api-client/tsconfig.json` | studio | packages/api-client | P3 | Web Studio | Configures or declares tsconfig for packages/api-client. |
| 1,370 | `packages/api-client/src/index.ts` | studio | packages/api-client | P3 | Web Studio | Implements index for packages/api-client. |
| 1,371 | `packages/api-client/src/client.ts` | studio | packages/api-client | P3 | Web Studio | Implements client for packages/api-client. |
| 1,372 | `packages/api-client/src/transport.ts` | studio | packages/api-client | P3 | Web Studio | Implements transport for packages/api-client. |
| 1,373 | `packages/api-client/src/auth.ts` | studio | packages/api-client | P3 | Web Studio | Implements auth for packages/api-client. |
| 1,374 | `packages/api-client/src/errors.ts` | studio | packages/api-client | P3 | Web Studio | Implements errors for packages/api-client. |
| 1,375 | `packages/api-client/src/pagination.ts` | studio | packages/api-client | P3 | Web Studio | Implements pagination for packages/api-client. |
| 1,376 | `packages/api-client/src/projects.ts` | studio | packages/api-client | P3 | Web Studio | Implements projects for packages/api-client. |
| 1,377 | `packages/api-client/src/datasets.ts` | studio | packages/api-client | P3 | Web Studio | Implements datasets for packages/api-client. |
| 1,378 | `packages/api-client/src/runs.ts` | studio | packages/api-client | P3 | Web Studio | Implements runs for packages/api-client. |
| 1,379 | `packages/api-client/src/worlds.ts` | studio | packages/api-client | P3 | Web Studio | Implements worlds for packages/api-client. |
| 1,380 | `packages/api-client/src/simulations.ts` | studio | packages/api-client | P3 | Web Studio | Implements simulations for packages/api-client. |
| 1,381 | `packages/api-client/src/artifacts.ts` | studio | packages/api-client | P3 | Web Studio | Implements artifacts for packages/api-client. |
| 1,382 | `packages/api-client/src/events.ts` | studio | packages/api-client | P3 | Web Studio | Implements events for packages/api-client. |
| 1,383 | `packages/api-client/src/uploads.ts` | studio | packages/api-client | P3 | Web Studio | Implements uploads for packages/api-client. |
| 1,384 | `packages/api-client/src/downloads.ts` | studio | packages/api-client | P3 | Web Studio | Implements downloads for packages/api-client. |
| 1,385 | `packages/api-client/src/generated.ts` | studio | packages/api-client | P3 | Web Studio | Implements generated for packages/api-client. |
| 1,386 | `packages/api-client/tests/client.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies client in packages/api-client. |
| 1,387 | `packages/api-client/tests/transport.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies transport in packages/api-client. |
| 1,388 | `packages/api-client/tests/auth.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies auth in packages/api-client. |
| 1,389 | `packages/api-client/tests/errors.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies errors in packages/api-client. |
| 1,390 | `packages/api-client/tests/pagination.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies pagination in packages/api-client. |
| 1,391 | `packages/api-client/tests/projects.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies projects in packages/api-client. |
| 1,392 | `packages/api-client/tests/datasets.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies datasets in packages/api-client. |
| 1,393 | `packages/api-client/tests/runs.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies runs in packages/api-client. |
| 1,394 | `packages/api-client/tests/worlds.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies worlds in packages/api-client. |
| 1,395 | `packages/api-client/tests/simulations.test.ts` | studio | packages/api-client | P3 | Web Studio | Verifies simulations in packages/api-client. |
| 1,396 | `packages/api-client/examples/artifacts.example.ts` | studio | packages/api-client | P3 | Web Studio | Demonstrates artifacts in packages/api-client. |
| 1,397 | `packages/api-client/examples/events.example.ts` | studio | packages/api-client | P3 | Web Studio | Demonstrates events in packages/api-client. |
| 1,398 | `packages/api-client/examples/uploads.example.ts` | studio | packages/api-client | P3 | Web Studio | Demonstrates uploads in packages/api-client. |
| 1,399 | `packages/api-client/examples/downloads.example.ts` | studio | packages/api-client | P3 | Web Studio | Demonstrates downloads in packages/api-client. |
| 1,400 | `packages/api-client/examples/generated.example.ts` | studio | packages/api-client | P3 | Web Studio | Demonstrates generated in packages/api-client. |
| 1,401 | `packages/api-client/fixtures/client.json` | studio | packages/api-client | P3 | Web Studio | Supplies client fixture data for packages/api-client. |
| 1,402 | `packages/api-client/fixtures/transport.json` | studio | packages/api-client | P3 | Web Studio | Supplies transport fixture data for packages/api-client. |
| 1,403 | `packages/api-client/fixtures/auth.json` | studio | packages/api-client | P3 | Web Studio | Supplies auth fixture data for packages/api-client. |
| 1,404 | `packages/api-client/fixtures/errors.json` | studio | packages/api-client | P3 | Web Studio | Supplies errors fixture data for packages/api-client. |
| 1,405 | `packages/api-client/fixtures/pagination.json` | studio | packages/api-client | P3 | Web Studio | Supplies pagination fixture data for packages/api-client. |
| 1,406 | `packages/world-schema/package.json` | studio | packages/world-schema | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/world-schema. |
| 1,407 | `packages/world-schema/README.md` | studio | packages/world-schema | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/world-schema. |
| 1,408 | `packages/world-schema/tsconfig.json` | studio | packages/world-schema | P3 | Web Studio | Configures or declares tsconfig for packages/world-schema. |
| 1,409 | `packages/world-schema/src/index.ts` | studio | packages/world-schema | P3 | Web Studio | Implements index for packages/world-schema. |
| 1,410 | `packages/world-schema/src/types.ts` | studio | packages/world-schema | P3 | Web Studio | Implements types for packages/world-schema. |
| 1,411 | `packages/world-schema/src/validators.ts` | studio | packages/world-schema | P3 | Web Studio | Implements validators for packages/world-schema. |
| 1,412 | `packages/world-schema/src/manifest.ts` | studio | packages/world-schema | P3 | Web Studio | Implements manifest for packages/world-schema. |
| 1,413 | `packages/world-schema/src/world.ts` | studio | packages/world-schema | P3 | Web Studio | Implements world for packages/world-schema. |
| 1,414 | `packages/world-schema/src/expression.ts` | studio | packages/world-schema | P3 | Web Studio | Implements expression for packages/world-schema. |
| 1,415 | `packages/world-schema/src/law.ts` | studio | packages/world-schema | P3 | Web Studio | Implements law for packages/world-schema. |
| 1,416 | `packages/world-schema/src/graph.ts` | studio | packages/world-schema | P3 | Web Studio | Implements graph for packages/world-schema. |
| 1,417 | `packages/world-schema/src/regime.ts` | studio | packages/world-schema | P3 | Web Studio | Implements regime for packages/world-schema. |
| 1,418 | `packages/world-schema/src/event.ts` | studio | packages/world-schema | P3 | Web Studio | Implements event for packages/world-schema. |
| 1,419 | `packages/world-schema/src/intervention.ts` | studio | packages/world-schema | P3 | Web Studio | Implements intervention for packages/world-schema. |
| 1,420 | `packages/world-schema/src/uncertainty.ts` | studio | packages/world-schema | P3 | Web Studio | Implements uncertainty for packages/world-schema. |
| 1,421 | `packages/world-schema/src/provenance.ts` | studio | packages/world-schema | P3 | Web Studio | Implements provenance for packages/world-schema. |
| 1,422 | `packages/world-schema/src/migrations.ts` | studio | packages/world-schema | P3 | Web Studio | Implements migrations for packages/world-schema. |
| 1,423 | `packages/world-schema/src/hash.ts` | studio | packages/world-schema | P3 | Web Studio | Implements hash for packages/world-schema. |
| 1,424 | `packages/world-schema/src/generated.ts` | studio | packages/world-schema | P3 | Web Studio | Implements generated for packages/world-schema. |
| 1,425 | `packages/world-schema/tests/types.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies types in packages/world-schema. |
| 1,426 | `packages/world-schema/tests/validators.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies validators in packages/world-schema. |
| 1,427 | `packages/world-schema/tests/manifest.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies manifest in packages/world-schema. |
| 1,428 | `packages/world-schema/tests/world.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies world in packages/world-schema. |
| 1,429 | `packages/world-schema/tests/expression.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies expression in packages/world-schema. |
| 1,430 | `packages/world-schema/tests/law.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies law in packages/world-schema. |
| 1,431 | `packages/world-schema/tests/graph.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies graph in packages/world-schema. |
| 1,432 | `packages/world-schema/tests/regime.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies regime in packages/world-schema. |
| 1,433 | `packages/world-schema/tests/event.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies event in packages/world-schema. |
| 1,434 | `packages/world-schema/tests/intervention.test.ts` | studio | packages/world-schema | P3 | Web Studio | Verifies intervention in packages/world-schema. |
| 1,435 | `packages/world-schema/examples/uncertainty.example.ts` | studio | packages/world-schema | P3 | Web Studio | Demonstrates uncertainty in packages/world-schema. |
| 1,436 | `packages/world-schema/examples/provenance.example.ts` | studio | packages/world-schema | P3 | Web Studio | Demonstrates provenance in packages/world-schema. |
| 1,437 | `packages/world-schema/examples/migrations.example.ts` | studio | packages/world-schema | P3 | Web Studio | Demonstrates migrations in packages/world-schema. |
| 1,438 | `packages/world-schema/examples/hash.example.ts` | studio | packages/world-schema | P3 | Web Studio | Demonstrates hash in packages/world-schema. |
| 1,439 | `packages/world-schema/examples/generated.example.ts` | studio | packages/world-schema | P3 | Web Studio | Demonstrates generated in packages/world-schema. |
| 1,440 | `packages/world-schema/fixtures/types.json` | studio | packages/world-schema | P3 | Web Studio | Supplies types fixture data for packages/world-schema. |
| 1,441 | `packages/world-schema/fixtures/validators.json` | studio | packages/world-schema | P3 | Web Studio | Supplies validators fixture data for packages/world-schema. |
| 1,442 | `packages/world-schema/fixtures/manifest.json` | studio | packages/world-schema | P3 | Web Studio | Supplies manifest fixture data for packages/world-schema. |
| 1,443 | `packages/world-schema/fixtures/world.json` | studio | packages/world-schema | P3 | Web Studio | Supplies world fixture data for packages/world-schema. |
| 1,444 | `packages/world-schema/fixtures/expression.json` | studio | packages/world-schema | P3 | Web Studio | Supplies expression fixture data for packages/world-schema. |
| 1,445 | `packages/world-viewer/package.json` | studio | packages/world-viewer | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/world-viewer. |
| 1,446 | `packages/world-viewer/README.md` | studio | packages/world-viewer | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/world-viewer. |
| 1,447 | `packages/world-viewer/tsconfig.json` | studio | packages/world-viewer | P3 | Web Studio | Configures or declares tsconfig for packages/world-viewer. |
| 1,448 | `packages/world-viewer/src/index.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements index for packages/world-viewer. |
| 1,449 | `packages/world-viewer/src/viewer.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements viewer for packages/world-viewer. |
| 1,450 | `packages/world-viewer/src/bundle.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements bundle for packages/world-viewer. |
| 1,451 | `packages/world-viewer/src/equation.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements equation for packages/world-viewer. |
| 1,452 | `packages/world-viewer/src/graph.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements graph for packages/world-viewer. |
| 1,453 | `packages/world-viewer/src/regime.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements regime for packages/world-viewer. |
| 1,454 | `packages/world-viewer/src/trajectory.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements trajectory for packages/world-viewer. |
| 1,455 | `packages/world-viewer/src/uncertainty.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements uncertainty for packages/world-viewer. |
| 1,456 | `packages/world-viewer/src/parameters.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements parameters for packages/world-viewer. |
| 1,457 | `packages/world-viewer/src/provenance.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements provenance for packages/world-viewer. |
| 1,458 | `packages/world-viewer/src/toolbar.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements toolbar for packages/world-viewer. |
| 1,459 | `packages/world-viewer/src/layout.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements layout for packages/world-viewer. |
| 1,460 | `packages/world-viewer/src/worker.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements worker for packages/world-viewer. |
| 1,461 | `packages/world-viewer/src/export.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements export for packages/world-viewer. |
| 1,462 | `packages/world-viewer/src/theme.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements theme for packages/world-viewer. |
| 1,463 | `packages/world-viewer/src/embed.ts` | studio | packages/world-viewer | P3 | Web Studio | Implements embed for packages/world-viewer. |
| 1,464 | `packages/world-viewer/tests/viewer.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies viewer in packages/world-viewer. |
| 1,465 | `packages/world-viewer/tests/bundle.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies bundle in packages/world-viewer. |
| 1,466 | `packages/world-viewer/tests/equation.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies equation in packages/world-viewer. |
| 1,467 | `packages/world-viewer/tests/graph.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies graph in packages/world-viewer. |
| 1,468 | `packages/world-viewer/tests/regime.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies regime in packages/world-viewer. |
| 1,469 | `packages/world-viewer/tests/trajectory.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies trajectory in packages/world-viewer. |
| 1,470 | `packages/world-viewer/tests/uncertainty.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies uncertainty in packages/world-viewer. |
| 1,471 | `packages/world-viewer/tests/parameters.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies parameters in packages/world-viewer. |
| 1,472 | `packages/world-viewer/tests/provenance.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies provenance in packages/world-viewer. |
| 1,473 | `packages/world-viewer/tests/toolbar.test.ts` | studio | packages/world-viewer | P3 | Web Studio | Verifies toolbar in packages/world-viewer. |
| 1,474 | `packages/world-viewer/examples/layout.example.ts` | studio | packages/world-viewer | P3 | Web Studio | Demonstrates layout in packages/world-viewer. |
| 1,475 | `packages/world-viewer/examples/worker.example.ts` | studio | packages/world-viewer | P3 | Web Studio | Demonstrates worker in packages/world-viewer. |
| 1,476 | `packages/world-viewer/examples/export.example.ts` | studio | packages/world-viewer | P3 | Web Studio | Demonstrates export in packages/world-viewer. |
| 1,477 | `packages/world-viewer/examples/theme.example.ts` | studio | packages/world-viewer | P3 | Web Studio | Demonstrates theme in packages/world-viewer. |
| 1,478 | `packages/world-viewer/examples/embed.example.ts` | studio | packages/world-viewer | P3 | Web Studio | Demonstrates embed in packages/world-viewer. |
| 1,479 | `packages/world-viewer/fixtures/viewer.json` | studio | packages/world-viewer | P3 | Web Studio | Supplies viewer fixture data for packages/world-viewer. |
| 1,480 | `packages/world-viewer/fixtures/bundle.json` | studio | packages/world-viewer | P3 | Web Studio | Supplies bundle fixture data for packages/world-viewer. |
| 1,481 | `packages/world-viewer/fixtures/equation.json` | studio | packages/world-viewer | P3 | Web Studio | Supplies equation fixture data for packages/world-viewer. |
| 1,482 | `packages/world-viewer/fixtures/graph.json` | studio | packages/world-viewer | P3 | Web Studio | Supplies graph fixture data for packages/world-viewer. |
| 1,483 | `packages/world-viewer/fixtures/regime.json` | studio | packages/world-viewer | P3 | Web Studio | Supplies regime fixture data for packages/world-viewer. |
| 1,484 | `packages/design-system/package.json` | studio | packages/design-system | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/design-system. |
| 1,485 | `packages/design-system/README.md` | studio | packages/design-system | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/design-system. |
| 1,486 | `packages/design-system/tsconfig.json` | studio | packages/design-system | P3 | Web Studio | Configures or declares tsconfig for packages/design-system. |
| 1,487 | `packages/design-system/src/index.ts` | studio | packages/design-system | P3 | Web Studio | Implements index for packages/design-system. |
| 1,488 | `packages/design-system/src/button.ts` | studio | packages/design-system | P3 | Web Studio | Implements button for packages/design-system. |
| 1,489 | `packages/design-system/src/input.ts` | studio | packages/design-system | P3 | Web Studio | Implements input for packages/design-system. |
| 1,490 | `packages/design-system/src/select.ts` | studio | packages/design-system | P3 | Web Studio | Implements select for packages/design-system. |
| 1,491 | `packages/design-system/src/dialog.ts` | studio | packages/design-system | P3 | Web Studio | Implements dialog for packages/design-system. |
| 1,492 | `packages/design-system/src/popover.ts` | studio | packages/design-system | P3 | Web Studio | Implements popover for packages/design-system. |
| 1,493 | `packages/design-system/src/tooltip.ts` | studio | packages/design-system | P3 | Web Studio | Implements tooltip for packages/design-system. |
| 1,494 | `packages/design-system/src/table.ts` | studio | packages/design-system | P3 | Web Studio | Implements table for packages/design-system. |
| 1,495 | `packages/design-system/src/tabs.ts` | studio | packages/design-system | P3 | Web Studio | Implements tabs for packages/design-system. |
| 1,496 | `packages/design-system/src/panel.ts` | studio | packages/design-system | P3 | Web Studio | Implements panel for packages/design-system. |
| 1,497 | `packages/design-system/src/badge.ts` | studio | packages/design-system | P3 | Web Studio | Implements badge for packages/design-system. |
| 1,498 | `packages/design-system/src/progress.ts` | studio | packages/design-system | P3 | Web Studio | Implements progress for packages/design-system. |
| 1,499 | `packages/design-system/src/toast.ts` | studio | packages/design-system | P3 | Web Studio | Implements toast for packages/design-system. |
| 1,500 | `packages/design-system/src/icons.ts` | studio | packages/design-system | P3 | Web Studio | Implements icons for packages/design-system. |
| 1,501 | `packages/design-system/src/tokens.ts` | studio | packages/design-system | P3 | Web Studio | Implements tokens for packages/design-system. |
| 1,502 | `packages/design-system/src/theme.ts` | studio | packages/design-system | P3 | Web Studio | Implements theme for packages/design-system. |
| 1,503 | `packages/design-system/tests/button.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies button in packages/design-system. |
| 1,504 | `packages/design-system/tests/input.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies input in packages/design-system. |
| 1,505 | `packages/design-system/tests/select.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies select in packages/design-system. |
| 1,506 | `packages/design-system/tests/dialog.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies dialog in packages/design-system. |
| 1,507 | `packages/design-system/tests/popover.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies popover in packages/design-system. |
| 1,508 | `packages/design-system/tests/tooltip.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies tooltip in packages/design-system. |
| 1,509 | `packages/design-system/tests/table.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies table in packages/design-system. |
| 1,510 | `packages/design-system/tests/tabs.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies tabs in packages/design-system. |
| 1,511 | `packages/design-system/tests/panel.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies panel in packages/design-system. |
| 1,512 | `packages/design-system/tests/badge.test.ts` | studio | packages/design-system | P3 | Web Studio | Verifies badge in packages/design-system. |
| 1,513 | `packages/design-system/examples/progress.example.ts` | studio | packages/design-system | P3 | Web Studio | Demonstrates progress in packages/design-system. |
| 1,514 | `packages/design-system/examples/toast.example.ts` | studio | packages/design-system | P3 | Web Studio | Demonstrates toast in packages/design-system. |
| 1,515 | `packages/design-system/examples/icons.example.ts` | studio | packages/design-system | P3 | Web Studio | Demonstrates icons in packages/design-system. |
| 1,516 | `packages/design-system/examples/tokens.example.ts` | studio | packages/design-system | P3 | Web Studio | Demonstrates tokens in packages/design-system. |
| 1,517 | `packages/design-system/examples/theme.example.ts` | studio | packages/design-system | P3 | Web Studio | Demonstrates theme in packages/design-system. |
| 1,518 | `packages/design-system/fixtures/button.json` | studio | packages/design-system | P3 | Web Studio | Supplies button fixture data for packages/design-system. |
| 1,519 | `packages/design-system/fixtures/input.json` | studio | packages/design-system | P3 | Web Studio | Supplies input fixture data for packages/design-system. |
| 1,520 | `packages/design-system/fixtures/select.json` | studio | packages/design-system | P3 | Web Studio | Supplies select fixture data for packages/design-system. |
| 1,521 | `packages/design-system/fixtures/dialog.json` | studio | packages/design-system | P3 | Web Studio | Supplies dialog fixture data for packages/design-system. |
| 1,522 | `packages/design-system/fixtures/popover.json` | studio | packages/design-system | P3 | Web Studio | Supplies popover fixture data for packages/design-system. |
| 1,523 | `packages/chart-core/package.json` | studio | packages/chart-core | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/chart-core. |
| 1,524 | `packages/chart-core/README.md` | studio | packages/chart-core | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/chart-core. |
| 1,525 | `packages/chart-core/tsconfig.json` | studio | packages/chart-core | P3 | Web Studio | Configures or declares tsconfig for packages/chart-core. |
| 1,526 | `packages/chart-core/src/index.ts` | studio | packages/chart-core | P3 | Web Studio | Implements index for packages/chart-core. |
| 1,527 | `packages/chart-core/src/chart.ts` | studio | packages/chart-core | P3 | Web Studio | Implements chart for packages/chart-core. |
| 1,528 | `packages/chart-core/src/scales.ts` | studio | packages/chart-core | P3 | Web Studio | Implements scales for packages/chart-core. |
| 1,529 | `packages/chart-core/src/axis.ts` | studio | packages/chart-core | P3 | Web Studio | Implements axis for packages/chart-core. |
| 1,530 | `packages/chart-core/src/series.ts` | studio | packages/chart-core | P3 | Web Studio | Implements series for packages/chart-core. |
| 1,531 | `packages/chart-core/src/tooltip.ts` | studio | packages/chart-core | P3 | Web Studio | Implements tooltip for packages/chart-core. |
| 1,532 | `packages/chart-core/src/legend.ts` | studio | packages/chart-core | P3 | Web Studio | Implements legend for packages/chart-core. |
| 1,533 | `packages/chart-core/src/brush.ts` | studio | packages/chart-core | P3 | Web Studio | Implements brush for packages/chart-core. |
| 1,534 | `packages/chart-core/src/zoom.ts` | studio | packages/chart-core | P3 | Web Studio | Implements zoom for packages/chart-core. |
| 1,535 | `packages/chart-core/src/downsample.ts` | studio | packages/chart-core | P3 | Web Studio | Implements downsample for packages/chart-core. |
| 1,536 | `packages/chart-core/src/palette.ts` | studio | packages/chart-core | P3 | Web Studio | Implements palette for packages/chart-core. |
| 1,537 | `packages/chart-core/src/trajectory.ts` | studio | packages/chart-core | P3 | Web Studio | Implements trajectory for packages/chart-core. |
| 1,538 | `packages/chart-core/src/phase_portrait.ts` | studio | packages/chart-core | P3 | Web Studio | Implements phase portrait for packages/chart-core. |
| 1,539 | `packages/chart-core/src/heatmap.ts` | studio | packages/chart-core | P3 | Web Studio | Implements heatmap for packages/chart-core. |
| 1,540 | `packages/chart-core/src/graph.ts` | studio | packages/chart-core | P3 | Web Studio | Implements graph for packages/chart-core. |
| 1,541 | `packages/chart-core/src/export.ts` | studio | packages/chart-core | P3 | Web Studio | Implements export for packages/chart-core. |
| 1,542 | `packages/chart-core/tests/chart.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies chart in packages/chart-core. |
| 1,543 | `packages/chart-core/tests/scales.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies scales in packages/chart-core. |
| 1,544 | `packages/chart-core/tests/axis.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies axis in packages/chart-core. |
| 1,545 | `packages/chart-core/tests/series.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies series in packages/chart-core. |
| 1,546 | `packages/chart-core/tests/tooltip.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies tooltip in packages/chart-core. |
| 1,547 | `packages/chart-core/tests/legend.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies legend in packages/chart-core. |
| 1,548 | `packages/chart-core/tests/brush.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies brush in packages/chart-core. |
| 1,549 | `packages/chart-core/tests/zoom.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies zoom in packages/chart-core. |
| 1,550 | `packages/chart-core/tests/downsample.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies downsample in packages/chart-core. |
| 1,551 | `packages/chart-core/tests/palette.test.ts` | studio | packages/chart-core | P3 | Web Studio | Verifies palette in packages/chart-core. |
| 1,552 | `packages/chart-core/examples/trajectory.example.ts` | studio | packages/chart-core | P3 | Web Studio | Demonstrates trajectory in packages/chart-core. |
| 1,553 | `packages/chart-core/examples/phase_portrait.example.ts` | studio | packages/chart-core | P3 | Web Studio | Demonstrates phase portrait in packages/chart-core. |
| 1,554 | `packages/chart-core/examples/heatmap.example.ts` | studio | packages/chart-core | P3 | Web Studio | Demonstrates heatmap in packages/chart-core. |
| 1,555 | `packages/chart-core/examples/graph.example.ts` | studio | packages/chart-core | P3 | Web Studio | Demonstrates graph in packages/chart-core. |
| 1,556 | `packages/chart-core/examples/export.example.ts` | studio | packages/chart-core | P3 | Web Studio | Demonstrates export in packages/chart-core. |
| 1,557 | `packages/chart-core/fixtures/chart.json` | studio | packages/chart-core | P3 | Web Studio | Supplies chart fixture data for packages/chart-core. |
| 1,558 | `packages/chart-core/fixtures/scales.json` | studio | packages/chart-core | P3 | Web Studio | Supplies scales fixture data for packages/chart-core. |
| 1,559 | `packages/chart-core/fixtures/axis.json` | studio | packages/chart-core | P3 | Web Studio | Supplies axis fixture data for packages/chart-core. |
| 1,560 | `packages/chart-core/fixtures/series.json` | studio | packages/chart-core | P3 | Web Studio | Supplies series fixture data for packages/chart-core. |
| 1,561 | `packages/chart-core/fixtures/tooltip.json` | studio | packages/chart-core | P3 | Web Studio | Supplies tooltip fixture data for packages/chart-core. |
| 1,562 | `packages/layout-engine/package.json` | studio | packages/layout-engine | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/layout-engine. |
| 1,563 | `packages/layout-engine/README.md` | studio | packages/layout-engine | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/layout-engine. |
| 1,564 | `packages/layout-engine/tsconfig.json` | studio | packages/layout-engine | P3 | Web Studio | Configures or declares tsconfig for packages/layout-engine. |
| 1,565 | `packages/layout-engine/src/index.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements index for packages/layout-engine. |
| 1,566 | `packages/layout-engine/src/layout.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements layout for packages/layout-engine. |
| 1,567 | `packages/layout-engine/src/graph_layout.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements graph layout for packages/layout-engine. |
| 1,568 | `packages/layout-engine/src/dag.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements dag for packages/layout-engine. |
| 1,569 | `packages/layout-engine/src/force.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements force for packages/layout-engine. |
| 1,570 | `packages/layout-engine/src/timeline.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements timeline for packages/layout-engine. |
| 1,571 | `packages/layout-engine/src/grid.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements grid for packages/layout-engine. |
| 1,572 | `packages/layout-engine/src/measure.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements measure for packages/layout-engine. |
| 1,573 | `packages/layout-engine/src/collision.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements collision for packages/layout-engine. |
| 1,574 | `packages/layout-engine/src/routing.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements routing for packages/layout-engine. |
| 1,575 | `packages/layout-engine/src/labels.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements labels for packages/layout-engine. |
| 1,576 | `packages/layout-engine/src/viewport.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements viewport for packages/layout-engine. |
| 1,577 | `packages/layout-engine/src/worker.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements worker for packages/layout-engine. |
| 1,578 | `packages/layout-engine/src/cache.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements cache for packages/layout-engine. |
| 1,579 | `packages/layout-engine/src/constraints.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements constraints for packages/layout-engine. |
| 1,580 | `packages/layout-engine/src/animation.ts` | studio | packages/layout-engine | P3 | Web Studio | Implements animation for packages/layout-engine. |
| 1,581 | `packages/layout-engine/tests/layout.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies layout in packages/layout-engine. |
| 1,582 | `packages/layout-engine/tests/graph_layout.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies graph layout in packages/layout-engine. |
| 1,583 | `packages/layout-engine/tests/dag.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies dag in packages/layout-engine. |
| 1,584 | `packages/layout-engine/tests/force.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies force in packages/layout-engine. |
| 1,585 | `packages/layout-engine/tests/timeline.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies timeline in packages/layout-engine. |
| 1,586 | `packages/layout-engine/tests/grid.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies grid in packages/layout-engine. |
| 1,587 | `packages/layout-engine/tests/measure.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies measure in packages/layout-engine. |
| 1,588 | `packages/layout-engine/tests/collision.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies collision in packages/layout-engine. |
| 1,589 | `packages/layout-engine/tests/routing.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies routing in packages/layout-engine. |
| 1,590 | `packages/layout-engine/tests/labels.test.ts` | studio | packages/layout-engine | P3 | Web Studio | Verifies labels in packages/layout-engine. |
| 1,591 | `packages/layout-engine/examples/viewport.example.ts` | studio | packages/layout-engine | P3 | Web Studio | Demonstrates viewport in packages/layout-engine. |
| 1,592 | `packages/layout-engine/examples/worker.example.ts` | studio | packages/layout-engine | P3 | Web Studio | Demonstrates worker in packages/layout-engine. |
| 1,593 | `packages/layout-engine/examples/cache.example.ts` | studio | packages/layout-engine | P3 | Web Studio | Demonstrates cache in packages/layout-engine. |
| 1,594 | `packages/layout-engine/examples/constraints.example.ts` | studio | packages/layout-engine | P3 | Web Studio | Demonstrates constraints in packages/layout-engine. |
| 1,595 | `packages/layout-engine/examples/animation.example.ts` | studio | packages/layout-engine | P3 | Web Studio | Demonstrates animation in packages/layout-engine. |
| 1,596 | `packages/layout-engine/fixtures/layout.json` | studio | packages/layout-engine | P3 | Web Studio | Supplies layout fixture data for packages/layout-engine. |
| 1,597 | `packages/layout-engine/fixtures/graph_layout.json` | studio | packages/layout-engine | P3 | Web Studio | Supplies graph layout fixture data for packages/layout-engine. |
| 1,598 | `packages/layout-engine/fixtures/dag.json` | studio | packages/layout-engine | P3 | Web Studio | Supplies dag fixture data for packages/layout-engine. |
| 1,599 | `packages/layout-engine/fixtures/force.json` | studio | packages/layout-engine | P3 | Web Studio | Supplies force fixture data for packages/layout-engine. |
| 1,600 | `packages/layout-engine/fixtures/timeline.json` | studio | packages/layout-engine | P3 | Web Studio | Supplies timeline fixture data for packages/layout-engine. |
| 1,601 | `packages/state-store/package.json` | studio | packages/state-store | P3 | Web Studio | Declares the build, dependencies, and package metadata for packages/state-store. |
| 1,602 | `packages/state-store/README.md` | studio | packages/state-store | P3 | Web Studio | Documents the purpose, boundaries, and usage of packages/state-store. |
| 1,603 | `packages/state-store/tsconfig.json` | studio | packages/state-store | P3 | Web Studio | Configures or declares tsconfig for packages/state-store. |
| 1,604 | `packages/state-store/src/index.ts` | studio | packages/state-store | P3 | Web Studio | Implements index for packages/state-store. |
| 1,605 | `packages/state-store/src/store.ts` | studio | packages/state-store | P3 | Web Studio | Implements store for packages/state-store. |
| 1,606 | `packages/state-store/src/workspace.ts` | studio | packages/state-store | P3 | Web Studio | Implements workspace for packages/state-store. |
| 1,607 | `packages/state-store/src/selection.ts` | studio | packages/state-store | P3 | Web Studio | Implements selection for packages/state-store. |
| 1,608 | `packages/state-store/src/panels.ts` | studio | packages/state-store | P3 | Web Studio | Implements panels for packages/state-store. |
| 1,609 | `packages/state-store/src/preferences.ts` | studio | packages/state-store | P3 | Web Studio | Implements preferences for packages/state-store. |
| 1,610 | `packages/state-store/src/history.ts` | studio | packages/state-store | P3 | Web Studio | Implements history for packages/state-store. |
| 1,611 | `packages/state-store/src/commands.ts` | studio | packages/state-store | P3 | Web Studio | Implements commands for packages/state-store. |
| 1,612 | `packages/state-store/src/events.ts` | studio | packages/state-store | P3 | Web Studio | Implements events for packages/state-store. |
| 1,613 | `packages/state-store/src/persistence.ts` | studio | packages/state-store | P3 | Web Studio | Implements persistence for packages/state-store. |
| 1,614 | `packages/state-store/src/sync.ts` | studio | packages/state-store | P3 | Web Studio | Implements sync for packages/state-store. |
| 1,615 | `packages/state-store/src/queries.ts` | studio | packages/state-store | P3 | Web Studio | Implements queries for packages/state-store. |
| 1,616 | `packages/state-store/src/mutations.ts` | studio | packages/state-store | P3 | Web Studio | Implements mutations for packages/state-store. |
| 1,617 | `packages/state-store/src/optimistic.ts` | studio | packages/state-store | P3 | Web Studio | Implements optimistic for packages/state-store. |
| 1,618 | `packages/state-store/src/undo.ts` | studio | packages/state-store | P3 | Web Studio | Implements undo for packages/state-store. |
| 1,619 | `packages/state-store/src/errors.ts` | studio | packages/state-store | P3 | Web Studio | Implements errors for packages/state-store. |
| 1,620 | `packages/state-store/tests/store.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies store in packages/state-store. |
| 1,621 | `packages/state-store/tests/workspace.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies workspace in packages/state-store. |
| 1,622 | `packages/state-store/tests/selection.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies selection in packages/state-store. |
| 1,623 | `packages/state-store/tests/panels.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies panels in packages/state-store. |
| 1,624 | `packages/state-store/tests/preferences.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies preferences in packages/state-store. |
| 1,625 | `packages/state-store/tests/history.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies history in packages/state-store. |
| 1,626 | `packages/state-store/tests/commands.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies commands in packages/state-store. |
| 1,627 | `packages/state-store/tests/events.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies events in packages/state-store. |
| 1,628 | `packages/state-store/tests/persistence.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies persistence in packages/state-store. |
| 1,629 | `packages/state-store/tests/sync.test.ts` | studio | packages/state-store | P3 | Web Studio | Verifies sync in packages/state-store. |
| 1,630 | `packages/state-store/examples/queries.example.ts` | studio | packages/state-store | P3 | Web Studio | Demonstrates queries in packages/state-store. |
| 1,631 | `packages/state-store/examples/mutations.example.ts` | studio | packages/state-store | P3 | Web Studio | Demonstrates mutations in packages/state-store. |
| 1,632 | `packages/state-store/examples/optimistic.example.ts` | studio | packages/state-store | P3 | Web Studio | Demonstrates optimistic in packages/state-store. |
| 1,633 | `packages/state-store/examples/undo.example.ts` | studio | packages/state-store | P3 | Web Studio | Demonstrates undo in packages/state-store. |
| 1,634 | `packages/state-store/examples/errors.example.ts` | studio | packages/state-store | P3 | Web Studio | Demonstrates errors in packages/state-store. |
| 1,635 | `packages/state-store/fixtures/store.json` | studio | packages/state-store | P3 | Web Studio | Supplies store fixture data for packages/state-store. |
| 1,636 | `packages/state-store/fixtures/workspace.json` | studio | packages/state-store | P3 | Web Studio | Supplies workspace fixture data for packages/state-store. |
| 1,637 | `packages/state-store/fixtures/selection.json` | studio | packages/state-store | P3 | Web Studio | Supplies selection fixture data for packages/state-store. |
| 1,638 | `packages/state-store/fixtures/panels.json` | studio | packages/state-store | P3 | Web Studio | Supplies panels fixture data for packages/state-store. |
| 1,639 | `packages/state-store/fixtures/preferences.json` | studio | packages/state-store | P3 | Web Studio | Supplies preferences fixture data for packages/state-store. |
| 1,640 | `services/api/README.md` | services | api | P5 | Backend Platform | Documents the purpose, boundaries, and usage of api. |
| 1,641 | `services/api/Dockerfile` | services | api | P5 | Backend Platform | Provides Dockerfile for api. |
| 1,642 | `services/api/pyproject.toml` | services | api | P5 | Backend Platform | Declares the build, dependencies, and package metadata for api. |
| 1,643 | `services/api/.env.example` | services | api | P5 | Backend Platform | Provides .env for api. |
| 1,644 | `services/api/src/lawsynth_api/main.py` | services | api | P5 | Backend Platform | Implements main for the api service. |
| 1,645 | `services/api/src/lawsynth_api/app.py` | services | api | P5 | Backend Platform | Implements app for the api service. |
| 1,646 | `services/api/src/lawsynth_api/settings.py` | services | api | P5 | Backend Platform | Implements settings for the api service. |
| 1,647 | `services/api/src/lawsynth_api/lifespan.py` | services | api | P5 | Backend Platform | Implements lifespan for the api service. |
| 1,648 | `services/api/src/lawsynth_api/auth.py` | services | api | P5 | Backend Platform | Implements auth for the api service. |
| 1,649 | `services/api/src/lawsynth_api/authorization.py` | services | api | P5 | Backend Platform | Implements authorization for the api service. |
| 1,650 | `services/api/src/lawsynth_api/projects.py` | services | api | P5 | Backend Platform | Implements projects for the api service. |
| 1,651 | `services/api/src/lawsynth_api/datasets.py` | services | api | P5 | Backend Platform | Implements datasets for the api service. |
| 1,652 | `services/api/src/lawsynth_api/runs.py` | services | api | P5 | Backend Platform | Implements runs for the api service. |
| 1,653 | `services/api/src/lawsynth_api/worlds.py` | services | api | P5 | Backend Platform | Implements worlds for the api service. |
| 1,654 | `services/api/src/lawsynth_api/simulations.py` | services | api | P5 | Backend Platform | Implements simulations for the api service. |
| 1,655 | `services/api/src/lawsynth_api/artifacts.py` | services | api | P5 | Backend Platform | Implements artifacts for the api service. |
| 1,656 | `services/api/src/lawsynth_api/events.py` | services | api | P5 | Backend Platform | Implements events for the api service. |
| 1,657 | `services/api/src/lawsynth_api/uploads.py` | services | api | P5 | Backend Platform | Implements uploads for the api service. |
| 1,658 | `services/api/src/lawsynth_api/downloads.py` | services | api | P5 | Backend Platform | Implements downloads for the api service. |
| 1,659 | `services/api/src/lawsynth_api/repositories.py` | services | api | P5 | Backend Platform | Implements repositories for the api service. |
| 1,660 | `services/api/src/lawsynth_api/database.py` | services | api | P5 | Backend Platform | Implements database for the api service. |
| 1,661 | `services/api/src/lawsynth_api/storage.py` | services | api | P5 | Backend Platform | Implements storage for the api service. |
| 1,662 | `services/api/src/lawsynth_api/middleware.py` | services | api | P5 | Backend Platform | Implements middleware for the api service. |
| 1,663 | `services/api/src/lawsynth_api/telemetry.py` | services | api | P5 | Backend Platform | Implements telemetry for the api service. |
| 1,664 | `services/api/tests/main_test.py` | services | api | P5 | Backend Platform | Verifies main in the api service. |
| 1,665 | `services/api/tests/app_test.py` | services | api | P5 | Backend Platform | Verifies app in the api service. |
| 1,666 | `services/api/tests/settings_test.py` | services | api | P5 | Backend Platform | Verifies settings in the api service. |
| 1,667 | `services/api/tests/lifespan_test.py` | services | api | P5 | Backend Platform | Verifies lifespan in the api service. |
| 1,668 | `services/api/tests/auth_test.py` | services | api | P5 | Backend Platform | Verifies auth in the api service. |
| 1,669 | `services/api/tests/authorization_test.py` | services | api | P5 | Backend Platform | Verifies authorization in the api service. |
| 1,670 | `services/api/tests/projects_test.py` | services | api | P5 | Backend Platform | Verifies projects in the api service. |
| 1,671 | `services/api/tests/datasets_test.py` | services | api | P5 | Backend Platform | Verifies datasets in the api service. |
| 1,672 | `services/api/tests/runs_test.py` | services | api | P5 | Backend Platform | Verifies runs in the api service. |
| 1,673 | `services/api/tests/worlds_test.py` | services | api | P5 | Backend Platform | Verifies worlds in the api service. |
| 1,674 | `services/api/tests/simulations_test.py` | services | api | P5 | Backend Platform | Verifies simulations in the api service. |
| 1,675 | `services/api/tests/artifacts_test.py` | services | api | P5 | Backend Platform | Verifies artifacts in the api service. |
| 1,676 | `services/api/config/development.yaml` | services | api | P5 | Backend Platform | Configures api for development operation. |
| 1,677 | `services/api/config/test.yaml` | services | api | P5 | Backend Platform | Configures api for test operation. |
| 1,678 | `services/api/config/staging.yaml` | services | api | P5 | Backend Platform | Configures api for staging operation. |
| 1,679 | `services/api/config/production.yaml` | services | api | P5 | Backend Platform | Configures api for production operation. |
| 1,680 | `services/api/config/logging.yaml` | services | api | P5 | Backend Platform | Configures api for logging operation. |
| 1,681 | `services/api/config/limits.yaml` | services | api | P5 | Backend Platform | Configures api for limits operation. |
| 1,682 | `services/api/docs/architecture.md` | services | api | P5 | Backend Platform | Documents architecture for the api service. |
| 1,683 | `services/api/docs/api.md` | services | api | P5 | Backend Platform | Documents api for the api service. |
| 1,684 | `services/api/docs/operations.md` | services | api | P5 | Backend Platform | Documents operations for the api service. |
| 1,685 | `services/api/docs/failures.md` | services | api | P5 | Backend Platform | Documents failures for the api service. |
| 1,686 | `services/api/docs/security.md` | services | api | P5 | Backend Platform | Documents security for the api service. |
| 1,687 | `services/scheduler/README.md` | services | scheduler | P5 | Backend Platform | Documents the purpose, boundaries, and usage of scheduler. |
| 1,688 | `services/scheduler/Dockerfile` | services | scheduler | P5 | Backend Platform | Provides Dockerfile for scheduler. |
| 1,689 | `services/scheduler/Cargo.toml` | services | scheduler | P5 | Backend Platform | Declares the build, dependencies, and package metadata for scheduler. |
| 1,690 | `services/scheduler/.env.example` | services | scheduler | P5 | Backend Platform | Provides .env for scheduler. |
| 1,691 | `services/scheduler/src/main.rs` | services | scheduler | P5 | Backend Platform | Implements main for the scheduler service. |
| 1,692 | `services/scheduler/src/config.rs` | services | scheduler | P5 | Backend Platform | Implements config for the scheduler service. |
| 1,693 | `services/scheduler/src/scheduler.rs` | services | scheduler | P5 | Backend Platform | Implements scheduler for the scheduler service. |
| 1,694 | `services/scheduler/src/queue.rs` | services | scheduler | P5 | Backend Platform | Implements queue for the scheduler service. |
| 1,695 | `services/scheduler/src/lease.rs` | services | scheduler | P5 | Backend Platform | Implements lease for the scheduler service. |
| 1,696 | `services/scheduler/src/policy.rs` | services | scheduler | P5 | Backend Platform | Implements policy for the scheduler service. |
| 1,697 | `services/scheduler/src/pool.rs` | services | scheduler | P5 | Backend Platform | Implements pool for the scheduler service. |
| 1,698 | `services/scheduler/src/quota.rs` | services | scheduler | P5 | Backend Platform | Implements quota for the scheduler service. |
| 1,699 | `services/scheduler/src/recovery.rs` | services | scheduler | P5 | Backend Platform | Implements recovery for the scheduler service. |
| 1,700 | `services/scheduler/src/metrics.rs` | services | scheduler | P5 | Backend Platform | Implements metrics for the scheduler service. |
| 1,701 | `services/scheduler/src/events.rs` | services | scheduler | P5 | Backend Platform | Implements events for the scheduler service. |
| 1,702 | `services/scheduler/src/database.rs` | services | scheduler | P5 | Backend Platform | Implements database for the scheduler service. |
| 1,703 | `services/scheduler/src/nats.rs` | services | scheduler | P5 | Backend Platform | Implements nats for the scheduler service. |
| 1,704 | `services/scheduler/src/backoff.rs` | services | scheduler | P5 | Backend Platform | Implements backoff for the scheduler service. |
| 1,705 | `services/scheduler/src/priority.rs` | services | scheduler | P5 | Backend Platform | Implements priority for the scheduler service. |
| 1,706 | `services/scheduler/src/placement.rs` | services | scheduler | P5 | Backend Platform | Implements placement for the scheduler service. |
| 1,707 | `services/scheduler/src/fairness.rs` | services | scheduler | P5 | Backend Platform | Implements fairness for the scheduler service. |
| 1,708 | `services/scheduler/src/shutdown.rs` | services | scheduler | P5 | Backend Platform | Implements shutdown for the scheduler service. |
| 1,709 | `services/scheduler/src/health.rs` | services | scheduler | P5 | Backend Platform | Implements health for the scheduler service. |
| 1,710 | `services/scheduler/src/errors.rs` | services | scheduler | P5 | Backend Platform | Implements errors for the scheduler service. |
| 1,711 | `services/scheduler/tests/main_test.rs` | services | scheduler | P5 | Backend Platform | Verifies main in the scheduler service. |
| 1,712 | `services/scheduler/tests/config_test.rs` | services | scheduler | P5 | Backend Platform | Verifies config in the scheduler service. |
| 1,713 | `services/scheduler/tests/scheduler_test.rs` | services | scheduler | P5 | Backend Platform | Verifies scheduler in the scheduler service. |
| 1,714 | `services/scheduler/tests/queue_test.rs` | services | scheduler | P5 | Backend Platform | Verifies queue in the scheduler service. |
| 1,715 | `services/scheduler/tests/lease_test.rs` | services | scheduler | P5 | Backend Platform | Verifies lease in the scheduler service. |
| 1,716 | `services/scheduler/tests/policy_test.rs` | services | scheduler | P5 | Backend Platform | Verifies policy in the scheduler service. |
| 1,717 | `services/scheduler/tests/pool_test.rs` | services | scheduler | P5 | Backend Platform | Verifies pool in the scheduler service. |
| 1,718 | `services/scheduler/tests/quota_test.rs` | services | scheduler | P5 | Backend Platform | Verifies quota in the scheduler service. |
| 1,719 | `services/scheduler/tests/recovery_test.rs` | services | scheduler | P5 | Backend Platform | Verifies recovery in the scheduler service. |
| 1,720 | `services/scheduler/tests/metrics_test.rs` | services | scheduler | P5 | Backend Platform | Verifies metrics in the scheduler service. |
| 1,721 | `services/scheduler/tests/events_test.rs` | services | scheduler | P5 | Backend Platform | Verifies events in the scheduler service. |
| 1,722 | `services/scheduler/tests/database_test.rs` | services | scheduler | P5 | Backend Platform | Verifies database in the scheduler service. |
| 1,723 | `services/scheduler/config/development.yaml` | services | scheduler | P5 | Backend Platform | Configures scheduler for development operation. |
| 1,724 | `services/scheduler/config/test.yaml` | services | scheduler | P5 | Backend Platform | Configures scheduler for test operation. |
| 1,725 | `services/scheduler/config/staging.yaml` | services | scheduler | P5 | Backend Platform | Configures scheduler for staging operation. |
| 1,726 | `services/scheduler/config/production.yaml` | services | scheduler | P5 | Backend Platform | Configures scheduler for production operation. |
| 1,727 | `services/scheduler/config/logging.yaml` | services | scheduler | P5 | Backend Platform | Configures scheduler for logging operation. |
| 1,728 | `services/scheduler/config/limits.yaml` | services | scheduler | P5 | Backend Platform | Configures scheduler for limits operation. |
| 1,729 | `services/scheduler/docs/architecture.md` | services | scheduler | P5 | Backend Platform | Documents architecture for the scheduler service. |
| 1,730 | `services/scheduler/docs/api.md` | services | scheduler | P5 | Backend Platform | Documents api for the scheduler service. |
| 1,731 | `services/scheduler/docs/operations.md` | services | scheduler | P5 | Backend Platform | Documents operations for the scheduler service. |
| 1,732 | `services/scheduler/docs/failures.md` | services | scheduler | P5 | Backend Platform | Documents failures for the scheduler service. |
| 1,733 | `services/scheduler/docs/security.md` | services | scheduler | P5 | Backend Platform | Documents security for the scheduler service. |
| 1,734 | `services/worker/README.md` | services | worker | P5 | Backend Platform | Documents the purpose, boundaries, and usage of worker. |
| 1,735 | `services/worker/Dockerfile` | services | worker | P5 | Backend Platform | Provides Dockerfile for worker. |
| 1,736 | `services/worker/Cargo.toml` | services | worker | P5 | Backend Platform | Declares the build, dependencies, and package metadata for worker. |
| 1,737 | `services/worker/.env.example` | services | worker | P5 | Backend Platform | Provides .env for worker. |
| 1,738 | `services/worker/src/main.rs` | services | worker | P5 | Backend Platform | Implements main for the worker service. |
| 1,739 | `services/worker/src/config.rs` | services | worker | P5 | Backend Platform | Implements config for the worker service. |
| 1,740 | `services/worker/src/worker.rs` | services | worker | P5 | Backend Platform | Implements worker for the worker service. |
| 1,741 | `services/worker/src/lease.rs` | services | worker | P5 | Backend Platform | Implements lease for the worker service. |
| 1,742 | `services/worker/src/heartbeat.rs` | services | worker | P5 | Backend Platform | Implements heartbeat for the worker service. |
| 1,743 | `services/worker/src/execute.rs` | services | worker | P5 | Backend Platform | Implements execute for the worker service. |
| 1,744 | `services/worker/src/checkpoint.rs` | services | worker | P5 | Backend Platform | Implements checkpoint for the worker service. |
| 1,745 | `services/worker/src/upload.rs` | services | worker | P5 | Backend Platform | Implements upload for the worker service. |
| 1,746 | `services/worker/src/resources.rs` | services | worker | P5 | Backend Platform | Implements resources for the worker service. |
| 1,747 | `services/worker/src/limits.rs` | services | worker | P5 | Backend Platform | Implements limits for the worker service. |
| 1,748 | `services/worker/src/sandbox.rs` | services | worker | P5 | Backend Platform | Implements sandbox for the worker service. |
| 1,749 | `services/worker/src/plugins.rs` | services | worker | P5 | Backend Platform | Implements plugins for the worker service. |
| 1,750 | `services/worker/src/artifacts.rs` | services | worker | P5 | Backend Platform | Implements artifacts for the worker service. |
| 1,751 | `services/worker/src/events.rs` | services | worker | P5 | Backend Platform | Implements events for the worker service. |
| 1,752 | `services/worker/src/telemetry.rs` | services | worker | P5 | Backend Platform | Implements telemetry for the worker service. |
| 1,753 | `services/worker/src/recovery.rs` | services | worker | P5 | Backend Platform | Implements recovery for the worker service. |
| 1,754 | `services/worker/src/cleanup.rs` | services | worker | P5 | Backend Platform | Implements cleanup for the worker service. |
| 1,755 | `services/worker/src/shutdown.rs` | services | worker | P5 | Backend Platform | Implements shutdown for the worker service. |
| 1,756 | `services/worker/src/health.rs` | services | worker | P5 | Backend Platform | Implements health for the worker service. |
| 1,757 | `services/worker/src/errors.rs` | services | worker | P5 | Backend Platform | Implements errors for the worker service. |
| 1,758 | `services/worker/tests/main_test.rs` | services | worker | P5 | Backend Platform | Verifies main in the worker service. |
| 1,759 | `services/worker/tests/config_test.rs` | services | worker | P5 | Backend Platform | Verifies config in the worker service. |
| 1,760 | `services/worker/tests/worker_test.rs` | services | worker | P5 | Backend Platform | Verifies worker in the worker service. |
| 1,761 | `services/worker/tests/lease_test.rs` | services | worker | P5 | Backend Platform | Verifies lease in the worker service. |
| 1,762 | `services/worker/tests/heartbeat_test.rs` | services | worker | P5 | Backend Platform | Verifies heartbeat in the worker service. |
| 1,763 | `services/worker/tests/execute_test.rs` | services | worker | P5 | Backend Platform | Verifies execute in the worker service. |
| 1,764 | `services/worker/tests/checkpoint_test.rs` | services | worker | P5 | Backend Platform | Verifies checkpoint in the worker service. |
| 1,765 | `services/worker/tests/upload_test.rs` | services | worker | P5 | Backend Platform | Verifies upload in the worker service. |
| 1,766 | `services/worker/tests/resources_test.rs` | services | worker | P5 | Backend Platform | Verifies resources in the worker service. |
| 1,767 | `services/worker/tests/limits_test.rs` | services | worker | P5 | Backend Platform | Verifies limits in the worker service. |
| 1,768 | `services/worker/tests/sandbox_test.rs` | services | worker | P5 | Backend Platform | Verifies sandbox in the worker service. |
| 1,769 | `services/worker/tests/plugins_test.rs` | services | worker | P5 | Backend Platform | Verifies plugins in the worker service. |
| 1,770 | `services/worker/config/development.yaml` | services | worker | P5 | Backend Platform | Configures worker for development operation. |
| 1,771 | `services/worker/config/test.yaml` | services | worker | P5 | Backend Platform | Configures worker for test operation. |
| 1,772 | `services/worker/config/staging.yaml` | services | worker | P5 | Backend Platform | Configures worker for staging operation. |
| 1,773 | `services/worker/config/production.yaml` | services | worker | P5 | Backend Platform | Configures worker for production operation. |
| 1,774 | `services/worker/config/logging.yaml` | services | worker | P5 | Backend Platform | Configures worker for logging operation. |
| 1,775 | `services/worker/config/limits.yaml` | services | worker | P5 | Backend Platform | Configures worker for limits operation. |
| 1,776 | `services/worker/docs/architecture.md` | services | worker | P5 | Backend Platform | Documents architecture for the worker service. |
| 1,777 | `services/worker/docs/api.md` | services | worker | P5 | Backend Platform | Documents api for the worker service. |
| 1,778 | `services/worker/docs/operations.md` | services | worker | P5 | Backend Platform | Documents operations for the worker service. |
| 1,779 | `services/worker/docs/failures.md` | services | worker | P5 | Backend Platform | Documents failures for the worker service. |
| 1,780 | `services/worker/docs/security.md` | services | worker | P5 | Backend Platform | Documents security for the worker service. |
| 1,781 | `services/artifact/README.md` | services | artifact | P5 | Backend Platform | Documents the purpose, boundaries, and usage of artifact. |
| 1,782 | `services/artifact/Dockerfile` | services | artifact | P5 | Backend Platform | Provides Dockerfile for artifact. |
| 1,783 | `services/artifact/Cargo.toml` | services | artifact | P5 | Backend Platform | Declares the build, dependencies, and package metadata for artifact. |
| 1,784 | `services/artifact/.env.example` | services | artifact | P5 | Backend Platform | Provides .env for artifact. |
| 1,785 | `services/artifact/src/main.rs` | services | artifact | P5 | Backend Platform | Implements main for the artifact service. |
| 1,786 | `services/artifact/src/config.rs` | services | artifact | P5 | Backend Platform | Implements config for the artifact service. |
| 1,787 | `services/artifact/src/routes.rs` | services | artifact | P5 | Backend Platform | Implements routes for the artifact service. |
| 1,788 | `services/artifact/src/object.rs` | services | artifact | P5 | Backend Platform | Implements object for the artifact service. |
| 1,789 | `services/artifact/src/upload.rs` | services | artifact | P5 | Backend Platform | Implements upload for the artifact service. |
| 1,790 | `services/artifact/src/download.rs` | services | artifact | P5 | Backend Platform | Implements download for the artifact service. |
| 1,791 | `services/artifact/src/metadata.rs` | services | artifact | P5 | Backend Platform | Implements metadata for the artifact service. |
| 1,792 | `services/artifact/src/checksum.rs` | services | artifact | P5 | Backend Platform | Implements checksum for the artifact service. |
| 1,793 | `services/artifact/src/signature.rs` | services | artifact | P5 | Backend Platform | Implements signature for the artifact service. |
| 1,794 | `services/artifact/src/retention.rs` | services | artifact | P5 | Backend Platform | Implements retention for the artifact service. |
| 1,795 | `services/artifact/src/gc.rs` | services | artifact | P5 | Backend Platform | Implements gc for the artifact service. |
| 1,796 | `services/artifact/src/multipart.rs` | services | artifact | P5 | Backend Platform | Implements multipart for the artifact service. |
| 1,797 | `services/artifact/src/authorization.rs` | services | artifact | P5 | Backend Platform | Implements authorization for the artifact service. |
| 1,798 | `services/artifact/src/storage.rs` | services | artifact | P5 | Backend Platform | Implements storage for the artifact service. |
| 1,799 | `services/artifact/src/database.rs` | services | artifact | P5 | Backend Platform | Implements database for the artifact service. |
| 1,800 | `services/artifact/src/cache.rs` | services | artifact | P5 | Backend Platform | Implements cache for the artifact service. |
| 1,801 | `services/artifact/src/telemetry.rs` | services | artifact | P5 | Backend Platform | Implements telemetry for the artifact service. |
| 1,802 | `services/artifact/src/health.rs` | services | artifact | P5 | Backend Platform | Implements health for the artifact service. |
| 1,803 | `services/artifact/src/limits.rs` | services | artifact | P5 | Backend Platform | Implements limits for the artifact service. |
| 1,804 | `services/artifact/src/errors.rs` | services | artifact | P5 | Backend Platform | Implements errors for the artifact service. |
| 1,805 | `services/artifact/tests/main_test.rs` | services | artifact | P5 | Backend Platform | Verifies main in the artifact service. |
| 1,806 | `services/artifact/tests/config_test.rs` | services | artifact | P5 | Backend Platform | Verifies config in the artifact service. |
| 1,807 | `services/artifact/tests/routes_test.rs` | services | artifact | P5 | Backend Platform | Verifies routes in the artifact service. |
| 1,808 | `services/artifact/tests/object_test.rs` | services | artifact | P5 | Backend Platform | Verifies object in the artifact service. |
| 1,809 | `services/artifact/tests/upload_test.rs` | services | artifact | P5 | Backend Platform | Verifies upload in the artifact service. |
| 1,810 | `services/artifact/tests/download_test.rs` | services | artifact | P5 | Backend Platform | Verifies download in the artifact service. |
| 1,811 | `services/artifact/tests/metadata_test.rs` | services | artifact | P5 | Backend Platform | Verifies metadata in the artifact service. |
| 1,812 | `services/artifact/tests/checksum_test.rs` | services | artifact | P5 | Backend Platform | Verifies checksum in the artifact service. |
| 1,813 | `services/artifact/tests/signature_test.rs` | services | artifact | P5 | Backend Platform | Verifies signature in the artifact service. |
| 1,814 | `services/artifact/tests/retention_test.rs` | services | artifact | P5 | Backend Platform | Verifies retention in the artifact service. |
| 1,815 | `services/artifact/tests/gc_test.rs` | services | artifact | P5 | Backend Platform | Verifies gc in the artifact service. |
| 1,816 | `services/artifact/tests/multipart_test.rs` | services | artifact | P5 | Backend Platform | Verifies multipart in the artifact service. |
| 1,817 | `services/artifact/config/development.yaml` | services | artifact | P5 | Backend Platform | Configures artifact for development operation. |
| 1,818 | `services/artifact/config/test.yaml` | services | artifact | P5 | Backend Platform | Configures artifact for test operation. |
| 1,819 | `services/artifact/config/staging.yaml` | services | artifact | P5 | Backend Platform | Configures artifact for staging operation. |
| 1,820 | `services/artifact/config/production.yaml` | services | artifact | P5 | Backend Platform | Configures artifact for production operation. |
| 1,821 | `services/artifact/config/logging.yaml` | services | artifact | P5 | Backend Platform | Configures artifact for logging operation. |
| 1,822 | `services/artifact/config/limits.yaml` | services | artifact | P5 | Backend Platform | Configures artifact for limits operation. |
| 1,823 | `services/artifact/docs/architecture.md` | services | artifact | P5 | Backend Platform | Documents architecture for the artifact service. |
| 1,824 | `services/artifact/docs/api.md` | services | artifact | P5 | Backend Platform | Documents api for the artifact service. |
| 1,825 | `services/artifact/docs/operations.md` | services | artifact | P5 | Backend Platform | Documents operations for the artifact service. |
| 1,826 | `services/artifact/docs/failures.md` | services | artifact | P5 | Backend Platform | Documents failures for the artifact service. |
| 1,827 | `services/artifact/docs/security.md` | services | artifact | P5 | Backend Platform | Documents security for the artifact service. |
| 1,828 | `services/gateway/README.md` | services | gateway | P5 | Backend Platform | Documents the purpose, boundaries, and usage of gateway. |
| 1,829 | `services/gateway/Dockerfile` | services | gateway | P5 | Backend Platform | Provides Dockerfile for gateway. |
| 1,830 | `services/gateway/Cargo.toml` | services | gateway | P5 | Backend Platform | Declares the build, dependencies, and package metadata for gateway. |
| 1,831 | `services/gateway/.env.example` | services | gateway | P5 | Backend Platform | Provides .env for gateway. |
| 1,832 | `services/gateway/src/main.rs` | services | gateway | P5 | Backend Platform | Implements main for the gateway service. |
| 1,833 | `services/gateway/src/config.rs` | services | gateway | P5 | Backend Platform | Implements config for the gateway service. |
| 1,834 | `services/gateway/src/proxy.rs` | services | gateway | P5 | Backend Platform | Implements proxy for the gateway service. |
| 1,835 | `services/gateway/src/routing.rs` | services | gateway | P5 | Backend Platform | Implements routing for the gateway service. |
| 1,836 | `services/gateway/src/auth.rs` | services | gateway | P5 | Backend Platform | Implements auth for the gateway service. |
| 1,837 | `services/gateway/src/rate_limit.rs` | services | gateway | P5 | Backend Platform | Implements rate limit for the gateway service. |
| 1,838 | `services/gateway/src/cors.rs` | services | gateway | P5 | Backend Platform | Implements cors for the gateway service. |
| 1,839 | `services/gateway/src/headers.rs` | services | gateway | P5 | Backend Platform | Implements headers for the gateway service. |
| 1,840 | `services/gateway/src/tls.rs` | services | gateway | P5 | Backend Platform | Implements tls for the gateway service. |
| 1,841 | `services/gateway/src/health.rs` | services | gateway | P5 | Backend Platform | Implements health for the gateway service. |
| 1,842 | `services/gateway/src/metrics.rs` | services | gateway | P5 | Backend Platform | Implements metrics for the gateway service. |
| 1,843 | `services/gateway/src/tracing.rs` | services | gateway | P5 | Backend Platform | Implements tracing for the gateway service. |
| 1,844 | `services/gateway/src/timeouts.rs` | services | gateway | P5 | Backend Platform | Implements timeouts for the gateway service. |
| 1,845 | `services/gateway/src/retry.rs` | services | gateway | P5 | Backend Platform | Implements retry for the gateway service. |
| 1,846 | `services/gateway/src/body_limits.rs` | services | gateway | P5 | Backend Platform | Implements body limits for the gateway service. |
| 1,847 | `services/gateway/src/uploads.rs` | services | gateway | P5 | Backend Platform | Implements uploads for the gateway service. |
| 1,848 | `services/gateway/src/downloads.rs` | services | gateway | P5 | Backend Platform | Implements downloads for the gateway service. |
| 1,849 | `services/gateway/src/events.rs` | services | gateway | P5 | Backend Platform | Implements events for the gateway service. |
| 1,850 | `services/gateway/src/shutdown.rs` | services | gateway | P5 | Backend Platform | Implements shutdown for the gateway service. |
| 1,851 | `services/gateway/src/errors.rs` | services | gateway | P5 | Backend Platform | Implements errors for the gateway service. |
| 1,852 | `services/gateway/tests/main_test.rs` | services | gateway | P5 | Backend Platform | Verifies main in the gateway service. |
| 1,853 | `services/gateway/tests/config_test.rs` | services | gateway | P5 | Backend Platform | Verifies config in the gateway service. |
| 1,854 | `services/gateway/tests/proxy_test.rs` | services | gateway | P5 | Backend Platform | Verifies proxy in the gateway service. |
| 1,855 | `services/gateway/tests/routing_test.rs` | services | gateway | P5 | Backend Platform | Verifies routing in the gateway service. |
| 1,856 | `services/gateway/tests/auth_test.rs` | services | gateway | P5 | Backend Platform | Verifies auth in the gateway service. |
| 1,857 | `services/gateway/tests/rate_limit_test.rs` | services | gateway | P5 | Backend Platform | Verifies rate limit in the gateway service. |
| 1,858 | `services/gateway/tests/cors_test.rs` | services | gateway | P5 | Backend Platform | Verifies cors in the gateway service. |
| 1,859 | `services/gateway/tests/headers_test.rs` | services | gateway | P5 | Backend Platform | Verifies headers in the gateway service. |
| 1,860 | `services/gateway/tests/tls_test.rs` | services | gateway | P5 | Backend Platform | Verifies tls in the gateway service. |
| 1,861 | `services/gateway/tests/health_test.rs` | services | gateway | P5 | Backend Platform | Verifies health in the gateway service. |
| 1,862 | `services/gateway/tests/metrics_test.rs` | services | gateway | P5 | Backend Platform | Verifies metrics in the gateway service. |
| 1,863 | `services/gateway/tests/tracing_test.rs` | services | gateway | P5 | Backend Platform | Verifies tracing in the gateway service. |
| 1,864 | `services/gateway/config/development.yaml` | services | gateway | P5 | Backend Platform | Configures gateway for development operation. |
| 1,865 | `services/gateway/config/test.yaml` | services | gateway | P5 | Backend Platform | Configures gateway for test operation. |
| 1,866 | `services/gateway/config/staging.yaml` | services | gateway | P5 | Backend Platform | Configures gateway for staging operation. |
| 1,867 | `services/gateway/config/production.yaml` | services | gateway | P5 | Backend Platform | Configures gateway for production operation. |
| 1,868 | `services/gateway/config/logging.yaml` | services | gateway | P5 | Backend Platform | Configures gateway for logging operation. |
| 1,869 | `services/gateway/config/limits.yaml` | services | gateway | P5 | Backend Platform | Configures gateway for limits operation. |
| 1,870 | `services/gateway/docs/architecture.md` | services | gateway | P5 | Backend Platform | Documents architecture for the gateway service. |
| 1,871 | `services/gateway/docs/api.md` | services | gateway | P5 | Backend Platform | Documents api for the gateway service. |
| 1,872 | `services/gateway/docs/operations.md` | services | gateway | P5 | Backend Platform | Documents operations for the gateway service. |
| 1,873 | `services/gateway/docs/failures.md` | services | gateway | P5 | Backend Platform | Documents failures for the gateway service. |
| 1,874 | `services/gateway/docs/security.md` | services | gateway | P5 | Backend Platform | Documents security for the gateway service. |
| 1,875 | `docs/getting-started/README.md` | docs | getting-started | P3 | Documentation | Introduces the getting-started documentation section. |
| 1,876 | `docs/getting-started/installation.md` | docs | getting-started | P3 | Documentation | Explains installation for getting-started. |
| 1,877 | `docs/getting-started/quickstart.md` | docs | getting-started | P3 | Documentation | Explains quickstart for getting-started. |
| 1,878 | `docs/getting-started/first-world.md` | docs | getting-started | P3 | Documentation | Explains first world for getting-started. |
| 1,879 | `docs/getting-started/studio.md` | docs | getting-started | P3 | Documentation | Explains studio for getting-started. |
| 1,880 | `docs/getting-started/python.md` | docs | getting-started | P3 | Documentation | Explains python for getting-started. |
| 1,881 | `docs/getting-started/cli.md` | docs | getting-started | P3 | Documentation | Explains cli for getting-started. |
| 1,882 | `docs/getting-started/concepts.md` | docs | getting-started | P3 | Documentation | Explains concepts for getting-started. |
| 1,883 | `docs/getting-started/examples.md` | docs | getting-started | P3 | Documentation | Explains examples for getting-started. |
| 1,884 | `docs/getting-started/troubleshooting.md` | docs | getting-started | P3 | Documentation | Explains troubleshooting for getting-started. |
| 1,885 | `docs/concepts/world-ir/README.md` | docs | concepts/world-ir | P3 | Documentation | Introduces the concepts/world-ir documentation section. |
| 1,886 | `docs/concepts/world-ir/world.md` | docs | concepts/world-ir | P3 | Documentation | Explains world for concepts/world-ir. |
| 1,887 | `docs/concepts/world-ir/variables.md` | docs | concepts/world-ir | P3 | Documentation | Explains variables for concepts/world-ir. |
| 1,888 | `docs/concepts/world-ir/types.md` | docs | concepts/world-ir | P3 | Documentation | Explains types for concepts/world-ir. |
| 1,889 | `docs/concepts/world-ir/units.md` | docs | concepts/world-ir | P3 | Documentation | Explains units for concepts/world-ir. |
| 1,890 | `docs/concepts/world-ir/time.md` | docs | concepts/world-ir | P3 | Documentation | Explains time for concepts/world-ir. |
| 1,891 | `docs/concepts/world-ir/laws.md` | docs | concepts/world-ir | P3 | Documentation | Explains laws for concepts/world-ir. |
| 1,892 | `docs/concepts/world-ir/events.md` | docs | concepts/world-ir | P3 | Documentation | Explains events for concepts/world-ir. |
| 1,893 | `docs/concepts/world-ir/provenance.md` | docs | concepts/world-ir | P3 | Documentation | Explains provenance for concepts/world-ir. |
| 1,894 | `docs/concepts/world-ir/versioning.md` | docs | concepts/world-ir | P3 | Documentation | Explains versioning for concepts/world-ir. |
| 1,895 | `docs/concepts/equations/README.md` | docs | concepts/equations | P3 | Documentation | Introduces the concepts/equations documentation section. |
| 1,896 | `docs/concepts/equations/expressions.md` | docs | concepts/equations | P3 | Documentation | Explains expressions for concepts/equations. |
| 1,897 | `docs/concepts/equations/operators.md` | docs | concepts/equations | P3 | Documentation | Explains operators for concepts/equations. |
| 1,898 | `docs/concepts/equations/continuous.md` | docs | concepts/equations | P3 | Documentation | Explains continuous for concepts/equations. |
| 1,899 | `docs/concepts/equations/discrete.md` | docs | concepts/equations | P3 | Documentation | Explains discrete for concepts/equations. |
| 1,900 | `docs/concepts/equations/algebraic.md` | docs | concepts/equations | P3 | Documentation | Explains algebraic for concepts/equations. |
| 1,901 | `docs/concepts/equations/stochastic.md` | docs | concepts/equations | P3 | Documentation | Explains stochastic for concepts/equations. |
| 1,902 | `docs/concepts/equations/constraints.md` | docs | concepts/equations | P3 | Documentation | Explains constraints for concepts/equations. |
| 1,903 | `docs/concepts/equations/simplification.md` | docs | concepts/equations | P3 | Documentation | Explains simplification for concepts/equations. |
| 1,904 | `docs/concepts/equations/alternatives.md` | docs | concepts/equations | P3 | Documentation | Explains alternatives for concepts/equations. |
| 1,905 | `docs/concepts/causality/README.md` | docs | concepts/causality | P3 | Documentation | Introduces the concepts/causality documentation section. |
| 1,906 | `docs/concepts/causality/assumptions.md` | docs | concepts/causality | P3 | Documentation | Explains assumptions for concepts/causality. |
| 1,907 | `docs/concepts/causality/graphs.md` | docs | concepts/causality | P3 | Documentation | Explains graphs for concepts/causality. |
| 1,908 | `docs/concepts/causality/lags.md` | docs | concepts/causality | P3 | Documentation | Explains lags for concepts/causality. |
| 1,909 | `docs/concepts/causality/identification.md` | docs | concepts/causality | P3 | Documentation | Explains identification for concepts/causality. |
| 1,910 | `docs/concepts/causality/interventions.md` | docs | concepts/causality | P3 | Documentation | Explains interventions for concepts/causality. |
| 1,911 | `docs/concepts/causality/counterfactuals.md` | docs | concepts/causality | P3 | Documentation | Explains counterfactuals for concepts/causality. |
| 1,912 | `docs/concepts/causality/equivalence.md` | docs | concepts/causality | P3 | Documentation | Explains equivalence for concepts/causality. |
| 1,913 | `docs/concepts/causality/stability.md` | docs | concepts/causality | P3 | Documentation | Explains stability for concepts/causality. |
| 1,914 | `docs/concepts/causality/limitations.md` | docs | concepts/causality | P3 | Documentation | Explains limitations for concepts/causality. |
| 1,915 | `docs/concepts/regimes/README.md` | docs | concepts/regimes | P3 | Documentation | Introduces the concepts/regimes documentation section. |
| 1,916 | `docs/concepts/regimes/change-points.md` | docs | concepts/regimes | P3 | Documentation | Explains change points for concepts/regimes. |
| 1,917 | `docs/concepts/regimes/states.md` | docs | concepts/regimes | P3 | Documentation | Explains states for concepts/regimes. |
| 1,918 | `docs/concepts/regimes/transitions.md` | docs | concepts/regimes | P3 | Documentation | Explains transitions for concepts/regimes. |
| 1,919 | `docs/concepts/regimes/guards.md` | docs | concepts/regimes | P3 | Documentation | Explains guards for concepts/regimes. |
| 1,920 | `docs/concepts/regimes/events.md` | docs | concepts/regimes | P3 | Documentation | Explains events for concepts/regimes. |
| 1,921 | `docs/concepts/regimes/shared-laws.md` | docs | concepts/regimes | P3 | Documentation | Explains shared laws for concepts/regimes. |
| 1,922 | `docs/concepts/regimes/specific-laws.md` | docs | concepts/regimes | P3 | Documentation | Explains specific laws for concepts/regimes. |
| 1,923 | `docs/concepts/regimes/probabilities.md` | docs | concepts/regimes | P3 | Documentation | Explains probabilities for concepts/regimes. |
| 1,924 | `docs/concepts/regimes/visualization.md` | docs | concepts/regimes | P3 | Documentation | Explains visualization for concepts/regimes. |
| 1,925 | `docs/concepts/uncertainty/README.md` | docs | concepts/uncertainty | P3 | Documentation | Introduces the concepts/uncertainty documentation section. |
| 1,926 | `docs/concepts/uncertainty/sources.md` | docs | concepts/uncertainty | P3 | Documentation | Explains sources for concepts/uncertainty. |
| 1,927 | `docs/concepts/uncertainty/parameters.md` | docs | concepts/uncertainty | P3 | Documentation | Explains parameters for concepts/uncertainty. |
| 1,928 | `docs/concepts/uncertainty/structure.md` | docs | concepts/uncertainty | P3 | Documentation | Explains structure for concepts/uncertainty. |
| 1,929 | `docs/concepts/uncertainty/trajectories.md` | docs | concepts/uncertainty | P3 | Documentation | Explains trajectories for concepts/uncertainty. |
| 1,930 | `docs/concepts/uncertainty/bootstrap.md` | docs | concepts/uncertainty | P3 | Documentation | Explains bootstrap for concepts/uncertainty. |
| 1,931 | `docs/concepts/uncertainty/ensembles.md` | docs | concepts/uncertainty | P3 | Documentation | Explains ensembles for concepts/uncertainty. |
| 1,932 | `docs/concepts/uncertainty/coverage.md` | docs | concepts/uncertainty | P3 | Documentation | Explains coverage for concepts/uncertainty. |
| 1,933 | `docs/concepts/uncertainty/propagation.md` | docs | concepts/uncertainty | P3 | Documentation | Explains propagation for concepts/uncertainty. |
| 1,934 | `docs/concepts/uncertainty/communication.md` | docs | concepts/uncertainty | P3 | Documentation | Explains communication for concepts/uncertainty. |
| 1,935 | `docs/guides/data/README.md` | docs | guides/data | P3 | Documentation | Introduces the guides/data documentation section. |
| 1,936 | `docs/guides/data/csv.md` | docs | guides/data | P3 | Documentation | Explains csv for guides/data. |
| 1,937 | `docs/guides/data/parquet.md` | docs | guides/data | P3 | Documentation | Explains parquet for guides/data. |
| 1,938 | `docs/guides/data/arrow.md` | docs | guides/data | P3 | Documentation | Explains arrow for guides/data. |
| 1,939 | `docs/guides/data/pandas.md` | docs | guides/data | P3 | Documentation | Explains pandas for guides/data. |
| 1,940 | `docs/guides/data/polars.md` | docs | guides/data | P3 | Documentation | Explains polars for guides/data. |
| 1,941 | `docs/guides/data/xarray.md` | docs | guides/data | P3 | Documentation | Explains xarray for guides/data. |
| 1,942 | `docs/guides/data/irregular-time.md` | docs | guides/data | P3 | Documentation | Explains irregular time for guides/data. |
| 1,943 | `docs/guides/data/missing-data.md` | docs | guides/data | P3 | Documentation | Explains missing data for guides/data. |
| 1,944 | `docs/guides/data/units.md` | docs | guides/data | P3 | Documentation | Explains units for guides/data. |
| 1,945 | `docs/guides/discovery/README.md` | docs | guides/discovery | P3 | Documentation | Introduces the guides/discovery documentation section. |
| 1,946 | `docs/guides/discovery/planning.md` | docs | guides/discovery | P3 | Documentation | Explains planning for guides/discovery. |
| 1,947 | `docs/guides/discovery/operators.md` | docs | guides/discovery | P3 | Documentation | Explains operators for guides/discovery. |
| 1,948 | `docs/guides/discovery/constraints.md` | docs | guides/discovery | P3 | Documentation | Explains constraints for guides/discovery. |
| 1,949 | `docs/guides/discovery/derivatives.md` | docs | guides/discovery | P3 | Documentation | Explains derivatives for guides/discovery. |
| 1,950 | `docs/guides/discovery/sparse.md` | docs | guides/discovery | P3 | Documentation | Explains sparse for guides/discovery. |
| 1,951 | `docs/guides/discovery/symbolic.md` | docs | guides/discovery | P3 | Documentation | Explains symbolic for guides/discovery. |
| 1,952 | `docs/guides/discovery/ranking.md` | docs | guides/discovery | P3 | Documentation | Explains ranking for guides/discovery. |
| 1,953 | `docs/guides/discovery/checkpoints.md` | docs | guides/discovery | P3 | Documentation | Explains checkpoints for guides/discovery. |
| 1,954 | `docs/guides/discovery/reproducibility.md` | docs | guides/discovery | P3 | Documentation | Explains reproducibility for guides/discovery. |
| 1,955 | `docs/guides/simulation/README.md` | docs | guides/simulation | P3 | Documentation | Introduces the guides/simulation documentation section. |
| 1,956 | `docs/guides/simulation/initial-state.md` | docs | guides/simulation | P3 | Documentation | Explains initial state for guides/simulation. |
| 1,957 | `docs/guides/simulation/horizon.md` | docs | guides/simulation | P3 | Documentation | Explains horizon for guides/simulation. |
| 1,958 | `docs/guides/simulation/controls.md` | docs | guides/simulation | P3 | Documentation | Explains controls for guides/simulation. |
| 1,959 | `docs/guides/simulation/interventions.md` | docs | guides/simulation | P3 | Documentation | Explains interventions for guides/simulation. |
| 1,960 | `docs/guides/simulation/shocks.md` | docs | guides/simulation | P3 | Documentation | Explains shocks for guides/simulation. |
| 1,961 | `docs/guides/simulation/events.md` | docs | guides/simulation | P3 | Documentation | Explains events for guides/simulation. |
| 1,962 | `docs/guides/simulation/ensembles.md` | docs | guides/simulation | P3 | Documentation | Explains ensembles for guides/simulation. |
| 1,963 | `docs/guides/simulation/comparison.md` | docs | guides/simulation | P3 | Documentation | Explains comparison for guides/simulation. |
| 1,964 | `docs/guides/simulation/export.md` | docs | guides/simulation | P3 | Documentation | Explains export for guides/simulation. |
| 1,965 | `docs/guides/studio/README.md` | docs | guides/studio | P3 | Documentation | Introduces the guides/studio documentation section. |
| 1,966 | `docs/guides/studio/workspace.md` | docs | guides/studio | P3 | Documentation | Explains workspace for guides/studio. |
| 1,967 | `docs/guides/studio/data-lens.md` | docs | guides/studio | P3 | Documentation | Explains data lens for guides/studio. |
| 1,968 | `docs/guides/studio/discovery-canvas.md` | docs | guides/studio | P3 | Documentation | Explains discovery canvas for guides/studio. |
| 1,969 | `docs/guides/studio/equation-explorer.md` | docs | guides/studio | P3 | Documentation | Explains equation explorer for guides/studio. |
| 1,970 | `docs/guides/studio/structure-map.md` | docs | guides/studio | P3 | Documentation | Explains structure map for guides/studio. |
| 1,971 | `docs/guides/studio/regime-timeline.md` | docs | guides/studio | P3 | Documentation | Explains regime timeline for guides/studio. |
| 1,972 | `docs/guides/studio/world-lab.md` | docs | guides/studio | P3 | Documentation | Explains world lab for guides/studio. |
| 1,973 | `docs/guides/studio/uncertainty-lens.md` | docs | guides/studio | P3 | Documentation | Explains uncertainty lens for guides/studio. |
| 1,974 | `docs/guides/studio/export.md` | docs | guides/studio | P3 | Documentation | Explains export for guides/studio. |
| 1,975 | `docs/methods/differentiation/README.md` | docs | methods/differentiation | P3 | Documentation | Introduces the methods/differentiation documentation section. |
| 1,976 | `docs/methods/differentiation/finite.md` | docs | methods/differentiation | P3 | Documentation | Explains finite for methods/differentiation. |
| 1,977 | `docs/methods/differentiation/savgol.md` | docs | methods/differentiation | P3 | Documentation | Explains savgol for methods/differentiation. |
| 1,978 | `docs/methods/differentiation/spline.md` | docs | methods/differentiation | P3 | Documentation | Explains spline for methods/differentiation. |
| 1,979 | `docs/methods/differentiation/tvreg.md` | docs | methods/differentiation | P3 | Documentation | Explains tvreg for methods/differentiation. |
| 1,980 | `docs/methods/differentiation/spectral.md` | docs | methods/differentiation | P3 | Documentation | Explains spectral for methods/differentiation. |
| 1,981 | `docs/methods/differentiation/weak-form.md` | docs | methods/differentiation | P3 | Documentation | Explains weak form for methods/differentiation. |
| 1,982 | `docs/methods/differentiation/irregular.md` | docs | methods/differentiation | P3 | Documentation | Explains irregular for methods/differentiation. |
| 1,983 | `docs/methods/differentiation/boundary.md` | docs | methods/differentiation | P3 | Documentation | Explains boundary for methods/differentiation. |
| 1,984 | `docs/methods/differentiation/selection.md` | docs | methods/differentiation | P3 | Documentation | Explains selection for methods/differentiation. |
| 1,985 | `docs/methods/sparse/README.md` | docs | methods/sparse | P3 | Documentation | Introduces the methods/sparse documentation section. |
| 1,986 | `docs/methods/sparse/libraries.md` | docs | methods/sparse | P3 | Documentation | Explains libraries for methods/sparse. |
| 1,987 | `docs/methods/sparse/stlsq.md` | docs | methods/sparse | P3 | Documentation | Explains stlsq for methods/sparse. |
| 1,988 | `docs/methods/sparse/sr3.md` | docs | methods/sparse | P3 | Documentation | Explains sr3 for methods/sparse. |
| 1,989 | `docs/methods/sparse/lasso.md` | docs | methods/sparse | P3 | Documentation | Explains lasso for methods/sparse. |
| 1,990 | `docs/methods/sparse/groups.md` | docs | methods/sparse | P3 | Documentation | Explains groups for methods/sparse. |
| 1,991 | `docs/methods/sparse/constraints.md` | docs | methods/sparse | P3 | Documentation | Explains constraints for methods/sparse. |
| 1,992 | `docs/methods/sparse/ensembles.md` | docs | methods/sparse | P3 | Documentation | Explains ensembles for methods/sparse. |
| 1,993 | `docs/methods/sparse/stability.md` | docs | methods/sparse | P3 | Documentation | Explains stability for methods/sparse. |
| 1,994 | `docs/methods/sparse/selection.md` | docs | methods/sparse | P3 | Documentation | Explains selection for methods/sparse. |
| 1,995 | `docs/methods/symbolic/README.md` | docs | methods/symbolic | P3 | Documentation | Introduces the methods/symbolic documentation section. |
| 1,996 | `docs/methods/symbolic/grammar.md` | docs | methods/symbolic | P3 | Documentation | Explains grammar for methods/symbolic. |
| 1,997 | `docs/methods/symbolic/initialization.md` | docs | methods/symbolic | P3 | Documentation | Explains initialization for methods/symbolic. |
| 1,998 | `docs/methods/symbolic/mutation.md` | docs | methods/symbolic | P3 | Documentation | Explains mutation for methods/symbolic. |
| 1,999 | `docs/methods/symbolic/crossover.md` | docs | methods/symbolic | P3 | Documentation | Explains crossover for methods/symbolic. |
| 2,000 | `docs/methods/symbolic/constants.md` | docs | methods/symbolic | P3 | Documentation | Explains constants for methods/symbolic. |
| 2,001 | `docs/methods/symbolic/simplification.md` | docs | methods/symbolic | P3 | Documentation | Explains simplification for methods/symbolic. |
| 2,002 | `docs/methods/symbolic/egraphs.md` | docs | methods/symbolic | P3 | Documentation | Explains egraphs for methods/symbolic. |
| 2,003 | `docs/methods/symbolic/frontiers.md` | docs | methods/symbolic | P3 | Documentation | Explains frontiers for methods/symbolic. |
| 2,004 | `docs/methods/symbolic/performance.md` | docs | methods/symbolic | P3 | Documentation | Explains performance for methods/symbolic. |
| 2,005 | `docs/methods/causal/README.md` | docs | methods/causal | P3 | Documentation | Introduces the methods/causal documentation section. |
| 2,006 | `docs/methods/causal/lagged.md` | docs | methods/causal | P3 | Documentation | Explains lagged for methods/causal. |
| 2,007 | `docs/methods/causal/granger.md` | docs | methods/causal | P3 | Documentation | Explains granger for methods/causal. |
| 2,008 | `docs/methods/causal/independence.md` | docs | methods/causal | P3 | Documentation | Explains independence for methods/causal. |
| 2,009 | `docs/methods/causal/score-based.md` | docs | methods/causal | P3 | Documentation | Explains score based for methods/causal. |
| 2,010 | `docs/methods/causal/time-order.md` | docs | methods/causal | P3 | Documentation | Explains time order for methods/causal. |
| 2,011 | `docs/methods/causal/bootstrap.md` | docs | methods/causal | P3 | Documentation | Explains bootstrap for methods/causal. |
| 2,012 | `docs/methods/causal/effects.md` | docs | methods/causal | P3 | Documentation | Explains effects for methods/causal. |
| 2,013 | `docs/methods/causal/sensitivity.md` | docs | methods/causal | P3 | Documentation | Explains sensitivity for methods/causal. |
| 2,014 | `docs/methods/causal/limits.md` | docs | methods/causal | P3 | Documentation | Explains limits for methods/causal. |
| 2,015 | `docs/methods/regime/README.md` | docs | methods/regime | P3 | Documentation | Introduces the methods/regime documentation section. |
| 2,016 | `docs/methods/regime/pelt.md` | docs | methods/regime | P3 | Documentation | Explains pelt for methods/regime. |
| 2,017 | `docs/methods/regime/binary.md` | docs | methods/regime | P3 | Documentation | Explains binary for methods/regime. |
| 2,018 | `docs/methods/regime/bocpd.md` | docs | methods/regime | P3 | Documentation | Explains bocpd for methods/regime. |
| 2,019 | `docs/methods/regime/hmm.md` | docs | methods/regime | P3 | Documentation | Explains hmm for methods/regime. |
| 2,020 | `docs/methods/regime/markov.md` | docs | methods/regime | P3 | Documentation | Explains markov for methods/regime. |
| 2,021 | `docs/methods/regime/guards.md` | docs | methods/regime | P3 | Documentation | Explains guards for methods/regime. |
| 2,022 | `docs/methods/regime/shared-structure.md` | docs | methods/regime | P3 | Documentation | Explains shared structure for methods/regime. |
| 2,023 | `docs/methods/regime/transitions.md` | docs | methods/regime | P3 | Documentation | Explains transitions for methods/regime. |
| 2,024 | `docs/methods/regime/selection.md` | docs | methods/regime | P3 | Documentation | Explains selection for methods/regime. |
| 2,025 | `docs/methods/uncertainty/README.md` | docs | methods/uncertainty | P3 | Documentation | Introduces the methods/uncertainty documentation section. |
| 2,026 | `docs/methods/uncertainty/residual.md` | docs | methods/uncertainty | P3 | Documentation | Explains residual for methods/uncertainty. |
| 2,027 | `docs/methods/uncertainty/bootstrap.md` | docs | methods/uncertainty | P3 | Documentation | Explains bootstrap for methods/uncertainty. |
| 2,028 | `docs/methods/uncertainty/profile.md` | docs | methods/uncertainty | P3 | Documentation | Explains profile for methods/uncertainty. |
| 2,029 | `docs/methods/uncertainty/covariance.md` | docs | methods/uncertainty | P3 | Documentation | Explains covariance for methods/uncertainty. |
| 2,030 | `docs/methods/uncertainty/ensembles.md` | docs | methods/uncertainty | P3 | Documentation | Explains ensembles for methods/uncertainty. |
| 2,031 | `docs/methods/uncertainty/structural.md` | docs | methods/uncertainty | P3 | Documentation | Explains structural for methods/uncertainty. |
| 2,032 | `docs/methods/uncertainty/trajectory.md` | docs | methods/uncertainty | P3 | Documentation | Explains trajectory for methods/uncertainty. |
| 2,033 | `docs/methods/uncertainty/summaries.md` | docs | methods/uncertainty | P3 | Documentation | Explains summaries for methods/uncertainty. |
| 2,034 | `docs/methods/uncertainty/calibration.md` | docs | methods/uncertainty | P3 | Documentation | Explains calibration for methods/uncertainty. |
| 2,035 | `docs/methods/simulation/README.md` | docs | methods/simulation | P3 | Documentation | Introduces the methods/simulation documentation section. |
| 2,036 | `docs/methods/simulation/discrete.md` | docs | methods/simulation | P3 | Documentation | Explains discrete for methods/simulation. |
| 2,037 | `docs/methods/simulation/ode.md` | docs | methods/simulation | P3 | Documentation | Explains ode for methods/simulation. |
| 2,038 | `docs/methods/simulation/sde.md` | docs | methods/simulation | P3 | Documentation | Explains sde for methods/simulation. |
| 2,039 | `docs/methods/simulation/hybrid.md` | docs | methods/simulation | P3 | Documentation | Explains hybrid for methods/simulation. |
| 2,040 | `docs/methods/simulation/events.md` | docs | methods/simulation | P3 | Documentation | Explains events for methods/simulation. |
| 2,041 | `docs/methods/simulation/noise.md` | docs | methods/simulation | P3 | Documentation | Explains noise for methods/simulation. |
| 2,042 | `docs/methods/simulation/interventions.md` | docs | methods/simulation | P3 | Documentation | Explains interventions for methods/simulation. |
| 2,043 | `docs/methods/simulation/ensembles.md` | docs | methods/simulation | P3 | Documentation | Explains ensembles for methods/simulation. |
| 2,044 | `docs/methods/simulation/diagnostics.md` | docs | methods/simulation | P3 | Documentation | Explains diagnostics for methods/simulation. |
| 2,045 | `docs/reference/python/README.md` | docs | reference/python | P3 | Documentation | Introduces the reference/python documentation section. |
| 2,046 | `docs/reference/python/dataset.md` | docs | reference/python | P3 | Documentation | Explains dataset for reference/python. |
| 2,047 | `docs/reference/python/plan.md` | docs | reference/python | P3 | Documentation | Explains plan for reference/python. |
| 2,048 | `docs/reference/python/run.md` | docs | reference/python | P3 | Documentation | Explains run for reference/python. |
| 2,049 | `docs/reference/python/candidate.md` | docs | reference/python | P3 | Documentation | Explains candidate for reference/python. |
| 2,050 | `docs/reference/python/world.md` | docs | reference/python | P3 | Documentation | Explains world for reference/python. |
| 2,051 | `docs/reference/python/scenario.md` | docs | reference/python | P3 | Documentation | Explains scenario for reference/python. |
| 2,052 | `docs/reference/python/trajectory.md` | docs | reference/python | P3 | Documentation | Explains trajectory for reference/python. |
| 2,053 | `docs/reference/python/bundle.md` | docs | reference/python | P3 | Documentation | Explains bundle for reference/python. |
| 2,054 | `docs/reference/python/errors.md` | docs | reference/python | P3 | Documentation | Explains errors for reference/python. |
| 2,055 | `docs/reference/rust/README.md` | docs | reference/rust | P3 | Documentation | Introduces the reference/rust documentation section. |
| 2,056 | `docs/reference/rust/core.md` | docs | reference/rust | P3 | Documentation | Explains core for reference/rust. |
| 2,057 | `docs/reference/rust/expr.md` | docs | reference/rust | P3 | Documentation | Explains expr for reference/rust. |
| 2,058 | `docs/reference/rust/world.md` | docs | reference/rust | P3 | Documentation | Explains world for reference/rust. |
| 2,059 | `docs/reference/rust/data.md` | docs | reference/rust | P3 | Documentation | Explains data for reference/rust. |
| 2,060 | `docs/reference/rust/discovery.md` | docs | reference/rust | P3 | Documentation | Explains discovery for reference/rust. |
| 2,061 | `docs/reference/rust/simulation.md` | docs | reference/rust | P3 | Documentation | Explains simulation for reference/rust. |
| 2,062 | `docs/reference/rust/bundle.md` | docs | reference/rust | P3 | Documentation | Explains bundle for reference/rust. |
| 2,063 | `docs/reference/rust/plugins.md` | docs | reference/rust | P3 | Documentation | Explains plugins for reference/rust. |
| 2,064 | `docs/reference/rust/errors.md` | docs | reference/rust | P3 | Documentation | Explains errors for reference/rust. |
| 2,065 | `docs/reference/cli/README.md` | docs | reference/cli | P3 | Documentation | Introduces the reference/cli documentation section. |
| 2,066 | `docs/reference/cli/discover.md` | docs | reference/cli | P3 | Documentation | Explains discover for reference/cli. |
| 2,067 | `docs/reference/cli/profile.md` | docs | reference/cli | P3 | Documentation | Explains profile for reference/cli. |
| 2,068 | `docs/reference/cli/inspect.md` | docs | reference/cli | P3 | Documentation | Explains inspect for reference/cli. |
| 2,069 | `docs/reference/cli/simulate.md` | docs | reference/cli | P3 | Documentation | Explains simulate for reference/cli. |
| 2,070 | `docs/reference/cli/intervene.md` | docs | reference/cli | P3 | Documentation | Explains intervene for reference/cli. |
| 2,071 | `docs/reference/cli/bundle.md` | docs | reference/cli | P3 | Documentation | Explains bundle for reference/cli. |
| 2,072 | `docs/reference/cli/plugin.md` | docs | reference/cli | P3 | Documentation | Explains plugin for reference/cli. |
| 2,073 | `docs/reference/cli/studio.md` | docs | reference/cli | P3 | Documentation | Explains studio for reference/cli. |
| 2,074 | `docs/reference/cli/serve.md` | docs | reference/cli | P3 | Documentation | Explains serve for reference/cli. |
| 2,075 | `docs/self-hosting/README.md` | docs | self-hosting | P3 | Documentation | Introduces the self-hosting documentation section. |
| 2,076 | `docs/self-hosting/compose.md` | docs | self-hosting | P3 | Documentation | Explains compose for self-hosting. |
| 2,077 | `docs/self-hosting/architecture.md` | docs | self-hosting | P3 | Documentation | Explains architecture for self-hosting. |
| 2,078 | `docs/self-hosting/database.md` | docs | self-hosting | P3 | Documentation | Explains database for self-hosting. |
| 2,079 | `docs/self-hosting/storage.md` | docs | self-hosting | P3 | Documentation | Explains storage for self-hosting. |
| 2,080 | `docs/self-hosting/workers.md` | docs | self-hosting | P3 | Documentation | Explains workers for self-hosting. |
| 2,081 | `docs/self-hosting/authentication.md` | docs | self-hosting | P3 | Documentation | Explains authentication for self-hosting. |
| 2,082 | `docs/self-hosting/backup.md` | docs | self-hosting | P3 | Documentation | Explains backup for self-hosting. |
| 2,083 | `docs/self-hosting/upgrade.md` | docs | self-hosting | P3 | Documentation | Explains upgrade for self-hosting. |
| 2,084 | `docs/self-hosting/airgap.md` | docs | self-hosting | P3 | Documentation | Explains airgap for self-hosting. |
| 2,085 | `docs/contributing/README.md` | docs | contributing | P3 | Documentation | Introduces the contributing documentation section. |
| 2,086 | `docs/contributing/development.md` | docs | contributing | P3 | Documentation | Explains development for contributing. |
| 2,087 | `docs/contributing/architecture.md` | docs | contributing | P3 | Documentation | Explains architecture for contributing. |
| 2,088 | `docs/contributing/operators.md` | docs | contributing | P3 | Documentation | Explains operators for contributing. |
| 2,089 | `docs/contributing/algorithms.md` | docs | contributing | P3 | Documentation | Explains algorithms for contributing. |
| 2,090 | `docs/contributing/datasets.md` | docs | contributing | P3 | Documentation | Explains datasets for contributing. |
| 2,091 | `docs/contributing/documentation.md` | docs | contributing | P3 | Documentation | Explains documentation for contributing. |
| 2,092 | `docs/contributing/benchmarks.md` | docs | contributing | P3 | Documentation | Explains benchmarks for contributing. |
| 2,093 | `docs/contributing/releases.md` | docs | contributing | P3 | Documentation | Explains releases for contributing. |
| 2,094 | `docs/contributing/governance.md` | docs | contributing | P3 | Documentation | Explains governance for contributing. |
| 2,095 | `docs/research/README.md` | docs | research | P3 | Documentation | Introduces the research documentation section. |
| 2,096 | `docs/research/methodology.md` | docs | research | P3 | Documentation | Explains methodology for research. |
| 2,097 | `docs/research/benchmarks.md` | docs | research | P3 | Documentation | Explains benchmarks for research. |
| 2,098 | `docs/research/limitations.md` | docs | research | P3 | Documentation | Explains limitations for research. |
| 2,099 | `docs/research/reading-list.md` | docs | research | P3 | Documentation | Explains reading list for research. |
| 2,100 | `docs/research/citations.md` | docs | research | P3 | Documentation | Explains citations for research. |
| 2,101 | `docs/research/reproducibility.md` | docs | research | P3 | Documentation | Explains reproducibility for research. |
| 2,102 | `docs/research/failure-cases.md` | docs | research | P3 | Documentation | Explains failure cases for research. |
| 2,103 | `docs/research/roadmap.md` | docs | research | P3 | Documentation | Explains roadmap for research. |
| 2,104 | `docs/research/collaboration.md` | docs | research | P3 | Documentation | Explains collaboration for research. |
| 2,105 | `examples/00-quickstart/README.md` | examples | 00-quickstart | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 00-quickstart. |
| 2,106 | `examples/00-quickstart/generate.py` | examples | 00-quickstart | P2 | Scientific Examples | Implements generate for 00-quickstart. |
| 2,107 | `examples/00-quickstart/discover.py` | examples | 00-quickstart | P2 | Scientific Examples | Implements discover for 00-quickstart. |
| 2,108 | `examples/00-quickstart/simulate.py` | examples | 00-quickstart | P2 | Scientific Examples | Implements simulate for 00-quickstart. |
| 2,109 | `examples/00-quickstart/config.toml` | examples | 00-quickstart | P2 | Scientific Examples | Configures or declares config for 00-quickstart. |
| 2,110 | `examples/00-quickstart/dataset-card.md` | examples | 00-quickstart | P2 | Scientific Examples | Documents dataset card for 00-quickstart. |
| 2,111 | `examples/00-quickstart/expected/world.json` | examples | 00-quickstart | P2 | Scientific Examples | Configures or declares world for 00-quickstart. |
| 2,112 | `examples/00-quickstart/expected/metrics.json` | examples | 00-quickstart | P2 | Scientific Examples | Configures or declares metrics for 00-quickstart. |
| 2,113 | `examples/00-quickstart/test_example.py` | examples | 00-quickstart | P2 | Scientific Examples | Verifies test example behavior for 00-quickstart. |
| 2,114 | `examples/01-lorenz/README.md` | examples | 01-lorenz | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 01-lorenz. |
| 2,115 | `examples/01-lorenz/generate.py` | examples | 01-lorenz | P2 | Scientific Examples | Implements generate for 01-lorenz. |
| 2,116 | `examples/01-lorenz/discover.py` | examples | 01-lorenz | P2 | Scientific Examples | Implements discover for 01-lorenz. |
| 2,117 | `examples/01-lorenz/simulate.py` | examples | 01-lorenz | P2 | Scientific Examples | Implements simulate for 01-lorenz. |
| 2,118 | `examples/01-lorenz/config.toml` | examples | 01-lorenz | P2 | Scientific Examples | Configures or declares config for 01-lorenz. |
| 2,119 | `examples/01-lorenz/dataset-card.md` | examples | 01-lorenz | P2 | Scientific Examples | Documents dataset card for 01-lorenz. |
| 2,120 | `examples/01-lorenz/expected/world.json` | examples | 01-lorenz | P2 | Scientific Examples | Configures or declares world for 01-lorenz. |
| 2,121 | `examples/01-lorenz/expected/metrics.json` | examples | 01-lorenz | P2 | Scientific Examples | Configures or declares metrics for 01-lorenz. |
| 2,122 | `examples/01-lorenz/test_example.py` | examples | 01-lorenz | P2 | Scientific Examples | Verifies test example behavior for 01-lorenz. |
| 2,123 | `examples/02-lotka-volterra/README.md` | examples | 02-lotka-volterra | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 02-lotka-volterra. |
| 2,124 | `examples/02-lotka-volterra/generate.py` | examples | 02-lotka-volterra | P2 | Scientific Examples | Implements generate for 02-lotka-volterra. |
| 2,125 | `examples/02-lotka-volterra/discover.py` | examples | 02-lotka-volterra | P2 | Scientific Examples | Implements discover for 02-lotka-volterra. |
| 2,126 | `examples/02-lotka-volterra/simulate.py` | examples | 02-lotka-volterra | P2 | Scientific Examples | Implements simulate for 02-lotka-volterra. |
| 2,127 | `examples/02-lotka-volterra/config.toml` | examples | 02-lotka-volterra | P2 | Scientific Examples | Configures or declares config for 02-lotka-volterra. |
| 2,128 | `examples/02-lotka-volterra/dataset-card.md` | examples | 02-lotka-volterra | P2 | Scientific Examples | Documents dataset card for 02-lotka-volterra. |
| 2,129 | `examples/02-lotka-volterra/expected/world.json` | examples | 02-lotka-volterra | P2 | Scientific Examples | Configures or declares world for 02-lotka-volterra. |
| 2,130 | `examples/02-lotka-volterra/expected/metrics.json` | examples | 02-lotka-volterra | P2 | Scientific Examples | Configures or declares metrics for 02-lotka-volterra. |
| 2,131 | `examples/02-lotka-volterra/test_example.py` | examples | 02-lotka-volterra | P2 | Scientific Examples | Verifies test example behavior for 02-lotka-volterra. |
| 2,132 | `examples/03-damped-pendulum/README.md` | examples | 03-damped-pendulum | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 03-damped-pendulum. |
| 2,133 | `examples/03-damped-pendulum/generate.py` | examples | 03-damped-pendulum | P2 | Scientific Examples | Implements generate for 03-damped-pendulum. |
| 2,134 | `examples/03-damped-pendulum/discover.py` | examples | 03-damped-pendulum | P2 | Scientific Examples | Implements discover for 03-damped-pendulum. |
| 2,135 | `examples/03-damped-pendulum/simulate.py` | examples | 03-damped-pendulum | P2 | Scientific Examples | Implements simulate for 03-damped-pendulum. |
| 2,136 | `examples/03-damped-pendulum/config.toml` | examples | 03-damped-pendulum | P2 | Scientific Examples | Configures or declares config for 03-damped-pendulum. |
| 2,137 | `examples/03-damped-pendulum/dataset-card.md` | examples | 03-damped-pendulum | P2 | Scientific Examples | Documents dataset card for 03-damped-pendulum. |
| 2,138 | `examples/03-damped-pendulum/expected/world.json` | examples | 03-damped-pendulum | P2 | Scientific Examples | Configures or declares world for 03-damped-pendulum. |
| 2,139 | `examples/03-damped-pendulum/expected/metrics.json` | examples | 03-damped-pendulum | P2 | Scientific Examples | Configures or declares metrics for 03-damped-pendulum. |
| 2,140 | `examples/03-damped-pendulum/test_example.py` | examples | 03-damped-pendulum | P2 | Scientific Examples | Verifies test example behavior for 03-damped-pendulum. |
| 2,141 | `examples/04-sir-epidemic/README.md` | examples | 04-sir-epidemic | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 04-sir-epidemic. |
| 2,142 | `examples/04-sir-epidemic/generate.py` | examples | 04-sir-epidemic | P2 | Scientific Examples | Implements generate for 04-sir-epidemic. |
| 2,143 | `examples/04-sir-epidemic/discover.py` | examples | 04-sir-epidemic | P2 | Scientific Examples | Implements discover for 04-sir-epidemic. |
| 2,144 | `examples/04-sir-epidemic/simulate.py` | examples | 04-sir-epidemic | P2 | Scientific Examples | Implements simulate for 04-sir-epidemic. |
| 2,145 | `examples/04-sir-epidemic/config.toml` | examples | 04-sir-epidemic | P2 | Scientific Examples | Configures or declares config for 04-sir-epidemic. |
| 2,146 | `examples/04-sir-epidemic/dataset-card.md` | examples | 04-sir-epidemic | P2 | Scientific Examples | Documents dataset card for 04-sir-epidemic. |
| 2,147 | `examples/04-sir-epidemic/expected/world.json` | examples | 04-sir-epidemic | P2 | Scientific Examples | Configures or declares world for 04-sir-epidemic. |
| 2,148 | `examples/04-sir-epidemic/expected/metrics.json` | examples | 04-sir-epidemic | P2 | Scientific Examples | Configures or declares metrics for 04-sir-epidemic. |
| 2,149 | `examples/04-sir-epidemic/test_example.py` | examples | 04-sir-epidemic | P2 | Scientific Examples | Verifies test example behavior for 04-sir-epidemic. |
| 2,150 | `examples/05-regime-switching/README.md` | examples | 05-regime-switching | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 05-regime-switching. |
| 2,151 | `examples/05-regime-switching/generate.py` | examples | 05-regime-switching | P2 | Scientific Examples | Implements generate for 05-regime-switching. |
| 2,152 | `examples/05-regime-switching/discover.py` | examples | 05-regime-switching | P2 | Scientific Examples | Implements discover for 05-regime-switching. |
| 2,153 | `examples/05-regime-switching/simulate.py` | examples | 05-regime-switching | P2 | Scientific Examples | Implements simulate for 05-regime-switching. |
| 2,154 | `examples/05-regime-switching/config.toml` | examples | 05-regime-switching | P2 | Scientific Examples | Configures or declares config for 05-regime-switching. |
| 2,155 | `examples/05-regime-switching/dataset-card.md` | examples | 05-regime-switching | P2 | Scientific Examples | Documents dataset card for 05-regime-switching. |
| 2,156 | `examples/05-regime-switching/expected/world.json` | examples | 05-regime-switching | P2 | Scientific Examples | Configures or declares world for 05-regime-switching. |
| 2,157 | `examples/05-regime-switching/expected/metrics.json` | examples | 05-regime-switching | P2 | Scientific Examples | Configures or declares metrics for 05-regime-switching. |
| 2,158 | `examples/05-regime-switching/test_example.py` | examples | 05-regime-switching | P2 | Scientific Examples | Verifies test example behavior for 05-regime-switching. |
| 2,159 | `examples/06-delayed-feedback/README.md` | examples | 06-delayed-feedback | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 06-delayed-feedback. |
| 2,160 | `examples/06-delayed-feedback/generate.py` | examples | 06-delayed-feedback | P2 | Scientific Examples | Implements generate for 06-delayed-feedback. |
| 2,161 | `examples/06-delayed-feedback/discover.py` | examples | 06-delayed-feedback | P2 | Scientific Examples | Implements discover for 06-delayed-feedback. |
| 2,162 | `examples/06-delayed-feedback/simulate.py` | examples | 06-delayed-feedback | P2 | Scientific Examples | Implements simulate for 06-delayed-feedback. |
| 2,163 | `examples/06-delayed-feedback/config.toml` | examples | 06-delayed-feedback | P2 | Scientific Examples | Configures or declares config for 06-delayed-feedback. |
| 2,164 | `examples/06-delayed-feedback/dataset-card.md` | examples | 06-delayed-feedback | P2 | Scientific Examples | Documents dataset card for 06-delayed-feedback. |
| 2,165 | `examples/06-delayed-feedback/expected/world.json` | examples | 06-delayed-feedback | P2 | Scientific Examples | Configures or declares world for 06-delayed-feedback. |
| 2,166 | `examples/06-delayed-feedback/expected/metrics.json` | examples | 06-delayed-feedback | P2 | Scientific Examples | Configures or declares metrics for 06-delayed-feedback. |
| 2,167 | `examples/06-delayed-feedback/test_example.py` | examples | 06-delayed-feedback | P2 | Scientific Examples | Verifies test example behavior for 06-delayed-feedback. |
| 2,168 | `examples/07-stochastic-volatility/README.md` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 07-stochastic-volatility. |
| 2,169 | `examples/07-stochastic-volatility/generate.py` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Implements generate for 07-stochastic-volatility. |
| 2,170 | `examples/07-stochastic-volatility/discover.py` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Implements discover for 07-stochastic-volatility. |
| 2,171 | `examples/07-stochastic-volatility/simulate.py` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Implements simulate for 07-stochastic-volatility. |
| 2,172 | `examples/07-stochastic-volatility/config.toml` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Configures or declares config for 07-stochastic-volatility. |
| 2,173 | `examples/07-stochastic-volatility/dataset-card.md` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Documents dataset card for 07-stochastic-volatility. |
| 2,174 | `examples/07-stochastic-volatility/expected/world.json` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Configures or declares world for 07-stochastic-volatility. |
| 2,175 | `examples/07-stochastic-volatility/expected/metrics.json` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Configures or declares metrics for 07-stochastic-volatility. |
| 2,176 | `examples/07-stochastic-volatility/test_example.py` | examples | 07-stochastic-volatility | P2 | Scientific Examples | Verifies test example behavior for 07-stochastic-volatility. |
| 2,177 | `examples/08-supply-demand/README.md` | examples | 08-supply-demand | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 08-supply-demand. |
| 2,178 | `examples/08-supply-demand/generate.py` | examples | 08-supply-demand | P2 | Scientific Examples | Implements generate for 08-supply-demand. |
| 2,179 | `examples/08-supply-demand/discover.py` | examples | 08-supply-demand | P2 | Scientific Examples | Implements discover for 08-supply-demand. |
| 2,180 | `examples/08-supply-demand/simulate.py` | examples | 08-supply-demand | P2 | Scientific Examples | Implements simulate for 08-supply-demand. |
| 2,181 | `examples/08-supply-demand/config.toml` | examples | 08-supply-demand | P2 | Scientific Examples | Configures or declares config for 08-supply-demand. |
| 2,182 | `examples/08-supply-demand/dataset-card.md` | examples | 08-supply-demand | P2 | Scientific Examples | Documents dataset card for 08-supply-demand. |
| 2,183 | `examples/08-supply-demand/expected/world.json` | examples | 08-supply-demand | P2 | Scientific Examples | Configures or declares world for 08-supply-demand. |
| 2,184 | `examples/08-supply-demand/expected/metrics.json` | examples | 08-supply-demand | P2 | Scientific Examples | Configures or declares metrics for 08-supply-demand. |
| 2,185 | `examples/08-supply-demand/test_example.py` | examples | 08-supply-demand | P2 | Scientific Examples | Verifies test example behavior for 08-supply-demand. |
| 2,186 | `examples/09-inventory-control/README.md` | examples | 09-inventory-control | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 09-inventory-control. |
| 2,187 | `examples/09-inventory-control/generate.py` | examples | 09-inventory-control | P2 | Scientific Examples | Implements generate for 09-inventory-control. |
| 2,188 | `examples/09-inventory-control/discover.py` | examples | 09-inventory-control | P2 | Scientific Examples | Implements discover for 09-inventory-control. |
| 2,189 | `examples/09-inventory-control/simulate.py` | examples | 09-inventory-control | P2 | Scientific Examples | Implements simulate for 09-inventory-control. |
| 2,190 | `examples/09-inventory-control/config.toml` | examples | 09-inventory-control | P2 | Scientific Examples | Configures or declares config for 09-inventory-control. |
| 2,191 | `examples/09-inventory-control/dataset-card.md` | examples | 09-inventory-control | P2 | Scientific Examples | Documents dataset card for 09-inventory-control. |
| 2,192 | `examples/09-inventory-control/expected/world.json` | examples | 09-inventory-control | P2 | Scientific Examples | Configures or declares world for 09-inventory-control. |
| 2,193 | `examples/09-inventory-control/expected/metrics.json` | examples | 09-inventory-control | P2 | Scientific Examples | Configures or declares metrics for 09-inventory-control. |
| 2,194 | `examples/09-inventory-control/test_example.py` | examples | 09-inventory-control | P2 | Scientific Examples | Verifies test example behavior for 09-inventory-control. |
| 2,195 | `examples/10-energy-load/README.md` | examples | 10-energy-load | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 10-energy-load. |
| 2,196 | `examples/10-energy-load/generate.py` | examples | 10-energy-load | P2 | Scientific Examples | Implements generate for 10-energy-load. |
| 2,197 | `examples/10-energy-load/discover.py` | examples | 10-energy-load | P2 | Scientific Examples | Implements discover for 10-energy-load. |
| 2,198 | `examples/10-energy-load/simulate.py` | examples | 10-energy-load | P2 | Scientific Examples | Implements simulate for 10-energy-load. |
| 2,199 | `examples/10-energy-load/config.toml` | examples | 10-energy-load | P2 | Scientific Examples | Configures or declares config for 10-energy-load. |
| 2,200 | `examples/10-energy-load/dataset-card.md` | examples | 10-energy-load | P2 | Scientific Examples | Documents dataset card for 10-energy-load. |
| 2,201 | `examples/10-energy-load/expected/world.json` | examples | 10-energy-load | P2 | Scientific Examples | Configures or declares world for 10-energy-load. |
| 2,202 | `examples/10-energy-load/expected/metrics.json` | examples | 10-energy-load | P2 | Scientific Examples | Configures or declares metrics for 10-energy-load. |
| 2,203 | `examples/10-energy-load/test_example.py` | examples | 10-energy-load | P2 | Scientific Examples | Verifies test example behavior for 10-energy-load. |
| 2,204 | `examples/11-customer-growth/README.md` | examples | 11-customer-growth | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 11-customer-growth. |
| 2,205 | `examples/11-customer-growth/generate.py` | examples | 11-customer-growth | P2 | Scientific Examples | Implements generate for 11-customer-growth. |
| 2,206 | `examples/11-customer-growth/discover.py` | examples | 11-customer-growth | P2 | Scientific Examples | Implements discover for 11-customer-growth. |
| 2,207 | `examples/11-customer-growth/simulate.py` | examples | 11-customer-growth | P2 | Scientific Examples | Implements simulate for 11-customer-growth. |
| 2,208 | `examples/11-customer-growth/config.toml` | examples | 11-customer-growth | P2 | Scientific Examples | Configures or declares config for 11-customer-growth. |
| 2,209 | `examples/11-customer-growth/dataset-card.md` | examples | 11-customer-growth | P2 | Scientific Examples | Documents dataset card for 11-customer-growth. |
| 2,210 | `examples/11-customer-growth/expected/world.json` | examples | 11-customer-growth | P2 | Scientific Examples | Configures or declares world for 11-customer-growth. |
| 2,211 | `examples/11-customer-growth/expected/metrics.json` | examples | 11-customer-growth | P2 | Scientific Examples | Configures or declares metrics for 11-customer-growth. |
| 2,212 | `examples/11-customer-growth/test_example.py` | examples | 11-customer-growth | P2 | Scientific Examples | Verifies test example behavior for 11-customer-growth. |
| 2,213 | `examples/12-macro-dynamics/README.md` | examples | 12-macro-dynamics | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 12-macro-dynamics. |
| 2,214 | `examples/12-macro-dynamics/generate.py` | examples | 12-macro-dynamics | P2 | Scientific Examples | Implements generate for 12-macro-dynamics. |
| 2,215 | `examples/12-macro-dynamics/discover.py` | examples | 12-macro-dynamics | P2 | Scientific Examples | Implements discover for 12-macro-dynamics. |
| 2,216 | `examples/12-macro-dynamics/simulate.py` | examples | 12-macro-dynamics | P2 | Scientific Examples | Implements simulate for 12-macro-dynamics. |
| 2,217 | `examples/12-macro-dynamics/config.toml` | examples | 12-macro-dynamics | P2 | Scientific Examples | Configures or declares config for 12-macro-dynamics. |
| 2,218 | `examples/12-macro-dynamics/dataset-card.md` | examples | 12-macro-dynamics | P2 | Scientific Examples | Documents dataset card for 12-macro-dynamics. |
| 2,219 | `examples/12-macro-dynamics/expected/world.json` | examples | 12-macro-dynamics | P2 | Scientific Examples | Configures or declares world for 12-macro-dynamics. |
| 2,220 | `examples/12-macro-dynamics/expected/metrics.json` | examples | 12-macro-dynamics | P2 | Scientific Examples | Configures or declares metrics for 12-macro-dynamics. |
| 2,221 | `examples/12-macro-dynamics/test_example.py` | examples | 12-macro-dynamics | P2 | Scientific Examples | Verifies test example behavior for 12-macro-dynamics. |
| 2,222 | `examples/13-market-microstructure/README.md` | examples | 13-market-microstructure | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 13-market-microstructure. |
| 2,223 | `examples/13-market-microstructure/generate.py` | examples | 13-market-microstructure | P2 | Scientific Examples | Implements generate for 13-market-microstructure. |
| 2,224 | `examples/13-market-microstructure/discover.py` | examples | 13-market-microstructure | P2 | Scientific Examples | Implements discover for 13-market-microstructure. |
| 2,225 | `examples/13-market-microstructure/simulate.py` | examples | 13-market-microstructure | P2 | Scientific Examples | Implements simulate for 13-market-microstructure. |
| 2,226 | `examples/13-market-microstructure/config.toml` | examples | 13-market-microstructure | P2 | Scientific Examples | Configures or declares config for 13-market-microstructure. |
| 2,227 | `examples/13-market-microstructure/dataset-card.md` | examples | 13-market-microstructure | P2 | Scientific Examples | Documents dataset card for 13-market-microstructure. |
| 2,228 | `examples/13-market-microstructure/expected/world.json` | examples | 13-market-microstructure | P2 | Scientific Examples | Configures or declares world for 13-market-microstructure. |
| 2,229 | `examples/13-market-microstructure/expected/metrics.json` | examples | 13-market-microstructure | P2 | Scientific Examples | Configures or declares metrics for 13-market-microstructure. |
| 2,230 | `examples/13-market-microstructure/test_example.py` | examples | 13-market-microstructure | P2 | Scientific Examples | Verifies test example behavior for 13-market-microstructure. |
| 2,231 | `examples/14-synthetic-control/README.md` | examples | 14-synthetic-control | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 14-synthetic-control. |
| 2,232 | `examples/14-synthetic-control/generate.py` | examples | 14-synthetic-control | P2 | Scientific Examples | Implements generate for 14-synthetic-control. |
| 2,233 | `examples/14-synthetic-control/discover.py` | examples | 14-synthetic-control | P2 | Scientific Examples | Implements discover for 14-synthetic-control. |
| 2,234 | `examples/14-synthetic-control/simulate.py` | examples | 14-synthetic-control | P2 | Scientific Examples | Implements simulate for 14-synthetic-control. |
| 2,235 | `examples/14-synthetic-control/config.toml` | examples | 14-synthetic-control | P2 | Scientific Examples | Configures or declares config for 14-synthetic-control. |
| 2,236 | `examples/14-synthetic-control/dataset-card.md` | examples | 14-synthetic-control | P2 | Scientific Examples | Documents dataset card for 14-synthetic-control. |
| 2,237 | `examples/14-synthetic-control/expected/world.json` | examples | 14-synthetic-control | P2 | Scientific Examples | Configures or declares world for 14-synthetic-control. |
| 2,238 | `examples/14-synthetic-control/expected/metrics.json` | examples | 14-synthetic-control | P2 | Scientific Examples | Configures or declares metrics for 14-synthetic-control. |
| 2,239 | `examples/14-synthetic-control/test_example.py` | examples | 14-synthetic-control | P2 | Scientific Examples | Verifies test example behavior for 14-synthetic-control. |
| 2,240 | `examples/15-user-constraints/README.md` | examples | 15-user-constraints | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 15-user-constraints. |
| 2,241 | `examples/15-user-constraints/generate.py` | examples | 15-user-constraints | P2 | Scientific Examples | Implements generate for 15-user-constraints. |
| 2,242 | `examples/15-user-constraints/discover.py` | examples | 15-user-constraints | P2 | Scientific Examples | Implements discover for 15-user-constraints. |
| 2,243 | `examples/15-user-constraints/simulate.py` | examples | 15-user-constraints | P2 | Scientific Examples | Implements simulate for 15-user-constraints. |
| 2,244 | `examples/15-user-constraints/config.toml` | examples | 15-user-constraints | P2 | Scientific Examples | Configures or declares config for 15-user-constraints. |
| 2,245 | `examples/15-user-constraints/dataset-card.md` | examples | 15-user-constraints | P2 | Scientific Examples | Documents dataset card for 15-user-constraints. |
| 2,246 | `examples/15-user-constraints/expected/world.json` | examples | 15-user-constraints | P2 | Scientific Examples | Configures or declares world for 15-user-constraints. |
| 2,247 | `examples/15-user-constraints/expected/metrics.json` | examples | 15-user-constraints | P2 | Scientific Examples | Configures or declares metrics for 15-user-constraints. |
| 2,248 | `examples/15-user-constraints/test_example.py` | examples | 15-user-constraints | P2 | Scientific Examples | Verifies test example behavior for 15-user-constraints. |
| 2,249 | `examples/16-custom-operator/README.md` | examples | 16-custom-operator | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 16-custom-operator. |
| 2,250 | `examples/16-custom-operator/generate.py` | examples | 16-custom-operator | P2 | Scientific Examples | Implements generate for 16-custom-operator. |
| 2,251 | `examples/16-custom-operator/discover.py` | examples | 16-custom-operator | P2 | Scientific Examples | Implements discover for 16-custom-operator. |
| 2,252 | `examples/16-custom-operator/simulate.py` | examples | 16-custom-operator | P2 | Scientific Examples | Implements simulate for 16-custom-operator. |
| 2,253 | `examples/16-custom-operator/config.toml` | examples | 16-custom-operator | P2 | Scientific Examples | Configures or declares config for 16-custom-operator. |
| 2,254 | `examples/16-custom-operator/dataset-card.md` | examples | 16-custom-operator | P2 | Scientific Examples | Documents dataset card for 16-custom-operator. |
| 2,255 | `examples/16-custom-operator/expected/world.json` | examples | 16-custom-operator | P2 | Scientific Examples | Configures or declares world for 16-custom-operator. |
| 2,256 | `examples/16-custom-operator/expected/metrics.json` | examples | 16-custom-operator | P2 | Scientific Examples | Configures or declares metrics for 16-custom-operator. |
| 2,257 | `examples/16-custom-operator/test_example.py` | examples | 16-custom-operator | P2 | Scientific Examples | Verifies test example behavior for 16-custom-operator. |
| 2,258 | `examples/17-custom-stage/README.md` | examples | 17-custom-stage | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 17-custom-stage. |
| 2,259 | `examples/17-custom-stage/generate.py` | examples | 17-custom-stage | P2 | Scientific Examples | Implements generate for 17-custom-stage. |
| 2,260 | `examples/17-custom-stage/discover.py` | examples | 17-custom-stage | P2 | Scientific Examples | Implements discover for 17-custom-stage. |
| 2,261 | `examples/17-custom-stage/simulate.py` | examples | 17-custom-stage | P2 | Scientific Examples | Implements simulate for 17-custom-stage. |
| 2,262 | `examples/17-custom-stage/config.toml` | examples | 17-custom-stage | P2 | Scientific Examples | Configures or declares config for 17-custom-stage. |
| 2,263 | `examples/17-custom-stage/dataset-card.md` | examples | 17-custom-stage | P2 | Scientific Examples | Documents dataset card for 17-custom-stage. |
| 2,264 | `examples/17-custom-stage/expected/world.json` | examples | 17-custom-stage | P2 | Scientific Examples | Configures or declares world for 17-custom-stage. |
| 2,265 | `examples/17-custom-stage/expected/metrics.json` | examples | 17-custom-stage | P2 | Scientific Examples | Configures or declares metrics for 17-custom-stage. |
| 2,266 | `examples/17-custom-stage/test_example.py` | examples | 17-custom-stage | P2 | Scientific Examples | Verifies test example behavior for 17-custom-stage. |
| 2,267 | `examples/18-bundle-interchange/README.md` | examples | 18-bundle-interchange | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 18-bundle-interchange. |
| 2,268 | `examples/18-bundle-interchange/generate.py` | examples | 18-bundle-interchange | P2 | Scientific Examples | Implements generate for 18-bundle-interchange. |
| 2,269 | `examples/18-bundle-interchange/discover.py` | examples | 18-bundle-interchange | P2 | Scientific Examples | Implements discover for 18-bundle-interchange. |
| 2,270 | `examples/18-bundle-interchange/simulate.py` | examples | 18-bundle-interchange | P2 | Scientific Examples | Implements simulate for 18-bundle-interchange. |
| 2,271 | `examples/18-bundle-interchange/config.toml` | examples | 18-bundle-interchange | P2 | Scientific Examples | Configures or declares config for 18-bundle-interchange. |
| 2,272 | `examples/18-bundle-interchange/dataset-card.md` | examples | 18-bundle-interchange | P2 | Scientific Examples | Documents dataset card for 18-bundle-interchange. |
| 2,273 | `examples/18-bundle-interchange/expected/world.json` | examples | 18-bundle-interchange | P2 | Scientific Examples | Configures or declares world for 18-bundle-interchange. |
| 2,274 | `examples/18-bundle-interchange/expected/metrics.json` | examples | 18-bundle-interchange | P2 | Scientific Examples | Configures or declares metrics for 18-bundle-interchange. |
| 2,275 | `examples/18-bundle-interchange/test_example.py` | examples | 18-bundle-interchange | P2 | Scientific Examples | Verifies test example behavior for 18-bundle-interchange. |
| 2,276 | `examples/19-server-api/README.md` | examples | 19-server-api | P2 | Scientific Examples | Documents the purpose, boundaries, and usage of 19-server-api. |
| 2,277 | `examples/19-server-api/generate.py` | examples | 19-server-api | P2 | Scientific Examples | Implements generate for 19-server-api. |
| 2,278 | `examples/19-server-api/discover.py` | examples | 19-server-api | P2 | Scientific Examples | Implements discover for 19-server-api. |
| 2,279 | `examples/19-server-api/simulate.py` | examples | 19-server-api | P2 | Scientific Examples | Implements simulate for 19-server-api. |
| 2,280 | `examples/19-server-api/config.toml` | examples | 19-server-api | P2 | Scientific Examples | Configures or declares config for 19-server-api. |
| 2,281 | `examples/19-server-api/dataset-card.md` | examples | 19-server-api | P2 | Scientific Examples | Documents dataset card for 19-server-api. |
| 2,282 | `examples/19-server-api/expected/world.json` | examples | 19-server-api | P2 | Scientific Examples | Configures or declares world for 19-server-api. |
| 2,283 | `examples/19-server-api/expected/metrics.json` | examples | 19-server-api | P2 | Scientific Examples | Configures or declares metrics for 19-server-api. |
| 2,284 | `examples/19-server-api/test_example.py` | examples | 19-server-api | P2 | Scientific Examples | Verifies test example behavior for 19-server-api. |
| 2,285 | `benchmarks/equation/algebraic-clean/README.md` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of equation/algebraic-clean. |
| 2,286 | `benchmarks/equation/algebraic-clean/benchmark.toml` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Configures or declares benchmark for equation/algebraic-clean. |
| 2,287 | `benchmarks/equation/algebraic-clean/generate.py` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Implements generate for equation/algebraic-clean. |
| 2,288 | `benchmarks/equation/algebraic-clean/run.py` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Implements run for equation/algebraic-clean. |
| 2,289 | `benchmarks/equation/algebraic-clean/score.py` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Implements score for equation/algebraic-clean. |
| 2,290 | `benchmarks/equation/algebraic-clean/baseline.json` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Configures or declares baseline for equation/algebraic-clean. |
| 2,291 | `benchmarks/equation/algebraic-clean/expected.json` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Provides deterministic expected fixture data for equation/algebraic-clean. |
| 2,292 | `benchmarks/equation/algebraic-clean/report.md` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Documents report for equation/algebraic-clean. |
| 2,293 | `benchmarks/equation/algebraic-clean/test_benchmark.py` | benchmarks | equation/algebraic-clean | P2 | Research Benchmarks | Verifies test benchmark behavior for equation/algebraic-clean. |
| 2,294 | `benchmarks/equation/algebraic-noisy/README.md` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of equation/algebraic-noisy. |
| 2,295 | `benchmarks/equation/algebraic-noisy/benchmark.toml` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Configures or declares benchmark for equation/algebraic-noisy. |
| 2,296 | `benchmarks/equation/algebraic-noisy/generate.py` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Implements generate for equation/algebraic-noisy. |
| 2,297 | `benchmarks/equation/algebraic-noisy/run.py` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Implements run for equation/algebraic-noisy. |
| 2,298 | `benchmarks/equation/algebraic-noisy/score.py` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Implements score for equation/algebraic-noisy. |
| 2,299 | `benchmarks/equation/algebraic-noisy/baseline.json` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Configures or declares baseline for equation/algebraic-noisy. |
| 2,300 | `benchmarks/equation/algebraic-noisy/expected.json` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Provides deterministic expected fixture data for equation/algebraic-noisy. |
| 2,301 | `benchmarks/equation/algebraic-noisy/report.md` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Documents report for equation/algebraic-noisy. |
| 2,302 | `benchmarks/equation/algebraic-noisy/test_benchmark.py` | benchmarks | equation/algebraic-noisy | P2 | Research Benchmarks | Verifies test benchmark behavior for equation/algebraic-noisy. |
| 2,303 | `benchmarks/equation/rational/README.md` | benchmarks | equation/rational | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of equation/rational. |
| 2,304 | `benchmarks/equation/rational/benchmark.toml` | benchmarks | equation/rational | P2 | Research Benchmarks | Configures or declares benchmark for equation/rational. |
| 2,305 | `benchmarks/equation/rational/generate.py` | benchmarks | equation/rational | P2 | Research Benchmarks | Implements generate for equation/rational. |
| 2,306 | `benchmarks/equation/rational/run.py` | benchmarks | equation/rational | P2 | Research Benchmarks | Implements run for equation/rational. |
| 2,307 | `benchmarks/equation/rational/score.py` | benchmarks | equation/rational | P2 | Research Benchmarks | Implements score for equation/rational. |
| 2,308 | `benchmarks/equation/rational/baseline.json` | benchmarks | equation/rational | P2 | Research Benchmarks | Configures or declares baseline for equation/rational. |
| 2,309 | `benchmarks/equation/rational/expected.json` | benchmarks | equation/rational | P2 | Research Benchmarks | Provides deterministic expected fixture data for equation/rational. |
| 2,310 | `benchmarks/equation/rational/report.md` | benchmarks | equation/rational | P2 | Research Benchmarks | Documents report for equation/rational. |
| 2,311 | `benchmarks/equation/rational/test_benchmark.py` | benchmarks | equation/rational | P2 | Research Benchmarks | Verifies test benchmark behavior for equation/rational. |
| 2,312 | `benchmarks/equation/transcendental/README.md` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of equation/transcendental. |
| 2,313 | `benchmarks/equation/transcendental/benchmark.toml` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Configures or declares benchmark for equation/transcendental. |
| 2,314 | `benchmarks/equation/transcendental/generate.py` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Implements generate for equation/transcendental. |
| 2,315 | `benchmarks/equation/transcendental/run.py` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Implements run for equation/transcendental. |
| 2,316 | `benchmarks/equation/transcendental/score.py` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Implements score for equation/transcendental. |
| 2,317 | `benchmarks/equation/transcendental/baseline.json` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Configures or declares baseline for equation/transcendental. |
| 2,318 | `benchmarks/equation/transcendental/expected.json` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Provides deterministic expected fixture data for equation/transcendental. |
| 2,319 | `benchmarks/equation/transcendental/report.md` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Documents report for equation/transcendental. |
| 2,320 | `benchmarks/equation/transcendental/test_benchmark.py` | benchmarks | equation/transcendental | P2 | Research Benchmarks | Verifies test benchmark behavior for equation/transcendental. |
| 2,321 | `benchmarks/equation/dimensional/README.md` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of equation/dimensional. |
| 2,322 | `benchmarks/equation/dimensional/benchmark.toml` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Configures or declares benchmark for equation/dimensional. |
| 2,323 | `benchmarks/equation/dimensional/generate.py` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Implements generate for equation/dimensional. |
| 2,324 | `benchmarks/equation/dimensional/run.py` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Implements run for equation/dimensional. |
| 2,325 | `benchmarks/equation/dimensional/score.py` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Implements score for equation/dimensional. |
| 2,326 | `benchmarks/equation/dimensional/baseline.json` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Configures or declares baseline for equation/dimensional. |
| 2,327 | `benchmarks/equation/dimensional/expected.json` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Provides deterministic expected fixture data for equation/dimensional. |
| 2,328 | `benchmarks/equation/dimensional/report.md` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Documents report for equation/dimensional. |
| 2,329 | `benchmarks/equation/dimensional/test_benchmark.py` | benchmarks | equation/dimensional | P2 | Research Benchmarks | Verifies test benchmark behavior for equation/dimensional. |
| 2,330 | `benchmarks/dynamics/ode-small/README.md` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of dynamics/ode-small. |
| 2,331 | `benchmarks/dynamics/ode-small/benchmark.toml` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Configures or declares benchmark for dynamics/ode-small. |
| 2,332 | `benchmarks/dynamics/ode-small/generate.py` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Implements generate for dynamics/ode-small. |
| 2,333 | `benchmarks/dynamics/ode-small/run.py` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Implements run for dynamics/ode-small. |
| 2,334 | `benchmarks/dynamics/ode-small/score.py` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Implements score for dynamics/ode-small. |
| 2,335 | `benchmarks/dynamics/ode-small/baseline.json` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Configures or declares baseline for dynamics/ode-small. |
| 2,336 | `benchmarks/dynamics/ode-small/expected.json` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Provides deterministic expected fixture data for dynamics/ode-small. |
| 2,337 | `benchmarks/dynamics/ode-small/report.md` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Documents report for dynamics/ode-small. |
| 2,338 | `benchmarks/dynamics/ode-small/test_benchmark.py` | benchmarks | dynamics/ode-small | P2 | Research Benchmarks | Verifies test benchmark behavior for dynamics/ode-small. |
| 2,339 | `benchmarks/dynamics/ode-chaotic/README.md` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of dynamics/ode-chaotic. |
| 2,340 | `benchmarks/dynamics/ode-chaotic/benchmark.toml` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Configures or declares benchmark for dynamics/ode-chaotic. |
| 2,341 | `benchmarks/dynamics/ode-chaotic/generate.py` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Implements generate for dynamics/ode-chaotic. |
| 2,342 | `benchmarks/dynamics/ode-chaotic/run.py` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Implements run for dynamics/ode-chaotic. |
| 2,343 | `benchmarks/dynamics/ode-chaotic/score.py` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Implements score for dynamics/ode-chaotic. |
| 2,344 | `benchmarks/dynamics/ode-chaotic/baseline.json` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Configures or declares baseline for dynamics/ode-chaotic. |
| 2,345 | `benchmarks/dynamics/ode-chaotic/expected.json` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Provides deterministic expected fixture data for dynamics/ode-chaotic. |
| 2,346 | `benchmarks/dynamics/ode-chaotic/report.md` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Documents report for dynamics/ode-chaotic. |
| 2,347 | `benchmarks/dynamics/ode-chaotic/test_benchmark.py` | benchmarks | dynamics/ode-chaotic | P2 | Research Benchmarks | Verifies test benchmark behavior for dynamics/ode-chaotic. |
| 2,348 | `benchmarks/dynamics/discrete/README.md` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of dynamics/discrete. |
| 2,349 | `benchmarks/dynamics/discrete/benchmark.toml` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Configures or declares benchmark for dynamics/discrete. |
| 2,350 | `benchmarks/dynamics/discrete/generate.py` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Implements generate for dynamics/discrete. |
| 2,351 | `benchmarks/dynamics/discrete/run.py` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Implements run for dynamics/discrete. |
| 2,352 | `benchmarks/dynamics/discrete/score.py` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Implements score for dynamics/discrete. |
| 2,353 | `benchmarks/dynamics/discrete/baseline.json` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Configures or declares baseline for dynamics/discrete. |
| 2,354 | `benchmarks/dynamics/discrete/expected.json` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Provides deterministic expected fixture data for dynamics/discrete. |
| 2,355 | `benchmarks/dynamics/discrete/report.md` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Documents report for dynamics/discrete. |
| 2,356 | `benchmarks/dynamics/discrete/test_benchmark.py` | benchmarks | dynamics/discrete | P2 | Research Benchmarks | Verifies test benchmark behavior for dynamics/discrete. |
| 2,357 | `benchmarks/dynamics/delay/README.md` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of dynamics/delay. |
| 2,358 | `benchmarks/dynamics/delay/benchmark.toml` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Configures or declares benchmark for dynamics/delay. |
| 2,359 | `benchmarks/dynamics/delay/generate.py` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Implements generate for dynamics/delay. |
| 2,360 | `benchmarks/dynamics/delay/run.py` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Implements run for dynamics/delay. |
| 2,361 | `benchmarks/dynamics/delay/score.py` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Implements score for dynamics/delay. |
| 2,362 | `benchmarks/dynamics/delay/baseline.json` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Configures or declares baseline for dynamics/delay. |
| 2,363 | `benchmarks/dynamics/delay/expected.json` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Provides deterministic expected fixture data for dynamics/delay. |
| 2,364 | `benchmarks/dynamics/delay/report.md` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Documents report for dynamics/delay. |
| 2,365 | `benchmarks/dynamics/delay/test_benchmark.py` | benchmarks | dynamics/delay | P2 | Research Benchmarks | Verifies test benchmark behavior for dynamics/delay. |
| 2,366 | `benchmarks/dynamics/stochastic/README.md` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of dynamics/stochastic. |
| 2,367 | `benchmarks/dynamics/stochastic/benchmark.toml` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Configures or declares benchmark for dynamics/stochastic. |
| 2,368 | `benchmarks/dynamics/stochastic/generate.py` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Implements generate for dynamics/stochastic. |
| 2,369 | `benchmarks/dynamics/stochastic/run.py` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Implements run for dynamics/stochastic. |
| 2,370 | `benchmarks/dynamics/stochastic/score.py` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Implements score for dynamics/stochastic. |
| 2,371 | `benchmarks/dynamics/stochastic/baseline.json` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Configures or declares baseline for dynamics/stochastic. |
| 2,372 | `benchmarks/dynamics/stochastic/expected.json` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Provides deterministic expected fixture data for dynamics/stochastic. |
| 2,373 | `benchmarks/dynamics/stochastic/report.md` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Documents report for dynamics/stochastic. |
| 2,374 | `benchmarks/dynamics/stochastic/test_benchmark.py` | benchmarks | dynamics/stochastic | P2 | Research Benchmarks | Verifies test benchmark behavior for dynamics/stochastic. |
| 2,375 | `benchmarks/dynamics/hybrid/README.md` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of dynamics/hybrid. |
| 2,376 | `benchmarks/dynamics/hybrid/benchmark.toml` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Configures or declares benchmark for dynamics/hybrid. |
| 2,377 | `benchmarks/dynamics/hybrid/generate.py` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Implements generate for dynamics/hybrid. |
| 2,378 | `benchmarks/dynamics/hybrid/run.py` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Implements run for dynamics/hybrid. |
| 2,379 | `benchmarks/dynamics/hybrid/score.py` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Implements score for dynamics/hybrid. |
| 2,380 | `benchmarks/dynamics/hybrid/baseline.json` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Configures or declares baseline for dynamics/hybrid. |
| 2,381 | `benchmarks/dynamics/hybrid/expected.json` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Provides deterministic expected fixture data for dynamics/hybrid. |
| 2,382 | `benchmarks/dynamics/hybrid/report.md` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Documents report for dynamics/hybrid. |
| 2,383 | `benchmarks/dynamics/hybrid/test_benchmark.py` | benchmarks | dynamics/hybrid | P2 | Research Benchmarks | Verifies test benchmark behavior for dynamics/hybrid. |
| 2,384 | `benchmarks/causal/linear/README.md` | benchmarks | causal/linear | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of causal/linear. |
| 2,385 | `benchmarks/causal/linear/benchmark.toml` | benchmarks | causal/linear | P2 | Research Benchmarks | Configures or declares benchmark for causal/linear. |
| 2,386 | `benchmarks/causal/linear/generate.py` | benchmarks | causal/linear | P2 | Research Benchmarks | Implements generate for causal/linear. |
| 2,387 | `benchmarks/causal/linear/run.py` | benchmarks | causal/linear | P2 | Research Benchmarks | Implements run for causal/linear. |
| 2,388 | `benchmarks/causal/linear/score.py` | benchmarks | causal/linear | P2 | Research Benchmarks | Implements score for causal/linear. |
| 2,389 | `benchmarks/causal/linear/baseline.json` | benchmarks | causal/linear | P2 | Research Benchmarks | Configures or declares baseline for causal/linear. |
| 2,390 | `benchmarks/causal/linear/expected.json` | benchmarks | causal/linear | P2 | Research Benchmarks | Provides deterministic expected fixture data for causal/linear. |
| 2,391 | `benchmarks/causal/linear/report.md` | benchmarks | causal/linear | P2 | Research Benchmarks | Documents report for causal/linear. |
| 2,392 | `benchmarks/causal/linear/test_benchmark.py` | benchmarks | causal/linear | P2 | Research Benchmarks | Verifies test benchmark behavior for causal/linear. |
| 2,393 | `benchmarks/causal/nonlinear/README.md` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of causal/nonlinear. |
| 2,394 | `benchmarks/causal/nonlinear/benchmark.toml` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Configures or declares benchmark for causal/nonlinear. |
| 2,395 | `benchmarks/causal/nonlinear/generate.py` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Implements generate for causal/nonlinear. |
| 2,396 | `benchmarks/causal/nonlinear/run.py` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Implements run for causal/nonlinear. |
| 2,397 | `benchmarks/causal/nonlinear/score.py` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Implements score for causal/nonlinear. |
| 2,398 | `benchmarks/causal/nonlinear/baseline.json` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Configures or declares baseline for causal/nonlinear. |
| 2,399 | `benchmarks/causal/nonlinear/expected.json` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Provides deterministic expected fixture data for causal/nonlinear. |
| 2,400 | `benchmarks/causal/nonlinear/report.md` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Documents report for causal/nonlinear. |
| 2,401 | `benchmarks/causal/nonlinear/test_benchmark.py` | benchmarks | causal/nonlinear | P2 | Research Benchmarks | Verifies test benchmark behavior for causal/nonlinear. |
| 2,402 | `benchmarks/causal/lagged/README.md` | benchmarks | causal/lagged | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of causal/lagged. |
| 2,403 | `benchmarks/causal/lagged/benchmark.toml` | benchmarks | causal/lagged | P2 | Research Benchmarks | Configures or declares benchmark for causal/lagged. |
| 2,404 | `benchmarks/causal/lagged/generate.py` | benchmarks | causal/lagged | P2 | Research Benchmarks | Implements generate for causal/lagged. |
| 2,405 | `benchmarks/causal/lagged/run.py` | benchmarks | causal/lagged | P2 | Research Benchmarks | Implements run for causal/lagged. |
| 2,406 | `benchmarks/causal/lagged/score.py` | benchmarks | causal/lagged | P2 | Research Benchmarks | Implements score for causal/lagged. |
| 2,407 | `benchmarks/causal/lagged/baseline.json` | benchmarks | causal/lagged | P2 | Research Benchmarks | Configures or declares baseline for causal/lagged. |
| 2,408 | `benchmarks/causal/lagged/expected.json` | benchmarks | causal/lagged | P2 | Research Benchmarks | Provides deterministic expected fixture data for causal/lagged. |
| 2,409 | `benchmarks/causal/lagged/report.md` | benchmarks | causal/lagged | P2 | Research Benchmarks | Documents report for causal/lagged. |
| 2,410 | `benchmarks/causal/lagged/test_benchmark.py` | benchmarks | causal/lagged | P2 | Research Benchmarks | Verifies test benchmark behavior for causal/lagged. |
| 2,411 | `benchmarks/causal/confounded/README.md` | benchmarks | causal/confounded | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of causal/confounded. |
| 2,412 | `benchmarks/causal/confounded/benchmark.toml` | benchmarks | causal/confounded | P2 | Research Benchmarks | Configures or declares benchmark for causal/confounded. |
| 2,413 | `benchmarks/causal/confounded/generate.py` | benchmarks | causal/confounded | P2 | Research Benchmarks | Implements generate for causal/confounded. |
| 2,414 | `benchmarks/causal/confounded/run.py` | benchmarks | causal/confounded | P2 | Research Benchmarks | Implements run for causal/confounded. |
| 2,415 | `benchmarks/causal/confounded/score.py` | benchmarks | causal/confounded | P2 | Research Benchmarks | Implements score for causal/confounded. |
| 2,416 | `benchmarks/causal/confounded/baseline.json` | benchmarks | causal/confounded | P2 | Research Benchmarks | Configures or declares baseline for causal/confounded. |
| 2,417 | `benchmarks/causal/confounded/expected.json` | benchmarks | causal/confounded | P2 | Research Benchmarks | Provides deterministic expected fixture data for causal/confounded. |
| 2,418 | `benchmarks/causal/confounded/report.md` | benchmarks | causal/confounded | P2 | Research Benchmarks | Documents report for causal/confounded. |
| 2,419 | `benchmarks/causal/confounded/test_benchmark.py` | benchmarks | causal/confounded | P2 | Research Benchmarks | Verifies test benchmark behavior for causal/confounded. |
| 2,420 | `benchmarks/causal/interventional/README.md` | benchmarks | causal/interventional | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of causal/interventional. |
| 2,421 | `benchmarks/causal/interventional/benchmark.toml` | benchmarks | causal/interventional | P2 | Research Benchmarks | Configures or declares benchmark for causal/interventional. |
| 2,422 | `benchmarks/causal/interventional/generate.py` | benchmarks | causal/interventional | P2 | Research Benchmarks | Implements generate for causal/interventional. |
| 2,423 | `benchmarks/causal/interventional/run.py` | benchmarks | causal/interventional | P2 | Research Benchmarks | Implements run for causal/interventional. |
| 2,424 | `benchmarks/causal/interventional/score.py` | benchmarks | causal/interventional | P2 | Research Benchmarks | Implements score for causal/interventional. |
| 2,425 | `benchmarks/causal/interventional/baseline.json` | benchmarks | causal/interventional | P2 | Research Benchmarks | Configures or declares baseline for causal/interventional. |
| 2,426 | `benchmarks/causal/interventional/expected.json` | benchmarks | causal/interventional | P2 | Research Benchmarks | Provides deterministic expected fixture data for causal/interventional. |
| 2,427 | `benchmarks/causal/interventional/report.md` | benchmarks | causal/interventional | P2 | Research Benchmarks | Documents report for causal/interventional. |
| 2,428 | `benchmarks/causal/interventional/test_benchmark.py` | benchmarks | causal/interventional | P2 | Research Benchmarks | Verifies test benchmark behavior for causal/interventional. |
| 2,429 | `benchmarks/regime/change-point/README.md` | benchmarks | regime/change-point | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of regime/change-point. |
| 2,430 | `benchmarks/regime/change-point/benchmark.toml` | benchmarks | regime/change-point | P2 | Research Benchmarks | Configures or declares benchmark for regime/change-point. |
| 2,431 | `benchmarks/regime/change-point/generate.py` | benchmarks | regime/change-point | P2 | Research Benchmarks | Implements generate for regime/change-point. |
| 2,432 | `benchmarks/regime/change-point/run.py` | benchmarks | regime/change-point | P2 | Research Benchmarks | Implements run for regime/change-point. |
| 2,433 | `benchmarks/regime/change-point/score.py` | benchmarks | regime/change-point | P2 | Research Benchmarks | Implements score for regime/change-point. |
| 2,434 | `benchmarks/regime/change-point/baseline.json` | benchmarks | regime/change-point | P2 | Research Benchmarks | Configures or declares baseline for regime/change-point. |
| 2,435 | `benchmarks/regime/change-point/expected.json` | benchmarks | regime/change-point | P2 | Research Benchmarks | Provides deterministic expected fixture data for regime/change-point. |
| 2,436 | `benchmarks/regime/change-point/report.md` | benchmarks | regime/change-point | P2 | Research Benchmarks | Documents report for regime/change-point. |
| 2,437 | `benchmarks/regime/change-point/test_benchmark.py` | benchmarks | regime/change-point | P2 | Research Benchmarks | Verifies test benchmark behavior for regime/change-point. |
| 2,438 | `benchmarks/regime/hmm/README.md` | benchmarks | regime/hmm | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of regime/hmm. |
| 2,439 | `benchmarks/regime/hmm/benchmark.toml` | benchmarks | regime/hmm | P2 | Research Benchmarks | Configures or declares benchmark for regime/hmm. |
| 2,440 | `benchmarks/regime/hmm/generate.py` | benchmarks | regime/hmm | P2 | Research Benchmarks | Implements generate for regime/hmm. |
| 2,441 | `benchmarks/regime/hmm/run.py` | benchmarks | regime/hmm | P2 | Research Benchmarks | Implements run for regime/hmm. |
| 2,442 | `benchmarks/regime/hmm/score.py` | benchmarks | regime/hmm | P2 | Research Benchmarks | Implements score for regime/hmm. |
| 2,443 | `benchmarks/regime/hmm/baseline.json` | benchmarks | regime/hmm | P2 | Research Benchmarks | Configures or declares baseline for regime/hmm. |
| 2,444 | `benchmarks/regime/hmm/expected.json` | benchmarks | regime/hmm | P2 | Research Benchmarks | Provides deterministic expected fixture data for regime/hmm. |
| 2,445 | `benchmarks/regime/hmm/report.md` | benchmarks | regime/hmm | P2 | Research Benchmarks | Documents report for regime/hmm. |
| 2,446 | `benchmarks/regime/hmm/test_benchmark.py` | benchmarks | regime/hmm | P2 | Research Benchmarks | Verifies test benchmark behavior for regime/hmm. |
| 2,447 | `benchmarks/regime/markov-switching/README.md` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of regime/markov-switching. |
| 2,448 | `benchmarks/regime/markov-switching/benchmark.toml` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Configures or declares benchmark for regime/markov-switching. |
| 2,449 | `benchmarks/regime/markov-switching/generate.py` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Implements generate for regime/markov-switching. |
| 2,450 | `benchmarks/regime/markov-switching/run.py` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Implements run for regime/markov-switching. |
| 2,451 | `benchmarks/regime/markov-switching/score.py` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Implements score for regime/markov-switching. |
| 2,452 | `benchmarks/regime/markov-switching/baseline.json` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Configures or declares baseline for regime/markov-switching. |
| 2,453 | `benchmarks/regime/markov-switching/expected.json` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Provides deterministic expected fixture data for regime/markov-switching. |
| 2,454 | `benchmarks/regime/markov-switching/report.md` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Documents report for regime/markov-switching. |
| 2,455 | `benchmarks/regime/markov-switching/test_benchmark.py` | benchmarks | regime/markov-switching | P2 | Research Benchmarks | Verifies test benchmark behavior for regime/markov-switching. |
| 2,456 | `benchmarks/regime/event-driven/README.md` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of regime/event-driven. |
| 2,457 | `benchmarks/regime/event-driven/benchmark.toml` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Configures or declares benchmark for regime/event-driven. |
| 2,458 | `benchmarks/regime/event-driven/generate.py` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Implements generate for regime/event-driven. |
| 2,459 | `benchmarks/regime/event-driven/run.py` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Implements run for regime/event-driven. |
| 2,460 | `benchmarks/regime/event-driven/score.py` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Implements score for regime/event-driven. |
| 2,461 | `benchmarks/regime/event-driven/baseline.json` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Configures or declares baseline for regime/event-driven. |
| 2,462 | `benchmarks/regime/event-driven/expected.json` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Provides deterministic expected fixture data for regime/event-driven. |
| 2,463 | `benchmarks/regime/event-driven/report.md` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Documents report for regime/event-driven. |
| 2,464 | `benchmarks/regime/event-driven/test_benchmark.py` | benchmarks | regime/event-driven | P2 | Research Benchmarks | Verifies test benchmark behavior for regime/event-driven. |
| 2,465 | `benchmarks/uncertainty/parameter-coverage/README.md` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of uncertainty/parameter-coverage. |
| 2,466 | `benchmarks/uncertainty/parameter-coverage/benchmark.toml` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Configures or declares benchmark for uncertainty/parameter-coverage. |
| 2,467 | `benchmarks/uncertainty/parameter-coverage/generate.py` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Implements generate for uncertainty/parameter-coverage. |
| 2,468 | `benchmarks/uncertainty/parameter-coverage/run.py` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Implements run for uncertainty/parameter-coverage. |
| 2,469 | `benchmarks/uncertainty/parameter-coverage/score.py` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Implements score for uncertainty/parameter-coverage. |
| 2,470 | `benchmarks/uncertainty/parameter-coverage/baseline.json` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Configures or declares baseline for uncertainty/parameter-coverage. |
| 2,471 | `benchmarks/uncertainty/parameter-coverage/expected.json` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Provides deterministic expected fixture data for uncertainty/parameter-coverage. |
| 2,472 | `benchmarks/uncertainty/parameter-coverage/report.md` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Documents report for uncertainty/parameter-coverage. |
| 2,473 | `benchmarks/uncertainty/parameter-coverage/test_benchmark.py` | benchmarks | uncertainty/parameter-coverage | P2 | Research Benchmarks | Verifies test benchmark behavior for uncertainty/parameter-coverage. |
| 2,474 | `benchmarks/uncertainty/structural-recovery/README.md` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of uncertainty/structural-recovery. |
| 2,475 | `benchmarks/uncertainty/structural-recovery/benchmark.toml` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Configures or declares benchmark for uncertainty/structural-recovery. |
| 2,476 | `benchmarks/uncertainty/structural-recovery/generate.py` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Implements generate for uncertainty/structural-recovery. |
| 2,477 | `benchmarks/uncertainty/structural-recovery/run.py` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Implements run for uncertainty/structural-recovery. |
| 2,478 | `benchmarks/uncertainty/structural-recovery/score.py` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Implements score for uncertainty/structural-recovery. |
| 2,479 | `benchmarks/uncertainty/structural-recovery/baseline.json` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Configures or declares baseline for uncertainty/structural-recovery. |
| 2,480 | `benchmarks/uncertainty/structural-recovery/expected.json` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Provides deterministic expected fixture data for uncertainty/structural-recovery. |
| 2,481 | `benchmarks/uncertainty/structural-recovery/report.md` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Documents report for uncertainty/structural-recovery. |
| 2,482 | `benchmarks/uncertainty/structural-recovery/test_benchmark.py` | benchmarks | uncertainty/structural-recovery | P2 | Research Benchmarks | Verifies test benchmark behavior for uncertainty/structural-recovery. |
| 2,483 | `benchmarks/uncertainty/trajectory-coverage/README.md` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of uncertainty/trajectory-coverage. |
| 2,484 | `benchmarks/uncertainty/trajectory-coverage/benchmark.toml` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Configures or declares benchmark for uncertainty/trajectory-coverage. |
| 2,485 | `benchmarks/uncertainty/trajectory-coverage/generate.py` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Implements generate for uncertainty/trajectory-coverage. |
| 2,486 | `benchmarks/uncertainty/trajectory-coverage/run.py` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Implements run for uncertainty/trajectory-coverage. |
| 2,487 | `benchmarks/uncertainty/trajectory-coverage/score.py` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Implements score for uncertainty/trajectory-coverage. |
| 2,488 | `benchmarks/uncertainty/trajectory-coverage/baseline.json` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Configures or declares baseline for uncertainty/trajectory-coverage. |
| 2,489 | `benchmarks/uncertainty/trajectory-coverage/expected.json` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Provides deterministic expected fixture data for uncertainty/trajectory-coverage. |
| 2,490 | `benchmarks/uncertainty/trajectory-coverage/report.md` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Documents report for uncertainty/trajectory-coverage. |
| 2,491 | `benchmarks/uncertainty/trajectory-coverage/test_benchmark.py` | benchmarks | uncertainty/trajectory-coverage | P2 | Research Benchmarks | Verifies test benchmark behavior for uncertainty/trajectory-coverage. |
| 2,492 | `benchmarks/performance/expression-eval/README.md` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/expression-eval. |
| 2,493 | `benchmarks/performance/expression-eval/benchmark.toml` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Configures or declares benchmark for performance/expression-eval. |
| 2,494 | `benchmarks/performance/expression-eval/generate.py` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Implements generate for performance/expression-eval. |
| 2,495 | `benchmarks/performance/expression-eval/run.py` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Implements run for performance/expression-eval. |
| 2,496 | `benchmarks/performance/expression-eval/score.py` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Implements score for performance/expression-eval. |
| 2,497 | `benchmarks/performance/expression-eval/baseline.json` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Configures or declares baseline for performance/expression-eval. |
| 2,498 | `benchmarks/performance/expression-eval/expected.json` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/expression-eval. |
| 2,499 | `benchmarks/performance/expression-eval/report.md` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Documents report for performance/expression-eval. |
| 2,500 | `benchmarks/performance/expression-eval/test_benchmark.py` | benchmarks | performance/expression-eval | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/expression-eval. |
| 2,501 | `benchmarks/performance/symbolic-search/README.md` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/symbolic-search. |
| 2,502 | `benchmarks/performance/symbolic-search/benchmark.toml` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Configures or declares benchmark for performance/symbolic-search. |
| 2,503 | `benchmarks/performance/symbolic-search/generate.py` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Implements generate for performance/symbolic-search. |
| 2,504 | `benchmarks/performance/symbolic-search/run.py` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Implements run for performance/symbolic-search. |
| 2,505 | `benchmarks/performance/symbolic-search/score.py` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Implements score for performance/symbolic-search. |
| 2,506 | `benchmarks/performance/symbolic-search/baseline.json` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Configures or declares baseline for performance/symbolic-search. |
| 2,507 | `benchmarks/performance/symbolic-search/expected.json` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/symbolic-search. |
| 2,508 | `benchmarks/performance/symbolic-search/report.md` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Documents report for performance/symbolic-search. |
| 2,509 | `benchmarks/performance/symbolic-search/test_benchmark.py` | benchmarks | performance/symbolic-search | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/symbolic-search. |
| 2,510 | `benchmarks/performance/sparse-discovery/README.md` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/sparse-discovery. |
| 2,511 | `benchmarks/performance/sparse-discovery/benchmark.toml` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Configures or declares benchmark for performance/sparse-discovery. |
| 2,512 | `benchmarks/performance/sparse-discovery/generate.py` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Implements generate for performance/sparse-discovery. |
| 2,513 | `benchmarks/performance/sparse-discovery/run.py` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Implements run for performance/sparse-discovery. |
| 2,514 | `benchmarks/performance/sparse-discovery/score.py` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Implements score for performance/sparse-discovery. |
| 2,515 | `benchmarks/performance/sparse-discovery/baseline.json` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Configures or declares baseline for performance/sparse-discovery. |
| 2,516 | `benchmarks/performance/sparse-discovery/expected.json` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/sparse-discovery. |
| 2,517 | `benchmarks/performance/sparse-discovery/report.md` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Documents report for performance/sparse-discovery. |
| 2,518 | `benchmarks/performance/sparse-discovery/test_benchmark.py` | benchmarks | performance/sparse-discovery | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/sparse-discovery. |
| 2,519 | `benchmarks/performance/simulation/README.md` | benchmarks | performance/simulation | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/simulation. |
| 2,520 | `benchmarks/performance/simulation/benchmark.toml` | benchmarks | performance/simulation | P2 | Research Benchmarks | Configures or declares benchmark for performance/simulation. |
| 2,521 | `benchmarks/performance/simulation/generate.py` | benchmarks | performance/simulation | P2 | Research Benchmarks | Implements generate for performance/simulation. |
| 2,522 | `benchmarks/performance/simulation/run.py` | benchmarks | performance/simulation | P2 | Research Benchmarks | Implements run for performance/simulation. |
| 2,523 | `benchmarks/performance/simulation/score.py` | benchmarks | performance/simulation | P2 | Research Benchmarks | Implements score for performance/simulation. |
| 2,524 | `benchmarks/performance/simulation/baseline.json` | benchmarks | performance/simulation | P2 | Research Benchmarks | Configures or declares baseline for performance/simulation. |
| 2,525 | `benchmarks/performance/simulation/expected.json` | benchmarks | performance/simulation | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/simulation. |
| 2,526 | `benchmarks/performance/simulation/report.md` | benchmarks | performance/simulation | P2 | Research Benchmarks | Documents report for performance/simulation. |
| 2,527 | `benchmarks/performance/simulation/test_benchmark.py` | benchmarks | performance/simulation | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/simulation. |
| 2,528 | `benchmarks/performance/bundle-io/README.md` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/bundle-io. |
| 2,529 | `benchmarks/performance/bundle-io/benchmark.toml` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Configures or declares benchmark for performance/bundle-io. |
| 2,530 | `benchmarks/performance/bundle-io/generate.py` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Implements generate for performance/bundle-io. |
| 2,531 | `benchmarks/performance/bundle-io/run.py` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Implements run for performance/bundle-io. |
| 2,532 | `benchmarks/performance/bundle-io/score.py` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Implements score for performance/bundle-io. |
| 2,533 | `benchmarks/performance/bundle-io/baseline.json` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Configures or declares baseline for performance/bundle-io. |
| 2,534 | `benchmarks/performance/bundle-io/expected.json` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/bundle-io. |
| 2,535 | `benchmarks/performance/bundle-io/report.md` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Documents report for performance/bundle-io. |
| 2,536 | `benchmarks/performance/bundle-io/test_benchmark.py` | benchmarks | performance/bundle-io | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/bundle-io. |
| 2,537 | `benchmarks/performance/python-boundary/README.md` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/python-boundary. |
| 2,538 | `benchmarks/performance/python-boundary/benchmark.toml` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Configures or declares benchmark for performance/python-boundary. |
| 2,539 | `benchmarks/performance/python-boundary/generate.py` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Implements generate for performance/python-boundary. |
| 2,540 | `benchmarks/performance/python-boundary/run.py` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Implements run for performance/python-boundary. |
| 2,541 | `benchmarks/performance/python-boundary/score.py` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Implements score for performance/python-boundary. |
| 2,542 | `benchmarks/performance/python-boundary/baseline.json` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Configures or declares baseline for performance/python-boundary. |
| 2,543 | `benchmarks/performance/python-boundary/expected.json` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/python-boundary. |
| 2,544 | `benchmarks/performance/python-boundary/report.md` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Documents report for performance/python-boundary. |
| 2,545 | `benchmarks/performance/python-boundary/test_benchmark.py` | benchmarks | performance/python-boundary | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/python-boundary. |
| 2,546 | `benchmarks/performance/end-to-end/README.md` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Documents the purpose, boundaries, and usage of performance/end-to-end. |
| 2,547 | `benchmarks/performance/end-to-end/benchmark.toml` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Configures or declares benchmark for performance/end-to-end. |
| 2,548 | `benchmarks/performance/end-to-end/generate.py` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Implements generate for performance/end-to-end. |
| 2,549 | `benchmarks/performance/end-to-end/run.py` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Implements run for performance/end-to-end. |
| 2,550 | `benchmarks/performance/end-to-end/score.py` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Implements score for performance/end-to-end. |
| 2,551 | `benchmarks/performance/end-to-end/baseline.json` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Configures or declares baseline for performance/end-to-end. |
| 2,552 | `benchmarks/performance/end-to-end/expected.json` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Provides deterministic expected fixture data for performance/end-to-end. |
| 2,553 | `benchmarks/performance/end-to-end/report.md` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Documents report for performance/end-to-end. |
| 2,554 | `benchmarks/performance/end-to-end/test_benchmark.py` | benchmarks | performance/end-to-end | P2 | Research Benchmarks | Verifies test benchmark behavior for performance/end-to-end. |
| 2,555 | `tests/conformance/minimal-world/README.md` | tests | conformance/minimal-world | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/minimal-world. |
| 2,556 | `tests/conformance/minimal-world/case.toml` | tests | conformance/minimal-world | P1 | Quality Engineering | Configures or declares case for conformance/minimal-world. |
| 2,557 | `tests/conformance/minimal-world/input.json` | tests | conformance/minimal-world | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/minimal-world. |
| 2,558 | `tests/conformance/minimal-world/expected.json` | tests | conformance/minimal-world | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/minimal-world. |
| 2,559 | `tests/conformance/minimal-world/run.py` | tests | conformance/minimal-world | P1 | Quality Engineering | Implements run for conformance/minimal-world. |
| 2,560 | `tests/conformance/continuous-world/README.md` | tests | conformance/continuous-world | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/continuous-world. |
| 2,561 | `tests/conformance/continuous-world/case.toml` | tests | conformance/continuous-world | P1 | Quality Engineering | Configures or declares case for conformance/continuous-world. |
| 2,562 | `tests/conformance/continuous-world/input.json` | tests | conformance/continuous-world | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/continuous-world. |
| 2,563 | `tests/conformance/continuous-world/expected.json` | tests | conformance/continuous-world | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/continuous-world. |
| 2,564 | `tests/conformance/continuous-world/run.py` | tests | conformance/continuous-world | P1 | Quality Engineering | Implements run for conformance/continuous-world. |
| 2,565 | `tests/conformance/discrete-world/README.md` | tests | conformance/discrete-world | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/discrete-world. |
| 2,566 | `tests/conformance/discrete-world/case.toml` | tests | conformance/discrete-world | P1 | Quality Engineering | Configures or declares case for conformance/discrete-world. |
| 2,567 | `tests/conformance/discrete-world/input.json` | tests | conformance/discrete-world | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/discrete-world. |
| 2,568 | `tests/conformance/discrete-world/expected.json` | tests | conformance/discrete-world | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/discrete-world. |
| 2,569 | `tests/conformance/discrete-world/run.py` | tests | conformance/discrete-world | P1 | Quality Engineering | Implements run for conformance/discrete-world. |
| 2,570 | `tests/conformance/stochastic-world/README.md` | tests | conformance/stochastic-world | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/stochastic-world. |
| 2,571 | `tests/conformance/stochastic-world/case.toml` | tests | conformance/stochastic-world | P1 | Quality Engineering | Configures or declares case for conformance/stochastic-world. |
| 2,572 | `tests/conformance/stochastic-world/input.json` | tests | conformance/stochastic-world | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/stochastic-world. |
| 2,573 | `tests/conformance/stochastic-world/expected.json` | tests | conformance/stochastic-world | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/stochastic-world. |
| 2,574 | `tests/conformance/stochastic-world/run.py` | tests | conformance/stochastic-world | P1 | Quality Engineering | Implements run for conformance/stochastic-world. |
| 2,575 | `tests/conformance/regime-world/README.md` | tests | conformance/regime-world | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/regime-world. |
| 2,576 | `tests/conformance/regime-world/case.toml` | tests | conformance/regime-world | P1 | Quality Engineering | Configures or declares case for conformance/regime-world. |
| 2,577 | `tests/conformance/regime-world/input.json` | tests | conformance/regime-world | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/regime-world. |
| 2,578 | `tests/conformance/regime-world/expected.json` | tests | conformance/regime-world | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/regime-world. |
| 2,579 | `tests/conformance/regime-world/run.py` | tests | conformance/regime-world | P1 | Quality Engineering | Implements run for conformance/regime-world. |
| 2,580 | `tests/conformance/hybrid-world/README.md` | tests | conformance/hybrid-world | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/hybrid-world. |
| 2,581 | `tests/conformance/hybrid-world/case.toml` | tests | conformance/hybrid-world | P1 | Quality Engineering | Configures or declares case for conformance/hybrid-world. |
| 2,582 | `tests/conformance/hybrid-world/input.json` | tests | conformance/hybrid-world | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/hybrid-world. |
| 2,583 | `tests/conformance/hybrid-world/expected.json` | tests | conformance/hybrid-world | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/hybrid-world. |
| 2,584 | `tests/conformance/hybrid-world/run.py` | tests | conformance/hybrid-world | P1 | Quality Engineering | Implements run for conformance/hybrid-world. |
| 2,585 | `tests/conformance/signed-bundle/README.md` | tests | conformance/signed-bundle | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/signed-bundle. |
| 2,586 | `tests/conformance/signed-bundle/case.toml` | tests | conformance/signed-bundle | P1 | Quality Engineering | Configures or declares case for conformance/signed-bundle. |
| 2,587 | `tests/conformance/signed-bundle/input.json` | tests | conformance/signed-bundle | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/signed-bundle. |
| 2,588 | `tests/conformance/signed-bundle/expected.json` | tests | conformance/signed-bundle | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/signed-bundle. |
| 2,589 | `tests/conformance/signed-bundle/run.py` | tests | conformance/signed-bundle | P1 | Quality Engineering | Implements run for conformance/signed-bundle. |
| 2,590 | `tests/conformance/bad-schema/README.md` | tests | conformance/bad-schema | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/bad-schema. |
| 2,591 | `tests/conformance/bad-schema/case.toml` | tests | conformance/bad-schema | P1 | Quality Engineering | Configures or declares case for conformance/bad-schema. |
| 2,592 | `tests/conformance/bad-schema/input.json` | tests | conformance/bad-schema | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/bad-schema. |
| 2,593 | `tests/conformance/bad-schema/expected.json` | tests | conformance/bad-schema | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/bad-schema. |
| 2,594 | `tests/conformance/bad-schema/run.py` | tests | conformance/bad-schema | P1 | Quality Engineering | Implements run for conformance/bad-schema. |
| 2,595 | `tests/conformance/bad-expression/README.md` | tests | conformance/bad-expression | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/bad-expression. |
| 2,596 | `tests/conformance/bad-expression/case.toml` | tests | conformance/bad-expression | P1 | Quality Engineering | Configures or declares case for conformance/bad-expression. |
| 2,597 | `tests/conformance/bad-expression/input.json` | tests | conformance/bad-expression | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/bad-expression. |
| 2,598 | `tests/conformance/bad-expression/expected.json` | tests | conformance/bad-expression | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/bad-expression. |
| 2,599 | `tests/conformance/bad-expression/run.py` | tests | conformance/bad-expression | P1 | Quality Engineering | Implements run for conformance/bad-expression. |
| 2,600 | `tests/conformance/bad-units/README.md` | tests | conformance/bad-units | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/bad-units. |
| 2,601 | `tests/conformance/bad-units/case.toml` | tests | conformance/bad-units | P1 | Quality Engineering | Configures or declares case for conformance/bad-units. |
| 2,602 | `tests/conformance/bad-units/input.json` | tests | conformance/bad-units | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/bad-units. |
| 2,603 | `tests/conformance/bad-units/expected.json` | tests | conformance/bad-units | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/bad-units. |
| 2,604 | `tests/conformance/bad-units/run.py` | tests | conformance/bad-units | P1 | Quality Engineering | Implements run for conformance/bad-units. |
| 2,605 | `tests/conformance/bad-hash/README.md` | tests | conformance/bad-hash | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/bad-hash. |
| 2,606 | `tests/conformance/bad-hash/case.toml` | tests | conformance/bad-hash | P1 | Quality Engineering | Configures or declares case for conformance/bad-hash. |
| 2,607 | `tests/conformance/bad-hash/input.json` | tests | conformance/bad-hash | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/bad-hash. |
| 2,608 | `tests/conformance/bad-hash/expected.json` | tests | conformance/bad-hash | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/bad-hash. |
| 2,609 | `tests/conformance/bad-hash/run.py` | tests | conformance/bad-hash | P1 | Quality Engineering | Implements run for conformance/bad-hash. |
| 2,610 | `tests/conformance/unsafe-archive/README.md` | tests | conformance/unsafe-archive | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of conformance/unsafe-archive. |
| 2,611 | `tests/conformance/unsafe-archive/case.toml` | tests | conformance/unsafe-archive | P1 | Quality Engineering | Configures or declares case for conformance/unsafe-archive. |
| 2,612 | `tests/conformance/unsafe-archive/input.json` | tests | conformance/unsafe-archive | P1 | Quality Engineering | Provides deterministic input fixture data for conformance/unsafe-archive. |
| 2,613 | `tests/conformance/unsafe-archive/expected.json` | tests | conformance/unsafe-archive | P1 | Quality Engineering | Provides deterministic expected fixture data for conformance/unsafe-archive. |
| 2,614 | `tests/conformance/unsafe-archive/run.py` | tests | conformance/unsafe-archive | P1 | Quality Engineering | Implements run for conformance/unsafe-archive. |
| 2,615 | `tests/cross-language/python-rust/README.md` | tests | cross-language/python-rust | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of cross-language/python-rust. |
| 2,616 | `tests/cross-language/python-rust/case.toml` | tests | cross-language/python-rust | P1 | Quality Engineering | Configures or declares case for cross-language/python-rust. |
| 2,617 | `tests/cross-language/python-rust/input.json` | tests | cross-language/python-rust | P1 | Quality Engineering | Provides deterministic input fixture data for cross-language/python-rust. |
| 2,618 | `tests/cross-language/python-rust/expected.json` | tests | cross-language/python-rust | P1 | Quality Engineering | Provides deterministic expected fixture data for cross-language/python-rust. |
| 2,619 | `tests/cross-language/python-rust/run.py` | tests | cross-language/python-rust | P1 | Quality Engineering | Implements run for cross-language/python-rust. |
| 2,620 | `tests/cross-language/rust-python/README.md` | tests | cross-language/rust-python | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of cross-language/rust-python. |
| 2,621 | `tests/cross-language/rust-python/case.toml` | tests | cross-language/rust-python | P1 | Quality Engineering | Configures or declares case for cross-language/rust-python. |
| 2,622 | `tests/cross-language/rust-python/input.json` | tests | cross-language/rust-python | P1 | Quality Engineering | Provides deterministic input fixture data for cross-language/rust-python. |
| 2,623 | `tests/cross-language/rust-python/expected.json` | tests | cross-language/rust-python | P1 | Quality Engineering | Provides deterministic expected fixture data for cross-language/rust-python. |
| 2,624 | `tests/cross-language/rust-python/run.py` | tests | cross-language/rust-python | P1 | Quality Engineering | Implements run for cross-language/rust-python. |
| 2,625 | `tests/cross-language/typescript-rust/README.md` | tests | cross-language/typescript-rust | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of cross-language/typescript-rust. |
| 2,626 | `tests/cross-language/typescript-rust/case.toml` | tests | cross-language/typescript-rust | P1 | Quality Engineering | Configures or declares case for cross-language/typescript-rust. |
| 2,627 | `tests/cross-language/typescript-rust/input.json` | tests | cross-language/typescript-rust | P1 | Quality Engineering | Provides deterministic input fixture data for cross-language/typescript-rust. |
| 2,628 | `tests/cross-language/typescript-rust/expected.json` | tests | cross-language/typescript-rust | P1 | Quality Engineering | Provides deterministic expected fixture data for cross-language/typescript-rust. |
| 2,629 | `tests/cross-language/typescript-rust/run.py` | tests | cross-language/typescript-rust | P1 | Quality Engineering | Implements run for cross-language/typescript-rust. |
| 2,630 | `tests/cross-language/bundle-roundtrip/README.md` | tests | cross-language/bundle-roundtrip | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of cross-language/bundle-roundtrip. |
| 2,631 | `tests/cross-language/bundle-roundtrip/case.toml` | tests | cross-language/bundle-roundtrip | P1 | Quality Engineering | Configures or declares case for cross-language/bundle-roundtrip. |
| 2,632 | `tests/cross-language/bundle-roundtrip/input.json` | tests | cross-language/bundle-roundtrip | P1 | Quality Engineering | Provides deterministic input fixture data for cross-language/bundle-roundtrip. |
| 2,633 | `tests/cross-language/bundle-roundtrip/expected.json` | tests | cross-language/bundle-roundtrip | P1 | Quality Engineering | Provides deterministic expected fixture data for cross-language/bundle-roundtrip. |
| 2,634 | `tests/cross-language/bundle-roundtrip/run.py` | tests | cross-language/bundle-roundtrip | P1 | Quality Engineering | Implements run for cross-language/bundle-roundtrip. |
| 2,635 | `tests/cross-language/schema-roundtrip/README.md` | tests | cross-language/schema-roundtrip | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of cross-language/schema-roundtrip. |
| 2,636 | `tests/cross-language/schema-roundtrip/case.toml` | tests | cross-language/schema-roundtrip | P1 | Quality Engineering | Configures or declares case for cross-language/schema-roundtrip. |
| 2,637 | `tests/cross-language/schema-roundtrip/input.json` | tests | cross-language/schema-roundtrip | P1 | Quality Engineering | Provides deterministic input fixture data for cross-language/schema-roundtrip. |
| 2,638 | `tests/cross-language/schema-roundtrip/expected.json` | tests | cross-language/schema-roundtrip | P1 | Quality Engineering | Provides deterministic expected fixture data for cross-language/schema-roundtrip. |
| 2,639 | `tests/cross-language/schema-roundtrip/run.py` | tests | cross-language/schema-roundtrip | P1 | Quality Engineering | Implements run for cross-language/schema-roundtrip. |
| 2,640 | `tests/scientific/lorenz-recovery/README.md` | tests | scientific/lorenz-recovery | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/lorenz-recovery. |
| 2,641 | `tests/scientific/lorenz-recovery/case.toml` | tests | scientific/lorenz-recovery | P1 | Quality Engineering | Configures or declares case for scientific/lorenz-recovery. |
| 2,642 | `tests/scientific/lorenz-recovery/input.json` | tests | scientific/lorenz-recovery | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/lorenz-recovery. |
| 2,643 | `tests/scientific/lorenz-recovery/expected.json` | tests | scientific/lorenz-recovery | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/lorenz-recovery. |
| 2,644 | `tests/scientific/lorenz-recovery/run.py` | tests | scientific/lorenz-recovery | P1 | Quality Engineering | Implements run for scientific/lorenz-recovery. |
| 2,645 | `tests/scientific/lotka-volterra-recovery/README.md` | tests | scientific/lotka-volterra-recovery | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/lotka-volterra-recovery. |
| 2,646 | `tests/scientific/lotka-volterra-recovery/case.toml` | tests | scientific/lotka-volterra-recovery | P1 | Quality Engineering | Configures or declares case for scientific/lotka-volterra-recovery. |
| 2,647 | `tests/scientific/lotka-volterra-recovery/input.json` | tests | scientific/lotka-volterra-recovery | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/lotka-volterra-recovery. |
| 2,648 | `tests/scientific/lotka-volterra-recovery/expected.json` | tests | scientific/lotka-volterra-recovery | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/lotka-volterra-recovery. |
| 2,649 | `tests/scientific/lotka-volterra-recovery/run.py` | tests | scientific/lotka-volterra-recovery | P1 | Quality Engineering | Implements run for scientific/lotka-volterra-recovery. |
| 2,650 | `tests/scientific/pendulum-recovery/README.md` | tests | scientific/pendulum-recovery | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/pendulum-recovery. |
| 2,651 | `tests/scientific/pendulum-recovery/case.toml` | tests | scientific/pendulum-recovery | P1 | Quality Engineering | Configures or declares case for scientific/pendulum-recovery. |
| 2,652 | `tests/scientific/pendulum-recovery/input.json` | tests | scientific/pendulum-recovery | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/pendulum-recovery. |
| 2,653 | `tests/scientific/pendulum-recovery/expected.json` | tests | scientific/pendulum-recovery | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/pendulum-recovery. |
| 2,654 | `tests/scientific/pendulum-recovery/run.py` | tests | scientific/pendulum-recovery | P1 | Quality Engineering | Implements run for scientific/pendulum-recovery. |
| 2,655 | `tests/scientific/sir-recovery/README.md` | tests | scientific/sir-recovery | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/sir-recovery. |
| 2,656 | `tests/scientific/sir-recovery/case.toml` | tests | scientific/sir-recovery | P1 | Quality Engineering | Configures or declares case for scientific/sir-recovery. |
| 2,657 | `tests/scientific/sir-recovery/input.json` | tests | scientific/sir-recovery | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/sir-recovery. |
| 2,658 | `tests/scientific/sir-recovery/expected.json` | tests | scientific/sir-recovery | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/sir-recovery. |
| 2,659 | `tests/scientific/sir-recovery/run.py` | tests | scientific/sir-recovery | P1 | Quality Engineering | Implements run for scientific/sir-recovery. |
| 2,660 | `tests/scientific/regime-recovery/README.md` | tests | scientific/regime-recovery | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/regime-recovery. |
| 2,661 | `tests/scientific/regime-recovery/case.toml` | tests | scientific/regime-recovery | P1 | Quality Engineering | Configures or declares case for scientific/regime-recovery. |
| 2,662 | `tests/scientific/regime-recovery/input.json` | tests | scientific/regime-recovery | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/regime-recovery. |
| 2,663 | `tests/scientific/regime-recovery/expected.json` | tests | scientific/regime-recovery | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/regime-recovery. |
| 2,664 | `tests/scientific/regime-recovery/run.py` | tests | scientific/regime-recovery | P1 | Quality Engineering | Implements run for scientific/regime-recovery. |
| 2,665 | `tests/scientific/uncertainty-coverage/README.md` | tests | scientific/uncertainty-coverage | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/uncertainty-coverage. |
| 2,666 | `tests/scientific/uncertainty-coverage/case.toml` | tests | scientific/uncertainty-coverage | P1 | Quality Engineering | Configures or declares case for scientific/uncertainty-coverage. |
| 2,667 | `tests/scientific/uncertainty-coverage/input.json` | tests | scientific/uncertainty-coverage | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/uncertainty-coverage. |
| 2,668 | `tests/scientific/uncertainty-coverage/expected.json` | tests | scientific/uncertainty-coverage | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/uncertainty-coverage. |
| 2,669 | `tests/scientific/uncertainty-coverage/run.py` | tests | scientific/uncertainty-coverage | P1 | Quality Engineering | Implements run for scientific/uncertainty-coverage. |
| 2,670 | `tests/scientific/adversarial-noise/README.md` | tests | scientific/adversarial-noise | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/adversarial-noise. |
| 2,671 | `tests/scientific/adversarial-noise/case.toml` | tests | scientific/adversarial-noise | P1 | Quality Engineering | Configures or declares case for scientific/adversarial-noise. |
| 2,672 | `tests/scientific/adversarial-noise/input.json` | tests | scientific/adversarial-noise | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/adversarial-noise. |
| 2,673 | `tests/scientific/adversarial-noise/expected.json` | tests | scientific/adversarial-noise | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/adversarial-noise. |
| 2,674 | `tests/scientific/adversarial-noise/run.py` | tests | scientific/adversarial-noise | P1 | Quality Engineering | Implements run for scientific/adversarial-noise. |
| 2,675 | `tests/scientific/irregular-sampling/README.md` | tests | scientific/irregular-sampling | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/irregular-sampling. |
| 2,676 | `tests/scientific/irregular-sampling/case.toml` | tests | scientific/irregular-sampling | P1 | Quality Engineering | Configures or declares case for scientific/irregular-sampling. |
| 2,677 | `tests/scientific/irregular-sampling/input.json` | tests | scientific/irregular-sampling | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/irregular-sampling. |
| 2,678 | `tests/scientific/irregular-sampling/expected.json` | tests | scientific/irregular-sampling | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/irregular-sampling. |
| 2,679 | `tests/scientific/irregular-sampling/run.py` | tests | scientific/irregular-sampling | P1 | Quality Engineering | Implements run for scientific/irregular-sampling. |
| 2,680 | `tests/scientific/missing-data/README.md` | tests | scientific/missing-data | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/missing-data. |
| 2,681 | `tests/scientific/missing-data/case.toml` | tests | scientific/missing-data | P1 | Quality Engineering | Configures or declares case for scientific/missing-data. |
| 2,682 | `tests/scientific/missing-data/input.json` | tests | scientific/missing-data | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/missing-data. |
| 2,683 | `tests/scientific/missing-data/expected.json` | tests | scientific/missing-data | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/missing-data. |
| 2,684 | `tests/scientific/missing-data/run.py` | tests | scientific/missing-data | P1 | Quality Engineering | Implements run for scientific/missing-data. |
| 2,685 | `tests/scientific/unit-consistency/README.md` | tests | scientific/unit-consistency | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of scientific/unit-consistency. |
| 2,686 | `tests/scientific/unit-consistency/case.toml` | tests | scientific/unit-consistency | P1 | Quality Engineering | Configures or declares case for scientific/unit-consistency. |
| 2,687 | `tests/scientific/unit-consistency/input.json` | tests | scientific/unit-consistency | P1 | Quality Engineering | Provides deterministic input fixture data for scientific/unit-consistency. |
| 2,688 | `tests/scientific/unit-consistency/expected.json` | tests | scientific/unit-consistency | P1 | Quality Engineering | Provides deterministic expected fixture data for scientific/unit-consistency. |
| 2,689 | `tests/scientific/unit-consistency/run.py` | tests | scientific/unit-consistency | P1 | Quality Engineering | Implements run for scientific/unit-consistency. |
| 2,690 | `tests/end-to-end/local-library/README.md` | tests | end-to-end/local-library | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/local-library. |
| 2,691 | `tests/end-to-end/local-library/case.toml` | tests | end-to-end/local-library | P1 | Quality Engineering | Configures or declares case for end-to-end/local-library. |
| 2,692 | `tests/end-to-end/local-library/input.json` | tests | end-to-end/local-library | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/local-library. |
| 2,693 | `tests/end-to-end/local-library/expected.json` | tests | end-to-end/local-library | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/local-library. |
| 2,694 | `tests/end-to-end/local-library/run.py` | tests | end-to-end/local-library | P1 | Quality Engineering | Implements run for end-to-end/local-library. |
| 2,695 | `tests/end-to-end/cli-discover/README.md` | tests | end-to-end/cli-discover | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/cli-discover. |
| 2,696 | `tests/end-to-end/cli-discover/case.toml` | tests | end-to-end/cli-discover | P1 | Quality Engineering | Configures or declares case for end-to-end/cli-discover. |
| 2,697 | `tests/end-to-end/cli-discover/input.json` | tests | end-to-end/cli-discover | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/cli-discover. |
| 2,698 | `tests/end-to-end/cli-discover/expected.json` | tests | end-to-end/cli-discover | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/cli-discover. |
| 2,699 | `tests/end-to-end/cli-discover/run.py` | tests | end-to-end/cli-discover | P1 | Quality Engineering | Implements run for end-to-end/cli-discover. |
| 2,700 | `tests/end-to-end/cli-simulate/README.md` | tests | end-to-end/cli-simulate | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/cli-simulate. |
| 2,701 | `tests/end-to-end/cli-simulate/case.toml` | tests | end-to-end/cli-simulate | P1 | Quality Engineering | Configures or declares case for end-to-end/cli-simulate. |
| 2,702 | `tests/end-to-end/cli-simulate/input.json` | tests | end-to-end/cli-simulate | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/cli-simulate. |
| 2,703 | `tests/end-to-end/cli-simulate/expected.json` | tests | end-to-end/cli-simulate | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/cli-simulate. |
| 2,704 | `tests/end-to-end/cli-simulate/run.py` | tests | end-to-end/cli-simulate | P1 | Quality Engineering | Implements run for end-to-end/cli-simulate. |
| 2,705 | `tests/end-to-end/local-studio/README.md` | tests | end-to-end/local-studio | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/local-studio. |
| 2,706 | `tests/end-to-end/local-studio/case.toml` | tests | end-to-end/local-studio | P1 | Quality Engineering | Configures or declares case for end-to-end/local-studio. |
| 2,707 | `tests/end-to-end/local-studio/input.json` | tests | end-to-end/local-studio | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/local-studio. |
| 2,708 | `tests/end-to-end/local-studio/expected.json` | tests | end-to-end/local-studio | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/local-studio. |
| 2,709 | `tests/end-to-end/local-studio/run.py` | tests | end-to-end/local-studio | P1 | Quality Engineering | Implements run for end-to-end/local-studio. |
| 2,710 | `tests/end-to-end/server-run/README.md` | tests | end-to-end/server-run | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/server-run. |
| 2,711 | `tests/end-to-end/server-run/case.toml` | tests | end-to-end/server-run | P1 | Quality Engineering | Configures or declares case for end-to-end/server-run. |
| 2,712 | `tests/end-to-end/server-run/input.json` | tests | end-to-end/server-run | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/server-run. |
| 2,713 | `tests/end-to-end/server-run/expected.json` | tests | end-to-end/server-run | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/server-run. |
| 2,714 | `tests/end-to-end/server-run/run.py` | tests | end-to-end/server-run | P1 | Quality Engineering | Implements run for end-to-end/server-run. |
| 2,715 | `tests/end-to-end/cancellation/README.md` | tests | end-to-end/cancellation | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/cancellation. |
| 2,716 | `tests/end-to-end/cancellation/case.toml` | tests | end-to-end/cancellation | P1 | Quality Engineering | Configures or declares case for end-to-end/cancellation. |
| 2,717 | `tests/end-to-end/cancellation/input.json` | tests | end-to-end/cancellation | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/cancellation. |
| 2,718 | `tests/end-to-end/cancellation/expected.json` | tests | end-to-end/cancellation | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/cancellation. |
| 2,719 | `tests/end-to-end/cancellation/run.py` | tests | end-to-end/cancellation | P1 | Quality Engineering | Implements run for end-to-end/cancellation. |
| 2,720 | `tests/end-to-end/resume/README.md` | tests | end-to-end/resume | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/resume. |
| 2,721 | `tests/end-to-end/resume/case.toml` | tests | end-to-end/resume | P1 | Quality Engineering | Configures or declares case for end-to-end/resume. |
| 2,722 | `tests/end-to-end/resume/input.json` | tests | end-to-end/resume | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/resume. |
| 2,723 | `tests/end-to-end/resume/expected.json` | tests | end-to-end/resume | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/resume. |
| 2,724 | `tests/end-to-end/resume/run.py` | tests | end-to-end/resume | P1 | Quality Engineering | Implements run for end-to-end/resume. |
| 2,725 | `tests/end-to-end/export/README.md` | tests | end-to-end/export | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/export. |
| 2,726 | `tests/end-to-end/export/case.toml` | tests | end-to-end/export | P1 | Quality Engineering | Configures or declares case for end-to-end/export. |
| 2,727 | `tests/end-to-end/export/input.json` | tests | end-to-end/export | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/export. |
| 2,728 | `tests/end-to-end/export/expected.json` | tests | end-to-end/export | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/export. |
| 2,729 | `tests/end-to-end/export/run.py` | tests | end-to-end/export | P1 | Quality Engineering | Implements run for end-to-end/export. |
| 2,730 | `tests/end-to-end/import/README.md` | tests | end-to-end/import | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of end-to-end/import. |
| 2,731 | `tests/end-to-end/import/case.toml` | tests | end-to-end/import | P1 | Quality Engineering | Configures or declares case for end-to-end/import. |
| 2,732 | `tests/end-to-end/import/input.json` | tests | end-to-end/import | P1 | Quality Engineering | Provides deterministic input fixture data for end-to-end/import. |
| 2,733 | `tests/end-to-end/import/expected.json` | tests | end-to-end/import | P1 | Quality Engineering | Provides deterministic expected fixture data for end-to-end/import. |
| 2,734 | `tests/end-to-end/import/run.py` | tests | end-to-end/import | P1 | Quality Engineering | Implements run for end-to-end/import. |
| 2,735 | `tests/compatibility/v0-bundles/README.md` | tests | compatibility/v0-bundles | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of compatibility/v0-bundles. |
| 2,736 | `tests/compatibility/v0-bundles/case.toml` | tests | compatibility/v0-bundles | P1 | Quality Engineering | Configures or declares case for compatibility/v0-bundles. |
| 2,737 | `tests/compatibility/v0-bundles/input.json` | tests | compatibility/v0-bundles | P1 | Quality Engineering | Provides deterministic input fixture data for compatibility/v0-bundles. |
| 2,738 | `tests/compatibility/v0-bundles/expected.json` | tests | compatibility/v0-bundles | P1 | Quality Engineering | Provides deterministic expected fixture data for compatibility/v0-bundles. |
| 2,739 | `tests/compatibility/v0-bundles/run.py` | tests | compatibility/v0-bundles | P1 | Quality Engineering | Implements run for compatibility/v0-bundles. |
| 2,740 | `tests/compatibility/v1-migrations/README.md` | tests | compatibility/v1-migrations | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of compatibility/v1-migrations. |
| 2,741 | `tests/compatibility/v1-migrations/case.toml` | tests | compatibility/v1-migrations | P1 | Quality Engineering | Configures or declares case for compatibility/v1-migrations. |
| 2,742 | `tests/compatibility/v1-migrations/input.json` | tests | compatibility/v1-migrations | P1 | Quality Engineering | Provides deterministic input fixture data for compatibility/v1-migrations. |
| 2,743 | `tests/compatibility/v1-migrations/expected.json` | tests | compatibility/v1-migrations | P1 | Quality Engineering | Provides deterministic expected fixture data for compatibility/v1-migrations. |
| 2,744 | `tests/compatibility/v1-migrations/run.py` | tests | compatibility/v1-migrations | P1 | Quality Engineering | Implements run for compatibility/v1-migrations. |
| 2,745 | `tests/compatibility/forward-fields/README.md` | tests | compatibility/forward-fields | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of compatibility/forward-fields. |
| 2,746 | `tests/compatibility/forward-fields/case.toml` | tests | compatibility/forward-fields | P1 | Quality Engineering | Configures or declares case for compatibility/forward-fields. |
| 2,747 | `tests/compatibility/forward-fields/input.json` | tests | compatibility/forward-fields | P1 | Quality Engineering | Provides deterministic input fixture data for compatibility/forward-fields. |
| 2,748 | `tests/compatibility/forward-fields/expected.json` | tests | compatibility/forward-fields | P1 | Quality Engineering | Provides deterministic expected fixture data for compatibility/forward-fields. |
| 2,749 | `tests/compatibility/forward-fields/run.py` | tests | compatibility/forward-fields | P1 | Quality Engineering | Implements run for compatibility/forward-fields. |
| 2,750 | `tests/compatibility/plugin-protocol/README.md` | tests | compatibility/plugin-protocol | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of compatibility/plugin-protocol. |
| 2,751 | `tests/compatibility/plugin-protocol/case.toml` | tests | compatibility/plugin-protocol | P1 | Quality Engineering | Configures or declares case for compatibility/plugin-protocol. |
| 2,752 | `tests/compatibility/plugin-protocol/input.json` | tests | compatibility/plugin-protocol | P1 | Quality Engineering | Provides deterministic input fixture data for compatibility/plugin-protocol. |
| 2,753 | `tests/compatibility/plugin-protocol/expected.json` | tests | compatibility/plugin-protocol | P1 | Quality Engineering | Provides deterministic expected fixture data for compatibility/plugin-protocol. |
| 2,754 | `tests/compatibility/plugin-protocol/run.py` | tests | compatibility/plugin-protocol | P1 | Quality Engineering | Implements run for compatibility/plugin-protocol. |
| 2,755 | `tests/chaos/worker-loss/README.md` | tests | chaos/worker-loss | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of chaos/worker-loss. |
| 2,756 | `tests/chaos/worker-loss/case.toml` | tests | chaos/worker-loss | P1 | Quality Engineering | Configures or declares case for chaos/worker-loss. |
| 2,757 | `tests/chaos/worker-loss/input.json` | tests | chaos/worker-loss | P1 | Quality Engineering | Provides deterministic input fixture data for chaos/worker-loss. |
| 2,758 | `tests/chaos/worker-loss/expected.json` | tests | chaos/worker-loss | P1 | Quality Engineering | Provides deterministic expected fixture data for chaos/worker-loss. |
| 2,759 | `tests/chaos/worker-loss/run.py` | tests | chaos/worker-loss | P1 | Quality Engineering | Implements run for chaos/worker-loss. |
| 2,760 | `tests/chaos/storage-timeout/README.md` | tests | chaos/storage-timeout | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of chaos/storage-timeout. |
| 2,761 | `tests/chaos/storage-timeout/case.toml` | tests | chaos/storage-timeout | P1 | Quality Engineering | Configures or declares case for chaos/storage-timeout. |
| 2,762 | `tests/chaos/storage-timeout/input.json` | tests | chaos/storage-timeout | P1 | Quality Engineering | Provides deterministic input fixture data for chaos/storage-timeout. |
| 2,763 | `tests/chaos/storage-timeout/expected.json` | tests | chaos/storage-timeout | P1 | Quality Engineering | Provides deterministic expected fixture data for chaos/storage-timeout. |
| 2,764 | `tests/chaos/storage-timeout/run.py` | tests | chaos/storage-timeout | P1 | Quality Engineering | Implements run for chaos/storage-timeout. |
| 2,765 | `tests/chaos/duplicate-delivery/README.md` | tests | chaos/duplicate-delivery | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of chaos/duplicate-delivery. |
| 2,766 | `tests/chaos/duplicate-delivery/case.toml` | tests | chaos/duplicate-delivery | P1 | Quality Engineering | Configures or declares case for chaos/duplicate-delivery. |
| 2,767 | `tests/chaos/duplicate-delivery/input.json` | tests | chaos/duplicate-delivery | P1 | Quality Engineering | Provides deterministic input fixture data for chaos/duplicate-delivery. |
| 2,768 | `tests/chaos/duplicate-delivery/expected.json` | tests | chaos/duplicate-delivery | P1 | Quality Engineering | Provides deterministic expected fixture data for chaos/duplicate-delivery. |
| 2,769 | `tests/chaos/duplicate-delivery/run.py` | tests | chaos/duplicate-delivery | P1 | Quality Engineering | Implements run for chaos/duplicate-delivery. |
| 2,770 | `tests/chaos/api-restart/README.md` | tests | chaos/api-restart | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of chaos/api-restart. |
| 2,771 | `tests/chaos/api-restart/case.toml` | tests | chaos/api-restart | P1 | Quality Engineering | Configures or declares case for chaos/api-restart. |
| 2,772 | `tests/chaos/api-restart/input.json` | tests | chaos/api-restart | P1 | Quality Engineering | Provides deterministic input fixture data for chaos/api-restart. |
| 2,773 | `tests/chaos/api-restart/expected.json` | tests | chaos/api-restart | P1 | Quality Engineering | Provides deterministic expected fixture data for chaos/api-restart. |
| 2,774 | `tests/chaos/api-restart/run.py` | tests | chaos/api-restart | P1 | Quality Engineering | Implements run for chaos/api-restart. |
| 2,775 | `tests/chaos/scheduler-restart/README.md` | tests | chaos/scheduler-restart | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of chaos/scheduler-restart. |
| 2,776 | `tests/chaos/scheduler-restart/case.toml` | tests | chaos/scheduler-restart | P1 | Quality Engineering | Configures or declares case for chaos/scheduler-restart. |
| 2,777 | `tests/chaos/scheduler-restart/input.json` | tests | chaos/scheduler-restart | P1 | Quality Engineering | Provides deterministic input fixture data for chaos/scheduler-restart. |
| 2,778 | `tests/chaos/scheduler-restart/expected.json` | tests | chaos/scheduler-restart | P1 | Quality Engineering | Provides deterministic expected fixture data for chaos/scheduler-restart. |
| 2,779 | `tests/chaos/scheduler-restart/run.py` | tests | chaos/scheduler-restart | P1 | Quality Engineering | Implements run for chaos/scheduler-restart. |
| 2,780 | `tests/security/archive-traversal/README.md` | tests | security/archive-traversal | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of security/archive-traversal. |
| 2,781 | `tests/security/archive-traversal/case.toml` | tests | security/archive-traversal | P1 | Quality Engineering | Configures or declares case for security/archive-traversal. |
| 2,782 | `tests/security/archive-traversal/input.json` | tests | security/archive-traversal | P1 | Quality Engineering | Provides deterministic input fixture data for security/archive-traversal. |
| 2,783 | `tests/security/archive-traversal/expected.json` | tests | security/archive-traversal | P1 | Quality Engineering | Provides deterministic expected fixture data for security/archive-traversal. |
| 2,784 | `tests/security/archive-traversal/run.py` | tests | security/archive-traversal | P1 | Quality Engineering | Implements run for security/archive-traversal. |
| 2,785 | `tests/security/decompression-limits/README.md` | tests | security/decompression-limits | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of security/decompression-limits. |
| 2,786 | `tests/security/decompression-limits/case.toml` | tests | security/decompression-limits | P1 | Quality Engineering | Configures or declares case for security/decompression-limits. |
| 2,787 | `tests/security/decompression-limits/input.json` | tests | security/decompression-limits | P1 | Quality Engineering | Provides deterministic input fixture data for security/decompression-limits. |
| 2,788 | `tests/security/decompression-limits/expected.json` | tests | security/decompression-limits | P1 | Quality Engineering | Provides deterministic expected fixture data for security/decompression-limits. |
| 2,789 | `tests/security/decompression-limits/run.py` | tests | security/decompression-limits | P1 | Quality Engineering | Implements run for security/decompression-limits. |
| 2,790 | `tests/security/plugin-permissions/README.md` | tests | security/plugin-permissions | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of security/plugin-permissions. |
| 2,791 | `tests/security/plugin-permissions/case.toml` | tests | security/plugin-permissions | P1 | Quality Engineering | Configures or declares case for security/plugin-permissions. |
| 2,792 | `tests/security/plugin-permissions/input.json` | tests | security/plugin-permissions | P1 | Quality Engineering | Provides deterministic input fixture data for security/plugin-permissions. |
| 2,793 | `tests/security/plugin-permissions/expected.json` | tests | security/plugin-permissions | P1 | Quality Engineering | Provides deterministic expected fixture data for security/plugin-permissions. |
| 2,794 | `tests/security/plugin-permissions/run.py` | tests | security/plugin-permissions | P1 | Quality Engineering | Implements run for security/plugin-permissions. |
| 2,795 | `tests/security/expression-limits/README.md` | tests | security/expression-limits | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of security/expression-limits. |
| 2,796 | `tests/security/expression-limits/case.toml` | tests | security/expression-limits | P1 | Quality Engineering | Configures or declares case for security/expression-limits. |
| 2,797 | `tests/security/expression-limits/input.json` | tests | security/expression-limits | P1 | Quality Engineering | Provides deterministic input fixture data for security/expression-limits. |
| 2,798 | `tests/security/expression-limits/expected.json` | tests | security/expression-limits | P1 | Quality Engineering | Provides deterministic expected fixture data for security/expression-limits. |
| 2,799 | `tests/security/expression-limits/run.py` | tests | security/expression-limits | P1 | Quality Engineering | Implements run for security/expression-limits. |
| 2,800 | `tests/security/authorization/README.md` | tests | security/authorization | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of security/authorization. |
| 2,801 | `tests/security/authorization/case.toml` | tests | security/authorization | P1 | Quality Engineering | Configures or declares case for security/authorization. |
| 2,802 | `tests/security/authorization/input.json` | tests | security/authorization | P1 | Quality Engineering | Provides deterministic input fixture data for security/authorization. |
| 2,803 | `tests/security/authorization/expected.json` | tests | security/authorization | P1 | Quality Engineering | Provides deterministic expected fixture data for security/authorization. |
| 2,804 | `tests/security/authorization/run.py` | tests | security/authorization | P1 | Quality Engineering | Implements run for security/authorization. |
| 2,805 | `tests/security/tenant-isolation/README.md` | tests | security/tenant-isolation | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of security/tenant-isolation. |
| 2,806 | `tests/security/tenant-isolation/case.toml` | tests | security/tenant-isolation | P1 | Quality Engineering | Configures or declares case for security/tenant-isolation. |
| 2,807 | `tests/security/tenant-isolation/input.json` | tests | security/tenant-isolation | P1 | Quality Engineering | Provides deterministic input fixture data for security/tenant-isolation. |
| 2,808 | `tests/security/tenant-isolation/expected.json` | tests | security/tenant-isolation | P1 | Quality Engineering | Provides deterministic expected fixture data for security/tenant-isolation. |
| 2,809 | `tests/security/tenant-isolation/run.py` | tests | security/tenant-isolation | P1 | Quality Engineering | Implements run for security/tenant-isolation. |
| 2,810 | `tests/performance/import-time/README.md` | tests | performance/import-time | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/import-time. |
| 2,811 | `tests/performance/import-time/case.toml` | tests | performance/import-time | P1 | Quality Engineering | Configures or declares case for performance/import-time. |
| 2,812 | `tests/performance/import-time/input.json` | tests | performance/import-time | P1 | Quality Engineering | Provides deterministic input fixture data for performance/import-time. |
| 2,813 | `tests/performance/import-time/expected.json` | tests | performance/import-time | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/import-time. |
| 2,814 | `tests/performance/import-time/run.py` | tests | performance/import-time | P1 | Quality Engineering | Implements run for performance/import-time. |
| 2,815 | `tests/performance/parquet-load/README.md` | tests | performance/parquet-load | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/parquet-load. |
| 2,816 | `tests/performance/parquet-load/case.toml` | tests | performance/parquet-load | P1 | Quality Engineering | Configures or declares case for performance/parquet-load. |
| 2,817 | `tests/performance/parquet-load/input.json` | tests | performance/parquet-load | P1 | Quality Engineering | Provides deterministic input fixture data for performance/parquet-load. |
| 2,818 | `tests/performance/parquet-load/expected.json` | tests | performance/parquet-load | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/parquet-load. |
| 2,819 | `tests/performance/parquet-load/run.py` | tests | performance/parquet-load | P1 | Quality Engineering | Implements run for performance/parquet-load. |
| 2,820 | `tests/performance/profile-million/README.md` | tests | performance/profile-million | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/profile-million. |
| 2,821 | `tests/performance/profile-million/case.toml` | tests | performance/profile-million | P1 | Quality Engineering | Configures or declares case for performance/profile-million. |
| 2,822 | `tests/performance/profile-million/input.json` | tests | performance/profile-million | P1 | Quality Engineering | Provides deterministic input fixture data for performance/profile-million. |
| 2,823 | `tests/performance/profile-million/expected.json` | tests | performance/profile-million | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/profile-million. |
| 2,824 | `tests/performance/profile-million/run.py` | tests | performance/profile-million | P1 | Quality Engineering | Implements run for performance/profile-million. |
| 2,825 | `tests/performance/expression-throughput/README.md` | tests | performance/expression-throughput | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/expression-throughput. |
| 2,826 | `tests/performance/expression-throughput/case.toml` | tests | performance/expression-throughput | P1 | Quality Engineering | Configures or declares case for performance/expression-throughput. |
| 2,827 | `tests/performance/expression-throughput/input.json` | tests | performance/expression-throughput | P1 | Quality Engineering | Provides deterministic input fixture data for performance/expression-throughput. |
| 2,828 | `tests/performance/expression-throughput/expected.json` | tests | performance/expression-throughput | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/expression-throughput. |
| 2,829 | `tests/performance/expression-throughput/run.py` | tests | performance/expression-throughput | P1 | Quality Engineering | Implements run for performance/expression-throughput. |
| 2,830 | `tests/performance/ode-simulation/README.md` | tests | performance/ode-simulation | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/ode-simulation. |
| 2,831 | `tests/performance/ode-simulation/case.toml` | tests | performance/ode-simulation | P1 | Quality Engineering | Configures or declares case for performance/ode-simulation. |
| 2,832 | `tests/performance/ode-simulation/input.json` | tests | performance/ode-simulation | P1 | Quality Engineering | Provides deterministic input fixture data for performance/ode-simulation. |
| 2,833 | `tests/performance/ode-simulation/expected.json` | tests | performance/ode-simulation | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/ode-simulation. |
| 2,834 | `tests/performance/ode-simulation/run.py` | tests | performance/ode-simulation | P1 | Quality Engineering | Implements run for performance/ode-simulation. |
| 2,835 | `tests/performance/bundle-open/README.md` | tests | performance/bundle-open | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/bundle-open. |
| 2,836 | `tests/performance/bundle-open/case.toml` | tests | performance/bundle-open | P1 | Quality Engineering | Configures or declares case for performance/bundle-open. |
| 2,837 | `tests/performance/bundle-open/input.json` | tests | performance/bundle-open | P1 | Quality Engineering | Provides deterministic input fixture data for performance/bundle-open. |
| 2,838 | `tests/performance/bundle-open/expected.json` | tests | performance/bundle-open | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/bundle-open. |
| 2,839 | `tests/performance/bundle-open/run.py` | tests | performance/bundle-open | P1 | Quality Engineering | Implements run for performance/bundle-open. |
| 2,840 | `tests/performance/cancellation-latency/README.md` | tests | performance/cancellation-latency | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/cancellation-latency. |
| 2,841 | `tests/performance/cancellation-latency/case.toml` | tests | performance/cancellation-latency | P1 | Quality Engineering | Configures or declares case for performance/cancellation-latency. |
| 2,842 | `tests/performance/cancellation-latency/input.json` | tests | performance/cancellation-latency | P1 | Quality Engineering | Provides deterministic input fixture data for performance/cancellation-latency. |
| 2,843 | `tests/performance/cancellation-latency/expected.json` | tests | performance/cancellation-latency | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/cancellation-latency. |
| 2,844 | `tests/performance/cancellation-latency/run.py` | tests | performance/cancellation-latency | P1 | Quality Engineering | Implements run for performance/cancellation-latency. |
| 2,845 | `tests/performance/studio-paint/README.md` | tests | performance/studio-paint | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/studio-paint. |
| 2,846 | `tests/performance/studio-paint/case.toml` | tests | performance/studio-paint | P1 | Quality Engineering | Configures or declares case for performance/studio-paint. |
| 2,847 | `tests/performance/studio-paint/input.json` | tests | performance/studio-paint | P1 | Quality Engineering | Provides deterministic input fixture data for performance/studio-paint. |
| 2,848 | `tests/performance/studio-paint/expected.json` | tests | performance/studio-paint | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/studio-paint. |
| 2,849 | `tests/performance/studio-paint/run.py` | tests | performance/studio-paint | P1 | Quality Engineering | Implements run for performance/studio-paint. |
| 2,850 | `tests/performance/event-latency/README.md` | tests | performance/event-latency | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/event-latency. |
| 2,851 | `tests/performance/event-latency/case.toml` | tests | performance/event-latency | P1 | Quality Engineering | Configures or declares case for performance/event-latency. |
| 2,852 | `tests/performance/event-latency/input.json` | tests | performance/event-latency | P1 | Quality Engineering | Provides deterministic input fixture data for performance/event-latency. |
| 2,853 | `tests/performance/event-latency/expected.json` | tests | performance/event-latency | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/event-latency. |
| 2,854 | `tests/performance/event-latency/run.py` | tests | performance/event-latency | P1 | Quality Engineering | Implements run for performance/event-latency. |
| 2,855 | `tests/performance/memory-budget/README.md` | tests | performance/memory-budget | P1 | Quality Engineering | Documents the purpose, boundaries, and usage of performance/memory-budget. |
| 2,856 | `tests/performance/memory-budget/case.toml` | tests | performance/memory-budget | P1 | Quality Engineering | Configures or declares case for performance/memory-budget. |
| 2,857 | `tests/performance/memory-budget/input.json` | tests | performance/memory-budget | P1 | Quality Engineering | Provides deterministic input fixture data for performance/memory-budget. |
| 2,858 | `tests/performance/memory-budget/expected.json` | tests | performance/memory-budget | P1 | Quality Engineering | Provides deterministic expected fixture data for performance/memory-budget. |
| 2,859 | `tests/performance/memory-budget/run.py` | tests | performance/memory-budget | P1 | Quality Engineering | Implements run for performance/memory-budget. |
| 2,860 | `plugins/custom-operator-rust/README.md` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of custom-operator-rust. |
| 2,861 | `plugins/custom-operator-rust/LICENSE` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Declares legal terms and notices for custom-operator-rust. |
| 2,862 | `plugins/custom-operator-rust/plugin.toml` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Configures or declares plugin for custom-operator-rust. |
| 2,863 | `plugins/custom-operator-rust/Cargo.toml` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for custom-operator-rust. |
| 2,864 | `plugins/custom-operator-rust/src/lib.rs` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Implements lib for custom-operator-rust. |
| 2,865 | `plugins/custom-operator-rust/tests/plugin_test.rs` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Verifies plugin test behavior for custom-operator-rust. |
| 2,866 | `plugins/custom-operator-rust/examples/basic.rs` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Implements basic for custom-operator-rust. |
| 2,867 | `plugins/custom-operator-rust/docs/usage.md` | plugins | custom-operator-rust | P5 | Extension Ecosystem | Documents usage for custom-operator-rust. |
| 2,868 | `plugins/custom-stage-python/README.md` | plugins | custom-stage-python | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of custom-stage-python. |
| 2,869 | `plugins/custom-stage-python/LICENSE` | plugins | custom-stage-python | P5 | Extension Ecosystem | Declares legal terms and notices for custom-stage-python. |
| 2,870 | `plugins/custom-stage-python/plugin.toml` | plugins | custom-stage-python | P5 | Extension Ecosystem | Configures or declares plugin for custom-stage-python. |
| 2,871 | `plugins/custom-stage-python/pyproject.toml` | plugins | custom-stage-python | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for custom-stage-python. |
| 2,872 | `plugins/custom-stage-python/src/custom_stage_python/plugin.py` | plugins | custom-stage-python | P5 | Extension Ecosystem | Implements plugin for custom-stage-python. |
| 2,873 | `plugins/custom-stage-python/tests/test_plugin.py` | plugins | custom-stage-python | P5 | Extension Ecosystem | Verifies test plugin behavior for custom-stage-python. |
| 2,874 | `plugins/custom-stage-python/examples/basic.py` | plugins | custom-stage-python | P5 | Extension Ecosystem | Implements basic for custom-stage-python. |
| 2,875 | `plugins/custom-stage-python/docs/usage.md` | plugins | custom-stage-python | P5 | Extension Ecosystem | Documents usage for custom-stage-python. |
| 2,876 | `plugins/csv-variant-adapter/README.md` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of csv-variant-adapter. |
| 2,877 | `plugins/csv-variant-adapter/LICENSE` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Declares legal terms and notices for csv-variant-adapter. |
| 2,878 | `plugins/csv-variant-adapter/plugin.toml` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Configures or declares plugin for csv-variant-adapter. |
| 2,879 | `plugins/csv-variant-adapter/pyproject.toml` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for csv-variant-adapter. |
| 2,880 | `plugins/csv-variant-adapter/src/csv_variant_adapter/plugin.py` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Implements plugin for csv-variant-adapter. |
| 2,881 | `plugins/csv-variant-adapter/tests/test_plugin.py` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Verifies test plugin behavior for csv-variant-adapter. |
| 2,882 | `plugins/csv-variant-adapter/examples/basic.py` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Implements basic for csv-variant-adapter. |
| 2,883 | `plugins/csv-variant-adapter/docs/usage.md` | plugins | csv-variant-adapter | P5 | Extension Ecosystem | Documents usage for csv-variant-adapter. |
| 2,884 | `plugins/external-simulator/README.md` | plugins | external-simulator | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of external-simulator. |
| 2,885 | `plugins/external-simulator/LICENSE` | plugins | external-simulator | P5 | Extension Ecosystem | Declares legal terms and notices for external-simulator. |
| 2,886 | `plugins/external-simulator/plugin.toml` | plugins | external-simulator | P5 | Extension Ecosystem | Configures or declares plugin for external-simulator. |
| 2,887 | `plugins/external-simulator/Cargo.toml` | plugins | external-simulator | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for external-simulator. |
| 2,888 | `plugins/external-simulator/src/lib.rs` | plugins | external-simulator | P5 | Extension Ecosystem | Implements lib for external-simulator. |
| 2,889 | `plugins/external-simulator/tests/plugin_test.rs` | plugins | external-simulator | P5 | Extension Ecosystem | Verifies plugin test behavior for external-simulator. |
| 2,890 | `plugins/external-simulator/examples/basic.rs` | plugins | external-simulator | P5 | Extension Ecosystem | Implements basic for external-simulator. |
| 2,891 | `plugins/external-simulator/docs/usage.md` | plugins | external-simulator | P5 | Extension Ecosystem | Documents usage for external-simulator. |
| 2,892 | `plugins/report-exporter/README.md` | plugins | report-exporter | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of report-exporter. |
| 2,893 | `plugins/report-exporter/LICENSE` | plugins | report-exporter | P5 | Extension Ecosystem | Declares legal terms and notices for report-exporter. |
| 2,894 | `plugins/report-exporter/plugin.toml` | plugins | report-exporter | P5 | Extension Ecosystem | Configures or declares plugin for report-exporter. |
| 2,895 | `plugins/report-exporter/pyproject.toml` | plugins | report-exporter | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for report-exporter. |
| 2,896 | `plugins/report-exporter/src/report_exporter/plugin.py` | plugins | report-exporter | P5 | Extension Ecosystem | Implements plugin for report-exporter. |
| 2,897 | `plugins/report-exporter/tests/test_plugin.py` | plugins | report-exporter | P5 | Extension Ecosystem | Verifies test plugin behavior for report-exporter. |
| 2,898 | `plugins/report-exporter/examples/basic.py` | plugins | report-exporter | P5 | Extension Ecosystem | Implements basic for report-exporter. |
| 2,899 | `plugins/report-exporter/docs/usage.md` | plugins | report-exporter | P5 | Extension Ecosystem | Documents usage for report-exporter. |
| 2,900 | `plugins/neural-prior/README.md` | plugins | neural-prior | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of neural-prior. |
| 2,901 | `plugins/neural-prior/LICENSE` | plugins | neural-prior | P5 | Extension Ecosystem | Declares legal terms and notices for neural-prior. |
| 2,902 | `plugins/neural-prior/plugin.toml` | plugins | neural-prior | P5 | Extension Ecosystem | Configures or declares plugin for neural-prior. |
| 2,903 | `plugins/neural-prior/pyproject.toml` | plugins | neural-prior | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for neural-prior. |
| 2,904 | `plugins/neural-prior/src/neural_prior/plugin.py` | plugins | neural-prior | P5 | Extension Ecosystem | Implements plugin for neural-prior. |
| 2,905 | `plugins/neural-prior/tests/test_plugin.py` | plugins | neural-prior | P5 | Extension Ecosystem | Verifies test plugin behavior for neural-prior. |
| 2,906 | `plugins/neural-prior/examples/basic.py` | plugins | neural-prior | P5 | Extension Ecosystem | Implements basic for neural-prior. |
| 2,907 | `plugins/neural-prior/docs/usage.md` | plugins | neural-prior | P5 | Extension Ecosystem | Documents usage for neural-prior. |
| 2,908 | `plugins/finance-data-adapter/README.md` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of finance-data-adapter. |
| 2,909 | `plugins/finance-data-adapter/LICENSE` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Declares legal terms and notices for finance-data-adapter. |
| 2,910 | `plugins/finance-data-adapter/plugin.toml` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Configures or declares plugin for finance-data-adapter. |
| 2,911 | `plugins/finance-data-adapter/pyproject.toml` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for finance-data-adapter. |
| 2,912 | `plugins/finance-data-adapter/src/finance_data_adapter/plugin.py` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Implements plugin for finance-data-adapter. |
| 2,913 | `plugins/finance-data-adapter/tests/test_plugin.py` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Verifies test plugin behavior for finance-data-adapter. |
| 2,914 | `plugins/finance-data-adapter/examples/basic.py` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Implements basic for finance-data-adapter. |
| 2,915 | `plugins/finance-data-adapter/docs/usage.md` | plugins | finance-data-adapter | P5 | Extension Ecosystem | Documents usage for finance-data-adapter. |
| 2,916 | `plugins/world-validator-wasi/README.md` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of world-validator-wasi. |
| 2,917 | `plugins/world-validator-wasi/LICENSE` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Declares legal terms and notices for world-validator-wasi. |
| 2,918 | `plugins/world-validator-wasi/plugin.toml` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Configures or declares plugin for world-validator-wasi. |
| 2,919 | `plugins/world-validator-wasi/Cargo.toml` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for world-validator-wasi. |
| 2,920 | `plugins/world-validator-wasi/src/lib.rs` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Implements lib for world-validator-wasi. |
| 2,921 | `plugins/world-validator-wasi/tests/plugin_test.rs` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Verifies plugin test behavior for world-validator-wasi. |
| 2,922 | `plugins/world-validator-wasi/examples/basic.rs` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Implements basic for world-validator-wasi. |
| 2,923 | `plugins/world-validator-wasi/docs/usage.md` | plugins | world-validator-wasi | P5 | Extension Ecosystem | Documents usage for world-validator-wasi. |
| 2,924 | `plugins/duckdb-source/README.md` | plugins | duckdb-source | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of duckdb-source. |
| 2,925 | `plugins/duckdb-source/LICENSE` | plugins | duckdb-source | P5 | Extension Ecosystem | Declares legal terms and notices for duckdb-source. |
| 2,926 | `plugins/duckdb-source/plugin.toml` | plugins | duckdb-source | P5 | Extension Ecosystem | Configures or declares plugin for duckdb-source. |
| 2,927 | `plugins/duckdb-source/pyproject.toml` | plugins | duckdb-source | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for duckdb-source. |
| 2,928 | `plugins/duckdb-source/src/duckdb_source/plugin.py` | plugins | duckdb-source | P5 | Extension Ecosystem | Implements plugin for duckdb-source. |
| 2,929 | `plugins/duckdb-source/tests/test_plugin.py` | plugins | duckdb-source | P5 | Extension Ecosystem | Verifies test plugin behavior for duckdb-source. |
| 2,930 | `plugins/duckdb-source/examples/basic.py` | plugins | duckdb-source | P5 | Extension Ecosystem | Implements basic for duckdb-source. |
| 2,931 | `plugins/duckdb-source/docs/usage.md` | plugins | duckdb-source | P5 | Extension Ecosystem | Documents usage for duckdb-source. |
| 2,932 | `plugins/scenario-exporter/README.md` | plugins | scenario-exporter | P5 | Extension Ecosystem | Documents the purpose, boundaries, and usage of scenario-exporter. |
| 2,933 | `plugins/scenario-exporter/LICENSE` | plugins | scenario-exporter | P5 | Extension Ecosystem | Declares legal terms and notices for scenario-exporter. |
| 2,934 | `plugins/scenario-exporter/plugin.toml` | plugins | scenario-exporter | P5 | Extension Ecosystem | Configures or declares plugin for scenario-exporter. |
| 2,935 | `plugins/scenario-exporter/Cargo.toml` | plugins | scenario-exporter | P5 | Extension Ecosystem | Declares the build, dependencies, and package metadata for scenario-exporter. |
| 2,936 | `plugins/scenario-exporter/src/lib.rs` | plugins | scenario-exporter | P5 | Extension Ecosystem | Implements lib for scenario-exporter. |
| 2,937 | `plugins/scenario-exporter/tests/plugin_test.rs` | plugins | scenario-exporter | P5 | Extension Ecosystem | Verifies plugin test behavior for scenario-exporter. |
| 2,938 | `plugins/scenario-exporter/examples/basic.rs` | plugins | scenario-exporter | P5 | Extension Ecosystem | Implements basic for scenario-exporter. |
| 2,939 | `plugins/scenario-exporter/docs/usage.md` | plugins | scenario-exporter | P5 | Extension Ecosystem | Documents usage for scenario-exporter. |
| 2,940 | `deploy/compose/local/README.md` | deploy | compose/local | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of compose/local. |
| 2,941 | `deploy/compose/local/.env.example` | deploy | compose/local | P5 | Platform Engineering | Provides .env for compose/local. |
| 2,942 | `deploy/compose/local/compose.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares compose for compose/local. |
| 2,943 | `deploy/compose/local/api.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares api for compose/local. |
| 2,944 | `deploy/compose/local/worker.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares worker for compose/local. |
| 2,945 | `deploy/compose/local/postgres.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares postgres for compose/local. |
| 2,946 | `deploy/compose/local/minio.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares minio for compose/local. |
| 2,947 | `deploy/compose/local/nats.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares nats for compose/local. |
| 2,948 | `deploy/compose/local/volumes.yaml` | deploy | compose/local | P5 | Platform Engineering | Configures or declares volumes for compose/local. |
| 2,949 | `deploy/compose/local/healthcheck.sh` | deploy | compose/local | P5 | Platform Engineering | Automates or operates healthcheck for compose/local. |
| 2,950 | `deploy/compose/production/README.md` | deploy | compose/production | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of compose/production. |
| 2,951 | `deploy/compose/production/.env.example` | deploy | compose/production | P5 | Platform Engineering | Provides .env for compose/production. |
| 2,952 | `deploy/compose/production/compose.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares compose for compose/production. |
| 2,953 | `deploy/compose/production/api.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares api for compose/production. |
| 2,954 | `deploy/compose/production/worker.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares worker for compose/production. |
| 2,955 | `deploy/compose/production/postgres.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares postgres for compose/production. |
| 2,956 | `deploy/compose/production/object-store.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares object store for compose/production. |
| 2,957 | `deploy/compose/production/nats.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares nats for compose/production. |
| 2,958 | `deploy/compose/production/proxy.yaml` | deploy | compose/production | P5 | Platform Engineering | Configures or declares proxy for compose/production. |
| 2,959 | `deploy/compose/production/backup.sh` | deploy | compose/production | P5 | Platform Engineering | Automates or operates backup for compose/production. |
| 2,960 | `deploy/docker/images/README.md` | deploy | docker/images | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of docker/images. |
| 2,961 | `deploy/docker/images/api.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides api for docker/images. |
| 2,962 | `deploy/docker/images/scheduler.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides scheduler for docker/images. |
| 2,963 | `deploy/docker/images/worker.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides worker for docker/images. |
| 2,964 | `deploy/docker/images/artifact.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides artifact for docker/images. |
| 2,965 | `deploy/docker/images/gateway.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides gateway for docker/images. |
| 2,966 | `deploy/docker/images/studio.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides studio for docker/images. |
| 2,967 | `deploy/docker/images/development.Dockerfile` | deploy | docker/images | P5 | Platform Engineering | Provides development for docker/images. |
| 2,968 | `deploy/docker/images/build.hcl` | deploy | docker/images | P5 | Platform Engineering | Provides build for docker/images. |
| 2,969 | `deploy/docker/images/.dockerignore` | deploy | docker/images | P5 | Platform Engineering | Provides  for docker/images. |
| 2,970 | `deploy/helm/lawsynth/README.md` | deploy | helm/lawsynth | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of helm/lawsynth. |
| 2,971 | `deploy/helm/lawsynth/Chart.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares Chart for helm/lawsynth. |
| 2,972 | `deploy/helm/lawsynth/values.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares values for helm/lawsynth. |
| 2,973 | `deploy/helm/lawsynth/values.schema.json` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares values.schema for helm/lawsynth. |
| 2,974 | `deploy/helm/lawsynth/templates-api.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares templates api for helm/lawsynth. |
| 2,975 | `deploy/helm/lawsynth/templates-worker.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares templates worker for helm/lawsynth. |
| 2,976 | `deploy/helm/lawsynth/templates-storage.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares templates storage for helm/lawsynth. |
| 2,977 | `deploy/helm/lawsynth/templates-ingress.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares templates ingress for helm/lawsynth. |
| 2,978 | `deploy/helm/lawsynth/templates-rbac.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares templates rbac for helm/lawsynth. |
| 2,979 | `deploy/helm/lawsynth/templates-migration.yaml` | deploy | helm/lawsynth | P5 | Platform Engineering | Configures or declares templates migration for helm/lawsynth. |
| 2,980 | `deploy/terraform/aws/README.md` | deploy | terraform/aws | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of terraform/aws. |
| 2,981 | `deploy/terraform/aws/main.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides main for terraform/aws. |
| 2,982 | `deploy/terraform/aws/variables.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides variables for terraform/aws. |
| 2,983 | `deploy/terraform/aws/outputs.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides outputs for terraform/aws. |
| 2,984 | `deploy/terraform/aws/versions.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides versions for terraform/aws. |
| 2,985 | `deploy/terraform/aws/network.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides network for terraform/aws. |
| 2,986 | `deploy/terraform/aws/database.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides database for terraform/aws. |
| 2,987 | `deploy/terraform/aws/storage.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides storage for terraform/aws. |
| 2,988 | `deploy/terraform/aws/cluster.tf` | deploy | terraform/aws | P5 | Platform Engineering | Provides cluster for terraform/aws. |
| 2,989 | `deploy/terraform/aws/example.tfvars` | deploy | terraform/aws | P5 | Platform Engineering | Configures or declares example for terraform/aws. |
| 2,990 | `deploy/terraform/gcp/README.md` | deploy | terraform/gcp | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of terraform/gcp. |
| 2,991 | `deploy/terraform/gcp/main.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides main for terraform/gcp. |
| 2,992 | `deploy/terraform/gcp/variables.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides variables for terraform/gcp. |
| 2,993 | `deploy/terraform/gcp/outputs.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides outputs for terraform/gcp. |
| 2,994 | `deploy/terraform/gcp/versions.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides versions for terraform/gcp. |
| 2,995 | `deploy/terraform/gcp/network.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides network for terraform/gcp. |
| 2,996 | `deploy/terraform/gcp/database.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides database for terraform/gcp. |
| 2,997 | `deploy/terraform/gcp/storage.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides storage for terraform/gcp. |
| 2,998 | `deploy/terraform/gcp/cluster.tf` | deploy | terraform/gcp | P5 | Platform Engineering | Provides cluster for terraform/gcp. |
| 2,999 | `deploy/terraform/gcp/example.tfvars` | deploy | terraform/gcp | P5 | Platform Engineering | Configures or declares example for terraform/gcp. |
| 3,000 | `deploy/terraform/azure/README.md` | deploy | terraform/azure | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of terraform/azure. |
| 3,001 | `deploy/terraform/azure/main.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides main for terraform/azure. |
| 3,002 | `deploy/terraform/azure/variables.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides variables for terraform/azure. |
| 3,003 | `deploy/terraform/azure/outputs.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides outputs for terraform/azure. |
| 3,004 | `deploy/terraform/azure/versions.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides versions for terraform/azure. |
| 3,005 | `deploy/terraform/azure/network.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides network for terraform/azure. |
| 3,006 | `deploy/terraform/azure/database.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides database for terraform/azure. |
| 3,007 | `deploy/terraform/azure/storage.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides storage for terraform/azure. |
| 3,008 | `deploy/terraform/azure/cluster.tf` | deploy | terraform/azure | P5 | Platform Engineering | Provides cluster for terraform/azure. |
| 3,009 | `deploy/terraform/azure/example.tfvars` | deploy | terraform/azure | P5 | Platform Engineering | Configures or declares example for terraform/azure. |
| 3,010 | `deploy/kubernetes/base/README.md` | deploy | kubernetes/base | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of kubernetes/base. |
| 3,011 | `deploy/kubernetes/base/namespace.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares namespace for kubernetes/base. |
| 3,012 | `deploy/kubernetes/base/api.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares api for kubernetes/base. |
| 3,013 | `deploy/kubernetes/base/scheduler.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares scheduler for kubernetes/base. |
| 3,014 | `deploy/kubernetes/base/worker.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares worker for kubernetes/base. |
| 3,015 | `deploy/kubernetes/base/artifact.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares artifact for kubernetes/base. |
| 3,016 | `deploy/kubernetes/base/gateway.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares gateway for kubernetes/base. |
| 3,017 | `deploy/kubernetes/base/configmap.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares configmap for kubernetes/base. |
| 3,018 | `deploy/kubernetes/base/rbac.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares rbac for kubernetes/base. |
| 3,019 | `deploy/kubernetes/base/kustomization.yaml` | deploy | kubernetes/base | P5 | Platform Engineering | Configures or declares kustomization for kubernetes/base. |
| 3,020 | `deploy/kubernetes/staging/README.md` | deploy | kubernetes/staging | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of kubernetes/staging. |
| 3,021 | `deploy/kubernetes/staging/kustomization.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares kustomization for kubernetes/staging. |
| 3,022 | `deploy/kubernetes/staging/replicas.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares replicas for kubernetes/staging. |
| 3,023 | `deploy/kubernetes/staging/resources.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares resources for kubernetes/staging. |
| 3,024 | `deploy/kubernetes/staging/ingress.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares ingress for kubernetes/staging. |
| 3,025 | `deploy/kubernetes/staging/config.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares config for kubernetes/staging. |
| 3,026 | `deploy/kubernetes/staging/secrets.example.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares secrets.example for kubernetes/staging. |
| 3,027 | `deploy/kubernetes/staging/network-policy.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares network policy for kubernetes/staging. |
| 3,028 | `deploy/kubernetes/staging/alerts.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares alerts for kubernetes/staging. |
| 3,029 | `deploy/kubernetes/staging/smoke-job.yaml` | deploy | kubernetes/staging | P5 | Platform Engineering | Configures or declares smoke job for kubernetes/staging. |
| 3,030 | `deploy/kubernetes/production/README.md` | deploy | kubernetes/production | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of kubernetes/production. |
| 3,031 | `deploy/kubernetes/production/kustomization.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares kustomization for kubernetes/production. |
| 3,032 | `deploy/kubernetes/production/replicas.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares replicas for kubernetes/production. |
| 3,033 | `deploy/kubernetes/production/resources.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares resources for kubernetes/production. |
| 3,034 | `deploy/kubernetes/production/ingress.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares ingress for kubernetes/production. |
| 3,035 | `deploy/kubernetes/production/config.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares config for kubernetes/production. |
| 3,036 | `deploy/kubernetes/production/secrets.example.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares secrets.example for kubernetes/production. |
| 3,037 | `deploy/kubernetes/production/network-policy.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares network policy for kubernetes/production. |
| 3,038 | `deploy/kubernetes/production/disruption-budget.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares disruption budget for kubernetes/production. |
| 3,039 | `deploy/kubernetes/production/backup-cronjob.yaml` | deploy | kubernetes/production | P5 | Platform Engineering | Configures or declares backup cronjob for kubernetes/production. |
| 3,040 | `deploy/systemd/single-node/README.md` | deploy | systemd/single-node | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of systemd/single-node. |
| 3,041 | `deploy/systemd/single-node/lawsynth-api.service` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates lawsynth api for systemd/single-node. |
| 3,042 | `deploy/systemd/single-node/lawsynth-scheduler.service` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates lawsynth scheduler for systemd/single-node. |
| 3,043 | `deploy/systemd/single-node/lawsynth-worker.service` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates lawsynth worker for systemd/single-node. |
| 3,044 | `deploy/systemd/single-node/lawsynth-artifact.service` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates lawsynth artifact for systemd/single-node. |
| 3,045 | `deploy/systemd/single-node/lawsynth-gateway.service` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates lawsynth gateway for systemd/single-node. |
| 3,046 | `deploy/systemd/single-node/lawsynth.target` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates lawsynth for systemd/single-node. |
| 3,047 | `deploy/systemd/single-node/environment.example` | deploy | systemd/single-node | P5 | Platform Engineering | Provides environment for systemd/single-node. |
| 3,048 | `deploy/systemd/single-node/install.sh` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates install for systemd/single-node. |
| 3,049 | `deploy/systemd/single-node/uninstall.sh` | deploy | systemd/single-node | P5 | Platform Engineering | Automates or operates uninstall for systemd/single-node. |
| 3,050 | `deploy/airgap/bundle/README.md` | deploy | airgap/bundle | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of airgap/bundle. |
| 3,051 | `deploy/airgap/bundle/manifest.yaml` | deploy | airgap/bundle | P5 | Platform Engineering | Configures or declares manifest for airgap/bundle. |
| 3,052 | `deploy/airgap/bundle/checksums.sha256` | deploy | airgap/bundle | P5 | Platform Engineering | Provides checksums for airgap/bundle. |
| 3,053 | `deploy/airgap/bundle/images.txt` | deploy | airgap/bundle | P5 | Platform Engineering | Provides images for airgap/bundle. |
| 3,054 | `deploy/airgap/bundle/packages.txt` | deploy | airgap/bundle | P5 | Platform Engineering | Provides packages for airgap/bundle. |
| 3,055 | `deploy/airgap/bundle/datasets.txt` | deploy | airgap/bundle | P5 | Platform Engineering | Provides datasets for airgap/bundle. |
| 3,056 | `deploy/airgap/bundle/export.sh` | deploy | airgap/bundle | P5 | Platform Engineering | Automates or operates export for airgap/bundle. |
| 3,057 | `deploy/airgap/bundle/import.sh` | deploy | airgap/bundle | P5 | Platform Engineering | Automates or operates import for airgap/bundle. |
| 3,058 | `deploy/airgap/bundle/verify.sh` | deploy | airgap/bundle | P5 | Platform Engineering | Automates or operates verify for airgap/bundle. |
| 3,059 | `deploy/airgap/bundle/install.sh` | deploy | airgap/bundle | P5 | Platform Engineering | Automates or operates install for airgap/bundle. |
| 3,060 | `deploy/observability/reference/README.md` | deploy | observability/reference | P5 | Platform Engineering | Documents the purpose, boundaries, and usage of observability/reference. |
| 3,061 | `deploy/observability/reference/otel-collector.yaml` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares otel collector for observability/reference. |
| 3,062 | `deploy/observability/reference/prometheus.yaml` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares prometheus for observability/reference. |
| 3,063 | `deploy/observability/reference/alerts.yaml` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares alerts for observability/reference. |
| 3,064 | `deploy/observability/reference/grafana-datasources.yaml` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares grafana datasources for observability/reference. |
| 3,065 | `deploy/observability/reference/api-dashboard.json` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares api dashboard for observability/reference. |
| 3,066 | `deploy/observability/reference/worker-dashboard.json` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares worker dashboard for observability/reference. |
| 3,067 | `deploy/observability/reference/science-dashboard.json` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares science dashboard for observability/reference. |
| 3,068 | `deploy/observability/reference/logging.yaml` | deploy | observability/reference | P5 | Platform Engineering | Configures or declares logging for observability/reference. |
| 3,069 | `deploy/observability/reference/runbook.md` | deploy | observability/reference | P5 | Platform Engineering | Documents runbook for observability/reference. |
| 3,070 | `tools/schema-gen/README.md` | tools | schema-gen | P4 | Developer Experience | Documents the purpose, boundaries, and usage of schema-gen. |
| 3,071 | `tools/schema-gen/pyproject.toml` | tools | schema-gen | P4 | Developer Experience | Declares the build, dependencies, and package metadata for schema-gen. |
| 3,072 | `tools/schema-gen/src/main.py` | tools | schema-gen | P4 | Developer Experience | Implements main for schema-gen. |
| 3,073 | `tools/schema-gen/src/schema.py` | tools | schema-gen | P4 | Developer Experience | Implements schema for schema-gen. |
| 3,074 | `tools/schema-gen/src/jsonschema.py` | tools | schema-gen | P4 | Developer Experience | Implements jsonschema for schema-gen. |
| 3,075 | `tools/schema-gen/src/typescript.py` | tools | schema-gen | P4 | Developer Experience | Implements typescript for schema-gen. |
| 3,076 | `tools/schema-gen/src/python.py` | tools | schema-gen | P4 | Developer Experience | Implements python for schema-gen. |
| 3,077 | `tools/schema-gen/tests/test_cli.py` | tools | schema-gen | P4 | Developer Experience | Verifies test cli behavior for schema-gen. |
| 3,078 | `tools/binding-gen/README.md` | tools | binding-gen | P4 | Developer Experience | Documents the purpose, boundaries, and usage of binding-gen. |
| 3,079 | `tools/binding-gen/pyproject.toml` | tools | binding-gen | P4 | Developer Experience | Declares the build, dependencies, and package metadata for binding-gen. |
| 3,080 | `tools/binding-gen/src/main.py` | tools | binding-gen | P4 | Developer Experience | Implements main for binding-gen. |
| 3,081 | `tools/binding-gen/src/protobuf.py` | tools | binding-gen | P4 | Developer Experience | Implements protobuf for binding-gen. |
| 3,082 | `tools/binding-gen/src/python.py` | tools | binding-gen | P4 | Developer Experience | Implements python for binding-gen. |
| 3,083 | `tools/binding-gen/src/rust.py` | tools | binding-gen | P4 | Developer Experience | Implements rust for binding-gen. |
| 3,084 | `tools/binding-gen/src/typescript.py` | tools | binding-gen | P4 | Developer Experience | Implements typescript for binding-gen. |
| 3,085 | `tools/binding-gen/tests/test_cli.py` | tools | binding-gen | P4 | Developer Experience | Verifies test cli behavior for binding-gen. |
| 3,086 | `tools/bundle-inspector/README.md` | tools | bundle-inspector | P4 | Developer Experience | Documents the purpose, boundaries, and usage of bundle-inspector. |
| 3,087 | `tools/bundle-inspector/pyproject.toml` | tools | bundle-inspector | P4 | Developer Experience | Declares the build, dependencies, and package metadata for bundle-inspector. |
| 3,088 | `tools/bundle-inspector/src/main.py` | tools | bundle-inspector | P4 | Developer Experience | Implements main for bundle-inspector. |
| 3,089 | `tools/bundle-inspector/src/manifest.py` | tools | bundle-inspector | P4 | Developer Experience | Implements manifest for bundle-inspector. |
| 3,090 | `tools/bundle-inspector/src/archive.py` | tools | bundle-inspector | P4 | Developer Experience | Implements archive for bundle-inspector. |
| 3,091 | `tools/bundle-inspector/src/checksum.py` | tools | bundle-inspector | P4 | Developer Experience | Implements checksum for bundle-inspector. |
| 3,092 | `tools/bundle-inspector/src/report.py` | tools | bundle-inspector | P4 | Developer Experience | Implements report for bundle-inspector. |
| 3,093 | `tools/bundle-inspector/tests/test_cli.py` | tools | bundle-inspector | P4 | Developer Experience | Verifies test cli behavior for bundle-inspector. |
| 3,094 | `tools/benchmark-site/README.md` | tools | benchmark-site | P4 | Developer Experience | Documents the purpose, boundaries, and usage of benchmark-site. |
| 3,095 | `tools/benchmark-site/pyproject.toml` | tools | benchmark-site | P4 | Developer Experience | Declares the build, dependencies, and package metadata for benchmark-site. |
| 3,096 | `tools/benchmark-site/src/main.py` | tools | benchmark-site | P4 | Developer Experience | Implements main for benchmark-site. |
| 3,097 | `tools/benchmark-site/src/results.py` | tools | benchmark-site | P4 | Developer Experience | Implements results for benchmark-site. |
| 3,098 | `tools/benchmark-site/src/charts.py` | tools | benchmark-site | P4 | Developer Experience | Implements charts for benchmark-site. |
| 3,099 | `tools/benchmark-site/src/compare.py` | tools | benchmark-site | P4 | Developer Experience | Implements compare for benchmark-site. |
| 3,100 | `tools/benchmark-site/src/publish.py` | tools | benchmark-site | P4 | Developer Experience | Implements publish for benchmark-site. |
| 3,101 | `tools/benchmark-site/tests/test_cli.py` | tools | benchmark-site | P4 | Developer Experience | Verifies test cli behavior for benchmark-site. |
| 3,102 | `tools/license-check/README.md` | tools | license-check | P4 | Developer Experience | Documents the purpose, boundaries, and usage of license-check. |
| 3,103 | `tools/license-check/pyproject.toml` | tools | license-check | P4 | Developer Experience | Declares the build, dependencies, and package metadata for license-check. |
| 3,104 | `tools/license-check/src/main.py` | tools | license-check | P4 | Developer Experience | Implements main for license-check. |
| 3,105 | `tools/license-check/src/scan.py` | tools | license-check | P4 | Developer Experience | Implements scan for license-check. |
| 3,106 | `tools/license-check/src/policy.py` | tools | license-check | P4 | Developer Experience | Implements policy for license-check. |
| 3,107 | `tools/license-check/src/notice.py` | tools | license-check | P4 | Developer Experience | Implements notice for license-check. |
| 3,108 | `tools/license-check/src/report.py` | tools | license-check | P4 | Developer Experience | Implements report for license-check. |
| 3,109 | `tools/license-check/tests/test_cli.py` | tools | license-check | P4 | Developer Experience | Verifies test cli behavior for license-check. |
| 3,110 | `tools/release-notes/README.md` | tools | release-notes | P4 | Developer Experience | Documents the purpose, boundaries, and usage of release-notes. |
| 3,111 | `tools/release-notes/pyproject.toml` | tools | release-notes | P4 | Developer Experience | Declares the build, dependencies, and package metadata for release-notes. |
| 3,112 | `tools/release-notes/src/main.py` | tools | release-notes | P4 | Developer Experience | Implements main for release-notes. |
| 3,113 | `tools/release-notes/src/commits.py` | tools | release-notes | P4 | Developer Experience | Implements commits for release-notes. |
| 3,114 | `tools/release-notes/src/changes.py` | tools | release-notes | P4 | Developer Experience | Implements changes for release-notes. |
| 3,115 | `tools/release-notes/src/render.py` | tools | release-notes | P4 | Developer Experience | Implements render for release-notes. |
| 3,116 | `tools/release-notes/src/publish.py` | tools | release-notes | P4 | Developer Experience | Implements publish for release-notes. |
| 3,117 | `tools/release-notes/tests/test_cli.py` | tools | release-notes | P4 | Developer Experience | Verifies test cli behavior for release-notes. |
| 3,118 | `tools/dataset-registry/README.md` | tools | dataset-registry | P4 | Developer Experience | Documents the purpose, boundaries, and usage of dataset-registry. |
| 3,119 | `tools/dataset-registry/pyproject.toml` | tools | dataset-registry | P4 | Developer Experience | Declares the build, dependencies, and package metadata for dataset-registry. |
| 3,120 | `tools/dataset-registry/src/main.py` | tools | dataset-registry | P4 | Developer Experience | Implements main for dataset-registry. |
| 3,121 | `tools/dataset-registry/src/manifest.py` | tools | dataset-registry | P4 | Developer Experience | Implements manifest for dataset-registry. |
| 3,122 | `tools/dataset-registry/src/download.py` | tools | dataset-registry | P4 | Developer Experience | Implements download for dataset-registry. |
| 3,123 | `tools/dataset-registry/src/verify.py` | tools | dataset-registry | P4 | Developer Experience | Implements verify for dataset-registry. |
| 3,124 | `tools/dataset-registry/src/card.py` | tools | dataset-registry | P4 | Developer Experience | Implements card for dataset-registry. |
| 3,125 | `tools/dataset-registry/tests/test_cli.py` | tools | dataset-registry | P4 | Developer Experience | Verifies test cli behavior for dataset-registry. |
| 3,126 | `tools/conformance-runner/README.md` | tools | conformance-runner | P4 | Developer Experience | Documents the purpose, boundaries, and usage of conformance-runner. |
| 3,127 | `tools/conformance-runner/pyproject.toml` | tools | conformance-runner | P4 | Developer Experience | Declares the build, dependencies, and package metadata for conformance-runner. |
| 3,128 | `tools/conformance-runner/src/main.py` | tools | conformance-runner | P4 | Developer Experience | Implements main for conformance-runner. |
| 3,129 | `tools/conformance-runner/src/discover.py` | tools | conformance-runner | P4 | Developer Experience | Implements discover for conformance-runner. |
| 3,130 | `tools/conformance-runner/src/execute.py` | tools | conformance-runner | P4 | Developer Experience | Implements execute for conformance-runner. |
| 3,131 | `tools/conformance-runner/src/compare.py` | tools | conformance-runner | P4 | Developer Experience | Implements compare for conformance-runner. |
| 3,132 | `tools/conformance-runner/src/report.py` | tools | conformance-runner | P4 | Developer Experience | Implements report for conformance-runner. |
| 3,133 | `tools/conformance-runner/tests/test_cli.py` | tools | conformance-runner | P4 | Developer Experience | Verifies test cli behavior for conformance-runner. |
| 3,134 | `tools/fixture-builder/README.md` | tools | fixture-builder | P4 | Developer Experience | Documents the purpose, boundaries, and usage of fixture-builder. |
| 3,135 | `tools/fixture-builder/pyproject.toml` | tools | fixture-builder | P4 | Developer Experience | Declares the build, dependencies, and package metadata for fixture-builder. |
| 3,136 | `tools/fixture-builder/src/main.py` | tools | fixture-builder | P4 | Developer Experience | Implements main for fixture-builder. |
| 3,137 | `tools/fixture-builder/src/generate.py` | tools | fixture-builder | P4 | Developer Experience | Implements generate for fixture-builder. |
| 3,138 | `tools/fixture-builder/src/canonicalize.py` | tools | fixture-builder | P4 | Developer Experience | Implements canonicalize for fixture-builder. |
| 3,139 | `tools/fixture-builder/src/checksum.py` | tools | fixture-builder | P4 | Developer Experience | Implements checksum for fixture-builder. |
| 3,140 | `tools/fixture-builder/src/package.py` | tools | fixture-builder | P4 | Developer Experience | Implements package for fixture-builder. |
| 3,141 | `tools/fixture-builder/tests/test_cli.py` | tools | fixture-builder | P4 | Developer Experience | Verifies test cli behavior for fixture-builder. |
| 3,142 | `tools/api-doc-gen/README.md` | tools | api-doc-gen | P4 | Developer Experience | Documents the purpose, boundaries, and usage of api-doc-gen. |
| 3,143 | `tools/api-doc-gen/pyproject.toml` | tools | api-doc-gen | P4 | Developer Experience | Declares the build, dependencies, and package metadata for api-doc-gen. |
| 3,144 | `tools/api-doc-gen/src/main.py` | tools | api-doc-gen | P4 | Developer Experience | Implements main for api-doc-gen. |
| 3,145 | `tools/api-doc-gen/src/openapi.py` | tools | api-doc-gen | P4 | Developer Experience | Implements openapi for api-doc-gen. |
| 3,146 | `tools/api-doc-gen/src/python.py` | tools | api-doc-gen | P4 | Developer Experience | Implements python for api-doc-gen. |
| 3,147 | `tools/api-doc-gen/src/rust.py` | tools | api-doc-gen | P4 | Developer Experience | Implements rust for api-doc-gen. |
| 3,148 | `tools/api-doc-gen/src/typescript.py` | tools | api-doc-gen | P4 | Developer Experience | Implements typescript for api-doc-gen. |
| 3,149 | `tools/api-doc-gen/tests/test_cli.py` | tools | api-doc-gen | P4 | Developer Experience | Verifies test cli behavior for api-doc-gen. |
| 3,150 | `assets/brand/logo.svg` | tools | brand-assets | P3 | Design | Provides logo for brand assets. |
| 3,151 | `assets/brand/logo-mark.svg` | tools | brand-assets | P3 | Design | Provides logo mark for brand assets. |
| 3,152 | `assets/brand/wordmark.svg` | tools | brand-assets | P3 | Design | Provides wordmark for brand assets. |
| 3,153 | `assets/brand/palette.json` | tools | brand-assets | P3 | Design | Configures or declares palette for brand assets. |
| 3,154 | `assets/brand/typography.md` | tools | brand-assets | P3 | Design | Documents typography for brand assets. |
| 3,155 | `assets/readme/hero.webp` | tools | brand-assets | P3 | Design | Provides hero for brand assets. |
| 3,156 | `assets/readme/lorenz-demo.gif` | tools | brand-assets | P3 | Design | Provides lorenz demo for brand assets. |
| 3,157 | `assets/readme/pipeline.svg` | tools | brand-assets | P3 | Design | Provides pipeline for brand assets. |
| 3,158 | `assets/readme/studio.webp` | tools | brand-assets | P3 | Design | Provides studio for brand assets. |
| 3,159 | `assets/social/github-card.png` | tools | brand-assets | P3 | Design | Provides github card for brand assets. |
| 3,160 | `assets/social/announcement.png` | tools | brand-assets | P3 | Design | Provides announcement for brand assets. |
| 3,161 | `assets/social/demo-thumbnail.png` | tools | brand-assets | P3 | Design | Provides demo thumbnail for brand assets. |

## 6. Implementation policy

1. Do not create all files on day one.
2. Create a file only when its phase is active and the module has a real implementation or contract.
3. A path change after P1 requires an architecture decision record when it crosses a public boundary.
4. Generated bindings and fixtures must be reproducible from checked-in source schemas or generators.
5. No file may exist solely to increase repository size.
6. The exact count is a v1 planning baseline, not a permanent cap.
7. World IR, `.lsworld`, Python API, and plugin protocol changes follow semantic versioning.

## 7. Validation record

- Expected files: **3,161**
- Generated files: **3,161**
- Unique paths: **3,161**
- Derived directories: **682**
- Duplicate paths: **0**
- Unsafe absolute or parent-traversal paths: **0**
- Subsystem totals: **matched**
