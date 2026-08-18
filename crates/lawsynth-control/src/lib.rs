//! Controlled (SINDYc) discovery for LawSynth forced systems.
//!
//! This crate extends sparse dynamics discovery from autonomous systems
//! `ẋ = f(x)` to **forced** systems `ẋ = f(x, u)`, where `u(t)` are exogenous,
//! measured control inputs. Ordinary SINDy fits `Ẋ = Θ(X)Ξ`; SINDYc augments the
//! candidate library to `Θ(X, U)` over both states and controls and regresses
//! each *state* derivative onto it.
//!
//! # The control contract
//!
//! Controls are treated fundamentally differently from states:
//!
//! - **Exogenous.** Controls are inputs to the library, evaluated at their
//!   measured values on the same time grid as the states.
//! - **Never differentiated.** Only state columns are passed to the derivative
//!   estimator; there is no `u̇`. This is enforced structurally in
//!   [`discover_controlled`], which builds a state-only sub-dataset before
//!   differentiating.
//! - **Never predicted.** The model contains exactly one equation per state and
//!   none for any control — controls appear only *inside* library terms.
//!
//! # Determinism
//!
//! The library variable order is fixed by the [`ControlSpec`] (`[states..,
//! controls..]`, as given), the library term order is fixed by
//! `lawsynth-features`, the derivative estimator is deterministic, and the
//! sparse solve (`stlsq_standardized`) is deterministic. Identical
//! `(Dataset, ControlSpec, ControlConfig)` inputs yield **bit-identical**
//! [`ControlledModel`] output.
//!
//! # Honest limits
//!
//! - Controls MUST be measured and sampled on the same time grid as the states;
//!   this crate does not resample or align signals.
//! - The control must be **persistently exciting**. If `u(t)` is constant or
//!   varies too little, its library columns are (near-)collinear with the
//!   constant term or with each other, and the control coefficients are
//!   unidentifiable — the sparse solve may attribute the control's effect to a
//!   state term or drop it entirely.
//! - Targets come from numerical differentiation, so the usual noise caveats of
//!   strong-form SINDy apply. See `specs/controlled-discovery/README.md`.
//!
//! # Example
//!
//! ```no_run
//! use lawsynth_control::{ControlConfig, ControlSpec, discover_controlled};
//! use lawsynth_core::Identifier;
//! # use lawsynth_data::Dataset;
//! # fn demo(dataset: &Dataset) -> Result<(), lawsynth_control::ControlError> {
//! let x = Identifier::new("x").unwrap();
//! let y = Identifier::new("y").unwrap();
//! let u = Identifier::new("u").unwrap();
//! let spec = ControlSpec::new([x, y], [u])?;
//! let model = discover_controlled(dataset, &spec, &ControlConfig::default())?;
//! for equation in &model.equations {
//!     for (term, coefficient) in equation.active_terms(&model.library_terms) {
//!         println!("d/dt {} += {coefficient} * {term}", equation.state);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod config;
mod discover;
mod error;
mod library;
mod model;
pub mod simulate;
mod spec;

pub use config::ControlConfig;
pub use discover::discover_controlled;
pub use error::ControlError;
pub use model::{ControlledModel, StateEquation};
pub use simulate::{
    ControlScore, ControlSignal, SimConfig, StateScore, Trajectory, ValidationConfig,
    simulate_controlled, validate_controlled,
};
pub use spec::ControlSpec;
