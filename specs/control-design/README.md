# Control-design boundary (v2-A)

This directory specifies **deterministic linear state-feedback design** — given a
linear(ized) system, computing a gain `K` that stabilizes it or makes it optimal
in the LQR sense, implemented in `crates/lawsynth-feedback`. It is a **boundary
specification** in the house style: it states what a conforming implementation
MUST do, and — crucially — what a designed gain is and is not allowed to claim.

## Motivation

A discovered dynamical law is an autonomous vector field `ẋ = f(x)`. Near a fixed
point `x*`, its behaviour is governed by the linearization `ẋ = A x` with
`A = ∂f/∂x` the Jacobian at `x*` (the same object `crates/lawsynth-stability`
classifies). If the system additionally admits actuation — a control input `u`
entering through an input matrix `B` — the linearized, actuated dynamics are

```
ẋ = A x + B u.
```

The engineering question is then: choose a **state-feedback law** `u = −K x` so
the closed loop `ẋ = (A − B K) x` behaves as desired. Two classical answers:

- **Pole placement** puts the closed-loop eigenvalues at chosen locations. For a
  single input it is solved exactly by **Ackermann's formula**. This is the tool
  when the target spectrum is known (a desired decay rate, a damped oscillation).
- **LQR** (linear–quadratic regulator) instead minimizes the infinite-horizon
  cost `∫₀^∞ (xᵀQx + uᵀRu) dt`. Its optimal gain is `K = R⁻¹BᵀP`, where `P`
  solves the **continuous-time algebraic Riccati equation (CARE)**
  `AᵀP + PA − PBR⁻¹BᵀP + Q = 0`. This is the tool when the trade-off between
  state error and control effort is what is specified, not the poles themselves.

LawSynth is **deterministic and offline**. This stage reuses the deterministic
eigensolver of `crates/lawsynth-koopman` (Householder–Hessenberg + Wilkinson
complex QR) to report the achieved closed-loop spectrum; the only other numerics
are local Gaussian elimination, a Kronecker-form Lyapunov solve, and matrix
polynomials. Identical inputs MUST yield bit-identical output.

## What a designed gain IS

A gain is a **property of the supplied linear pair `(A, B)` and the design
weights**, not of any data or of the underlying nonlinear field. The contract is:

1. **A gain for the linearization, not the nonlinear law.** `K` stabilizes
   `A − B K`. On the true nonlinear system it is valid only in the neighbourhood
   of the fixed point where `A` is a faithful linearization (Hartman–Grobman);
   the crate makes no basin-of-attraction or global claim.
2. **Exact placement, honestly verified.** Pole placement returns the achieved
   poles as `eigen(A − B K)`, so a caller reads the actually realized spectrum —
   not merely the requested one — and can see any numerical drift.
3. **The Riccati solution is checkable.** LQR returns the solved `P`; plugging it
   back into the CARE gives a residual a caller can measure. The optimality claim
   is exactly "this `(K, P)` is a stabilizing solution of the stated CARE", no
   more.
4. **A property of the expression, not of any fit.** The design carries no
   discovery confidence or fit residual. Those are separate, upstream concerns.

## Requirements

1. **Pole placement is single-input and exact (Ackermann).** For a controllable
   pair `(A, b)` with `b` of shape `n × 1` and `n` desired poles, an
   implementation MUST compute `K = eₙᵀ C⁻¹ p(A)`, where `C = [b, Ab, …, Aⁿ⁻¹b]`
   is the controllability matrix, `eₙᵀ = [0 … 0 1]`, and `p` is the monic desired
   characteristic polynomial `∏(s − λᵢ)`. `C⁻¹` and `p(A)` MUST be formed with
   local deterministic Gaussian elimination and matrix multiplication.

2. **Real gains only.** The desired poles MUST be closed under complex
   conjugation, so `p` has real coefficients and `K` is real. An implementation
   MUST reject a pole set whose expanded polynomial retains an imaginary part
   above tolerance (a lone complex pole) rather than return a complex gain.

3. **Controllability is required, not assumed.** If `C` is singular the pair is
   uncontrollable and no gain can place the poles; the implementation MUST return
   a distinct `Uncontrollable` error, never a fabricated or least-squares gain.

4. **LQR solves the CARE by a deterministic Kleinman iteration.** From an initial
   stabilizing gain `K₀`, each step MUST solve the Lyapunov equation
   `(A − BKᵢ)ᵀ P + P (A − BKᵢ) = −(Q + KᵢᵀRKᵢ)` and update `K = R⁻¹BᵀP`, iterating
   to a fixed convergence tolerance. The Lyapunov step MUST be solved exactly by
   vectorization — the Kronecker system `((I ⊗ Aᶜᵀ) + (Aᶜᵀ ⊗ I)) vec(P) = −vec(W)`
   solved by local Gaussian elimination — not by a second iterative scheme. The
   returned `P` MUST be symmetrized (`(P + Pᵀ)/2`), exact in infinite precision.

5. **The initial gain is constructed deterministically.** `K₀` MUST come from a
   fixed rule (Bass's algorithm: with `β > ‖A‖`, solve
   `(A + βI) Z + Z (A + βI)ᵀ = 2BBᵀ` and set `K₀ = BᵀZ⁻¹`), never from a random
   seed or an eigenvalue guess. If no stabilizing `K₀` can be built the
   implementation MUST return a distinct `NotStabilizable` error.

6. **Weights are validated at the boundary.** `Q` MUST be symmetric and positive
   semidefinite; `R` MUST be symmetric and positive definite (hence invertible).
   Symmetry is checked directly; definiteness is checked from the shared
   eigensolver's eigenvalues. A violated precondition MUST surface as a distinct
   typed error (`NotSymmetric`, `NotPositiveSemidefinite`, `NotPositiveDefinite`),
   never a silently repaired weight.

7. **Reuse the deterministic eigensolver.** The achieved closed-loop poles MUST
   come from the `lawsynth-koopman` eigensolver applied to `A − B K`, not a second
   hand-rolled solver, so the whole pipeline shares one audited decomposition, and
   MUST be reported in that solver's canonical order.

8. **Determinism.** Polynomial expansion, controllability, Gaussian elimination
   (largest-magnitude pivot, lowest index on a tie), the Lyapunov solve, and the
   Kleinman iteration MUST all be deterministic. Identical inputs MUST produce a
   **bit-identical** result: identical `K` and `P` (to `f64` bit patterns) and an
   identical achieved-pole set in identical order.

## Public API

```text
place_poles(&Matrix /*A*/, &Matrix /*b, n×1*/, &[Complex] /*desired*/)
    -> Result<Gain, FeedbackError>

lqr(&Matrix /*A*/, &Matrix /*B*/, &Matrix /*Q*/, &Matrix /*R*/)
    -> Result<Gain, FeedbackError>

Gain { k: Matrix, achieved_poles: Vec<Complex>, p: Option<Matrix> }
Gain::is_stable(margin) -> bool
```

`Matrix` and `Complex` are re-exported from `lawsynth-koopman` so a caller builds
`(A, B)` and reads achieved poles without a separate dependency. Pole placement
returns `p: None` (it forms no value function); LQR returns `p: Some(P)`. This
crate delivers the **linear feedback-design library** only; wiring it into a
controller, a discovery report, or a simulation loop is downstream and out of
scope here.

## Honest scope & limits

- **Pole placement is single-input only.** Ackermann's formula applies to one
  actuator (`b` of width 1). Multi-input placement has extra freedom that must be
  resolved by a robustness criterion (Kautsky–Nichols / robust eigenstructure
  assignment); that is deliberately **not** implemented here, and a wider `B` is
  rejected rather than silently reduced.
- **LQR assumes stabilizability and `R ≻ 0`.** The Kleinman iteration needs an
  initial stabilizing gain; the Bass bootstrap constructs one for a controllable
  (hence stabilizable) pair, but a genuinely non-stabilizable pair — an unstable
  mode with no actuation — has no solution and is reported as such. `R` must be
  positive definite for `R⁻¹` to exist.
- **The gain is only as local as the linearization.** `A` is a Jacobian at a
  point; the closed-loop guarantee is local. Nothing here certifies a region of
  attraction or behaviour away from the fixed point.
- **Numerical conditioning degrades near the edges.** For large `n`, a
  near-uncontrollable pair, or a fast desired spectrum, the controllability
  matrix, the matrix polynomial, and the `n² × n²` Lyapunov system become
  ill-conditioned; the achieved poles then drift from the requested ones, which is
  precisely why the achieved spectrum is reported rather than assumed. A Riccati
  residual is likewise a modelling check, not a proof of optimality to machine
  precision.
- **Continuous-time only.** The CARE and the `Re(λ) < 0` stability notion are
  continuous-time; discrete-time placement / DARE are not covered.

## Non-goals

- No multi-input / robust pole placement, no output feedback, no observer or
  Kalman-filter design (the separation principle is out of scope).
- No discrete-time design (DARE, deadbeat), no finite-horizon / time-varying LQR,
  no `H∞` or robust-control synthesis.
- No region-of-attraction estimate, Lyapunov-function search, or any nonlinear or
  global stability guarantee for the underlying field.
