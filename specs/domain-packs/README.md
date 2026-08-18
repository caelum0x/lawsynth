# Domain packs boundary (v2-A)

This directory specifies **curated, self-validated domain presets** — the starting
points implemented in `crates/lawsynth-domains` (`catalog.rs`, `reference.rs`,
`preset.rs`, `registry.rs`). It is a **boundary specification** in the house
style: it states what a conforming preset MUST provide and MUST NOT claim.

## Motivation

Sparse and symbolic discovery need three things chosen well before they can
recover a law: which candidate terms to search over (the feature library), which
structural assumptions to impose (an optional template prior), and clean, richly
excited data. A newcomer to a domain has to guess all three. A **domain preset**
packages a defensible default for a common scientific domain — mechanics,
population dynamics, chemical kinetics — so LawSynth is usable out of the box.

A preset is not a black box that promises the right answer. It is a **transparent,
self-validated bundle** whose every claim is checked by a round-trip test against
its own reference law.

## What a preset IS

A domain preset bundles four things for one domain:

1. **A feature-library configuration** — polynomial degree plus optional
   trigonometric / rational families — tuned so the domain's law lies inside the
   candidate span.
2. **An optional template prior** (`specs/template-priors/`) encoding a hard
   structural assumption (e.g. "populations have no spontaneous source term") that
   shrinks the candidate set.
3. **Optional, honestly-expressible SI unit hints** for the state variables. A
   preset attaches a unit only when it can actually express it; abstract counts or
   concentrations are left unannotated rather than given an invented dimension.
4. **A documented reference system** — the canonical governing law together with a
   deterministic fixed-step RK4 trajectory generator built directly from that law.

The binding contract is **round-trip recovery**: integrating the reference law
into a clean trajectory and running discovery with the preset's own configuration
recovers that same law — the same active-term structure and coefficients to a
tight tolerance. Each shipped preset carries one integration test asserting this;
a preset that cannot recover its own law **is not shipped**.

## Requirements

1. **Self-validation is mandatory.** Every preset MUST have an integration test
   that (a) generates the reference trajectory, (b) builds a `Dataset`, (c) runs
   discovery through the preset's assembled `DiscoveryConfig` (degree, families,
   sparse threshold, and prior), and (d) asserts the discovered law's monomial
   support matches the reference exactly and every coefficient matches within a
   tight tolerance. The tolerance MUST be far below the coefficient magnitudes and
   only comfortably above the observed finite-difference error — never widened to
   mask a miss. If a domain cannot be made to round-trip under the std-only,
   offline constraints, it MUST be dropped, not faked.
2. **Determinism.** Preset lookup, trajectory generation, and discovery are pure
   functions of their inputs. The RK4 integrator reads no clock and draws no
   randomness; identical `(initial condition, step size, step count)` yield a
   bit-identical trajectory, and preset lookup is a fixed-order slice scan, never a
   hashed iteration. Identical inputs MUST produce bit-identical outputs.
3. **Reference laws are standard textbook forms.** The canonical law shipped for a
   domain MUST be the recognized standard form (with named parameters), so the
   preset teaches the real governing equation rather than a contrived fit target.
4. **Offline, std-only.** The crate is std-only with internal path dependencies;
   `net.offline = true`. No external crates, no network, no platform service.
5. **Deterministic registry.** Presets are exposed through a fixed-order enum
   (`DomainPresetKind`) with lookup-by-name; an unknown name MUST return an
   explicit error listing the available names, never a silent default.

## The shipped presets

| Name | Domain | Reference law | Notes |
|---|---|---|---|
| `damped-oscillator` | Mechanics | `dx/dt = v`, `dv/dt = -x - 0.5·v` | Degree-1 library; SI units (m, m/s). The sparse threshold sits below the damping term's standardized magnitude so the real damping is kept, not pruned. |
| `lotka-volterra` | Population dynamics | `dprey/dt = 1.5·prey - prey·predator`, `dpredator/dt = 0.75·prey·predator - predator` | Degree-2 library for the interaction term; an ecological template prior admits only polynomial kinds, dropping the constant intercept (no spontaneous generation). |
| `brusselator` | Chemical kinetics | `dx/dt = 1 - 4·x + x²·y`, `dy/dt = 3·x - x²·y` | Degree-3 library for the cubic autocatalytic term. `B > 1 + A²` puts the system on a limit cycle, giving the persistent excitation the cubic term needs; the constant source term rules out a kind/degree prior. |

All three recover their reference law with coefficient error `≤ 1.3e-4`
(Lotka-Volterra's prey growth term is the tightest); the round-trip tests assert a
`1e-3` coefficient tolerance.

## Honest scope & limits

- **A preset is a starting point, not a guarantee.** It is tuned on *clean
  synthetic data* generated from a standard law. Real measurements carry noise,
  irregular sampling, and unmodeled effects; recovering a law from them generally
  needs the smoothing / preprocessing / bootstrap knobs on `DiscoveryConfig` and
  user judgement. A preset shrinks and centers the search; it does not certify the
  result on real data.
- **The reference is textbook, the parameters are fixed.** The synthetic
  trajectory uses one fixed initial condition, step size, and parameter set chosen
  for identifiability. A different operating regime (near a bifurcation, an
  over-damped oscillator, a Brusselator below its Hopf threshold) may need retuned
  degree, threshold, or prior — the preset is a default, not a universal law
  detector.
- **A prior can only exclude, never supply.** Where a preset ships a template
  prior, it inherits that spec's contract: the prior expresses the user's stated
  structural assumption and can make an excluded law unrecoverable. Preset priors
  are chosen so they never exclude a term the reference law needs.
- **Round-trip is necessary, not sufficient, for real use.** Recovering a law from
  its own clean synthetic data proves the search space and solver are correctly
  configured for that law; it does not prove the law holds for a user's data.

## Non-goals

- No noisy-data robustness claims, no automatic domain detection, no learned or
  data-adaptive presets — a preset is a fixed, authored bundle.
- No new candidate families or solver algorithms; a preset only *configures* the
  existing feature library, template-prior, and discovery machinery.
- No network or platform service; the whole crate is a pure in-process library.
