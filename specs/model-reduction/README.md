# Model-reduction boundary (v2-A)

This directory specifies **deterministic linear model-order reduction by balanced
truncation** — given a stable linear(ized) system, producing a lower-order model
that preserves the dominant input-output response, implemented in
`crates/lawsynth-modelreduce`. It is a **boundary specification** in the house
style: it states what a conforming implementation MUST do, and — crucially — what
a reduced model is and is not allowed to claim.

## Motivation

A discovered dynamical law linearized at a fixed point, or a Koopman/DMD operator
fit from data, is a linear state-space realization

```
ẋ = A x + B u,   y = C x
```

of some order `n`. Many such realizations carry states that barely participate in
the map from input `u` to output `y`: directions that are weakly reachable from
`B` (nearly *uncontrollable*) or weakly visible in `C` (nearly *unobservable*).
Carrying them costs simulation, analysis, and interpretation effort for almost no
fidelity.

**Balanced truncation** (Moore 1981) removes them in a coordinate-free way. It is
built on two Gramians:

- the **controllability Gramian** `Wc`, the unique solution of the Lyapunov
  equation `A Wc + Wc Aᵀ + B Bᵀ = 0`, whose energy measures how strongly each
  direction is driven by the input;
- the **observability Gramian** `Wo`, solving `Aᵀ Wo + Wo A + Cᵀ C = 0`, whose
  energy measures how strongly each direction shows up at the output.

Both exist and are positive (semi)definite exactly when `A` is **Hurwitz** (every
eigenvalue with `Re < 0`). A **balancing** change of coordinates `T` makes the two
Gramians equal and diagonal, `T⁻¹ Wc T⁻ᵀ = Tᵀ Wo T = Σ = diag(σ₁ ≥ … ≥ σₙ ≥ 0)`.
The `σᵢ` are the **Hankel singular values** — coordinate-independent invariants of
the input-output map. Truncating the states with the smallest `σᵢ` yields a
reduced model `(Aᵣ, Bᵣ, Cᵣ)` with the celebrated a priori error bound

```
‖G − Gᵣ‖∞ ≤ 2 (σ_{k+1} + … + σₙ).
```

LawSynth is **deterministic and offline**. This stage reuses the deterministic
eigensolver of `crates/lawsynth-koopman` (Householder–Hessenberg + Wilkinson
complex QR) to check the Hurwitz precondition and to verify the reduced spectrum;
the only other numerics are local Gaussian elimination, a Kronecker-form Lyapunov
solve, and a cyclic Jacobi symmetric eigensolver. Identical inputs MUST yield
bit-identical output.

## What a reduced model IS

A reduced model is a **property of the supplied linear realization `(A, B, C)`**,
not of any data or of the underlying nonlinear field. The contract is:

1. **A reduction of the linearization, not the nonlinear law.** The reduced model
   approximates the transfer function of the given `(A, B, C)`. On a true
   nonlinear system it is valid only where that linear realization is itself a
   faithful model; the crate makes no global or nonlinear claim.
2. **The Hankel singular values are the honest diagnostic.** They are reported in
   full (all `n` values), so a caller reads exactly how much response energy each
   truncated state carried and where the natural truncation order lies (a large
   gap `σ_k ≫ σ_{k+1}`).
3. **The error is bounded, and the bound is exposed.** A conforming reduced model
   reports the a priori bound `2 · Σ_{i>k} σᵢ`. This is a statement about the
   `H∞` distance between the full and reduced transfer functions, exact in
   infinite-precision arithmetic — not a claim about any particular trajectory.
4. **A property of the realization, not of any fit.** The reduction carries no
   discovery confidence or fit residual. Those are separate, upstream concerns.

## Requirements

1. **Hurwitz precondition, checked not assumed.** An implementation MUST verify
   that every eigenvalue of `A` has strictly negative real part, using the shared
   `lawsynth-koopman` eigensolver, and MUST return a distinct `NotStable` error
   otherwise. The Gramians do not exist for a non-Hurwitz `A`, and no reduction
   MUST be fabricated in that case.

2. **Gramians by exact Lyapunov solve.** `Wc` and `Wo` MUST be obtained by solving
   their continuous Lyapunov equations via vectorization — the Kronecker system
   `((I ⊗ A) + (A ⊗ I)) vec(Wc) = −vec(B Bᵀ)` (and the transposed form for `Wo`) —
   with local Gaussian elimination (largest-magnitude pivot, lowest index on a
   tie), not a second iterative scheme. Each Gramian MUST be symmetrized
   (`(W + Wᵀ)/2`), exact in infinite precision.

3. **Square-root balancing (numerically sound).** The balancing transform MUST be
   built in square-root form: factor `Wc = R Rᵀ` (from a symmetric
   eigendecomposition `Wc = Q diag(λ) Qᵀ`, so `R = Q diag(√λ)`), diagonalize
   `M = Rᵀ Wo R = U diag(σ²) Uᵀ`, and set `T = R U Σ^{-1/2}`,
   `T⁻¹ = Σ^{1/2} Uᵀ R⁻¹`. The Hankel singular values are `σ = √diag(σ²)`, sorted
   non-increasing. The symmetric eigendecompositions MUST come from a deterministic
   solver (a cyclic Jacobi sweep with fixed order and tolerance).

4. **Balanced realization is genuinely balanced.** With no truncation (`k = n`)
   the transformed Gramians `T⁻¹ Wc T⁻ᵀ` and `Tᵀ Wo T` MUST both equal
   `diag(σ)` to numerical tolerance — a checkable invariant an implementation
   MUST be able to demonstrate.

5. **Truncation keeps the dominant states.** For a retained order `k`, an
   implementation MUST form the full balanced realization and keep the leading
   block: `Aᵣ = (T⁻¹ A T)[0..k, 0..k]`, `Bᵣ = (T⁻¹ B)[0..k, :]`,
   `Cᵣ = (C T)[:, 0..k]`, indexed so the retained states carry the largest Hankel
   singular values. The order MUST be selectable **either** by an explicit `k`
   **or** by an energy tolerance — the smallest `k` whose discarded tail
   `Σ_{i>k} σᵢ` is at most a caller-given fraction of `Σᵢ σᵢ`.

6. **Stability is preserved.** Balanced truncation of a Hurwitz system is Hurwitz;
   a conforming reduced `Aᵣ` MUST remain stable (verifiable via the shared
   eigensolver), and the a priori error bound `2 · Σ_{i>k} σᵢ` MUST be reported.

7. **Preconditions and shapes are validated at the boundary.** A non-square `A`, a
   `B` without one row per state, a `C` without one column per state, an
   out-of-range order (`k ∉ 1..=n`), or an energy tolerance outside `[0, 1)` MUST
   surface as distinct typed errors. A singular Gramian (a zero Hankel singular
   value — a mode both uncontrollable and unobservable, leaving the balancing
   transform undefined) MUST be reported as `SingularSystem`, never silently
   repaired.

8. **Determinism.** The Lyapunov solve, Gaussian elimination, the Jacobi
   eigensolver (fixed sweep order, canonicalized eigenvector signs), the Hankel
   singular value ordering, and the truncation MUST all be deterministic.
   Identical inputs MUST produce a **bit-identical** result: identical `Aᵣ`, `Bᵣ`,
   `Cᵣ` (to `f64` bit patterns) and an identical Hankel-singular-value vector.

## Public API

```text
balanced_truncation(&Matrix /*A*/, &Matrix /*B*/, &Matrix /*C*/, &ReductionSpec)
    -> Result<ReducedModel, ModelReduceError>

hankel_singular_values(&Matrix /*A*/, &Matrix /*B*/, &Matrix /*C*/)
    -> Result<Vec<f64>, ModelReduceError>

controllability_gramian(&Matrix /*A*/, &Matrix /*B*/) -> Result<Matrix, ModelReduceError>
observability_gramian (&Matrix /*A*/, &Matrix /*C*/) -> Result<Matrix, ModelReduceError>

ReductionSpec { Order(usize), EnergyTolerance(f64) }
ReducedModel  { a, b, c, hankel_singular_values, order }
ReducedModel::error_bound() -> f64        // 2 · Σ_{i>k} σ_i
```

`Matrix` and `Complex` are re-exported from `lawsynth-koopman` so a caller builds
`(A, B, C)` and reads results without a separate dependency. The Gramians and the
Hankel singular values are exposed directly because they are the auditable
intermediate quantities of the method. This crate delivers the **linear
model-reduction library** only; wiring it into a discovery report, a controller,
or a simulation loop is downstream and out of scope here.

## Honest scope & limits

- **Continuous-time stable systems only.** The Gramians, the `Re(λ) < 0` stability
  notion, and the error bound are continuous-time and require a Hurwitz `A`.
  Discrete-time balancing (the Stein equation) and reduction of unstable systems
  (coprime-factor or additive-decomposition variants) are **not** implemented; a
  non-Hurwitz `A` is rejected, never reduced.
- **`D` passes through untouched.** Balanced truncation reduces the state map only;
  a feedthrough term `D` (if any) is unchanged and is deliberately out of scope
  here, so the crate takes `(A, B, C)`.
- **The a priori bound assumes exact arithmetic.** `‖G − Gᵣ‖∞ ≤ 2 · Σ_{i>k} σᵢ`
  is a theorem about the exact balanced truncation; finite-precision balancing
  adds a small extra error, which is precisely why the full Hankel spectrum and
  the reduced spectrum are reported rather than assumed.
- **Conditioning degrades at the edges.** A highly non-normal `A`, a near-minimal
  realization with a tiny Hankel-singular-value gap, or a nearly singular Gramian
  makes `R`, `Σ^{-1/2}`, and the balancing transform ill-conditioned; the reduced
  model then drifts, and a zero Hankel singular value makes the transform outright
  undefined (reported, not repaired). Truncating across a small `σ_k − σ_{k+1}`
  gap is a modelling choice the caller owns.
- **A reduction of the given realization, not the world.** The reduced model is
  faithful to `(A, B, C)` as supplied; it inherits whatever modelling error that
  linear realization already carries relative to the true (possibly nonlinear)
  system.

## Non-goals

- No discrete-time balancing, no reduction of unstable or marginally stable
  systems, no coprime-factor / balanced-stochastic / positive-real variants.
- No frequency-weighted balancing, no Hankel-norm optimal approximation, no
  moment-matching / Krylov (rational interpolation) or POD-based reduction.
- No nonlinear model reduction, no reduction of the underlying symbolic field, and
  no global or basin-of-attraction claim about the reduced dynamics.
