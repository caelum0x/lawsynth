# Dimensional search boundary (units-in-discovery)

This directory specifies **dimensional pruning** and **Buckingham-π** support for
equation discovery. It is a **boundary specification**: it states what a
conforming implementation MUST do when variable units are available, not that any
particular search strategy is built. The primitives it constrains
(`lawsynth-units`, the discovery pipeline, the symbolic enumerator) already exist
and remain fully functional offline with units absent.

The technique mirrors in-loop dimensional analysis in PySR (`WildcardQuantity`),
AI-Feynman (units as the first structural reduction), and the SciML/PySINDy
practice of restricting a library to dimensionally-admissible terms. It is
**additive and opt-in**: with units absent or disabled the discovery path is
byte-identical to before.

## Inputs

A conforming implementation MAY accept, per discovery run, a partial map from
variable name to SI unit (hence to an integer **dimension vector** over the seven
base dimensions length, mass, time, current, temperature, amount, luminous
intensity). Units MAY come from the dataset, the `World` variables, or an
explicit `--units NAME=UNIT[,NAME=UNIT...]` input. The time axis the derivatives
are taken against carries the time dimension; only the *dimension* matters, never
a unit's numeric scale.

A variable absent from the map is a **dimensional wildcard** (see below), so a
partially-annotated dataset MUST NOT be over-pruned.

## The wildcard rule (free constants)

Free numeric constants — fitted coefficients, additive offsets, and literals
inside transcendental arguments — are **dimension wildcards**: each MAY take
whatever dimension keeps its enclosing term consistent. Concretely, inference
over an expression tree MUST treat a numeric constant (and an undeclared symbol)
as an undetermined dimension that:

- absorbs into an addition/subtraction to match the other operand's dimension;
- leaves a product/quotient undetermined;
- satisfies the dimensionless-argument requirement of `exp`/`log`/`sin`/`cos`;
- is a valid (dimensionless) power exponent.

A subexpression is **impossible** only when no wildcard assignment can make it
consistent — e.g. adding a length to a velocity, or taking `sin` of a dimensioned
quantity. Inference MUST reject exactly the impossible subexpressions and no
others.

## Target dimension and admissibility

For a state `xᵢ`, the discovery target is the derivative `dxᵢ/dt`, whose target
dimension MUST be `[xᵢ] / [time]`. A candidate term or law is **admissible** for
that target when its wildcard-aware inferred dimension either (a) is undetermined
(a wildcard, matching any target) or (b) equals the target dimension. An
impossible term is inadmissible.

Because each library term (and each enumerated symbolic candidate) is fitted with
its **own free coefficient** — itself a wildcard constant — a dimensionful
coefficient MAY rescale any internally-consistent term to the target. Therefore a
conforming implementation, when pruning coefficient-fitted candidate terms, MUST
reject a term **iff it is dimensionally impossible** (inference fails); an
internally-consistent term of any determined dimension is retained, since its
coefficient carries the residual dimension. This is the physical SINDy-with-units
rule: coefficients carry units, and only terms that cannot be made consistent for
*any* coefficient (e.g. `sin(x)` with dimensioned `x`) are removed.

An implementation MAY additionally offer a stricter mode that treats coefficients
as dimensionless and requires the term's determined dimension to equal the
target; that mode MUST be opt-in and is not the default.

## Pruning obligations

A conforming discovery implementation:

- MUST apply the admissibility test to every candidate feature term / law
  **before** scoring, so inadmissible candidates never influence the fit or the
  reported metrics;
- MUST apply the same rule in any candidate enumerator it exposes (e.g. the
  symbolic search), per-state against that state's target dimension;
- MUST be **deterministic**: the retained set and the pruned count depend only on
  the inputs, never on iteration order, wall-clock, or randomness;
- MUST NOT change results when units are absent or the filter is disabled — the
  default path is byte-identical, and a term set in which nothing is impossible
  yields an identical world;
- SHOULD report the number of candidates pruned by dimensional inconsistency as a
  diagnostic that does not otherwise affect the returned world.

## Buckingham-π (dimensionless groups)

An implementation MAY expose a helper that, given the variables' dimension
vectors, returns a basis of **dimensionless groups**: integer exponent vectors
`p` with `D · p = 0`, where `D` is the `7 × n` dimension matrix. This is the
integer nullspace of `D`; its dimension is `n − rank(D)` (the Buckingham-π count).
The computation MUST be exact (rational arithmetic, standard library only) and
deterministic — each basis vector normalized to a primitive integer vector with a
positive leading entry, and the basis returned in a fixed sorted order. The helper
is a documented, tested primitive; wiring it into a full nondimensionalized search
is permitted but not required by this boundary.

## Offline & determinism guarantee

Dimensional search is additive. A conforming implementation MUST run the entire
discovery path offline with no units configured (the pre-units behavior), with
dimensional pruning and Buckingham-π activating only when units are supplied. No
dimensional feature may make the local core require a network or a non-determin­istic
dependency.
