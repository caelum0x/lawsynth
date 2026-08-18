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
