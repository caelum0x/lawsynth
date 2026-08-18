//! Deterministic reconstruction of a Cartesian sample grid from a `Dataset`,
//! plus axis-wise numerical differentiation built on `lawsynth-differentiate`.
//!
//! Separability and symmetry screening both need partials of `f` with respect to
//! one variable while the others are held fixed. That is only well defined when
//! the samples form a full tensor grid, so this module reconstructs that grid
//! (or honestly reports that it cannot) and exposes partials along each axis.

use lawsynth_differentiate::differentiate_series;

use crate::ReduceError;

/// A single reconstructed grid axis: strictly increasing distinct levels.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Axis {
    pub name: String,
    pub coords: Vec<f64>,
}

/// An `n`-dimensional scalar field sampled on a Cartesian grid, stored flat in
/// row-major order (axis 0 is the slowest-varying index).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GridField {
    pub axes: Vec<Axis>,
    pub values: Vec<f64>,
}

/// Why a Cartesian grid could not be reconstructed from the samples.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GridFailure {
    /// An axis had fewer than the required number of distinct levels.
    AxisTooShort { name: String, distinct: usize, required: usize },
    /// The product of axis lengths did not equal the sample count, so the
    /// samples are not a complete tensor grid.
    NotComplete { product: usize, samples: usize },
    /// A sample value did not fall on any reconstructed axis level.
    OffGrid { name: String },
    /// Two samples mapped to the same grid cell (duplicate coordinates).
    DuplicateCell,
    /// A grid cell had no sample (incomplete grid).
    MissingCell,
}

impl GridFailure {
    pub fn reason(&self) -> String {
        match self {
            GridFailure::AxisTooShort { name, distinct, required } => {
                format!("axis `{name}` has {distinct} distinct level(s); {required} are required")
            }
            GridFailure::NotComplete { product, samples } => {
                format!("samples ({samples}) do not fill the Cartesian product of axes ({product})")
            }
            GridFailure::OffGrid { name } => {
                format!("a sample did not align to any level of axis `{name}`")
            }
            GridFailure::DuplicateCell => "two samples share the same grid cell".to_string(),
            GridFailure::MissingCell => "the sample grid has an empty cell".to_string(),
        }
    }
}

impl GridField {
    pub fn ndim(&self) -> usize {
        self.axes.len()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Stride (flat step) for a one-step increment along `axis`.
    pub fn stride(&self, axis: usize) -> usize {
        self.axes[axis + 1..].iter().map(|a| a.coords.len()).product()
    }

    /// The index along `axis` for flat position `flat`.
    pub fn axis_index(&self, flat: usize, axis: usize) -> usize {
        (flat / self.stride(axis)) % self.axes[axis].coords.len()
    }

    /// The coordinate value along `axis` at flat position `flat`.
    pub fn coord_at(&self, flat: usize, axis: usize) -> f64 {
        self.axes[axis].coords[self.axis_index(flat, axis)]
    }

    /// Flat indices of cells that are interior along every axis in `axes` (i.e.
    /// not on an end level of those axes). Derivative-based residuals are only
    /// accurate on interior cells, where the three-point derivative is central;
    /// endpoints use a less accurate one-sided rule.
    pub fn interior_cells(&self, axes: &[usize]) -> Vec<usize> {
        (0..self.len())
            .filter(|&cell| {
                axes.iter().all(|&axis| {
                    let idx = self.axis_index(cell, axis);
                    idx > 0 && idx + 1 < self.axes[axis].coords.len()
                })
            })
            .collect()
    }

    /// Numerical partial `∂F/∂x_axis` over the whole grid, using the
    /// deterministic three-point derivative from `lawsynth-differentiate` along
    /// each line parallel to `axis`.
    pub fn partial(&self, axis: usize) -> Result<GridField, ReduceError> {
        let stride = self.stride(axis);
        let len = self.axes[axis].coords.len();
        let coords = &self.axes[axis].coords;
        let mut out = vec![0.0; self.values.len()];
        for base in 0..self.values.len() {
            // Process each line once, keyed by its start (axis index == 0).
            if (base / stride) % len != 0 {
                continue;
            }
            let line: Vec<f64> = (0..len).map(|k| self.values[base + k * stride]).collect();
            let derivative = differentiate_series(coords, &line)
                .map_err(|err| ReduceError::Differentiation(format!("{err:?}")))?;
            for (k, value) in derivative.into_iter().enumerate() {
                out[base + k * stride] = value;
            }
        }
        Ok(GridField { axes: self.axes.clone(), values: out })
    }
}

/// Root-mean-square of a slice (deterministic left-to-right accumulation).
pub(crate) fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f64 = values.iter().map(|v| v * v).sum();
    (sum / values.len() as f64).sqrt()
}

pub(crate) fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

pub(crate) fn range(values: &[f64]) -> f64 {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in values {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if lo.is_finite() && hi.is_finite() { hi - lo } else { 0.0 }
}

/// Deterministically reduces a coordinate column to its sorted distinct levels,
/// merging values within a relative tolerance.
pub(crate) fn distinct_levels(values: &[f64], rel_tol: f64) -> Vec<f64> {
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut levels: Vec<f64> = Vec::new();
    for value in sorted {
        match levels.last() {
            Some(&last)
                if (value - last).abs() <= last.abs().max(value.abs()).max(1.0) * rel_tol => {}
            _ => levels.push(value),
        }
    }
    levels
}

/// Finds the index of `value` among `coords` within `rel_tol`, or `None`.
fn level_index(coords: &[f64], value: f64, rel_tol: f64) -> Option<usize> {
    coords
        .iter()
        .position(|&level| (value - level).abs() <= level.abs().max(value.abs()).max(1.0) * rel_tol)
}

/// Attempts to reconstruct a Cartesian grid: `inputs` are the named coordinate
/// columns (in variable order), `target` is the scalar field sampled per row.
pub(crate) fn reconstruct(
    inputs: &[(String, Vec<f64>)],
    target: &[f64],
    min_axis_len: usize,
    rel_tol: f64,
) -> Result<GridField, GridFailure> {
    let samples = target.len();
    let mut axes: Vec<Axis> = Vec::with_capacity(inputs.len());
    for (name, column) in inputs {
        let coords = distinct_levels(column, rel_tol);
        if coords.len() < min_axis_len {
            return Err(GridFailure::AxisTooShort {
                name: name.clone(),
                distinct: coords.len(),
                required: min_axis_len,
            });
        }
        axes.push(Axis { name: name.clone(), coords });
    }

    let product: usize = axes.iter().map(|a| a.coords.len()).product();
    if product != samples {
        return Err(GridFailure::NotComplete { product, samples });
    }

    let field = GridField { axes, values: vec![f64::NAN; product] };
    let mut filled = vec![false; product];
    let mut values = vec![0.0; product];
    for row in 0..samples {
        let mut flat = 0usize;
        for (axis, (name, column)) in inputs.iter().enumerate() {
            let coords = &field.axes[axis].coords;
            let idx = level_index(coords, column[row], rel_tol)
                .ok_or_else(|| GridFailure::OffGrid { name: name.clone() })?;
            flat = flat * coords.len() + idx;
        }
        if filled[flat] {
            return Err(GridFailure::DuplicateCell);
        }
        filled[flat] = true;
        values[flat] = target[row];
    }
    if filled.iter().any(|&f| !f) {
        return Err(GridFailure::MissingCell);
    }
    Ok(GridField { axes: field.axes, values })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_2d(xs: &[f64], ys: &[f64], f: impl Fn(f64, f64) -> f64) -> GridField {
        let mut values = Vec::new();
        for &x in xs {
            for &y in ys {
                values.push(f(x, y));
            }
        }
        GridField {
            axes: vec![
                Axis { name: "x".into(), coords: xs.to_vec() },
                Axis { name: "y".into(), coords: ys.to_vec() },
            ],
            values,
        }
    }

    #[test]
    fn distinct_levels_merges_near_equal_and_sorts() {
        let levels = distinct_levels(&[2.0, 1.0, 2.0, 1.0, 3.0], 1e-9);
        assert_eq!(levels, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn partial_recovers_linear_slope() {
        // f = 3x + y  ->  ∂f/∂x = 3 everywhere.
        let xs = [0.0, 1.0, 2.0, 3.0];
        let ys = [0.0, 1.0, 2.0];
        let field = grid_2d(&xs, &ys, |x, y| 3.0 * x + y);
        let fx = field.partial(0).unwrap();
        for v in fx.values {
            assert!((v - 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn reconstruct_builds_row_major_grid() {
        let xs = vec![10.0, 20.0, 30.0];
        let ys = vec![1.0, 2.0, 3.0];
        let mut xcol = Vec::new();
        let mut ycol = Vec::new();
        let mut target = Vec::new();
        for &x in &xs {
            for &y in &ys {
                xcol.push(x);
                ycol.push(y);
                target.push(x + y);
            }
        }
        let field =
            reconstruct(&[("x".into(), xcol), ("y".into(), ycol)], &target, 3, 1e-9).unwrap();
        assert_eq!(field.axes[0].coords, xs);
        assert_eq!(field.axes[1].coords, ys);
        // Cell (x=20, y=3) is flat index 1*3 + 2 = 5.
        assert_eq!(field.values[5], 23.0);
    }

    #[test]
    fn reconstruct_rejects_incomplete_grid() {
        // Two rows only: not a full 2x2 product.
        let result = reconstruct(
            &[("x".into(), vec![0.0, 1.0]), ("y".into(), vec![0.0, 1.0])],
            &[0.0, 1.0],
            2,
            1e-9,
        );
        assert!(matches!(result, Err(GridFailure::NotComplete { .. })));
    }
}
