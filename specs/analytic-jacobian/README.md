# Analytic Jacobian boundary (v2-A)

This directory specifies **deterministic analytic Jacobian codegen** — the
symbolic differentiation of a discovered vector field implemented in
`crates/lawsynth-jacobian`. It is a **boundary specification** in the house
style: it states what a conforming implementation MUST do, and — crucially —
what the emitted Jacobian is and is not allowed to claim.

## Motivation

A discovered dynamical law is a vector field: one right-hand side per state,
`dx_i/dt = f_i(x, …)`, each `f_i` an expression tree in the `lawsynth-expr` IR.
Downstream stages need the **Jacobian** `J[i][j] = ∂f_i/∂x_j`:

- **Stiff / implicit integration.** Implicit solvers (backward Euler, BDF,
  Rosenbrock) solve a linear system in `J` each step; a numerically
  finite-differenced `J` costs `n` extra field evaluations per column and injects
  differencing noise. An exact analytic `J` is cheaper and noise-free.
- **Local stability.** The eigenvalues of `J` evaluated at a fixed point classify
  that point (stable node, saddle, focus, …) — a direct, honest readout of the
  discovered law's local behaviour.
- **Sensitivity.** `J` is the linearization used by forward sensitivity and by
  continuation methods.

LawSynth is **deterministic and offline**. It differentiates **symbolically** —
exact rules on the expression IR — rather than probing a trained surrogate or
finite-differencing the field. Identical inputs MUST yield bit-identical output.

## What the Jacobian IS

The emitted `J` is the **exact partial-derivative matrix of the given
expressions**, under the standard convention that every symbol other than the
differentiation variable (parameters, constants, other states) is treated as
independent of it. It is:

1. **Exact, not estimated.** Each entry is produced by symbolic differentiation
   rules, not by a difference quotient. Where a derivative exists in closed real
   form, the entry is that derivative.
2. **A property of the expression, not of any data.** The Jacobian says nothing
   about how well `f_i` fits observations; it differentiates whatever field it is
   handed. Discovery quality is a separate, upstream concern.
3. **Simplified conservatively, not canonicalized.** Entries are reduced by local
   identities and constant folding for readability and cheaper evaluation. The
   result is *not* a canonical normal form: two mathematically equal entries may
   print differently. No correctness claim rests on simplification.

## Requirements

1. **Differentiation rules.** A conforming implementation MUST implement, for
   every node kind in the `lawsynth-expr` IR, the correct rule:

   | Node | Rule |
   |---|---|
   | `Constant(c)` | `0` |
   | `Symbol(s)` | `1` if `s` is the variable, else `0` |
   | `Negate(u)` | `−u′` |
   | `Exp(u)` | `exp(u)·u′` |
   | `Log(u)` | `u′/u` |
   | `Sin(u)` | `cos(u)·u′` |
   | `Cos(u)` | `−(sin(u)·u′)` |
   | `Add(l,r)` | `l′ + r′` |
   | `Subtract(l,r)` | `l′ − r′` |
   | `Multiply(l,r)` | `l′·r + l·r′` (product rule) |
   | `Divide(l,r)` | `(l′·r − l·r′)/r²` (quotient rule) |
   | `Power(f,g)` | see below |

   The chain rule MUST compose correctly through arbitrary nesting.

2. **Power rule, chosen for widest validity.** `Power` MUST be differentiated by
   the rule that is valid over the widest real domain:
   - **Constant exponent** `f^c`: `c·f^(c−1)·f′`. This MUST be preferred whenever
     the exponent is constant, because it never introduces `log(f)` and therefore
     stays correct for negative bases (e.g. `d/dx x² = 2x` at `x < 0`).
   - **Positive constant base** `b^g`, `b > 0`: `b^g·ln(b)·g′`.
   - **General** `f^g`: `f^g·(g′·ln(f) + g·f′/f)`. This is exact wherever `f > 0`;
     the implementation MUST document that its numerical validity is limited to
     positive bases.

3. **No silent zeros.** Differentiating a node that has no real closed-form
   derivative MUST return a **typed error**, never a wrong or silently-zero
   result. In the reference implementation the only such case is `b^g` with a
   **non-positive constant base and a variable exponent** (the generalized power
   rule would require `log` of a non-positive base). The error variant MUST also
   serve as a forward-compatible guard for any IR node kind added later.

4. **Determinism.** Row and column ordering MUST follow the caller-supplied
   `states` ordering exactly. Fields MUST be matched to states by identifier
   through a fixed-order scan — no hash-map iteration order may leak into the
   output. Any floating-point produced by simplification (constant folding,
   `ln(b)`, `c−1`) MUST be computed by a fixed evaluation order. Identical
   `(fields, states)` inputs MUST produce a **bit-identical** Jacobian: identical
   expression structure and, where floats appear, identical `f64` bit patterns.

5. **Assembly is square and total.** For `n` states the Jacobian MUST be `n × n`.
   The implementation MUST reject, with distinct typed errors:
   - a repeated identifier in `states` (ambiguous indexing),
   - two fields sharing a derivative target (ambiguous row),
   - a state with no corresponding field (unformable row).
   It MUST NOT fabricate, reorder, or drop a row to paper over these.

6. **Evaluation.** The implementation MUST provide numeric evaluation of the
   assembled Jacobian at a point, reusing the `lawsynth-expr` evaluator, and
   returning a dense `n × n` matrix. A symbol referenced by an entry but absent
   from the environment MUST surface as a typed evaluation error (unknown
   symbol), never a substituted default.

7. **Honest verification.** Any claim that the symbolic Jacobian is correct MUST
   be backed by a reproducible cross-check against an **independent** numerical
   derivative — the three-point / finite-difference estimators in
   `lawsynth-differentiate` — at several sampled points and to a stated tolerance.
   The reference test suite checks a 2×2 (Lotka–Volterra), a 3×3 (Lorenz), and a
   transcendental field against central differences and asserts agreement to
   `1e-6`.

## Public API

```text
differentiate(&Expr, &Identifier) -> Result<Expr, JacobianError>
analytic_jacobian(&[(Identifier, Expr)], &[Identifier]) -> Result<Jacobian, JacobianError>

Jacobian::dimension() -> usize
Jacobian::states() -> &[Identifier]
Jacobian::rows() -> &[Vec<Expr>]
Jacobian::entry(row, col) -> Option<&Expr>
Jacobian::evaluate(&Environment) -> Result<Vec<Vec<f64>>, JacobianError>
Jacobian::to_canonical_string() -> String
```

`differentiate` exposes the single-expression rule engine; `analytic_jacobian`
assembles the square matrix in `states` order. `Jacobian::to_canonical_string`
is a stable structural fingerprint for determinism checks. This crate delivers
the **differentiation and assembly library** only — wiring `J` into an implicit
integrator or an eigenvalue-based stability report is downstream and out of scope
here.

## Honest scope & limits

- **Supported functions are exactly the IR's:** `+ − × ÷`, `^`, unary negate,
  `exp`, `log`, `sin`, `cos`. There is no `tan`, `sqrt` (use `^0.5`), `abs`, or
  piecewise/conditional node, so no such derivative is emitted or claimed.
- **General `f^g` is valid where `f > 0`.** The emitted derivative is the correct
  real derivative on that domain; it is not extended to negative or complex bases.
- **Simplification is conservative.** It folds constants and applies the `+0`,
  `−0`, `*1`, `*0`, `^0`, `^1`, and double-negation identities. It does **not**
  factor, expand, collect like terms, or canonicalize, so it is not a decision
  procedure for expression equality.
- **The Jacobian differentiates the given field, nothing more.** It carries no
  discovery confidence, no fit residual, and no stability verdict; those belong to
  the stages that consume it.

## Non-goals

- No numerical/finite-difference Jacobian as a product (finite differences are
  used only to *verify* the symbolic result in tests).
- No higher derivatives (Hessian / tensor), no automatic differentiation graph,
  no complex-step differentiation.
- No integrator, eigensolver, or stability classifier — those consume `J` and
  live in their own crates with their own contracts.
