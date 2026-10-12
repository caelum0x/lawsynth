# Release manifest — v0.1.0

This document defines the **v0.1.0 cut**: what ships as the product, how a first
user installs it, and the exact, owner-gated path to publishing it. It is the
source of truth for the release surface.

## Flagship & install paths (work today, from source)

The flagship is the `lawsynth` CLI; the Python `lawsynth.Study` API is the
second face. Both install and run offline from a source checkout — **no
crates.io / PyPI access is required to use v0.1**:

```sh
# CLI — the tested install path
cargo install --path crates/lawsynth-cli

# Python SDK — builds the lawsynth._native extension
cd python/lawsynth && maturin develop
# ...or, maturin-free / offline:
python/lawsynth/scripts/build-native.sh
```

Acceptance smoke (run on `examples/00-quickstart`, logistic growth):
`discover` recovers `dpopulation/dt = 0.7999*population - 0.07999*population^2`
(the true r=0.8, K=10 law) at R²=1.0000; `explain`, `forecast`, and `report`
complete; two `discover` runs produce byte-identical `.lsworld` bundles.

## Published crate surface

The published surface is the CLI + Python SDK and their transitive workspace
dependencies. Because the CLI compiles every subcommand into the binary, its
dependency closure includes the advanced dynamics crates as well as the core
pipeline.

**Core pipeline library (20)** — also the Python SDK's full closure:

```
lawsynth-core lawsynth-expr lawsynth-units lawsynth-world lawsynth-bundle
lawsynth-data lawsynth-differentiate lawsynth-preprocess lawsynth-features
lawsynth-sparse lawsynth-score lawsynth-stats lawsynth-opt lawsynth-symbolic
lawsynth-profile lawsynth-regime lawsynth-causal lawsynth-uncertainty
lawsynth-sim lawsynth-discovery
```

**Advanced crates surfaced as CLI subcommands (22)** — in the CLI closure, not
the Python closure:

```
lawsynth-jacobian lawsynth-koopman lawsynth-stability lawsynth-lyapunov
lawsynth-invariants lawsynth-basins lawsynth-bifurcation lawsynth-feedback
lawsynth-mpc lawsynth-estimate lawsynth-modelreduce lawsynth-modelselect
lawsynth-sensitivity lawsynth-control lawsynth-network lawsynth-pde
lawsynth-sde lawsynth-weakform lawsynth-domains lawsynth-egraph
lawsynth-plugin-api lawsynth-plugin-host
```

**Faces (2):** `lawsynth-report`, `lawsynth-python`, plus the `lawsynth-cli`
binary.

## Explicitly internal for v0.1 (not published)

These stay in the workspace with `publish = false` and are **not** part of the
v0.1 release surface:

- Non-surface workspace crates: `lawsynth-api-types`, `lawsynth-runner`,
  `lawsynth-store`, `lawsynth-quant`, `lawsynth-discrete`, `lawsynth-dynamics`,
  `lawsynth-implicit`, `lawsynth-integration`, `lawsynth-propagate`,
  `lawsynth-reduce`, `lawsynth-wasm`, `lawsynth-wasm-bindings`.
- Services (`services/`): `lawsynth-gateway`, `lawsynth-worker`,
  `lawsynth-scheduler`, `lawsynth-artifact-service`.
- Apps (`apps/`): `studio`, `playground`, `docs-site`, `gridsynth`,
  `information-diffusion`.

## Publishing (owner-gated — one command each)

Publishing is disabled in `release-plz.toml` (`publish = false`,
`git_tag_enable = false`, `git_release_enable = false`) until the owner
provisions package ownership and credentials. When un-gated, publish leaf-first
in this topological order (each line is one `cargo publish -p <crate>`):

```
 1. lawsynth-causal        16. lawsynth-score          31. lawsynth-invariants
 2. lawsynth-core          17. lawsynth-sensitivity    32. lawsynth-mpc
 3. lawsynth-data          18. lawsynth-sparse         33. lawsynth-network
 4. lawsynth-differentiate 19. lawsynth-stability      34. lawsynth-pde
 5. lawsynth-expr          20. lawsynth-stats          35. lawsynth-plugin-host
 6. lawsynth-features      21. lawsynth-uncertainty    36. lawsynth-sde
 7. lawsynth-jacobian      22. lawsynth-units          37. lawsynth-sim
 8. lawsynth-koopman       23. lawsynth-weakform       38. lawsynth-symbolic
 9. lawsynth-lyapunov      24. lawsynth-world          39. lawsynth-discovery
10. lawsynth-modelreduce   25. lawsynth-basins         40. lawsynth-domains
11. lawsynth-opt           26. lawsynth-bifurcation    41. lawsynth-estimate
12. lawsynth-plugin-api    27. lawsynth-bundle         42. lawsynth-modelselect
13. lawsynth-preprocess    28. lawsynth-control        43. lawsynth-python
14. lawsynth-profile       29. lawsynth-egraph         44. lawsynth-report
15. lawsynth-regime        30. lawsynth-feedback       45. lawsynth-cli
```

Pre-publish checklist:

- [x] Every crate carries `description`, `version`, `license`, `repository`
      metadata (via `[workspace.package]`).
- [x] Every intra-workspace `lawsynth-*` path dependency in the surface also
      carries `version = "0.1.0"` (required by `cargo publish`).
- [ ] Owner provisions crates.io ownership + `CARGO_REGISTRY_TOKEN`, then flips
      `publish` in `release-plz.toml` and runs the order above.
- [ ] Python: `maturin publish` for `lawsynth` (PyPI, owner-gated).

Note: the `lawsynth` CLI *binary* depends transitively on the 22 advanced
crates (they are compiled-in subcommands). Its crates.io publish therefore
requires the whole closure above; for v0.1 the CLI's primary distribution is the
`cargo install --path` source install, which needs none of it.
