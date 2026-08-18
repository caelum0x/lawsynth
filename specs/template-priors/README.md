# Template priors boundary (v2-A)

This directory specifies **grammar-constrained candidate libraries** — the
declarative *template priors* implemented in `crates/lawsynth-discovery`
(`template.rs`, wired through `execute.rs`). It is a **boundary specification** in
the house style: it states what a conforming implementation MUST do, and —
crucially — what a prior is and is not allowed to claim.

## Motivation

Sparse regression (SINDy) and symbolic search recover a law by selecting terms
from a fixed candidate library `Θ(X)`. The larger and less structured that
library, the harder identification becomes: irrelevant columns invite spurious
selections, inflate variance, and blur the Pareto frontier. Symbolic-regression
tools (e.g. PySR) let a user impose *templates / constraints* — "the law is a
rational function", "no cross terms", "these variables only" — that shrink the
search space using a physical or domain prior.

LawSynth encodes the same idea as a **template prior**: a declarative, immutable
description of which candidate terms are admissible. It is applied as a
**deterministic hard filter** to the materialised feature library *before* the
sparse solve, so the solver only ever sees terms the prior permits.

## What a prior IS

A template prior is a **HARD admissibility filter over candidate TERMS**. The
contract is:

1. **It restricts the candidate library, nothing more.** A term the filter drops
   can never appear in a discovered law. A term the filter keeps is merely
   *offered* to the solver — the sparse solve may still zero its coefficient.
2. **It is not a soft penalty.** There is no weight, temperature, or score nudge.
   A term is either admissible or it is not.
3. **It is not a proof about the truth.** A prior expresses the *user's* stated
   assumption about the law's form. If the prior excludes a term the true law
   needs, discovery **cannot** recover that law — that non-recovery is the honest,
   enforced consequence of the assumption, not a bug.
4. **Every drop is recorded.** Applying a prior produces a `TemplateFilterReport`
   listing every dropped term and the single reason it was dropped. A prior MUST
   NOT silently narrow the search; the report makes every application auditable.

## Requirements

1. **Determinism & offline.** The filter is a pure function of `(candidate terms,
   prior)`. It reads no clock, draws no randomness, and iterates only in stable
   slice order (never hashmap iteration order). Identical inputs MUST yield a
   bit-identical admissible set and report. The crate is std-only with internal
   path dependencies; `net.offline = true`.
2. **Per-term rules, fixed evaluation order.** Each candidate term is tested
   against the enabled rules in the order: total degree, allowed variables,
   allowed kinds, forbid-interactions. The **first** violated rule is the recorded
   drop reason, so reports are deterministic and never double-count a term.
3. **Total-degree definition.** Total degree is defined recursively: a constant is
   `0`, a variable is `1`, a product adds the degrees of its factors, a sum or
   quotient takes the max of its sides, an integer power multiplies, and a
   transcendental (`sin`, `cos`, `exp`, `log`, negate) takes the degree of its
   argument. Thus `x·y` and the bounded rational `x/(1+x²)` both have degree `2`,
   and `sin(x)` has degree `1`. The cap `max_total_degree(d)` admits only terms of
   degree `≤ d`.
4. **Kind classification is total and priority-ordered.** Every term maps to
   exactly one `TermKind`: `Rational` if it contains any division, else
   `Trigonometric` if it contains a sine/cosine, else `Exponential` if it contains
   an exp/log, else `Constant` if it has no variables, else `Polynomial`.
   `allowed_kinds(K)` admits only terms whose kind is in `K`.
5. **Interactions.** An *interaction* (cross) term is any term referencing two or
   more distinct variables. `forbid_interactions` drops exactly those, keeping
   single-variable and constant terms.
6. **Active-term cap.** `max_active_terms(n)` retains the first `n` otherwise
   admissible terms in the library's own deterministic order (which for the
   standard polynomial library is ascending degree — the simplest terms) and drops
   the rest as `MaxActiveExceeded`. Because a discovered law's active terms are a
   subset of the candidates, this bounds the law to **at most `n` active terms**.
   The cap is applied after the per-term rules so it counts only admissible terms.
7. **Required kinds are checked against the final admitted set.** `required_kinds`
   asserts that the *admitted candidate set* contains at least one term of each
   required kind. If it does not — because the base library had none, or every one
   was dropped by another rule or the active-term cap — the prior is
   **unsatisfiable** and the filter fails with
   `TemplateError::UnsatisfiableRequiredKind`. This is checked last, so conflicts
   with any other rule surface honestly rather than silently.
8. **Backward compatibility.** The prior is opt-in via
   `DiscoveryConfig::template_prior: Option<TemplatePrior>`. The default `None`
   admits every candidate term; discovery is then **byte-identical** to the
   pre-template pipeline. Supplying `TemplatePrior::unconstrained()` is equivalent
   in output (the discovered world matches the no-prior run) but additionally
   emits a report recording zero drops.

## Where the filter applies

The prior is applied once, in `run_discovery`, immediately after the feature
library is materialised (polynomial + optional trigonometric + optional rational)
and evaluated, and **before** any per-state sparse fit. The admitted column set is
intersected, per state, with the (independent) dimensional-pruning admissibility
so the two filters compose without either overriding the other. The resulting
`TemplateFilterReport` is surfaced on `DiscoveryResult::template_filter` (`None`
on the default path).

## Honest scope & limits

- **A prior can only exclude the truth, never supply it.** If the true law
  requires a term the prior forbids, the constrained fit returns whatever best
  fits within the admitted set — and when the admitted set is empty for a state,
  the implementation emits an **honest zero law** (`0`) rather than fabricating
  structure. Recovery of an excluded law is impossible by construction.
- **`required_kinds` constrains the candidate set, not the selection.** Requiring
  a kind guarantees the solver is *offered* such a term; it does **not** guarantee
  the final law retains one (the sparse threshold may still zero it). This is the
  strongest guarantee a pre-solve candidate filter can honestly make; the
  implementation MUST NOT claim the selected law contains the required kind.
- **`max_active_terms` truncates by library order.** It bounds active-term count
  by dropping later-ordered candidates. If the true term falls outside the first
  `n`, it is dropped (and recorded) and cannot be recovered — the cap is a
  simplicity prior, not a guarantee that the best `n` terms are kept.

## Non-goals

- No soft/penalised constraints, no learned or data-adaptive templates, no
  automatic prior inference — a prior is supplied by the user as data.
- No new candidate *families*: the filter only removes terms from the existing
  library; it never synthesises terms the base library did not generate.
- No network or platform service; the filter is a pure in-process function.
