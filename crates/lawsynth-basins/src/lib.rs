//! Deterministic basin-of-attraction mapping for LawSynth.
//!
//! Given a discovered **autonomous** vector field `ẋ = f(x)` — one expression
//! tree per state — with SEVERAL stable attractors, this crate answers the
//! global question that stability and bifurcation do not: **from which initial
//! conditions do you reach each attractor?** It:
//!
//! 1. **locates the stable attractors** by delegating to
//!    [`lawsynth_stability::analyze_stability`] and keeping only the stable
//!    nodes and stable spirals (saddles, unstable points, and non-hyperbolic
//!    `Center`/`Marginal` points are not attractors);
//! 2. **lays a deterministic grid** of initial conditions over the search box;
//! 3. **integrates each initial condition forward** with a local fixed-step RK4
//!    flow; and
//! 4. **classifies each trajectory** by the attractor it converges to — or by
//!    the honest outcomes [`Label::Escaped`] and [`Label::Undetermined`].
//!
//! # Design
//!
//! - **Reuse, not reinvention.** Attractors come from the deterministic
//!   fixed-point + linear-stability analysis of [`lawsynth_stability`]; field
//!   evaluation uses the [`lawsynth_expr`] IR. Only the fixed-step RK4 forward
//!   flow is local (std-only).
//! - **Deterministic grid and flow.** The initial-condition grid is a fixed,
//!   content-independent even lattice over the box, enumerated row-major; RK4
//!   performs its arithmetic in a fixed order. Nothing is random or wall-clock
//!   derived.
//! - **Honest classification.** A trajectory is labelled with an attractor only
//!   if it comes within a tolerance of one. Trajectories that leave the box or
//!   diverge are `Escaped`; trajectories that do not settle within `max_time`
//!   are `Undetermined`. The classification is never forced.
//! - **Determinism.** Identical `(fields, states, BasinConfig)` inputs yield a
//!   bit-identical [`BasinReport`] — coordinates, fractions, and labels down to
//!   their `f64` bit patterns.
//! - **Offline, std-only.** No external crates; only internal LawSynth paths.
//!
//! See `specs/basin-mapping/README.md` for the conformance contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_basins::{BasinConfig, Label, map_basins};
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{BinaryOperator, Expr};
//!
//! // Bistable 1-D flow x' = x - x^3: stable attractors at x = ±1, saddle at 0.
//! let x = Identifier::new("x").unwrap();
//! let cube = Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(3.0));
//! let field = Expr::difference(Expr::symbol(x.clone()), cube);
//! let fields = vec![(x.clone(), field)];
//!
//! let config = BasinConfig::new(vec![(-2.0, 2.0)])
//!     .with_grid_resolution(5)
//!     .with_convergence_tolerance(1e-2)
//!     .with_max_time(30.0);
//! let report = map_basins(&fields, &[x], &config).unwrap();
//!
//! // Two attractors, and a symmetric box splits its settled mass evenly.
//! assert_eq!(report.attractors.len(), 2);
//! assert_eq!(report.fractions, vec![0.5, 0.5]);
//! // x = -2 flows to the x = -1 well (attractor index 0).
//! assert_eq!(report.grid_labels[0], Label::Attractor(0));
//! // x = 0 sits on the saddle and never settles: an honest Undetermined.
//! assert_eq!(report.grid_labels[2], Label::Undetermined);
//! ```

mod analysis;
mod classify;
mod config;
mod error;
mod grid;
mod integrate;
mod report;

pub use analysis::map_basins;
pub use config::{
    BasinConfig, DEFAULT_CONVERGENCE_TOLERANCE, DEFAULT_DIVERGENCE_LIMIT, DEFAULT_DT,
    DEFAULT_ESCAPE_MARGIN, DEFAULT_GRID_RESOLUTION, DEFAULT_MAX_TIME,
};
pub use error::BasinError;
pub use report::{Attractor, BasinReport, Label};

// Re-exported so callers can read an attractor's class without depending on
// `lawsynth-stability` directly.
pub use lawsynth_stability::Classification;
