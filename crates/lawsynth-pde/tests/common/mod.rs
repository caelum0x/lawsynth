//! Deterministic field generators for the PDE-discovery integration tests.
//!
//! Two of the three fixtures are **exact analytic solutions** (heat, advection),
//! so there is no solver error to confound the finite-difference truncation the
//! discovery incurs. The Burgers fixture is produced by a stable, deterministic
//! RK4 forward solve on a fine substep grid, so the snapshot-grid central
//! differences remain the honest error source.
//!
//! All fields live on the periodic domain `[0, 2π)` with `dx = 2π / nx`. The
//! discovery drops the spatial boundary points, which the periodic, smooth data
//! tolerates without special treatment.
// Shared across several test binaries; not every helper is used by every one.
#![allow(dead_code)]

use std::f64::consts::TAU;

/// A generated field together with the grid steps discovery needs.
pub struct Fixture {
    pub field: Vec<Vec<f64>>,
    pub dx: f64,
    pub dt: f64,
}

/// The spatial step for `nx` points on `[0, 2π)`.
fn spatial_step(nx: usize) -> f64 {
    TAU / nx as f64
}

/// A **two-mode exact solution of the heat equation** `u_t = α u_xx`:
///
/// ```text
/// u(x, t) = e^{-α k1² t} sin(k1 x) + amp2 · e^{-α k2² t} sin(k2 x).
/// ```
///
/// Each Fourier mode solves the heat equation, so their sum does too — exactly,
/// with no solver error. Using two distinct wavenumbers is essential: a *single*
/// mode makes `u` and `u_xx` perfectly collinear (`u_xx = −k² u`), so PDE-FIND
/// could not tell `α u_xx` from `−α k² u`. Two modes break that degeneracy.
pub fn heat_two_mode(
    alpha: f64,
    k1: f64,
    k2: f64,
    amp2: f64,
    nx: usize,
    nt: usize,
    dt: f64,
) -> Fixture {
    let dx = spatial_step(nx);
    let field = (0..nt)
        .map(|ti| {
            let t = ti as f64 * dt;
            (0..nx)
                .map(|xi| {
                    let x = xi as f64 * dx;
                    (-alpha * k1 * k1 * t).exp() * (k1 * x).sin()
                        + amp2 * (-alpha * k2 * k2 * t).exp() * (k2 * x).sin()
                })
                .collect()
        })
        .collect();
    Fixture { field, dx, dt }
}

/// A **two-mode exact travelling wave** for advection `u_t = −c u_x`:
///
/// ```text
/// u(x, t) = sin(k1 (x − c t)) + amp2 · sin(k2 (x − c t)).
/// ```
///
/// Any profile translating at speed `c` satisfies `u_t = −c u_x` exactly.
pub fn advection_two_mode(
    c: f64,
    k1: f64,
    k2: f64,
    amp2: f64,
    nx: usize,
    nt: usize,
    dt: f64,
) -> Fixture {
    let dx = spatial_step(nx);
    let field = (0..nt)
        .map(|ti| {
            let t = ti as f64 * dt;
            (0..nx)
                .map(|xi| {
                    let x = xi as f64 * dx;
                    let phase = x - c * t;
                    (k1 * phase).sin() + amp2 * (k2 * phase).sin()
                })
                .collect()
        })
        .collect();
    Fixture { field, dx, dt }
}

/// A **stable RK4 forward solve of viscous Burgers** `u_t = ν u_xx − u u_x` on
/// the periodic domain, from the smooth initial condition `u₀(x) = sin(x)`.
///
/// The internal step is `dt / substeps`; only every `substeps`-th state is kept,
/// so the emitted snapshot grid is coarse enough for its central time difference
/// to be the dominant (and honest) error source, not the solver.
pub fn burgers_forward(nu: f64, nx: usize, nt: usize, dt: f64, substeps: usize) -> Fixture {
    let dx = spatial_step(nx);
    let mut u: Vec<f64> = (0..nx).map(|xi| (xi as f64 * dx).sin()).collect();
    let inner_dt = dt / substeps as f64;

    let mut field = Vec::with_capacity(nt);
    field.push(u.clone());
    for _ in 1..nt {
        for _ in 0..substeps {
            u = rk4_step(&u, nu, dx, inner_dt);
        }
        field.push(u.clone());
    }
    Fixture { field, dx, dt }
}

/// One classical RK4 step of the Burgers right-hand side.
fn rk4_step(u: &[f64], nu: f64, dx: f64, dt: f64) -> Vec<f64> {
    let k1 = burgers_rhs(u, nu, dx);
    let k2 = burgers_rhs(&axpy(u, &k1, dt / 2.0), nu, dx);
    let k3 = burgers_rhs(&axpy(u, &k2, dt / 2.0), nu, dx);
    let k4 = burgers_rhs(&axpy(u, &k3, dt), nu, dx);
    (0..u.len()).map(|i| u[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i])).collect()
}

/// `u + a·v`, elementwise.
fn axpy(u: &[f64], v: &[f64], a: f64) -> Vec<f64> {
    (0..u.len()).map(|i| u[i] + a * v[i]).collect()
}

/// The Burgers right-hand side `ν u_xx − u u_x` with periodic central differences.
fn burgers_rhs(u: &[f64], nu: f64, dx: f64) -> Vec<f64> {
    let n = u.len();
    (0..n)
        .map(|i| {
            let left = u[(i + n - 1) % n];
            let right = u[(i + 1) % n];
            let u_x = (right - left) / (2.0 * dx);
            let u_xx = (right - 2.0 * u[i] + left) / (dx * dx);
            nu * u_xx - u[i] * u_x
        })
        .collect()
}
