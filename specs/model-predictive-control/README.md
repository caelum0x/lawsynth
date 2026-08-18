# Model-predictive-control boundary (v2-A)

This directory specifies **deterministic successive-linearization model-
predictive control (MPC)** — given a discovered nonlinear model `ẋ = f(x, u)`,
driving the state to a setpoint by repeatedly linearizing about the current
operating point, designing a local LQR feedback, applying the first control
move, and advancing the true nonlinear plant one step. It is implemented in
`crates/lawsynth-mpc`. It is a **boundary specification** in the house style: it
states what a conforming implementation MUST do, and — crucially — what the
resulting controller is and is **not** allowed to claim.

## Motivation

A discovered dynamical law with actuation is a controlled vector field
`ẋ = f(x, u)`, where the state fields are symbolic `Expr` trees over the state
symbols and one or more control symbols. The engineering question is: choose a
control policy `u(x)` that drives the state to a target `x_ref` and holds it
there. The classical linear answer — LQR (`crates/lawsynth-feedback`) — applies
only to a linear pair `(A, B)`; the field here is nonlinear.

**Successive-linearization MPC** bridges the two. At each control step, with the
current state `x`, it linearizes the nonlinear field about the operating point
`(x, u_ref)`,

```
A = ∂f/∂x |(x, u_ref)      B = ∂f/∂u |(x, u_ref),
```

designs an infinite-horizon LQR gain `K` for that local `(A, B)`, applies the
first move `u = u_ref − K (x − x_ref)` (saturated to actuator limits), advances
the true nonlinear plant one fixed step, and re-linearizes at the new state.
This is the receding-horizon idea with the horizon subproblem solved by a local
LQR rather than a full trajectory optimization — equivalently, a **gain-
scheduled LQR** whose schedule is generated on the fly by the analytic Jacobian.

`A` is the same object `crates/lawsynth-jacobian` produces (analytic, exact
symbolic differentiation); `B` is the analogous partial `∂f/∂u`, one column per
control symbol. The local design reuses the deterministic Kleinman LQR of
`crates/lawsynth-feedback` verbatim. LawSynth is **deterministic and offline**,
so identical inputs MUST yield a bit-identical trajectory.

## What the controller IS

The returned trajectory is a **property of the supplied field, the weights, the
initial condition, and the step size** — not of any data or of a global
guarantee. The contract is:

1. **A closed loop of local designs, not a global optimum.** At each step the
   applied gain is the exact LQR gain of the *local* linearization. The
   controller makes no claim of global optimality, of a horizon cost minimum, or
   of the constrained optimum when saturation is active.
2. **The plant advanced is the true nonlinear field.** Linearization is used
   only to compute the feedback gain; the state is advanced by RK4 on `f(x, u)`
   itself, so the recorded trajectory is the genuine nonlinear closed-loop
   response, not a linear prediction.
3. **Exactness where the model is linear.** If `f` is affine in `x` and `u`, the
   linearization is exact and constant, so every step designs the *same* gain,
   bit-identical to a direct `lqr(A, B, Q, R)` solve, and the closed loop matches
   an LQR-controlled linear rollout. This is the conformance anchor.
4. **Saturation by clamping, honestly labelled.** When `[u_min, u_max]` are
   given, each applied move is clamped element-wise. Clamping is *not* the
   constraint-optimal projection a QP-MPC would compute; it is a simple, honest
   actuator model that keeps every applied `u` within bounds.
5. **A property of the expression, not of any fit.** The controller carries no
   discovery confidence or fit residual. Those are separate, upstream concerns.

## Requirements

1. **Relinearize every step from the analytic model.** At each control step an
   implementation MUST form `A = ∂f/∂x` from the analytic Jacobian
   (`analytic_jacobian`) and `B = ∂f/∂u` by symbolically differentiating each
   field with respect to each control symbol (`differentiate`), and MUST
   evaluate both at the current `(x, u_ref)`. The symbolic differentiation MAY be
   done once at construction and only *evaluated* per step; it MUST NOT be
   replaced by a finite-difference approximation.

2. **Design the local gain with the shared LQR.** The per-step gain MUST come
   from `lawsynth-feedback::lqr` applied to the local `(A, B, Q, R)`, so the
   whole pipeline shares one audited Riccati/Kleinman solve. A design failure
   (e.g. `R` not positive definite, an unstabilizable linearization, non-
   convergence) MUST surface as a distinct typed error, never a fabricated gain.

3. **Apply the first move with the stated law.** The applied control MUST be
   `u = u_ref − K (x − x_ref)`, then clamped element-wise to `[u_min, u_max]`
   when those bounds are supplied. `u_ref` defaults to zero.

4. **Advance the true nonlinear plant by fixed-step RK4.** The state MUST be
   advanced one step of size `dt` by the classical four-stage Runge–Kutta method
   applied to the nonlinear field `f(x, u)` with the applied `u` **held
   constant** across the four stages. No linear surrogate may be substituted for
   the plant step.

5. **Validate the problem at the boundary.** Dimensions (`x₀`, `x_ref` length
   `n`; `u_ref`, saturation bounds length `m`; `Q` shape `n×n`; `R` shape
   `m×m`), finiteness of all configuration values, a strictly positive finite
   `dt`, a non-zero horizon, a non-empty state and control set, every state
   having a field, and `u_min ≤ u_max` per channel MUST each be checked, and a
   violation MUST surface as a distinct typed error before any integration runs.

6. **Reuse, do not re-derive.** Linearization, LQR, and evaluation MUST reuse the
   `lawsynth-jacobian`, `lawsynth-feedback`, and `lawsynth-expr` implementations.
   No second Jacobian, Riccati solver, or expression evaluator may be hand-rolled
   here.

7. **Determinism.** The symbolic Jacobian and control partials, their numeric
   evaluation, the Kleinman LQR iteration, the saturation clamp, and the RK4
   advance MUST all be deterministic — no RNG, no clock, fixed loop order.
   Identical inputs MUST produce a **bit-identical** trajectory: identical state
   and control sequences to `f64` bit patterns.

## Public API

```text
mpc_control(
    fields:   &[(Identifier, Expr)],   // ẋ_i = f_i(x, u), matched to states
    states:   &[Identifier],           // state ordering, length n
    controls: &[Identifier],           // control ordering, length m
    config:   &MpcConfig,
) -> Result<MpcTrajectory, MpcError>

MpcConfig {
    initial_state, setpoint,          // x₀, x_ref (length n)
    control_reference,                // u_ref (length m, default 0)
    state_weight, control_weight,     // Q (n×n), R (m×m)
    dt, steps,                        // fixed step, horizon length
    control_min, control_max,         // optional saturation (length m)
}
MpcConfig::new(x0, x_ref, Q, R, dt, steps)          // u_ref = 0, unsaturated
        ::with_control_reference(u_ref) / ::with_saturation(u_min, u_max)

MpcTrajectory::states() / controls() / gains() / times()
             ::final_state() / error_norm(k, x_ref) / final_error_norm(x_ref)
             ::bit_fingerprint()          // raw f64 bits, for determinism checks
```

`Matrix` is re-exported from `lawsynth-koopman` (through `lawsynth-feedback`) so a
caller builds `Q`/`R` and reads per-step gains without a separate dependency.
This crate delivers the **controller** only; discovering the field, choosing the
weights, and reporting results are upstream/downstream and out of scope here.

## Honest scope & limits

- **Successive-linearization LQR-MPC, not QP-MPC.** There is no horizon cost
  optimization and no constrained quadratic program. Each step solves an
  *infinite-horizon* LQR for the local linearization and applies its first move.
  Saturation is handled by **clamping**, which is *not* constraint-optimal — a
  true constrained MPC would re-optimize subject to the bounds. When the actuator
  saturates, the clamped move is generally suboptimal.
- **Local stability only; no feasibility or recursive-stability guarantee.** The
  closed-loop guarantee is exactly LQR's local one, applied pointwise along the
  trajectory. There is no proof of convergence for arbitrary initial conditions,
  no region-of-attraction estimate, and no recursive-feasibility / terminal-set
  argument of the kind constrained MPC provides. A run may fail to reach the
  setpoint (or diverge) for a strongly nonlinear plant, a distant start, or an
  actuator too weak or too saturated.
- **A stabilizable linearization is required every step.** Each step needs
  `lqr(A, B, Q, R)` to succeed: `(A, B)` must be stabilizable and `R ≻ 0`. If a
  linearization along the path is not stabilizable (e.g. the local `B` loses
  rank, an uncontrollable unstable mode), the step fails and the error is
  propagated — the controller does not silently substitute a fallback.
- **Fixed-step accuracy.** The plant is advanced by fixed-step RK4; there is no
  adaptive step control or error estimate. Too large a `dt` degrades the plant
  simulation and can destabilize the closed loop irrespective of the gain. `dt`
  is the discretization of the *plant simulation*, not a control-horizon length.
- **Setpoint consistency is the caller's responsibility.** The law
  `u = u_ref − K (x − x_ref)` regulates to `x_ref` only if `(x_ref, u_ref)` is
  (near) an equilibrium of `f` — i.e. `f(x_ref, u_ref) ≈ 0`. For a setpoint that
  is not an equilibrium the state settles to a nearby offset, not exactly
  `x_ref`; the crate does not solve for a feedforward `u_ref`.
- **The gain is only as local as the linearization.** As in
  `specs/control-design`, `A` is a Jacobian at a point and the design is local;
  nothing here certifies behaviour away from the current operating point.
- **Continuous-time model, single fixed weighting.** `f` is a continuous-time
  field and `Q`, `R` are fixed across the run (no time-varying or terminal
  weighting). Discrete-time / DARE design is out of scope.

## Non-goals

- No constrained QP-MPC, no finite-horizon trajectory optimization, no terminal
  cost / terminal constraint set, no move-blocking or input-rate constraints.
- No output feedback, observer, or state estimation (full state is assumed
  measured); the separation principle is out of scope.
- No robustness / tube MPC, no disturbance model, no offset-free tracking via
  integral augmentation or feedforward equilibrium solve.
- No adaptive or multiple-shooting integration, no region-of-attraction or
  Lyapunov certificate for the nonlinear closed loop.
