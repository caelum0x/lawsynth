# State-estimation boundary (v2-A)

This directory specifies **deterministic state estimation** — given a
linear(ized) system with a *partial* output map, designing and running an
estimator that reconstructs the *full* state from partial, noisy measurements,
implemented in `crates/lawsynth-estimate`. It is a **boundary specification** in
the house style: it states what a conforming implementation MUST do, and —
crucially — what an estimator is and is not allowed to claim.

## Motivation

A discovered dynamical law linearized at a fixed point is `ẋ = A x + B u` with
`A = ∂f/∂x` the Jacobian (the object `crates/lawsynth-stability` classifies and
`crates/lawsynth-feedback` controls). In practice not every state is measured:
sensing produces an **output** `y = C x`, where `C` (`p × n`) selects or mixes
the observed states. When `p < n` the state is only *partially* observed.

The estimation question is then: reconstruct the full `x` from the stream of
`y` (and the known input `u`). The standard device is an **observer**, a copy of
the model corrected by the measurement residual (the *innovation* `y − C x̂`):

```
x̂̇ = A x̂ + B u + L (y − C x̂),   so the error e = x − x̂ obeys ė = (A − L C) e.
```

Choosing the gain `L` so `A − L C` is Hurwitz makes `x̂ → x`. Two classical
answers:

- **Luenberger observer / pole placement** puts the error eigenvalues at chosen
  locations (a desired estimation decay rate). For a single output it is solved
  exactly by **Ackermann's formula**.
- **Kalman filter** instead chooses `L` optimally given a stochastic model —
  process covariance `Q` and measurement covariance `R`. The steady-state
  continuous gain is `L = P Cᵀ R⁻¹`, where `P` solves the **filter** continuous
  algebraic Riccati equation (CARE) `A P + P Aᵀ − P Cᵀ R⁻¹ C P + Q = 0`.

### Duality (why this reuses the feedback crate)

Estimator design is the **exact dual** of feedback design, because
`(A − L C)ᵀ = Aᵀ − Cᵀ Lᵀ`. Placing the error poles of `(A, C)` is placing the
feedback poles of `(Aᵀ, Cᵀ)`; the optimal filter is the LQR of `(Aᵀ, Cᵀ)`. So
this crate **reuses** `crates/lawsynth-feedback` rather than re-deriving numerics:

- **Observer** `L = place_poles(Aᵀ, Cᵀ, desired)ᵀ`. With a single output, `Cᵀ`
  is `n × 1` — the SISO case Ackermann handles.
- **Kalman** `L = lqr(Aᵀ, Cᵀ, Q, R)ᵀ`. The feedback CARE
  `AᵀP + PA − PBR⁻¹BᵀP + Q = 0`, under `(A, B) → (Aᵀ, Cᵀ)`, becomes exactly the
  filter CARE above with solution `X = P`; its gain `R⁻¹ C P` transposes to
  `P Cᵀ R⁻¹`. The feedback crate's CARE convention therefore maps to the filter
  CARE **with no sign or transpose adjustment** — the substitution is direct.

LawSynth is **deterministic and offline**. This stage reuses the deterministic
eigensolver of `crates/lawsynth-koopman` (Householder–Hessenberg + Wilkinson
complex QR) to report the achieved error spectrum `eigen(A − L C)`, and the
deterministic Ackermann / Kleinman–Riccati machinery of `crates/lawsynth-feedback`
for the gains. Identical inputs MUST yield bit-identical output.

## What an estimator IS

An estimator is a **property of the supplied linear triple `(A, C)` (plus the
design targets or covariances)**, not of any data or of the underlying nonlinear
field. The contract is:

1. **A gain for the linearization, not the nonlinear law.** `L` shapes the error
   dynamics `A − L C`. On the true nonlinear system the estimate is valid only in
   the neighbourhood of the fixed point where `A` is a faithful linearization;
   the crate makes no global reconstruction claim.
2. **Exact placement, honestly verified.** The observer returns the achieved
   error poles as `eigen(A − L C)`, so a caller reads the actually realized
   spectrum — not merely the requested one — and can see any numerical drift.
3. **The Riccati solution is checkable.** The Kalman filter returns the solved
   error covariance `P`; plugging it back into the filter CARE gives a residual a
   caller can measure. The optimality claim is exactly "this `(L, P)` is a
   stabilizing solution of the stated filter CARE", no more.
4. **A property of the expression, not of any fit.** The design carries no
   discovery confidence or fit residual. Those are separate, upstream concerns.

## Requirements

1. **Observer placement is single-output and exact (dual Ackermann).** For an
   observable pair `(A, C)` with `C` of shape `1 × n` and `n` desired error
   poles, an implementation MUST compute `L = place_poles(Aᵀ, Cᵀ, desired)ᵀ`,
   reusing the feedback crate's Ackermann routine on the dual pair, never a
   re-derived formula. A taller `C` (`p > 1`) MUST be rejected with a distinct
   `MultiOutput` error.

2. **Real gains only.** The desired error poles MUST be closed under complex
   conjugation, so the placement polynomial is real and `L` is real. A lone
   complex pole MUST be rejected (`NonRealDesignPoles`), inherited from the dual
   placement, rather than returning a complex gain.

3. **Observability is required, not assumed.** The observability matrix
   `O = [C; CA; …; CAⁿ⁻¹]` (`np × n`) MUST have full column rank `n`, checked by
   deterministic Gaussian elimination (largest-magnitude pivot, lowest index on a
   tie). A rank-deficient pair is unobservable and MUST return a distinct
   `Unobservable` error — the dual of an uncontrollable feedback pair — never a
   fabricated gain.

4. **The Kalman filter solves the filter CARE by the dual LQR.** For covariances
   `Q ⪰ 0` (process) and `R ≻ 0` (measurement), an implementation MUST compute
   `(L, P)` from `lqr(Aᵀ, Cᵀ, Q, R)`: the returned Riccati matrix is the error
   covariance `P` and `L = (lqr gain)ᵀ = P Cᵀ R⁻¹`. The underlying solve MUST be
   the feedback crate's deterministic Kleinman iteration (exact vectorized
   Lyapunov steps), not a second scheme, and `P` MUST be symmetric. The Kalman
   path admits multiple outputs (`p ≥ 1`), since the dual LQR is multi-input.

5. **Covariances are validated at the boundary.** `Q` MUST be symmetric and
   positive semidefinite; `R` MUST be symmetric and positive definite (hence
   invertible). Violations MUST surface as distinct typed errors (`NotSymmetric`,
   `NotPositiveSemidefinite`, `NotPositiveDefinite`), inherited from the dual LQR,
   never a silently repaired covariance. A non-detectable pair (the dual of
   non-stabilizable) MUST return a distinct `NotDetectable` error.

6. **Reuse the deterministic eigensolver.** The achieved error poles MUST come
   from the `lawsynth-koopman` eigensolver applied to `A − L C`, in that solver's
   canonical order, so the whole pipeline shares one audited decomposition.

7. **Simulation integrates the coupled system with fixed-step RK4.** The
   estimator demonstration MUST integrate the plant `ẋ = A x + B u` and the
   observer `x̂̇ = A x̂ + B u + L (y − C x̂)` together with a fixed-step classical
   RK4, from a caller-supplied (possibly wrong) initial estimate `x̂(0) ≠ x(0)`,
   returning both trajectories, the recorded measurements `y = C x (+ noise)`, and
   the estimation error `‖x − x̂‖₂` over time. Inputs and any measurement noise
   are held over each step (zero-order hold).

8. **Any noise is seeded and deterministic.** Optional measurement noise MUST be
   drawn from the project's SplitMix64 generator (`lawsynth_core::DeterministicRng`)
   shaped by Box–Muller, seeded from a fixed `u64` — never the wall clock.

9. **Determinism.** Observability, dual placement / LQR, the eigen-verification,
   RK4 integration, and seeded noise MUST all be deterministic. Identical inputs
   MUST produce a **bit-identical** result: identical gain `L`, covariance `P`,
   and trajectories (to `f64` bit patterns), and an identical error-pole set in
   identical order.

## Public API

```text
design_observer(&Matrix /*A*/, &Matrix /*C, 1×n*/, &[Complex] /*error poles*/)
    -> Result<Observer, EstimateError>

kalman_filter(&Matrix /*A*/, &Matrix /*C, p×n*/, &Matrix /*Q, n×n*/, &Matrix /*R, p×p*/)
    -> Result<Observer, EstimateError>

run_observer(&Observer, &Matrix /*A*/, &Matrix /*B*/, &Matrix /*C*/,
             &[f64] /*true x0*/, &[f64] /*est x0*/, &[Vec<f64>] /*inputs*/,
             Option<MeasurementNoise>, f64 /*dt*/, usize /*steps*/)
    -> Result<EstimateTrajectory, EstimateError>

observability_matrix(&Matrix, &Matrix) -> Result<Matrix, EstimateError>
is_observable(&Matrix, &Matrix) -> Result<bool, EstimateError>

Observer { gain: Matrix, error_poles: Vec<Complex>, covariance: Option<Matrix>,
           method: ObserverMethod }
Observer::is_convergent(margin) -> bool
EstimateTrajectory { times, true_states, estimates, measurements, errors }
MeasurementNoise { seed, std_dev }
```

`Matrix` and `Complex` are re-exported from `lawsynth-koopman` so a caller builds
`(A, B, C)` and reads error poles / covariance without a separate dependency. The
observer returns `covariance: None` (it forms no covariance); the Kalman filter
returns `Some(P)`. This crate delivers the **estimation library** only; wiring it
into a controller (the separation principle), a discovery report, or a live sensor
loop is downstream and out of scope here.

## Honest scope & limits

- **Observer placement is single-output only.** Dual Ackermann applies to one
  measured output (`C` of height 1). Multi-output placement has extra freedom that
  must be resolved by a robustness criterion (dual Kautsky–Nichols / robust
  eigenstructure assignment); that is deliberately **not** implemented, and a
  taller `C` is rejected rather than silently reduced. The Kalman filter *does*
  handle multiple outputs, via the multi-input dual LQR.
- **Kalman is steady-state and continuous-time only.** This is the
  infinite-horizon continuous filter (a constant gain from the CARE). There is no
  time-varying Riccati, no discrete-time (DARE) filter, and no extended /
  unscented filter for nonlinear models — the estimate is linear and local to the
  linearization.
- **Optimality is Gaussian and asymptotic.** The Kalman gain is optimal only
  under the linear-Gaussian assumptions (white `Q`, `R`) at steady state; the
  returned `P` is the *steady-state* error covariance, and the CARE residual is a
  modelling check, not a proof of optimality to machine precision.
- **Detectability and `R ≻ 0` are assumed.** The dual LQR needs a stabilizing
  solution; a non-detectable pair (an unstable, unobservable mode) has none and is
  reported as such. `R` must be positive definite for `R⁻¹` to exist.
- **The estimate is only as local as the linearization.** `A` is a Jacobian at a
  point; convergence `x̂ → x` is guaranteed for the linear error dynamics, not for
  the true nonlinear flow away from the fixed point.
- **Numerical conditioning degrades near the edges.** For large `n`, a
  near-unobservable pair, or a fast desired error spectrum, the observability
  matrix, the placement polynomial, and the `n² × n²` Lyapunov system become
  ill-conditioned; the achieved error poles then drift from the requested ones,
  which is precisely why the achieved spectrum is reported rather than assumed.
  RK4 integration adds a fixed-step truncation error to the simulated trajectories.

## Non-goals

- No multi-output / robust observer placement, no reduced-order (Gopinath)
  observer, no unknown-input or disturbance observer.
- No discrete-time filter (DARE), no time-varying / finite-horizon Kalman, no
  extended, unscented, or particle filtering for nonlinear models.
- No separation-principle controller synthesis (combining an observer with a
  feedback gain), no output-feedback design — estimation only.
- No region-of-attraction estimate or any nonlinear / global reconstruction
  guarantee for the underlying field.
