# Controlled discovery (SINDYc) boundary (v2-A)

This directory specifies **controlled sparse dynamics discovery** — the SINDYc
formulation implemented in `crates/lawsynth-control`. It is a **boundary
specification** in the house style: it states what a conforming implementation
MUST do, and — crucially — how a measured control input is and is not allowed to
be treated.

## Motivation

Ordinary SINDy fits autonomous dynamics `Ẋ = Θ(X)Ξ`: it estimates a state
derivative and regresses it onto a candidate library built over the states.
Many real systems are **forced**: `ẋ = f(x, u)`, where `u(t)` is an exogenous,
measured control input (a drive, an actuator command, an environmental signal).
SINDYc handles forcing by **augmenting the candidate library** to `Θ(X, U)` over
both states and controls, so control-only terms (`u`, `u²`) and state×control
cross terms (`x·u`, `y·u`) can enter the regression alongside the ordinary state
terms. Each *state* derivative is then regressed onto the augmented library.

The control is data, not a variable to be solved for. That asymmetry is the
whole point of the boundary below.

## What a control IS

A control is a **measured exogenous input**, and this fixes three hard rules:

1. **Exogenous input, on the same grid.** A control column is evaluated at its
   measured values and enters the augmented library `Θ(x, u)` exactly like a
   state column enters `Θ(x)`. It MUST be sampled on the same time grid as the
   states; a conforming implementation does not resample or align signals.
2. **Never differentiated.** A conforming implementation MUST NOT form any
   target from a derivative of a control. There is no `u̇`. Only the state
   columns are handed to the derivative estimator. The reference implementation
   enforces this *structurally* by differentiating a state-only sub-dataset, and
   proves it with a test that perturbs the control and confirms every
   state-derivative target is byte-identical.
3. **Never predicted.** The discovered model contains **exactly one equation per
   state and none for any control**. Controls appear only *inside* library
   terms, never on the left-hand side. A conforming `ControlledModel` MUST expose
   the state/control designation so a caller can verify this.

## Requirements

1. **Explicit state/control designation.** The caller MUST designate which
   dataset columns are states and which are controls (a `ControlSpec`). Both
   groups MUST be non-empty — a run with no controls is ordinary SINDy and
   belongs to a different entry point. An identifier MUST NOT appear in both
   groups, MUST NOT repeat within a group, and every designated identifier MUST
   exist in the dataset. Dataset columns named by neither group are ignored.
2. **Augmented library over the combined variables.** The candidate library MUST
   be built over the combined variable set `[states.., controls..]` using the
   shared feature machinery (`crates/lawsynth-features`), NOT re-implemented, so
   that control and state×control terms arise from the same deterministic
   polynomial expansion as the state terms.
3. **Numerically differentiated state targets.** Targets `ẋ_i` MUST come from the
   deterministic derivative estimators in `crates/lawsynth-differentiate` applied
   to the **state** columns only. The usual strong-form noise caveats apply: the
   estimator amplifies observation noise, so the controlled fit is only as clean
   as the state derivatives.
4. **Determinism.** The library variable order is fixed by the spec
   (`[states.., controls..]`, as given, documented), the library term order is
   fixed by the feature crate, the derivative estimator is deterministic, and the
   sparse solve is deterministic. Identical `(Dataset, ControlSpec, ControlConfig)`
   inputs MUST produce **bit-identical** `ControlledModel` output. The reference
   implementation verifies this bit-for-bit.
5. **Honest reporting.** Each state equation MUST carry its sparse coefficient row
   aligned with human-readable augmented-library term labels, plus a residual
   signal, so a caller can read off the discovered right-hand side and judge fit
   quality.

## Honest scope & limits

- **Same-grid, measured controls only.** If the control is not measured, or is
  sampled on a different grid than the states, this boundary does not apply;
  align/resample upstream.
- **Persistent excitation is required.** The control must be *persistently
  exciting*. If `u(t)` is constant or varies too little, its library columns are
  (near-)collinear with the constant term or with each other and the control
  coefficients are **unidentifiable** — the solve may attribute the control's
  effect to a state or constant term, split it arbitrarily, or drop it. A
  conforming implementation MUST NOT silently present an unidentifiable control
  coefficient as a recovered law. The reference implementation demonstrates both
  sides: a persistently-exciting multi-sine recovers the control gain to ~1e-5,
  while a constant control makes the control term add no identifiable information
  (a states-only fit is essentially as good).
- **Strong-form noise sensitivity.** Because targets are differentiated from the
  data, heavy observation noise degrades the fit exactly as it does for ordinary
  SINDy; the weak-form boundary (`specs/weak-form/`) is the noise-robust
  companion and could be extended to the controlled case as future work.

## Public API

```text
discover_controlled(&Dataset, &ControlSpec, &ControlConfig)
    -> Result<ControlledModel, ControlError>
```

`ControlSpec` designates ordered states and controls. `ControlConfig` reuses the
feature, derivative, and sparse configuration types of the crates it drives.
`ControlledModel` returns one `StateEquation` per state — each a sparse
coefficient row over the shared augmented library — together with the library
term labels and the state/control designation. There is deliberately no equation
for any control.

## Non-goals

- No control *design* or optimal control: this crate discovers `f(x, u)`, it does
  not choose `u`.
- No resampling, alignment, or delay embedding of controls; inputs are assumed
  measured on the state grid.
- No weak/integral controlled form and no claim of noise robustness beyond what
  the strong-form derivative estimator provides.
- No claim of proof: a discovered controlled law is a sparse, quantified fit,
  subject to the persistent-excitation and noise limits above.

## Forward simulation & validation

Discovery fits `ẋ = Θ(x, u) Ξ` but does not roll it forward. The `simulate`
module closes the loop `discover → simulate → score`: it integrates a
`ControlledModel` under a supplied control and scores the rollout against
held-out data.

### Simulator contract

```text
simulate_controlled(&ControlledModel, initial: &[f64], &ControlSignal, &SimConfig)
    -> Result<Trajectory, ControlError>
validate_controlled(&ControlledModel, &Dataset, &ControlSpec, &ValidationConfig)
    -> Result<ControlScore, ControlError>
```

- **Fixed-step RK4.** Integration uses classical fourth-order Runge-Kutta with a
  fixed step `SimConfig { t0, dt, steps }`, producing `steps + 1` samples. There
  is no adaptive stepping and no stiffness handling; a stiff system needs a small
  `dt`.
- **Structural term evaluation (no string parsing).** The model carries the
  structured augmented library `ControlledModel::library`. At every RK4 stage the
  current `(state, control)` values are bound into an expression `Environment`
  and each library term's **expression tree** is evaluated with
  `lawsynth_expr::evaluate`. Term `k`'s value is multiplied by
  `equation.coefficients[k]` and summed: `ẋ_i = Σ_k coef[i][k] · term_k(x, u)`.
  The human-readable `library_terms` labels are never re-parsed — the rolled-out
  right-hand side is exactly the fitted model.
- **Control at RK4 stages.** RK4 samples the right-hand side at `t`, `t + dt/2`,
  and `t + dt`. A `ControlSignal` supplies one value per control channel at each
  of those times. The channels MUST equal the model's controls, in order.
- **Control interpolation rule.** A control given as a closure `t ↦ [u(t)]` is
  evaluated exactly at the stage times. A control given as sampled columns on a
  strictly ascending time axis is **linearly interpolated** between adjacent
  samples and **clamped (held constant)** outside the sampled range. Linear
  interpolation is continuous and deterministic, so RK4 mid-step values are
  well-defined and reproducible.

### Determinism

Every operation is a pure `f64` computation over deterministically ordered data
(library terms fixed at discovery, states/controls in model order, `BTreeMap`
environments). Identical `(model, initial, control, config)` inputs yield a
**bit-identical** `Trajectory` (compared via `f64::to_bits`), and identical
validation inputs yield a bit-identical `ControlScore`. Both are covered by
tests.

### R² and RMSE definition

`validate_controlled` simulates from the dataset's own initial condition under
the dataset's own control columns, then compares the simulated state columns to
the observed ones. For each state it reports, via `lawsynth_score::fit_statistics`:

- **R²** `= 1 − SS_res / SS_tot`, where `SS_res = Σ (observed − simulated)²` and
  `SS_tot = Σ (observed − mean(observed))²`.
- **RMSE** `= sqrt(SS_res / N)`.

The **aggregate** figures pool every state's samples (in model-state order) into
one observed/predicted pair and score that, weighting each sample equally.
`ValidationConfig { substeps }` optionally takes `substeps` RK4 steps between
consecutive observed samples (comparing only at the sample points) so the
integrator's own step error can be shrunk below the model error; the default is
one step per interval.

### Honest limits

- **Open-loop rollout.** The rollout is open-loop: model-coefficient error
  accumulates over the horizon, so predictive R² degrades as the horizon grows.
  On the reference forced oscillator the discovered model reproduces the full
  20 s trajectory with aggregate R² ≈ 1.0 (RMSE ≈ 4e-5); a longer or less
  accurate model would drift more.
- **Compatible control grid.** A sampled control must cover the simulated horizon
  on a compatible grid. Values requested outside the samples are **clamped**, not
  extrapolated, and validation assumes an (approximately) regular grid so every
  `substeps`-th simulated sample lands on an observed time.
- **Fixed step only.** No adaptive stepping and no stiffness handling — accuracy
  is entirely governed by `dt`.
- **Score discriminates model quality.** Zeroing the discovered control
  coefficient (an unmodelled forcing) collapses the reference aggregate R² from
  ≈ 1.0 to ≈ 0.11 and inflates RMSE ~14×, demonstrating that the predictive
  score genuinely separates good models from bad ones.
