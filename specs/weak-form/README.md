# Weak / integral-form discovery boundary (v2-A)

This directory specifies weak/integral-form system identification — the
noise-robust formulation implemented in `crates/lawsynth-weakform`. It is a
**boundary specification** in the house style: it states what a conforming
implementation MUST do.

## Motivation

Strong-form SINDy fits `Ẋ = Θ(X)Ξ` using derivatives Ẋ *estimated from the data*.
Numerical differentiation amplifies observation noise, so the strong form
degrades sharply as noise grows. The weak form multiplies the ODE by a smooth,
compactly-supported test function φ and integrates by parts, moving the
derivative onto the analytic φ:

```
∫ φ̇_k · x dt  =  ∫ φ_k · Θ(x) dt · Ξ
```

No derivative of the noisy data is ever taken; only integrals against φ and φ̇.

## Requirements

1. **No differentiation of observed data.** A conforming weak-form discovery MUST
   form its targets and library rows from integrals of the data against test
   functions (and their analytic derivatives), never from a finite-difference /
   smoothing derivative of the observations.
2. **Compactly-supported test functions.** Each φ_k MUST vanish (with its
   derivative) at the boundary of its subdomain, so integration by parts carries
   no boundary term. The standard bump `φ(t) = (1 − ((t−c)/r)²)^p`, `p ≥ 2`, is
   sufficient; the implementation MUST document its family, order, and radius.
3. **Determinism.** Test-function placement (centers/radii), quadrature, and the
   sparse solve MUST be deterministic — centers derived from the usable time
   window by a fixed rule, quadrature a fixed scheme, any sampling seeded from
   content. Identical inputs MUST yield bit-identical output.
4. **Honest scope.** The weak form estimates the SAME law family as the strong
   form (the sparse coefficient matrix Ξ over the same candidate library); it is
   a more noise-robust estimator of the same object, not a different model class.
   The implementation MUST report the number of test functions used and a
   conditioning/health signal so a caller can judge whether the placed test
   functions excite the candidate columns independently.
5. **Noise-robustness claim.** Any claim that the weak form beats the strong form
   under noise MUST be backed by a reproducible comparison on the same noisy data
   (the reference implementation's `noise_robustness` test does this).

## Non-goals

Full PDE / spatiotemporal weak forms and adaptive test-function selection are out
of scope for this boundary; they may be added as extensions with their own
contracts.
