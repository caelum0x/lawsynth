# E-graph simplification boundary (v2-A)

This directory specifies **deterministic, value-preserving expression
simplification** — the equality-saturation simplifier implemented in
`crates/lawsynth-egraph`. It is a **boundary specification** in the house style:
it states what a conforming implementation MUST do, and — crucially — what a
simplified expression is and is not allowed to claim.

## Motivation

A discovered law is a set of expression trees in the `lawsynth-expr` IR, one
right-hand side per field. Raw discovery output is rarely tidy: it carries
`x + 0`, `x * 1`, `x^1`, un-folded numeric constants, `log(exp(x))` round-trips,
and un-factored sums like `a*b + a*c`. Before a law is displayed, exported, or
handed to a downstream stage, those artefacts should be reduced to a cheaper,
more readable equivalent **without changing what the law computes**.

LawSynth is **deterministic and offline**. It simplifies by applying a fixed set
of local algebraic identities and constant folding to a bounded fixpoint, then
extracting the cost-minimal form. It does **not** call a CAS, search the web, or
consult a trained model. Identical inputs MUST yield bit-identical output.

## What a simplified expression IS

The result of `simplify_expr` is a **value-preserving reduction** of its input:

1. **Value-preserving on the input's domain.** For every assignment of the free
   symbols at which the *original* expression evaluates to a finite number, the
   simplified expression evaluates to the **same** number (to within
   floating-point rounding). No rewrite changes the function's value where the
   original is defined.
2. **A property of the expression, not of any data.** Simplification says nothing
   about how well the law fits observations; it reduces whatever tree it is
   handed. Discovery quality is a separate, upstream concern.
3. **Cost-minimal among the forms it reaches, not globally canonical.**
   Extraction picks the member with the fewest scalar AST nodes (ties broken by
   canonical representation) from the bounded set of forms saturation produces.
   Two mathematically equal laws MAY still simplify to different trees.

## The rewrite rules

Every rule below is a value-preserving identity. Rules marked **(domain-widening)**
produce a form defined on a *wider* domain than the original (for example
`x / x -> 1` also has a value at `x = 0`); they remain value-preserving on the
original's domain, and the widening is intentional and documented — never a
change of value where the original was already defined.

| Rule | Identity | Notes |
|---|---|---|
| Additive identity | `x+0 -> x`, `0+x -> x` | |
| Subtractive identity | `x-0 -> x` | |
| Negation of difference | `0-x -> -x` | |
| Self-difference | `x-x -> 0` | domain-widening |
| Multiplicative identity | `x*1 -> x`, `1*x -> x` | |
| Annihilation | `x*0 -> 0`, `0*x -> 0` | domain-widening |
| Division identity | `x/1 -> x` | |
| Zero numerator | `0/x -> 0` | domain-widening, guarded so `0/0` is left intact |
| Self-quotient | `x/x -> 1` | domain-widening, guarded so `0/0` is left intact |
| Power of one | `x^1 -> x` | |
| Power of zero | `x^0 -> 1` | matches `powf`, including `0^0 = 1` |
| Power product | `x^a · x^b -> x^(a+b)` | exact for base `> 0` |
| Power tower | `(x^a)^b -> x^(a·b)` | exact for base `> 0` |
| Exp product | `exp(a)·exp(b) -> exp(a+b)` | exact for all reals |
| Log/exp inverse | `log(exp(x)) -> x` | exact for all reals (`exp(x) > 0`) |
| Exp/log inverse | `exp(log(x)) -> x` | exact on `x > 0`, the domain of `log` |
| Double negation | `-(-x) -> x` | |
| Sine parity | `sin(-x) -> -sin(x)` | sine is odd |
| Cosine parity | `cos(-x) -> cos(x)` | cosine is even |
| Pythagorean | `sin(u)^2 + cos(u)^2 -> 1` | exact for all reals |
| Distributive factoring | `a*b + a*c -> a*(b+c)` | cost-reducing direction of distributivity |
| Constant folding | `2*3 -> 6`, `1+1 -> 2`, … | folds only to finite results |
| Canonical order | `y+x -> x+y`, `y*x -> x*y` | total, deterministic operand order |

Constant folding never produces a non-finite value: `1/0`, an overflowing
`exp`, and `(-2)^0.5` are left un-folded rather than collapsed to `inf`/`NaN`,
matching the evaluator's own refusal to return non-finite results.

## Requirements

1. **Soundness (the central contract).** Every rewrite MUST preserve the
   expression's value at every point where the original is defined. A conforming
   implementation MUST NOT include a rule that changes the value on the
   original's domain. Domain-widening rules are permitted only when they agree
   with the original wherever it had a value.

2. **Constant folding.** Numeric sub-expressions MUST fold to a single constant
   whenever the result is finite, and MUST be left intact otherwise.

3. **Determinism.** Simplification MUST be a pure function of the input
   expression and configuration. No hash-map iteration order may leak into the
   result: commutative operands are ordered by a total canonical-string
   comparison, and candidate extraction breaks cost ties by canonical
   representation. Identical inputs MUST produce a **bit-identical** result —
   identical structure and, where floats appear, identical `f64` bit patterns.

4. **Bounded and terminating.** Saturation MUST run under an explicit pass bound
   (`RewriteConfig::max_passes`, rejected when zero) and an explicit node ceiling
   (`RewriteLimits`, default 256). Each normalization sweep that changes the
   expression strictly reduces node count or re-sorts commutative operands into
   an idempotent canonical order, so a fixpoint is always reached; the bounds
   guarantee termination even on adversarial input, surfacing over-large inputs
   as a typed `LimitExceeded` error rather than looping.

5. **Extraction picks the cost-minimal member.** The returned expression MUST be
   the lowest-cost form (scalar node count) among the forms saturation reaches,
   with ties broken deterministically. Simplification MUST NOT return a form more
   expensive than the input.

6. **Honest verification.** Any claim that a rule is sound MUST be backed by a
   reproducible numerical cross-check: evaluate the original and the simplified
   form via `lawsynth-expr` at many sampled points and assert agreement wherever
   the original is defined. The reference suite samples 256 pseudo-random points
   per expression across a battery covering every rule and asserts agreement to a
   `1e-12` mixed absolute/relative tolerance — several thousand checks in total.
   A single unsound rule fails this test.

## Public API

```text
simplify_expr(&Expr, &RewriteConfig) -> Result<Expr, RewriteError>
simplify_law(&[(Identifier, Expr)], &RewriteConfig) -> Result<Vec<(Identifier, Expr)>, RewriteError>

normalize(Expr) -> Expr        // one deterministic reduction to a local fixpoint
expression_cost(&Expr) -> usize
extract_lowest_cost(&[Expr]) -> Option<Expr>
EquivalenceGraph::add / classes / equivalent
```

`simplify_expr` is the single-expression entry point; `simplify_law` cleans up a
whole law's fields in caller order for display or export. `normalize` exposes the
underlying reduction, and `EquivalenceGraph` groups locally equivalent
expressions by their normalized form. This crate delivers the **simplification
library** only — deciding *when* to simplify a law for a report or a bundle is
downstream and out of scope here.

## Honest scope & limits

- **Not a decision procedure.** Simplification is best-effort within the pass and
  node bounds. It does **not** guarantee a canonical or globally minimal form:
  two equal expressions may reduce to different trees, and `equivalent` can return
  `false` for expressions this rule set cannot bring to the same normal form. A
  `true` result is sound (the forms are genuinely equal); a `false` result is not
  a proof of inequality.
- **Supported functions are exactly the IR's:** `+ − × ÷`, `^`, unary negate,
  `exp`, `log`, `sin`, `cos`. No `tan`, `sqrt` (use `^0.5`), `abs`, or
  conditional node exists, so none is simplified.
- **Domain-widening is intentional, not accidental.** Rules like `x/x -> 1`,
  `x*0 -> 0`, and `x-x -> 0` define a value where the original was undefined
  (e.g. at `x = 0` or where a sub-expression fails). This is documented per rule
  and is never a change of value where the original already had one.
- **Floating-point, not exact algebra.** Constant folding and transcendental
  round-trips use `f64`; agreement is to rounding, and folding refuses any
  non-finite result rather than inventing one.
- **Simplification carries no discovery semantics.** It preserves value and
  reduces cost; it carries no fit residual, no confidence, and no stability
  verdict. Those belong to the stages that consume the law.

## Non-goals

- No canonicalization or normal-form guarantee, and therefore no general
  expression-equality oracle.
- No expansion, term collection, or trigonometric angle-sum machinery beyond the
  single Pythagorean collapse; only the cost-reducing direction of distributivity
  (factoring) is applied.
- No symbolic algebra beyond the fixed local rule set — no polynomial division,
  no partial fractions, no algebraic-number handling.
- No numeric approximation of an expression as a product; evaluation is used only
  to *verify* soundness in tests.
