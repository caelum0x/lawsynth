# Lyapunov-exponent boundary (v2-A)

This directory specifies **deterministic Lyapunov-spectrum estimation** —
integrating the variational flow of a discovered autonomous field to estimate
the average exponential rates at which nearby trajectories separate, implemented
in `crates/lawsynth-lyapunov`. It is a **boundary specification** in the house
style: it states what a conforming implementation MUST do, and — crucially —
what an estimated exponent is and is not allowed to claim.

## Motivation

A discovered dynamical law is an autonomous vector field: one right-hand side per
state, `ẋ_i = f_i(x)`, each `f_i` an expression tree in the `lawsynth-expr` IR.
The **stability** crate reads the *local* behaviour near an equilibrium off the
Jacobian eigenvalues there; the **bifurcation** work tracks how equilibria move
as a parameter varies. Neither answers the *global, long-term* question: does the
flow, wherever it settles, **stretch** nearby states apart or **fold** them
together?

The Lyapunov spectrum is that answer. For an `n`-dimensional flow it is a set of
`n` numbers `λ_1 ≥ λ_2 ≥ … ≥ λ_n`, the average exponential rates of separation
along `n` independent directions carried with the trajectory:

- A **positive largest exponent** is the signature of **chaos** — sensitive
  dependence on initial conditions, the hallmark of a strange attractor.
- The exponents that are **zero** correspond to neutral directions (the flow
  direction of any non-fixed bounded trajectory contributes a zero exponent).
- The **sum** of the exponents is the average rate of phase-space **volume**
  change — the time-averaged divergence of the field. A dissipative system
  contracts volume, so its exponents sum to a negative number.

LawSynth is **deterministic and offline**. The spectrum is obtained by
**integrating the variational (linearized) flow with the analytic Jacobian** —
not by finite-differencing the field, and not by probing a trained surrogate.
Identical inputs MUST yield bit-identical output.

## What an estimated spectrum IS

The emitted spectrum is a **time-averaged estimate for the reported discrete
trajectory**, from a **fixed initial state and a fixed identity frame**. The
contract is:

1. **An estimate, not an eigenvalue.** Lyapunov exponents are *asymptotic time
   averages*; a finite integration gives an approximation whose accuracy grows
   with the integration length. This is not the exact linear-algebra readout the
   stability crate returns at a fixed point — it is a Monte-Carlo-free but still
   *converging* average along a trajectory.
2. **Analytic in its derivatives, numeric in its integration.** The Jacobian
   `J(x) = ∂f/∂x` is the exact symbolic derivative of the given field; only the
   time integration and the QR renormalization are numerical. No difference
   quotient of the field appears in the integrator.
3. **A property of the expression and the chosen trajectory, not of any data.**
   The analysis differentiates and integrates whatever field and initial state it
   is handed; it carries no discovery confidence or fit residual. Those are
   separate, upstream concerns.
4. **Consistent between state and frame.** The augmented system `(x, Q)` is
   advanced by **one shared** fixed-step integrator, so the state and every
   perturbation column see identical stage points — the exponents are those of
   the *reported* discrete trajectory, not of some other discretisation.
5. **Honest about what it explored.** The estimate describes the region the
   trajectory actually visited over the averaging window. An initial condition
   outside the basin of the intended attractor, or a transient that was not
   discarded, is reflected in the numbers as-is, never corrected by fiat.

## The method (Benettin / QR)

For state `x ∈ Rⁿ` and `ẋ = f(x)`, evolve `x` together with a frame of `n`
perturbation vectors, the columns of `Q ∈ Rⁿˣⁿ`, under the variational flow:

```text
ẋ   = f(x)
q̇_j = J(x) · q_j        J(x) = ∂f/∂x        (j = 0 … n-1)
```

Every `k` steps, QR-decompose the evolved frame `Q = Q'·R` (with `Q'`
orthonormal and `R` upper-triangular with non-negative diagonal), keep `Q'` as
the new frame, and accumulate `ln R_ii` for each `i`. The reorthonormalization
prevents every column from collapsing onto the single fastest-growing direction
and keeps the frame well-conditioned. After discarding a transient, the `i`-th
Lyapunov exponent is

```text
λ_i = (Σ ln R_ii) / T,
```

where `T` is the elapsed time of the averaging window. The exponents are returned
sorted **descending**.

## Requirements

1. **Analytic Jacobian, reused.** A conforming implementation MUST obtain `J(x)`
   from the analytic Jacobian of `crates/lawsynth-jacobian`. It MUST NOT
   finite-difference the field to form the variational dynamics. The supported
   node kinds, the power-rule choice, and the "no silent zeros" guarantee are
   inherited from the analytic-Jacobian contract.

2. **One shared fixed-step integrator on the augmented system.** The augmented
   vector `y = [x, q_0, …, q_{n-1}]` (length `n·(1 + n)`) MUST be advanced by a
   single fixed-step fourth-order Runge–Kutta scheme, so the state block and every
   frame column are integrated with the **same** stages and step. The state MUST
   NOT be integrated separately from, or with a different step than, the frame.

3. **Deterministic frame and reorthonormalization.** The initial frame MUST be a
   fixed, content-independent orthonormal set — the identity `Q = I`. The frame
   MUST be reorthonormalized every `k` steps by a deterministic Gram–Schmidt /
   Householder QR (fixed column-then-row order, no pivoting, no RNG). The diagonal
   `R_ii` (the per-interval local expansion factors) MUST be non-negative, and
   `ln R_ii` accumulated in a fixed index order.

4. **Transient discard, then time-average.** A leading fraction of the run MUST be
   discardable before the log-accumulation begins, so the estimate reflects the
   attractor rather than the approach to it. The exponent is the accumulated log
   sum divided by the elapsed time `T` of the averaging window. The
   reorthonormalization MUST continue *through* the transient (only the
   accumulation is suppressed), so the frame does not degenerate.

5. **Derived diagnostics.** From the descending spectrum the implementation MUST
   report:
   - the **largest** exponent `λ_1` (chaos when positive);
   - the **sum** `Σ λ_i`, equal to the time-averaged divergence (mean trace of
     `J` along the trajectory);
   - the **Kaplan–Yorke (Lyapunov) dimension**
     `D_KY = j + (Σ_{i≤j} λ_i)/|λ_{j+1}|`, where `j` is the largest index whose
     partial sum `Σ_{i≤j} λ_i` is non-negative. If **no** partial sum is
     non-negative (`λ_1 < 0`) the dimension is `0`; if **every** partial sum stays
     non-negative (`j = n`) the fraction is undefined and the full dimension `n`
     is reported. Neither boundary is forced into the fractional formula.

6. **Determinism.** Frame initialization, the RK4 stages, the QR, the transient
   boundary, the accumulation, and the final sort MUST be deterministic. Identical
   `(fields, states, initial, LyapunovConfig)` inputs MUST produce a
   **bit-identical** `LyapunovReport`: identical exponents, sum, and dimension to
   their `f64` bit patterns.

7. **Autonomy and totality.** The field MUST be autonomous: every symbol it
   references MUST be one of the states, otherwise there is no value to bind and
   the implementation MUST return a typed error. It MUST reject, with distinct
   typed errors and never a fabricated or silently dropped result: an empty state
   space; an initial vector whose length differs from `states`; a non-finite
   initial value; an invalid config (`dt ≤ 0`, zero `steps`, a zero
   reorthonormalization interval, a transient fraction outside `[0, 1)`); any
   structural or numeric failure surfaced by the analytic Jacobian; a blow-up to a
   non-finite state; or a perturbation-frame column that collapses to zero length.

## Public API

```text
lyapunov_spectrum(&[(Identifier, Expr)], &[Identifier], &[f64], &LyapunovConfig)
    -> Result<LyapunovReport, LyapunovError>
largest_lyapunov(&[(Identifier, Expr)], &[Identifier], &[f64], &LyapunovConfig)
    -> Result<f64, LyapunovError>

LyapunovConfig::new(dt, steps, reorthonormalization_interval, transient_fraction)
    -> Self                                       // + with_* builder setters
LyapunovReport::exponents() -> &[f64]             // sorted descending
LyapunovReport::largest() -> f64
LyapunovReport::sum() -> f64
LyapunovReport::kaplan_yorke_dimension() -> f64
LyapunovReport::integration_time() -> f64         // averaging-window length T
LyapunovReport::dimension() -> usize
LyapunovReport::to_canonical_string() -> String   // determinism fingerprint
```

`LyapunovConfig` carries the fixed step, the step count, the
reorthonormalization interval `k`, and the transient fraction. This crate
delivers the **spectrum-estimation library** only; wiring the diagnostic into a
discovery report, an attractor classifier, or a forecasting-horizon estimate is
downstream and out of scope here.

## Honest verification

The reference suite pins the estimator against cases with a known spectrum or a
known divergence, using tolerances appropriate to a time-averaged estimator (not
machine precision), with the integration length stated per case:

- **Linear decay** `ẋ = −x` → `{−1}`, and `ẋ = −x, ẏ = −2y` → `{−1, −2}`, to
  `1e-3` over a short run (the field is linear, so the estimate is essentially
  exact up to RK4 error).
- **Harmonic oscillator** `ẋ = y, ẏ = −x` (conservative) → both exponents ≈ 0 to
  `1e-3`; a discriminating "no separation, no chaos" check.
- **Damped oscillator** `ẋ = y, ẏ = −x − 0.3y` → both exponents negative and the
  **sum** equal to the constant trace `−0.3` to `1e-3` — a strong
  sum-equals-divergence check independent of the individual values.
- **Lorenz** `σ=10, ρ=28, β=8/3` → a positive largest exponent (≈ 0.906) to a
  **broad** `±0.25` (honestly reflecting slow chaotic convergence), a middle
  exponent ≈ 0, a fractional Kaplan–Yorke dimension in `(2, 3)`, and a **sum**
  equal to the constant divergence `−(σ+1+β) ≈ −13.667` to `0.05` — the tight,
  reliable identity. The Lorenz run integrates `70 000` steps of `dt = 0.01`
  (to `t = 700`, averaging over ≈ 600 time units), reorthonormalizing every
  10 steps.

Determinism (bit-identical reports and exponents) and the typed error paths are
exercised alongside.

## Honest scope & limits

- **A time-averaged estimate, not an exact spectrum.** Accuracy depends on the
  integration length, the step `dt`, and the reorthonormalization interval `k`.
  The individual chaotic (largest) exponent converges **slowly** — its running
  estimate fluctuates like `1/√T` — while the **sum** (divergence) is tight
  because it equals a constant or slowly-varying trace independent of the
  separation dynamics. When in doubt, trust the sum and give the individual
  chaotic exponent a broad tolerance.
- **The trajectory must explore the attractor.** The exponents describe the
  region the trajectory actually visited. The initial condition MUST lie in the
  basin of the intended attractor, and enough transient MUST be discarded for the
  trajectory to settle onto it; otherwise the numbers describe the transient, not
  the attractor.
- **Fixed-step RK4 error.** There is no adaptive step or error control. On stiff
  or fast systems a step that resolves the slow dynamics may under-resolve the
  fast variational directions; tighten `dt` (and shorten `k` to keep the frame
  well-conditioned) when the estimate is unstable. Too large a `k` lets the
  columns grow so disparate that the Gram–Schmidt loses precision; too small a `k`
  wastes work.
- **Smooth autonomous fields only.** The method assumes a smooth autonomous field
  with a bounded trajectory. A blow-up (unbounded trajectory) is reported as a
  typed error, not as a spurious exponent. Supported functions are exactly the
  IR's (`+ − × ÷`, `^`, unary negate, `exp`, `log`, `sin`, `cos`), inherited from
  the analytic Jacobian.
- **The spectrum describes the given field, nothing more.** It carries no
  discovery confidence, no fit residual, and no forecasting guarantee; those
  belong to the stages that consume it.

## Non-goals

- No adaptive step size, error control, or stiff/implicit integrator — a single
  fixed-step RK4 only.
- No basin analysis, attractor reconstruction, or automatic transient detection —
  the initial state and transient fraction are caller-supplied.
- No estimation from a time series alone (no delay embedding / Rosenstein /
  Wolf-from-data method) — this crate integrates a *known* analytic field, not a
  measured trajectory.
- No downstream chaos verdict, forecasting-horizon, or attractor-dimension
  classification report — those consume the spectrum and live in their own
  crates with their own contracts.
