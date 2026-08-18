# Structural reductions boundary (v2-A)

This directory specifies **deterministic symmetry & separability reductions** —
the structural pre-analysis implemented in `crates/lawsynth-reduce`. It is a
**boundary specification** in the house style: it states what a conforming
implementation MUST do, and — crucially — what a detected reduction is and is not
allowed to claim.

## Motivation

AI-Feynman shrinks combinatorial symbolic search by *structurally reducing* a
target `f(x1..xn)` before searching: dimensional analysis, then **symmetry**,
then **separability** (additive `f = g(A) + h(B)` / multiplicative `f = g(A)·h(B)`)
split an `n`-variable problem into smaller independent sub-problems
(divide-and-conquer). Each reduction that holds removes whole regions of the
search space.

AI-Feynman detects these structures by probing a **trained neural network** that
interpolates the data, then querying its gradients. LawSynth is **deterministic
and offline** and ships no trained probe. Instead this crate uses the data's own
**numerical partial derivatives** — the finite-difference / three-point
derivatives already provided by `crates/lawsynth-differentiate` — evaluated on
the reconstructed sample grid. The differentiation is exact-arithmetic
deterministic, so the whole reduction pipeline preserves the reproducibility
contract: identical inputs MUST yield bit-identical output.

## What a reduction IS

A detected reduction is a **HYPOTHESIS about structure**, never a proof. The
contract is:

1. **Detection is a screening test.** Separability is screened by estimating a
   mixed second partial `∂²f/∂x_i∂x_j` numerically across a candidate partition
   and testing whether it is ≈ 0 (additive), or the mixed partial of `log|f|` is
   ≈ 0 (multiplicative). Symmetry is screened by first-derivative invariance
   under a coordinated transform (see below). A screen that passes only makes the
   structure a candidate.
2. **Every detection MUST be verified.** A screened reduction MUST be checked by
   *reconstructing* the data from the reduced form and measuring how well it
   reproduces the observations (a relative RMS residual, i.e. `1 − R²` of the
   reconstruction). A reduction is only reported when both the screen and the
   verification pass their tolerances.
3. **Honest reporting.** Every reported reduction MUST carry its screening
   residual, its reconstruction residual, and a confidence derived from the
   reconstruction (`confidence = 1 − reconstruction_residual`, clamped to
   `[0,1]`). A reduction MUST NOT be asserted as an established fact; it is a
   ranked, quantified hypothesis for a downstream divide-and-conquer stage.
4. **No spurious claims.** On data with no structure (e.g. `f = x·y + sin(x+y)`),
   a conforming implementation MUST report nothing above tolerance. Silence is a
   valid and required answer.

## Requirements

1. **Determinism.** Grid reconstruction (distinct-value detection, cell
   ordering), differentiation, residual aggregation (fixed iteration order), and
   partition enumeration MUST be deterministic. No hashing with a randomized
   seed, no floating-point reduction whose order depends on iteration nondeterminism.
   Identical `(Dataset, ReduceConfig)` inputs MUST produce a bit-identical
   `ReductionReport`.
2. **Numerical derivatives, not a learned probe.** All gradient / mixed-partial
   estimates MUST come from the deterministic derivative estimators in
   `lawsynth-differentiate` applied along reconstructed grid axes. No trained
   surrogate model is used or required.
3. **Grid reconstruction, honestly gated.** Detection requires the input columns
   to form a full Cartesian (tensor) grid so that partials along one variable can
   be estimated with the others held fixed. An implementation MUST attempt to
   reconstruct such a grid from the samples and, if the samples do not form a
   complete grid (missing cells, duplicates, too few distinct values per axis),
   MUST report `GridStatus::NotReconstructed` with a reason and detect nothing —
   never fabricate a partial from scattered data.
4. **Separability.** For a bipartition `(A, B)` of the input variables:
   - **Additive** `f = g(A) + h(B)` is screened by requiring every cross mixed
     partial `∂²f/∂x_i∂x_j` (`i∈A, j∈B`) to be ≈ 0, and verified by the additive
     (two-way main-effects) reconstruction
     `f̂ = mean_B f + mean_A f − mean f`.
   - **Multiplicative** `f = g(A)·h(B)` is screened and verified identically on
     `log|f|`, and is only attempted when `f` is sign-consistent and bounded away
     from zero on the grid (otherwise reported as not applicable, not as absent).
5. **Symmetry.** For a variable pair `(x, y)`, four simple symmetries are screened
   from first partials `f_x, f_y` (all deterministic invariance tests):
   | Symmetry | `f` depends only on | Invariant (≈ 0) |
   |---|---|---|
   | Difference | `x − y` | `f_x + f_y` |
   | Sum | `x + y` | `f_x − f_y` |
   | Product | `x · y` | `x·f_x − y·f_y` |
   | Ratio | `x / y` | `x·f_x + y·f_y` |
   Each residual is normalized by the gradient scale and reported with a
   confidence.
6. **Honest scope & limits.** Detection is inherently tolerance- and
   noise-limited: numerical mixed partials amplify noise, coarse or short axes
   raise the residual floor, and a large tolerance risks false positives while a
   small one risks false negatives. The implementation MUST expose these
   tolerances in `ReduceConfig`, MUST document that reductions are hypotheses,
   and MUST NOT silently widen tolerances to force a detection.

## Public API

```text
detect_reductions(&Dataset, &ReduceConfig) -> Result<ReductionReport, ReduceError>
```

`ReductionReport` lists the detected separabilities (kind, partition, screening
residual, reconstruction residual, confidence) and symmetries (pair, kind,
residual, confidence), plus the grid status. These reductions are intended to
feed a later divide-and-conquer discovery stage; this crate delivers the
**detection library and honest reporting** only — wiring into the discovery
pipeline is a follow-up and is out of scope here.

## Non-goals

- No trained/neural probe, no stochastic sampling, no network or platform service.
- No PDE / spatiotemporal reductions; the boundary is scalar `f(x1..xn)` sampled
  on a Cartesian grid (or a per-state derivative surface presented as such).
- No claim of proof: a reduction is a verifiable, quantified hypothesis.
