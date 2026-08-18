# Discrete-time-control boundary (v2-A)

This directory specifies **deterministic discrete-time control and estimation** —
given a sampled-data linear(ized) system, computing a feedback gain `K`, a filter
gain `L`, or a Luenberger observer gain that drive the closed-loop / error
dynamics inside the unit circle, implemented in `crates/lawsynth-discrete`. It is
a **boundary specification** in the house style: it states what a conforming
implementation MUST do, and — crucially — what a designed gain is and is not
allowed to claim.

## Motivation

A discovered dynamical law sampled at a fixed step `Δt` — or a Koopman/DMD
operator, which is *natively* discrete (`x' ≈ A x + B u`) — advances the state one
step at a time:

```
x_{k+1} = A x_k + B u_k,   y_k = C x_k.
```

This is the discrete-time counterpart of the continuous pair `ẋ = A x + B u` that
`crates/lawsynth-feedback` controls and `crates/lawsynth-estimate` observes. The
engineering questions are the same — choose `u = −K x` so the loop behaves, and
reconstruct `x` from partial `y` — but the **stability notion changes**. A
continuous system is stable when every eigenvalue lies in the open **left
half-plane** (`Re λ < 0`); a discrete system is stable when every eigenvalue lies
strictly inside the **unit circle** (`|λ| < 1`, i.e. spectral radius `< 1`). The
governing equations are therefore the *discrete* algebraic Riccati equations, not
the continuous ones — they are genuinely different and are solved here directly,
not by mapping to the continuous crate.

- **Discrete LQR (DLQR)** minimizes `Σ_k (xₖᵀQxₖ + uₖᵀRuₖ)`. The optimal gain is
  `K = (R + BᵀPB)⁻¹BᵀPA`, where `P` solves the **discrete algebraic Riccati
  equation (DARE)** `P = AᵀPA − AᵀPB(R + BᵀPB)⁻¹BᵀPA + Q`.
- **Discrete Kalman filter** chooses the estimator gain optimally given process
  covariance `Q` and measurement covariance `R`. The steady-state **predictor**
  gain is `L = APCᵀ(R + CPCᵀ)⁻¹`, where `P` solves the **dual (filter) DARE**
  `P = APAᵀ − APCᵀ(R + CPCᵀ)⁻¹CPAᵀ + Q`.
- **Discrete Luenberger observer** places the error poles `A − LC` exactly at
  chosen z-plane locations by the dual of Ackermann's formula.

### Duality

The filter DARE for `(A, C)` is exactly the control DARE for the transposed pair
`(Aᵀ, Cᵀ)`. The implementation exploits this directly: `discrete_kalman` reuses
the same DARE value iteration on `(Aᵀ, Cᵀ)` and reads the predictor gain
`L = APCᵀ(R + CPCᵀ)⁻¹` off the solution — one audited Riccati routine serves both
control and estimation.

LawSynth is **deterministic and offline**. This stage reuses the deterministic
eigensolver of `crates/lawsynth-koopman` (Householder–Hessenberg + Wilkinson
complex QR) to report the achieved closed-loop / error spectrum; the only other
numerics are local Gaussian elimination and matrix polynomials. Identical inputs
MUST yield bit-identical output.

## What a designed gain IS

A gain is a **property of the supplied linear system and the design weights**, not
of any data or of the underlying nonlinear field. The contract is:

1. **A gain for the linearization, not the nonlinear law.** `K` stabilizes
   `A − BK`; `L` shapes the error dynamics `A − LC`. On the true nonlinear system
   the guarantee is valid only in the neighbourhood of the fixed point where `A`
   is a faithful one-step linearization; the crate makes no basin-of-attraction or
   global claim.
2. **Discrete stability, honestly verified.** Every design returns its achieved
   spectrum as `eigen(A − BK)` / `eigen(A − LC)`, so a caller reads the actually
   realized eigenvalues — and confirms the discrete condition `|λ| < 1` directly —
   rather than trusting the requested spectrum.
3. **The Riccati solution is checkable.** DLQR and the Kalman filter return the
   solved `P`; plugging it back into the relevant DARE gives a residual a caller
   can measure. The optimality claim is exactly "this `(K, P)` / `(L, P)` is a
   stabilizing solution of the stated DARE", no more.
4. **A property of the expression, not of any fit.** The design carries no
   discovery confidence or fit residual. Those are separate, upstream concerns.

## Requirements

1. **DLQR solves the DARE by a deterministic value iteration.** From `P₀ = Q`, an
   implementation MUST iterate
   `P ← AᵀPA − AᵀPB(R + BᵀPB)⁻¹BᵀPA + Q` to a fixed relative convergence
   tolerance, bounded by a maximum iteration count. Each iterate MUST be
   symmetrized (`(P + Pᵀ)/2`), exact in infinite precision. The inner solve of
   `(R + BᵀPB)⁻¹` MUST use local deterministic Gaussian elimination. The gain is
   then `K = (R + BᵀPB)⁻¹BᵀPA` for the control law `u = −K x`.

2. **The Kalman filter solves the dual filter DARE.** For covariances `Q ⪰ 0`
   (process) and `R ≻ 0` (measurement), an implementation MUST compute `P` from
   the same DARE iteration applied to `(Aᵀ, Cᵀ)` and return the predictor gain
   `L = APCᵀ(R + CPCᵀ)⁻¹`, whose error dynamics are `A − LC`. The Kalman path
   admits multiple outputs (`p ≥ 1`), since the dual control problem is
   multi-input.

3. **Discrete observer placement is single-output and exact (dual Ackermann).**
   For an observable pair `(A, C)` with `C` of shape `1 × n` and `n` desired
   z-plane error poles, an implementation MUST compute `L = p(A) O⁻¹ eₙ`, where
   `O = [C; CA; …; CAⁿ⁻¹]` is the observability matrix, `eₙ = [0 … 0 1]ᵀ`, and `p`
   is the monic desired characteristic polynomial `∏(z − λᵢ)`. `O⁻¹` and `p(A)`
   MUST be formed with local deterministic Gaussian elimination and matrix
   multiplication. A taller `C` (`p > 1`) MUST be rejected with a distinct
   `MultiOutput` error.

4. **Real gains only.** The desired observer poles MUST be closed under complex
   conjugation, so `p` has real coefficients and `L` is real. A lone complex pole
   MUST be rejected (`NonRealDesignPoles`) rather than returning a complex gain.

5. **Observability / stabilizability is required, not assumed.** If the
   observability matrix is singular the pair is unobservable and MUST return a
   distinct `Unobservable` error. If the DARE iteration fails to converge — a
   diverging iterate or an exhausted budget, as happens for an unstabilizable /
   undetectable unstable mode — the implementation MUST return a distinct
   `NotConvergent` error, never a fabricated or least-squares gain.

6. **Weights are validated at the boundary.** `Q` MUST be symmetric and positive
   semidefinite; `R` MUST be symmetric and positive definite (hence invertible).
   Symmetry is checked directly; definiteness is checked from the shared
   eigensolver's eigenvalues. A violated precondition MUST surface as a distinct
   typed error (`NotSymmetric`, `NotPositiveSemidefinite`, `NotPositiveDefinite`),
   never a silently repaired weight.

7. **Reuse the deterministic eigensolver.** The achieved closed-loop / error poles
   MUST come from the `lawsynth-koopman` eigensolver applied to `A − BK` /
   `A − LC`, in that solver's canonical order, so the whole pipeline shares one
   audited decomposition. Discrete stability is then read off as
   spectral radius `< 1`.

8. **Determinism.** The DARE value iteration, the polynomial expansion,
   observability, Gaussian elimination (largest-magnitude pivot, lowest index on a
   tie), and the eigen-verification MUST all be deterministic — fixed loop order,
   no RNG, no clock. Identical inputs MUST produce a **bit-identical** result:
   identical `K` / `L` / `P` (to `f64` bit patterns) and an identical achieved
   spectrum in identical order.

## Public API

```text
dlqr(&Matrix /*A*/, &Matrix /*B*/, &Matrix /*Q*/, &Matrix /*R*/)
    -> Result<DiscreteGain, DiscreteError>

discrete_kalman(&Matrix /*A*/, &Matrix /*C, p×n*/, &Matrix /*Q, n×n*/, &Matrix /*R, p×p*/)
    -> Result<DiscreteObserver, DiscreteError>

discrete_observer_from_poles(&Matrix /*A*/, &Matrix /*C, 1×n*/, &[Complex] /*z-plane poles*/)
    -> Result<DiscreteObserver, DiscreteError>

DiscreteGain { k: Matrix, achieved_poles: Vec<Complex>, p: Matrix }
DiscreteGain::spectral_radius() -> f64
DiscreteGain::is_stable(margin) -> bool   // all |λ| < 1 − margin

DiscreteObserver { l: Matrix, error_poles: Vec<Complex>, p: Option<Matrix>, method: ObserverMethod }
DiscreteObserver::spectral_radius() -> f64
DiscreteObserver::is_convergent(margin) -> bool   // all |λ| < 1 − margin
```

`Matrix` and `Complex` are re-exported from `lawsynth-koopman` so a caller builds
`(A, B, C)` and reads achieved spectra without a separate dependency. DLQR always
returns `P`; the Kalman filter returns `Some(P)` (the error covariance); observer
placement returns `None` (it forms no covariance). This crate delivers the
**discrete design library** only; wiring it into a controller, a discovery report,
or a simulation loop is downstream and out of scope here.

## Honest scope & limits

- **Discrete stability means the unit circle.** The whole crate's stability notion
  is `|λ| < 1` (spectral radius below one), *not* the continuous `Re λ < 0`. The
  DARE, the DLQR/Kalman gains, and `is_stable` / `is_convergent` are all
  discrete-time; do not confuse them with the continuous CARE of
  `crates/lawsynth-feedback`.
- **DARE convergence requires stabilizability / detectability.** The value
  iteration converges to the stabilizing solution only for a stabilizable `(A, B)`
  (detectable `(A, C)` for the filter). A genuinely unstabilizable unstable mode
  has no stabilizing DARE solution: the iterate diverges and is reported as
  `NotConvergent` rather than returned as a fabricated gain.
- **Value iteration converges linearly — slow near marginal stability.** The
  fixed-point map contracts at a rate governed by the closed-loop spectral radius
  squared, so convergence is fast for well-damped systems but **slows sharply as
  poles approach the unit circle**. The iteration is bounded by a maximum count; a
  near-marginal problem may hit that bound and report `NotConvergent`. (A
  quadratically convergent doubling scheme is a possible future refinement, out of
  scope here.)
- **`R ≻ 0` is assumed.** `R` must be positive definite for `R + BᵀPB` (resp.
  `R + CPCᵀ`) to be invertible; a semidefinite or indefinite `R` is rejected.
- **The gain is only as local as the linearization.** `A` is a one-step
  linearization at a point; the closed-loop / error guarantee is local. Nothing
  here certifies a region of attraction or behaviour away from the fixed point.
- **Observer placement is single-output only.** Dual Ackermann applies to one
  measured output (`C` of height 1); a taller `C` is rejected rather than silently
  reduced. The Kalman filter *does* handle multiple outputs, via the multi-input
  dual DARE.
- **Numerical conditioning degrades near the unit circle.** For large `n`, a
  near-uncontrollable / near-unobservable pair, or a fast desired spectrum, the
  DARE, the observability matrix, and the matrix polynomial become
  ill-conditioned; the achieved spectrum then drifts from the requested one, which
  is precisely why the achieved spectrum is reported rather than assumed. A DARE
  residual is likewise a modelling check, not a proof of optimality to machine
  precision.
- **Optimality is asymptotic and (for Kalman) Gaussian.** The Kalman gain is the
  *steady-state* constant gain, optimal only under linear-Gaussian assumptions at
  steady state; `P` is the steady-state error covariance.

## Non-goals

- No continuous-time design (that is `crates/lawsynth-feedback` /
  `crates/lawsynth-estimate`); no mapping between the two here.
- No multi-output / robust observer placement, no reduced-order observer, no
  unknown-input or disturbance observer.
- No finite-horizon / time-varying DARE, no LQG / separation-principle controller
  synthesis, no deadbeat or `H∞` design.
- No time-varying or extended/unscented filtering for nonlinear models; the
  estimate is linear and local to the linearization.
- No region-of-attraction estimate or any nonlinear / global guarantee for the
  underlying field.
