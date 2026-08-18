# Implicit / rational dynamics boundary (v2-A)

This directory specifies implicit and rational dynamics discovery — the SINDy-PI
style method implemented in `crates/lawsynth-implicit`, which recovers relations
`f(x, ẋ) = 0` (including rational laws `ẋ = P(x)/Q(x)`) that explicit
`ẋ = Θ(x)Ξ` regression cannot express. It is a **boundary specification**.

## Method

Build an augmented candidate library `Θ(x, ẋ)` — monomials in the states, plus
those monomials multiplied by the target derivative ẋ. A rational law
`ẋ = P(x)/Q(x)` is equivalent to the implicit relation `Q(x)·ẋ − P(x) = 0`, i.e.
a sparse vector ξ with `Θ(x, ẋ) · ξ ≈ 0`. Discovery finds that sparse nonzero ξ.

## Requirements

1. **Nontrivial solution.** ξ = 0 trivially satisfies `Θξ = 0` and MUST be
   excluded. The reference uses the SINDy-PI alternating-LHS scheme: for each
   candidate column j, normalise its coefficient to 1 (move it to the LHS) and
   solve a sparse regression for the remainder; a left-hand side whose column is
   identically zero MUST be skipped.
2. **Selection.** Among candidate LHS choices, selection MUST be deterministic.
   Near-perfect fits (relative residual below a fixed tolerance) MUST be treated
   as tied so that machine-level residual noise does not decide the outcome;
   ties break toward fewer active terms, then toward a **derivative-bearing** LHS
   (so the chosen relation is affine in ẋ and reconstructs the canonical
   `Q(x)·ẋ = P(x)`), then toward the earlier library index.
3. **Explicit reconstruction.** When the relation is affine in ẋ, the
   implementation MUST reconstruct the explicit rational law `ẋ = P(x)/Q(x)` and
   report whether Q(x) is non-vanishing over the observed window (a pole makes the
   explicit form invalid there).
4. **Determinism.** Identical inputs MUST yield bit-identical output.

## Honest identifiability limit

Implicit discovery has a genuine identifiability limit: **at library degree ≥ 2
the augmented nullspace is multi-dimensional.** If `r(x, ẋ) = 0` is the true
relation, then `m(x)·r(x, ẋ) = 0` for any monomial `m(x)` also fits to machine
precision. Therefore an implementation is NOT guaranteed to return the minimal,
spurious-free relation at higher degree. The achievable, conforming guarantee is:

- the returned relation is a **valid** member of the nullspace (fits to
  tolerance) and, when affine in ẋ, reconstructs an explicit rational law that
  **reproduces the true derivative** across the observed window; and
- at the **minimal degree** (a one-dimensional nullspace) the clean, spurious-free
  law is recovered.

An implementation MUST NOT claim minimal-form recovery at higher degree; it MUST
document this limit (as the reference's degree-2 test does).
