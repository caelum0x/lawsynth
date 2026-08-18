# Invariant-detection boundary (v2-A)

This directory specifies **deterministic conserved-quantity (invariant)
detection** — the search for functions that stay constant along a discovered
flow, implemented in `crates/lawsynth-invariants`. It is a **boundary
specification** in the house style: it states what a conforming implementation
MUST do, and — crucially — what a detected invariant is and is not allowed to
claim.

## Motivation

A discovered dynamical law is an autonomous vector field: one right-hand side per
state, `dx_i/dt = f_i(x)`, each `f_i` an expression tree in the `lawsynth-expr`
IR. Many physical systems conserve a quantity along their flow — energy for an
undamped oscillator, the Lotka–Volterra invariant, angular momentum — and
surfacing such a **conserved quantity** `H(x)` is a strong, interpretable readout
of the law's structure: it certifies that trajectories are confined to level sets
of `H`, exposes symmetries, and is a sanity check on the discovery itself.

A function `H` is conserved exactly when it does not change along the flow, i.e.
when its **Lie derivative** (the directional derivative along `f`) vanishes
everywhere:

```text
L_f H = ∇H · f = Σ_i (∂H/∂x_i) · f_i(x) = 0.
```

LawSynth is **deterministic and offline**. It does not train a neural surrogate
or sample stochastically; it differentiates the field **symbolically** and solves
a linear-algebra problem with a fixed algorithm. Identical inputs MUST yield
bit-identical output.

## What an invariant IS

A detected invariant is a **HYPOTHESIS about conservation**, never a proof. The
contract is:

1. **A library-relative discovery.** `H` is parametrized over a finite candidate
   library `{φ_1, …, φ_m}` as `H(x) = Σ_j c_j φ_j(x)`. The method can only find
   invariants **expressible in that library**. It finds a coefficient vector, not
   a closed-form theorem about the system.
2. **A nullspace vector of a sampled operator.** Because `L_f φ_j = ∇φ_j · f` is a
   known function of `x`, conservation is the linear constraint
   `Σ_j c_j (L_f φ_j) = 0`. Sampling it at points `x^(1)…x^(N)` builds a matrix
   `M[k][j] = (L_f φ_j)(x^(k))`; an invariant is a nonzero `c` with `M c ≈ 0` — a
   vector in the numerical nullspace of `M`. Because `M` is sampled on a finite
   grid, a near-null `c` certifies `L_f H ≈ 0` **at the sample points**, not a
   symbolic identity `L_f H ≡ 0`.
3. **Quantified, not asserted.** Every reported invariant MUST carry its singular
   value `σ` (the SVD's evidence of near-nullity) and its residual `‖M c‖` (how
   nearly `L_f H` vanishes over the grid). A small residual makes conservation a
   well-supported hypothesis; it is not promoted to a proof.
4. **No trivial or spurious claims.** The constant function is **excluded from the
   library**, so `H = const` is never reported. On a system with no invariant
   expressible in the library (e.g. a damped oscillator), a conforming
   implementation MUST report **nothing** above tolerance. Silence is a valid and
   required answer.

## Requirements

1. **Library.** The candidate library MUST be built deterministically: every
   monomial of total degree `1..=degree` over the states (constant excluded), in a
   fixed order, optionally followed by `sin(x_i)` and `cos(x_i)` per state. Each
   basis function carries a stable label (`x^2`, `x*y`, `cos(x)`) so a coefficient
   vector is interpretable.
2. **Lie derivative, symbolically.** `L_f φ_j = Σ_i (∂φ_j/∂x_i) · f_i` MUST use the
   **exact symbolic** partial derivatives from `lawsynth-jacobian`, then be
   evaluated numerically at each sample. Partials are a property of the library
   term and the field, never finite-differenced.
3. **Deterministic sampling.** The sample points MUST form a deterministic
   tensor-product grid over an axis-aligned box (`sample_lo`, `sample_hi`,
   `resolution`), enumerated in a fixed order. The grid MUST hold at least as many
   points as the library has functions (otherwise the nullspace is
   underdetermined and the implementation MUST reject the request).
4. **Nullspace via SVD.** The right-singular vectors of `M` with singular values
   `σ_j <= tolerance · σ_max` MUST be reported as invariants, using the
   deterministic one-sided Jacobi SVD in `lawsynth-koopman`. The threshold is
   relative to the largest singular value so it is scale-aware.
5. **Canonical normalization.** Each coefficient vector MUST be normalized to unit
   Euclidean norm with a fixed sign convention — the largest-magnitude entry
   (earliest index on ties) made positive — so the reported invariant is canonical
   and two runs cannot differ only by scale or sign. Reported invariants MUST be
   ordered by ascending singular value with a total order over floats.
6. **Determinism.** Library construction, grid enumeration, matrix assembly, SVD,
   residual aggregation (fixed iteration order), and normalization MUST be
   deterministic. Identical `(fields, states, InvariantConfig)` inputs MUST produce
   a **bit-identical** `InvariantReport`: identical labels and identical `f64` bit
   patterns for every coefficient, residual, and singular value.
7. **Total, typed boundary.** The implementation MUST reject, with distinct typed
   errors: no states, no fields, a repeated state identifier, a state with no
   field, a field that references a symbol outside the declared states, a
   zero degree, a resolution below two, a degenerate or non-finite sample box, a
   non-finite or negative tolerance, and a grid smaller than the library. It MUST
   NOT fabricate, reorder, or drop data to paper over these.

## Public API

```text
detect_invariants(&[(Identifier, Expr)], &[Identifier], &InvariantConfig)
    -> Result<InvariantReport, InvariantError>

build_basis(&[Identifier], &InvariantConfig) -> Result<Vec<BasisFunction>, InvariantError>

InvariantConfig { degree, include_trigonometric, sample_lo, sample_hi, resolution, tolerance }
InvariantReport { basis_labels: Vec<String>, invariants: Vec<Invariant> }
Invariant       { coefficients: Vec<f64>, residual: f64, singular_value: f64 }
Invariant::coefficient(&[String], &str) -> Option<f64>
InvariantReport::to_bits() -> Vec<u64>   // determinism digest
```

`detect_invariants` is the whole boundary; `build_basis` exposes the library
construction for inspection. `InvariantReport::to_bits` is the flat float digest
used by determinism tests. This crate delivers the **detection and honest
reporting library** only — using a conserved quantity to constrain integration,
reduce the model, or classify a regime is downstream and out of scope here.

## Honest scope & limits

- **Library-bounded expressivity.** The method finds only invariants expressible
  as a linear combination of the chosen basis. A **purely polynomial library
  cannot represent a transcendental invariant** — e.g. the pendulum energy
  `½y² + (1 − cos x)` is invisible without `cos` terms, and a conforming
  implementation reports *nothing* rather than a spurious polynomial fit. Enabling
  the trigonometric terms recovers it; no finite library is complete.
- **A hypothesis, not a proof.** A near-null coefficient vector certifies
  `L_f H ≈ 0` at the sample points to within the reported residual. It is not a
  symbolic proof of exact conservation; a genuine invariant and an accidental
  near-coincidence on a too-small grid are distinguished only by the residual and
  by sampling more of the state space.
- **Tolerance trades errors.** The relative singular-value tolerance trades false
  positives against false negatives: too loose invents invariants, too tight
  misses weakly-resolved ones. The tolerance is an explicit config field and MUST
  NOT be silently widened to force a detection.
- **Sampling must cover the space.** The grid must span the region of interest and
  have enough distinct points per axis to resolve the Lie derivatives (at least
  `degree + 1` points per axis for a polynomial library); a coarse or ill-placed
  grid can both miss invariants and admit spurious ones.
- **Degeneracy inflates the nullspace.** A symmetric system can conserve *more*
  than the "obvious" quantities (an isotropic pair of equal-frequency oscillators
  conserves a whole `u(2)` of quadratics). The reported invariants then span that
  larger space; the SVD returns an orthonormal — not physically-aligned — basis of
  it, which is correct but must be read as "a basis of the conserved space", not
  "the" canonical invariants.
- **It analyzes the given field, nothing more.** The report carries no discovery
  confidence and no fit residual against data; it differentiates and samples
  whatever field it is handed. Discovery quality is a separate, upstream concern.

## Non-goals

- No trained/neural probe, no stochastic sampling, no network or platform service.
- No symbolic proof of conservation and no computer-algebra nullspace over exact
  arithmetic; the nullspace is numerical, over sampled points, with a reported
  residual.
- No automatic library selection, dimensional analysis, or symmetry-group
  reduction; the library is the caller's explicit choice.
- No downstream use of the invariant (constrained integration, model reduction,
  regime classification) — those consume the report and live in their own crates.
