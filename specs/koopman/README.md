# Koopman / DMD contract

`lawsynth-koopman` discovers a **linear operator** that advances a system one
step in time. Given snapshot pairs — states `X` and their one-step successors
`X'` — it fits `X' ≈ A X` (Dynamic Mode Decomposition, DMD), the controlled
form `X' ≈ A X + B U` (DMD with control, DMDc), and the lifted form
`ψ(X') ≈ K ψ(X)` over a feature dictionary (Extended DMD, EDMD). It returns the
identified operator, its eigenvalues and modes, and a forward `predict` roll-out.

This is the compact linear-operator discovery method that SciML's
`DataDrivenDMD` provides and that LawSynth previously lacked (see
[`docs/research/competitive-analysis.md`](../../docs/research/competitive-analysis.md)
recommendation #3 and [`docs/roadmap/expansion-v2.md`](../../docs/roadmap/expansion-v2.md)
milestone v2-A). It ports the *public algorithm*, not any competitor code.

## Boundary

The public construction paths are:

- `dmd(&X, &Xprime, rank)` — exact/SVD DMD. Truncated SVD `X = U Σ Vᵀ`, reduced
  operator `Ã = Uᵣᵀ X' Vᵣ Σᵣ⁻¹`, its eigendecomposition (DMD eigenvalues and
  exact DMD modes), and the identified full operator `A = X' Vᵣ Σᵣ⁻¹ Uᵣᵀ`.
- `dmdc(&X, &Xprime, &U, rank)` — DMD with control. Fits `[A B]` from the
  stacked regression `X' = [A B] · [X; U]` via a truncated SVD pseudo-inverse of
  the stacked matrix, then splits the state and control blocks.
- `edmd(&dataset, &dictionary, rank)` — Extended DMD. Lifts each state snapshot
  through a deterministic polynomial feature dictionary and fits the linear
  operator in the lifted space; this is the bridge to (mildly) nonlinear
  dynamics.

Snapshot matrices are column-major in meaning: each **column** is one state
observation, each **row** is one state coordinate. `X` and `X'` must share
shape; `X'` holds the one-step successors of `X`.

## Determinism

Every fit is deterministic. The singular value decomposition is a one-sided
Jacobi iteration with a fixed sweep order and a fixed convergence tolerance; the
eigendecomposition is a Householder Hessenberg reduction followed by a
Wilkinson-shifted complex QR iteration with fixed deflation tolerances; modes are
recovered by inverse iteration from a fixed starting vector and a canonical
phase/scale normalisation. Identical inputs and rank produce bit-identical
outputs. No randomness, no wall-clock, no threads, no external solver.

## Honesty about the model

The discovered object is a **linear** operator (DMD/DMDc) or a **lifted-linear**
operator over an explicit finite dictionary (EDMD). It is not a nonlinear
symbolic law. DMD is exact only for genuinely linear dynamics; on nonlinear
systems it is the best linear approximation in the least-squares sense over the
supplied snapshots, and EDMD extends that reach only as far as the chosen
dictionary spans the observables. Eigenvalues are reported in discrete time (per
step); `continuous_eigenvalues(dt)` maps them to `ln(λ)/dt` for growth
(real part) and oscillation (imaginary part), and is undefined for
non-positive-real eigenvalues (returns the principal branch).

## Numerical limits

The linear algebra is hand-rolled in `f64` with no external crate. The one-sided
Jacobi SVD is accurate to a few ulp of the singular values for well-conditioned
inputs; recovery of a well-separated linear operator's eigenvalues is typically
to ~1e-10 or better. Extremely ill-conditioned snapshot matrices, repeated or
clustered eigenvalues, and defective (non-diagonalisable) operators degrade the
accuracy of the modes (eigenvectors) before the eigenvalues; the crate reports
the singular values so callers can judge the effective rank. This is not a
general-purpose LAPACK replacement.
