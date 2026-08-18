//! The concrete preset catalog: three distinct, self-validated domains.
//!
//! Each builder returns a fully-formed [`DomainPreset`] whose reference law is a
//! standard textbook form. The tuning (polynomial degree, sparse threshold,
//! optional prior, initial condition, and RK4 schedule) is chosen so that the
//! trajectory synthesized from the reference law is rich enough for the deterministic
//! sparse solver to recover that same law — the property the integration tests assert.

use lawsynth_core::Identifier;
use lawsynth_discovery::{TemplatePrior, TermKind};
use lawsynth_units::Unit;

use crate::preset::{DomainPreset, UnitHint};
use crate::reference::{ReferenceLaw, ReferenceSystem, ReferenceTerm};

fn ident(name: &str) -> Identifier {
    Identifier::new(name).expect("preset identifiers are valid by construction")
}

fn unit(name: &str) -> Unit {
    Unit::parse(name).expect("preset unit strings are valid by construction")
}

/// Damped linear harmonic oscillator (mechanics).
///
/// Reference law (position `x`, velocity `v`), with `ω² = 1`, damping `c = 0.5`:
///
/// ```text
/// dx/dt = v
/// dv/dt = -x - 0.5·v
/// ```
///
/// A degree-1 polynomial library is exactly the linear span the law lives in, so
/// no structural prior is needed. The sparse threshold is set below the damping
/// term's standardized magnitude so the (physically real) damping is retained
/// rather than pruned. State variables carry honest SI units.
pub fn damped_oscillator() -> DomainPreset {
    let x = ident("x");
    let v = ident("v");
    // Variable order [x, v]: exponents below are [power of x, power of v].
    let reference = ReferenceSystem::new(
        vec![x.clone(), v.clone()],
        vec![
            // dx/dt = v
            ReferenceLaw::new([ReferenceTerm::new(1.0, [0, 1])]),
            // dv/dt = -x - 0.5 v
            ReferenceLaw::new([ReferenceTerm::new(-1.0, [1, 0]), ReferenceTerm::new(-0.5, [0, 1])]),
        ],
        vec![1.0, 0.0],
        0.01,
        2_000,
    );
    DomainPreset::new(
        "damped-oscillator",
        "Damped linear harmonic oscillator: dx/dt = v, dv/dt = -x - 0.5 v.",
        reference,
        1,
        false,
        false,
        0.05,
        None,
        vec![
            UnitHint { variable: x, unit: unit("m") },
            UnitHint { variable: v, unit: unit("m/s") },
        ],
    )
}

/// Lotka–Volterra predator–prey population dynamics.
///
/// Reference law (`prey`, `predator`):
///
/// ```text
/// dprey/dt     = 1.5·prey - prey·predator
/// dpredator/dt = 0.75·prey·predator - predator
/// ```
///
/// A degree-2 polynomial library supplies the required `prey·predator`
/// interaction. An ecological template prior restricts admissible terms to the
/// polynomial family, dropping the constant intercept — populations have no
/// spontaneous source term.
pub fn lotka_volterra() -> DomainPreset {
    let prey = ident("prey");
    let predator = ident("predator");
    // Variable order [prey, predator]: exponents are [power of prey, power of predator].
    let reference = ReferenceSystem::new(
        vec![prey.clone(), predator.clone()],
        vec![
            // dprey/dt = 1.5 prey - prey*predator
            ReferenceLaw::new([ReferenceTerm::new(1.5, [1, 0]), ReferenceTerm::new(-1.0, [1, 1])]),
            // dpredator/dt = 0.75 prey*predator - predator
            ReferenceLaw::new([ReferenceTerm::new(0.75, [1, 1]), ReferenceTerm::new(-1.0, [0, 1])]),
        ],
        vec![10.0, 5.0],
        0.001,
        4_000,
    );
    let prior = TemplatePrior::unconstrained().with_allowed_kinds([TermKind::Polynomial]);
    DomainPreset::new(
        "lotka-volterra",
        "Lotka-Volterra predator-prey: dprey/dt = 1.5 prey - prey predator, \
         dpredator/dt = 0.75 prey predator - predator.",
        reference,
        2,
        false,
        false,
        0.1,
        Some(prior),
        Vec::new(),
    )
}

/// Brusselator autocatalytic chemical kinetics.
///
/// Reference law (`x`, `y`) with `A = 1`, `B = 3`:
///
/// ```text
/// dx/dt = 1 - 4·x + x²·y
/// dy/dt = 3·x - x²·y
/// ```
///
/// The cubic autocatalytic term `x²·y` requires a degree-3 polynomial library.
/// Because `B > 1 + A²` the system settles onto a limit cycle, giving persistent
/// excitation across the state space — enough for the sparse solver to isolate the
/// cubic term. The constant source term rules out a degree or kind prior here.
pub fn brusselator() -> DomainPreset {
    let x = ident("x");
    let y = ident("y");
    // Variable order [x, y]: exponents are [power of x, power of y].
    let reference = ReferenceSystem::new(
        vec![x.clone(), y.clone()],
        vec![
            // dx/dt = 1 - 4 x + x^2 y
            ReferenceLaw::new([
                ReferenceTerm::new(1.0, [0, 0]),
                ReferenceTerm::new(-4.0, [1, 0]),
                ReferenceTerm::new(1.0, [2, 1]),
            ]),
            // dy/dt = 3 x - x^2 y
            ReferenceLaw::new([ReferenceTerm::new(3.0, [1, 0]), ReferenceTerm::new(-1.0, [2, 1])]),
        ],
        vec![1.0, 1.0],
        0.001,
        8_000,
    );
    DomainPreset::new(
        "brusselator",
        "Brusselator autocatalytic kinetics: dx/dt = 1 - 4 x + x^2 y, dy/dt = 3 x - x^2 y.",
        reference,
        3,
        false,
        false,
        0.1,
        None,
        Vec::new(),
    )
}
