# Forward sensitivity boundary (v2-A)

This directory specifies **deterministic forward sensitivity analysis** —
integrating the variational (sensitivity) equations of a discovered model to
compute the trajectory sensitivities `S_j(t) = ∂x(t)/∂θ_j`, implemented in
`crates/lawsynth-sensitivity`. It is a **boundary specification** in the house
style: it states what a conforming implementation MUST do, and — crucially —
what the emitted sensitivities are and are not allowed to claim.

## Motivation

A discovered dynamical law is a vector field with discovered coefficients: one
right-hand side per state, `ẋ_i = f_i(x; θ)`, each `f_i` an expression tree in
the `lawsynth-expr` IR referencing both state symbols `x` and parameter symbols
`θ`. Once the coefficients `θ` are fixed, the natural next question is
**sensitivity**: how does a change in each discovered coefficient perturb the
forecast? The answer is the matrix of trajectory sensitivities

```text
S_j(t) = ∂x(t)/∂θ_j        (an n-vector for each parameter j),
```

which is the workhorse of three downstream stages:

- **Uncertainty propagation.** To first order a parameter covariance `Σ_θ` maps
  to a state covariance `S(t)·Σ_θ·S(t)ᵀ`; the sensitivities are the linear map.
- **Identifiability.** A coefficient whose sensitivity is (near) zero everywhere
  cannot be pinned down by the data — the forecast does not respond to it. The
  Fisher information `∫ S(t)ᵀ S(t) dt` is assembled directly from `S`.
- **Optimal experimental design.** Choosing when and what to measure to best
  constrain `θ` is an optimisation over functionals of `S(t)`.

LawSynth is **deterministic and offline**. The sensitivities are obtained by
**integrating the variational equations with analytic derivatives** — not by
finite-differencing the field inside the integrator, and not by probing a
trained surrogate. Identical inputs MUST yield bit-identical output.

## What the sensitivities ARE

The emitted `S_j(t)` is the **forward-integrated solution of the variational
system for the given field**, under the convention that the initial state is
independent of the parameters (`S_j(0) = 0`). It is:

1. **Analytic in its derivatives, numeric in its integration.** The Jacobian
   `J_x = ∂f/∂x` and each parameter partial `f_{θ_j} = ∂f/∂θ_j` are exact
   symbolic derivatives of the given expressions; only the time integration is
   numerical. No difference quotient of the field appears in the integrator.
2. **A property of the expression at fixed `θ`, not of any data.** The analysis
   differentiates and integrates whatever field and nominal coefficients it is
   handed; it carries no discovery confidence or fit residual. Those are
   separate, upstream concerns.
3. **Consistent between state and sensitivity.** The augmented system
   `(x, S_1, …, S_p)` is advanced by **one shared** fixed-step integrator, so the
   state and the sensitivities see identical stage points — the sensitivities are
   the sensitivities of the *reported* discrete trajectory, not of some other
   discretisation.
4. **Honest about absent parameters.** A coefficient that does not appear in any
   field has `f_{θ_j} ≡ 0`, so its sensitivity is **exactly zero** for all time.
   That zero is reported as-is, never suppressed and never fabricated into a
   spurious nonzero response.

## The variational system

For state `x ∈ Rⁿ`, parameters `θ ∈ Rᵖ`, and `ẋ = f(x; θ)`:

```text
ẋ   = f(x; θ)
Ṡ_j = J_x · S_j + f_{θ_j}        S_j(0) = 0        (j = 1 … p)
```

where `J_x = ∂f/∂x` is the `n × n` analytic Jacobian and `f_{θ_j} = ∂f/∂θ_j` is
the `n`-vector of field partials with respect to parameter `θ_j`. The initial
condition `S_j(0) = 0` encodes that the initial state does not depend on the
parameters. (Sensitivity to the initial condition, which would set `S(0) = I` for
those directions, is out of scope for this crate and is not emitted.)

## Requirements

1. **Analytic derivatives, reused.** A conforming implementation MUST obtain
   `J_x` from the analytic Jacobian of `crates/lawsynth-jacobian`, and each
   `f_{θ_j}` by symbolically differentiating the field with that crate's
   `differentiate(&Expr, &Identifier)` and evaluating the result. It MUST NOT
   finite-difference the field to form either quantity. The supported node kinds,
   the power-rule choice, and the "no silent zeros" guarantee are inherited from
   the analytic-Jacobian contract.

2. **One shared fixed-step integrator on the augmented system.** The augmented
   vector `y = [x, S_1, …, S_p]` (length `n·(1 + p)`) MUST be advanced by a
   single fixed-step fourth-order Runge–Kutta scheme, so that the state block and
   every sensitivity block are integrated with the **same** stages and step. The
   state MUST NOT be integrated separately from, or with a different step than,
   the sensitivities.

3. **Fixed layout and accumulation order.** The augmented vector MUST pack the
   state first, then one contiguous `n`-length block per parameter in
   `parameters` order. The matrix–vector product `J_x · S_j` and every stage
   combination MUST be accumulated in a fixed index order, so no reordering can
   perturb the low bits of the result.

4. **Determinism.** Environment construction, field/Jacobian/partial evaluation,
   the RK4 stages, and the time grid MUST be deterministic. The sample times MUST
   be computed as `t0 + k·dt` from the step index `k` (not accumulated), so
   rounding does not drift. Identical `(fields, states, parameters, initial,
   parameter_values, SensitivityConfig)` inputs MUST produce a **bit-identical**
   trajectory: identical times, state values, and sensitivity values to their
   `f64` bit patterns.

5. **Totality and typed errors.** The implementation MUST reject, with distinct
   typed errors and never a fabricated or silently dropped result:
   - an empty state space;
   - an initial vector whose length differs from `states`, or a parameter-value
     vector whose length differs from `parameters`;
   - a non-finite initial or parameter value;
   - a repeated parameter, or an identifier declared as both a state and a
     parameter;
   - a field symbol that is neither a declared state nor a declared parameter
     (the "unknown parameter symbol" case — there is no value to bind);
   - an invalid config (`dt ≤ 0`, non-finite `t0`, zero `steps`);
   - any structural or numeric failure surfaced by the analytic Jacobian
     (duplicate/missing field, undifferentiable node, evaluation error).
   A parameter that is well-formed but simply **absent from the fields** is NOT
   an error: its sensitivity is exactly zero and MUST be reported as such.

6. **Query surface.** The produced trajectory MUST expose the shared time grid,
   the state trajectory `x(t)`, the per-parameter sensitivity blocks `S_j(t)`,
   and a helper returning the scalar `∂x_i(t)/∂θ_j` for any `(state, parameter,
   step)` index, with out-of-range indices returning `None` rather than panicking.
   It MUST also expose a canonical-string fingerprint encoding every float by its
   bit pattern, for determinism checks.

7. **Honest verification.** Any claim that the integrated sensitivities are
   correct MUST be backed by reproducible cross-checks: (a) an **analytic check**
   against a closed-form sensitivity — the linear scalar law `ẋ = −θ·x` has
   `∂x/∂θ = −t·x₀·e^{−θ t}` — to a tight tolerance; and (b) a **finite-difference
   cross-check** for nonlinear models, comparing the integrated `∂x(t)/∂θ_j`
   against a central difference `[x(t; θ + h e_j) − x(t; θ − h e_j)] / (2h)`
   obtained by re-simulating the **state only** at perturbed parameters, for
   every parameter at several times and to a stated tolerance. The reference
   suite checks the linear law to `1e-6`, and a logistic (1 state, 2 parameters)
   and a 2D Lotka–Volterra (2 states, 4 parameters) against central differences
   (`h = 1e-4`) to `1e-6`.

## Public API

```text
forward_sensitivities(
    &[(Identifier, Expr)],   // fields  ẋ_i = f_i(x; θ)
    &[Identifier],           // states  (output ordering)
    &[Identifier],           // parameters
    &[f64],                  // initial state x(t0)
    &[f64],                  // nominal parameter values θ
    &SensitivityConfig,
) -> Result<SensitivityTrajectory, SensitivityError>

SensitivityConfig::new(t0, dt, steps) -> Self     // + with_* builder setters
SensitivityTrajectory::states() -> &[Identifier]
SensitivityTrajectory::parameters() -> &[Identifier]
SensitivityTrajectory::times() -> &[f64]
SensitivityTrajectory::state_at(step) -> Option<&[f64]>
SensitivityTrajectory::sensitivity_at(parameter, step) -> Option<&[f64]>
SensitivityTrajectory::partial(state, parameter, step) -> Option<f64>   // ∂x_i(t)/∂θ_j
SensitivityTrajectory::to_canonical_string() -> String                  // determinism fingerprint
```

`SensitivityConfig` carries the integration start time, the fixed step, and the
step count; the trajectory holds `steps + 1` samples, the first at `t0`. This
crate delivers the **variational-integration library** only; wiring the
sensitivities into an uncertainty band, a Fisher-information / identifiability
report, or an experimental-design objective is downstream and out of scope here.

## Honest scope & limits

- **Fixed-step accuracy.** The sensitivities inherit the integrator's global
  `O(dt⁴)` error. The variational equations are often **stiffer** than the state
  equations (the sensitivity can grow while the state settles), so a step that
  resolves the state may under-resolve `S`; there is no adaptive step or error
  control here. Tighten `dt` when the finite-difference cross-check widens.
- **Parameters must appear symbolically in the fields.** Sensitivity is computed
  by differentiating the field expressions with respect to the parameter symbol.
  A coefficient baked into a constant, or entering only through data
  preprocessing, has no symbolic handle and its sensitivity cannot be recovered
  here (it reads as exactly zero, which is the correct answer *for the given
  field*).
- **Supported functions are exactly the IR's** (`+ − × ÷`, `^`, unary negate,
  `exp`, `log`, `sin`, `cos`), inherited from the analytic Jacobian and the
  differentiator; no derivative is emitted or claimed for anything outside it,
  and the general `f^g` partial is valid only where the base is positive.
- **First-order, local in `θ`.** `S_j(t)` is the exact first derivative at the
  nominal `θ`; using it to extrapolate a finite parameter change is a linear
  approximation whose error grows with the change and with model nonlinearity.
- **Long horizons accumulate error.** Like any forward integration, both the
  state and the sensitivities drift over long time spans; a growing sensitivity
  compounds this. The trajectory is honest about the discretisation it reports,
  not about the true continuous flow beyond the integrator's accuracy.
- **The sensitivities describe the given field, nothing more.** They carry no
  discovery confidence, no fit residual, and no identifiability verdict; those
  belong to the stages that consume them.

## Non-goals

- No adjoint / reverse sensitivity, no second-order sensitivities, and no
  sensitivity to the initial condition (`S(0) = I`).
- No adaptive step size, error control, or stiff/implicit integrator — a single
  fixed-step RK4 only.
- No uncertainty band, Fisher-information matrix, identifiability ranking, or
  experimental-design objective — those consume `S(t)` and live in their own
  crates with their own contracts.
- No finite-difference sensitivity as a product (finite differences are used only
  to *verify* the analytic result in tests).
