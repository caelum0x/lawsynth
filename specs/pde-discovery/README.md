# Evolution-PDE discovery boundary (v2-A)

This directory specifies evolution-PDE discovery — the PDE-FIND-style estimator
implemented in `crates/lawsynth-pde`, which recovers a 1-D evolution law

```text
u_t = F(u, u_x, u_xx, ...)
```

from snapshots of a scalar field `u(x, t)` on a regular space–time grid. It is a
**boundary specification** in the house style: it states what a conforming
implementation MUST do, and is explicit about the numerical nature of the result.

It is the PDE analogue of strong-form SINDy: `lawsynth-sim` can *simulate* a
field forward; this crate *discovers* the evolution law from data. Rudy, Brunton,
Proctor & Kutz, *"Data-driven discovery of partial differential equations"*
(PDE-FIND), is the reference method.

## Method

The field is presented as `field[t][x]` — rows are time snapshots, columns are
spatial points — with uniform steps `dx` and `dt`.

1. **Finite-difference derivatives on the interior.** The time derivative is the
   central difference `u_t = (u[t+1] − u[t−1]) / (2·dt)`, and the spatial
   derivatives are central differences of the required order:

   ```text
   u_x   = (u[i+1] − u[i−1]) / (2·dx)                          (half-width 1, O(dx²))
   u_xx  = (u[i+1] − 2·u[i] + u[i−1]) / dx²                    (half-width 1, O(dx²))
   u_xxx = (u[i+2] − 2·u[i+1] + 2·u[i−1] − u[i−2]) / (2·dx³)   (half-width 2, O(dx²))
   ```

   Only interior points where the central stencil is valid are used: the
   outermost snapshot on each side in time, and the outermost `h` columns on each
   spatial edge (`h` the widest stencil's half-width), are **dropped**.

2. **Differential-term library.** Each candidate column is a product `uᵖ · D_m`
   of a field power and a spatial-derivative factor (`D_0 = 1`, `D_1 = u_x`,
   `D_2 = u_xx`, `D_3 = u_xxx`), with `p` up to a configured max degree and `m`
   up to a configured max order. The default `[1, u, u², u_x, u·u_x, u²·u_x,
   u_xx, u·u_xx, u²·u_xx]` covers the heat, Burgers and advection families. Every
   column carries a human-readable label.

3. **Sparse regression.** The flattened `u_t` (over every interior `(x, t)`) is
   sequentially-thresholded-least-squares regressed onto the flattened library
   matrix (`lawsynth-sparse`). The surviving labelled terms are the discovered
   PDE.

## Requirements

1. **Central finite differences on the interior only.** A conforming discovery
   MUST estimate `u_t` by a central time difference and the spatial derivatives
   by central spatial differences of the stated order, and MUST evaluate the
   library **only** at points where every required stencil fits inside the grid.
   Boundary points MUST be dropped, never one-sided-extrapolated silently. The
   stencil order (`O(dx²)`/`O(dt²)`) MUST be documented.

2. **Fixed, labelled differential library.** The candidate set MUST be the
   cross-product of field powers `uᵖ` (`p = 0..P`) and derivative factors `D_m`
   (`m = 0..M`), with `P` and `M` configurable and each column labelled. The
   constant intercept (`p = 0, m = 0`) MUST be optional.

3. **Flattened sparse regression.** The target MUST be the flattened interior
   `u_t` and the design matrix the flattened library evaluated at the same
   points, in one fixed point order. The result MUST report each labelled term's
   fitted coefficient and the residual sum of squares of the fit.

4. **Determinism.** The interior traversal order (this implementation uses
   row-major, **time outer, space inner**), the library evaluation, and the
   sparse solve MUST run in a fixed order with no hidden randomness and no
   wall-clock input. Any synthetic field used to validate the method MUST be
   generated deterministically (an exact analytic solution, or a seeded/fixed
   forward solve). Identical `(field, dx, dt, PdeConfig)` inputs MUST yield a
   bit-identical `PdeModel`.

5. **Degenerate input.** A non-rectangular field, a non-finite sample, a grid too
   small for the required central stencil (fewer than three snapshots in time, or
   fewer than `2h+1` spatial columns), a non-finite or non-positive step, or a
   field with no time evolution MUST return a typed error, never a fabricated law.

## Honest limits

This is a **numerical differentiation + regression** estimator, and the
specification is deliberate about what that does and does not guarantee:

- Finite differences carry `O(dx²)`/`O(dt²)` **truncation error**, so recovered
  coefficients are approximate on any finite grid. **Finer grids tighten the
  recovery**; a conforming implementation MUST NOT claim machine-precision
  recovery. The reference tests assert recovery only to tolerances matched to
  their grid resolution (see below), and include a test that a finer grid does
  not worsen the error.
- The method **differentiates the observed data — twice in space for `u_xx`** —
  which amplifies observation noise. It is therefore noise-sensitive; the
  reference fixtures are clean (exact solutions or a fine-substep forward solve).
  A **weak / integral form** (see `specs/weak-form`) is the noise-robust
  counterpart, which moves the derivatives onto analytic test functions; a
  spatiotemporal weak form is deferred future work, matching that boundary's own
  deferral of full PDE weak forms.
- The scope is **1-D evolution PDEs on a regular grid** with a **fixed
  differential-term library**. Single-Fourier-mode fields make `u` and `u_xx`
  collinear (`u_xx = −k² u`), so the library cannot separate them — a field with
  spectral content in more than one mode is required to identify diffusion
  uniquely.
- Boundaries are **dropped**, not modelled. Higher derivative orders beyond
  `u_xxx`, multi-component or 2-D fields, non-uniform grids, and arbitrary
  (non-library) right-hand sides are **out of scope** for this boundary; they may
  be added as extensions with their own contracts.

## Reference recovery

The `lawsynth-pde` integration tests recover, on periodic `[0, 2π)` grids:

| Equation | Field source | Grid | Recovered | Truth |
| --- | --- | --- | --- | --- |
| Heat `u_t = α u_xx` | exact two-mode analytic | `nx = 96`, `nt = 40`, `dt = 0.01` | `u_xx ≈ 0.2002` | `α = 0.2` |
| Advection `u_t = −c u_x` | exact two-mode travelling wave | `nx = 96`, `nt = 40`, `dt = 0.01` | `u_x ≈ −0.8014` | `c = 0.8` |
| Burgers `u_t = ν u_xx − u u_x` | stable RK4 forward solve | `nx = 128`, `nt = 80`, `dt = 0.004` | `u_xx ≈ 0.1000`, `u·u_x ≈ −1.0000` | `ν = 0.1`, `−1` |

Each is recovered as a clean sparse law (the exact expected terms active, spurious
terms thresholded to zero), and the discovery is verified bit-identical across
repeated runs.
