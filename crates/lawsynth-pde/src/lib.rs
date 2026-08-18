//! Deterministic evolution-PDE discovery (PDE-FIND style) for LawSynth.
//!
//! Given snapshots of a 1-D field `u(x, t)` on a regular space–time grid, this
//! crate discovers an evolution law
//!
//! ```text
//! u_t = F(u, u_x, u_xx, ...)
//! ```
//!
//! by estimating the time derivative and the spatial derivatives with central
//! finite differences on the grid **interior**, building a candidate library of
//! differential terms (powers of `u` times derivative factors), and
//! **sparse-regressing** the flattened `u_t` onto that library. It is the PDE
//! analogue of SINDy — Rudy, Brunton, Proctor & Kutz, *"Data-driven discovery of
//! partial differential equations"* (PDE-FIND).
//!
//! # Method
//!
//! 1. **Finite differences.** `u_t` is a central time difference
//!    `(u[t+1] − u[t−1]) / (2·dt)`; `u_x`, `u_xx`, `u_xxx` are central spatial
//!    differences (all `O(h²)`). Only interior points where the central stencil
//!    is valid are used — the outermost points along each axis are dropped. See
//!    [`derivatives`](crate) for the stencils.
//! 2. **Library.** Each candidate column is a product `uᵖ · D_m` with
//!    `D_0 = 1, D_1 = u_x, D_2 = u_xx, ...`, `p` up to
//!    [`PdeConfig::max_u_degree`] and `m` up to
//!    [`PdeConfig::max_derivative_order`]. The default `[1, u, u², u_x, u·u_x,
//!    u²·u_x, u_xx, u·u_xx, u²·u_xx]` covers the heat, Burgers and advection
//!    families. Every term is labelled.
//! 3. **Sparse regression.** The flattened `u_t` (over all interior `(x, t)`) is
//!    regressed onto the library via STLSQ (`lawsynth-sparse`). The design matrix
//!    and target are internally rescaled by `RMS(u_t)`, so the sparse threshold
//!    is a dimensionless fraction of the dominant balance.
//!
//! # Determinism
//!
//! The interior is visited row-major, time outer / space inner, and the sparse
//! solve is deterministic. Identical `(field, dx, dt, PdeConfig)` inputs produce
//! a bit-identical [`PdeModel`].
//!
//! # Honest limits
//!
//! Finite differences differentiate the data (twice in space for `u_xx`), which
//! **amplifies noise**; recovery is therefore sensitive to noise and to grid
//! resolution. The truncation error is `O(dx²)` / `O(dt²)`, so **finer grids
//! tighten the recovered coefficients** — the reference tests assert recovery
//! only to tolerances matched to their grid, never machine precision. The scope
//! is 1-D evolution PDEs on a regular grid with a fixed differential-term
//! library; boundaries are dropped, and arbitrary PDEs, 2-D fields, and a
//! noise-robust weak form are out of scope. See `specs/pde-discovery/README.md`
//! for the full contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_pde::{PdeConfig, discover_pde};
//!
//! // Two-mode exact solution of the heat equation u_t = α u_xx on [0, 2π):
//! //   u(x, t) = e^{-α k1² t} sin(k1 x) + ½ e^{-α k2² t} sin(k2 x).
//! // Two modes (k1 ≠ k2) break the single-mode collinearity of u and u_xx.
//! let alpha = 0.2_f64;
//! let (nx, nt) = (48, 24);
//! let dx = std::f64::consts::TAU / nx as f64;
//! let dt = 0.02_f64;
//! let field: Vec<Vec<f64>> = (0..nt)
//!     .map(|ti| {
//!         let t = ti as f64 * dt;
//!         (0..nx)
//!             .map(|xi| {
//!                 let x = xi as f64 * dx;
//!                 (-alpha * 1.0 * t).exp() * (1.0 * x).sin()
//!                     + 0.5 * (-alpha * 4.0 * t).exp() * (2.0 * x).sin()
//!             })
//!             .collect()
//!     })
//!     .collect();
//!
//! let model = discover_pde(&field, dx, dt, &PdeConfig::default()).unwrap();
//! // The discovered law is u_t = α·u_xx (coefficient of u^0 · u_xx ≈ α).
//! assert!((model.coefficient_of(0, 2) - alpha).abs() < 0.05);
//! ```

mod config;
mod derivatives;
mod discover;
mod error;
mod library;
mod model;

pub use config::PdeConfig;
pub use discover::discover_pde;
pub use error::PdeError;
pub use model::{PdeModel, PdeTerm};
