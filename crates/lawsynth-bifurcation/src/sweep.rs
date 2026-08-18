//! The deterministic parameter sweep and its numeric knobs.

use crate::error::BifurcationError;

/// Default number of grid points across the parameter range.
pub const DEFAULT_STEPS: usize = 21;
/// Default per-coordinate radius within which a fixed point at one parameter
/// value is treated as the continuation of a fixed point at the previous value.
pub const DEFAULT_MATCH_TOLERANCE: f64 = 1e-2;
/// Default number of bisection iterations used to localize a critical parameter.
pub const DEFAULT_LOCALIZATION_ITERATIONS: usize = 60;
/// Default half-width of the band around zero in which a real part is treated as
/// "on the imaginary axis" for crossing detection.
pub const DEFAULT_CROSSING_BAND: f64 = 1e-9;
/// Default threshold above which an eigenvalue's imaginary part counts as a
/// genuine complex pair (distinguishing Hopf from a real zero-eigenvalue fold).
pub const DEFAULT_IMAGINARY_TOLERANCE: f64 = 1e-6;
/// Default bound on `|Re(λ)|` at a localized branch birth/death below which the
/// event is accepted as a zero-eigenvalue (fold) bifurcation rather than a fixed
/// point merely wandering across the search-box boundary.
pub const DEFAULT_FOLD_EIGENVALUE_TOLERANCE: f64 = 1e-2;
/// Default parameter-distance within which two detected bifurcations are merged.
pub const DEFAULT_DEDUP_PARAMETER_TOLERANCE: f64 = 1e-4;
/// Default coordinate-distance within which two detected bifurcations are merged.
pub const DEFAULT_DEDUP_COORDINATE_TOLERANCE: f64 = 1e-2;

/// A deterministic sweep of a scalar parameter over `[min, max]`.
///
/// The sweep fixes the parameter grid and every tolerance the continuation and
/// bifurcation detection use, so a run is fully reproducible. All setters return
/// a new value (builder style); nothing is mutated in place.
#[derive(Clone, Debug, PartialEq)]
pub struct Sweep {
    min: f64,
    max: f64,
    steps: usize,
    match_tolerance: f64,
    localization_iterations: usize,
    crossing_band: f64,
    imaginary_tolerance: f64,
    fold_eigenvalue_tolerance: f64,
    dedup_parameter_tolerance: f64,
    dedup_coordinate_tolerance: f64,
}

impl Sweep {
    /// Builds a sweep over `[min, max]` with `steps` grid points and default
    /// tolerances. `steps` must be at least 2 and `min <= max`; both are checked
    /// when [`Sweep::validate`] runs.
    pub fn new(min: f64, max: f64, steps: usize) -> Self {
        Self {
            min,
            max,
            steps,
            match_tolerance: DEFAULT_MATCH_TOLERANCE,
            localization_iterations: DEFAULT_LOCALIZATION_ITERATIONS,
            crossing_band: DEFAULT_CROSSING_BAND,
            imaginary_tolerance: DEFAULT_IMAGINARY_TOLERANCE,
            fold_eigenvalue_tolerance: DEFAULT_FOLD_EIGENVALUE_TOLERANCE,
            dedup_parameter_tolerance: DEFAULT_DEDUP_PARAMETER_TOLERANCE,
            dedup_coordinate_tolerance: DEFAULT_DEDUP_COORDINATE_TOLERANCE,
        }
    }

    /// Sets the branch-matching per-coordinate tolerance.
    pub fn with_match_tolerance(mut self, tolerance: f64) -> Self {
        self.match_tolerance = tolerance;
        self
    }

    /// Sets the number of bisection iterations used to localize a critical value.
    pub fn with_localization_iterations(mut self, iterations: usize) -> Self {
        self.localization_iterations = iterations;
        self
    }

    /// Sets the half-width of the "real part ≈ 0" band for crossing detection.
    pub fn with_crossing_band(mut self, band: f64) -> Self {
        self.crossing_band = band;
        self
    }

    /// Sets the imaginary-part threshold distinguishing Hopf from a real fold.
    pub fn with_imaginary_tolerance(mut self, tolerance: f64) -> Self {
        self.imaginary_tolerance = tolerance;
        self
    }

    /// Sets the `|Re(λ)|` bound accepted as a fold at a branch birth/death.
    pub fn with_fold_eigenvalue_tolerance(mut self, tolerance: f64) -> Self {
        self.fold_eigenvalue_tolerance = tolerance;
        self
    }

    /// Sets the parameter-distance within which two bifurcations are merged.
    pub fn with_dedup_parameter_tolerance(mut self, tolerance: f64) -> Self {
        self.dedup_parameter_tolerance = tolerance;
        self
    }

    /// Sets the coordinate-distance within which two bifurcations are merged.
    pub fn with_dedup_coordinate_tolerance(mut self, tolerance: f64) -> Self {
        self.dedup_coordinate_tolerance = tolerance;
        self
    }

    /// The lower end of the parameter range.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// The upper end of the parameter range.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// The number of grid points.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// The branch-matching per-coordinate tolerance.
    pub fn match_tolerance(&self) -> f64 {
        self.match_tolerance
    }

    /// The bisection iteration budget for localization.
    pub fn localization_iterations(&self) -> usize {
        self.localization_iterations
    }

    /// The "real part ≈ 0" band half-width.
    pub fn crossing_band(&self) -> f64 {
        self.crossing_band
    }

    /// The Hopf-vs-fold imaginary-part threshold.
    pub fn imaginary_tolerance(&self) -> f64 {
        self.imaginary_tolerance
    }

    /// The fold-acceptance `|Re(λ)|` bound at a branch birth/death.
    pub fn fold_eigenvalue_tolerance(&self) -> f64 {
        self.fold_eigenvalue_tolerance
    }

    /// The parameter-distance bifurcation-merge tolerance.
    pub fn dedup_parameter_tolerance(&self) -> f64 {
        self.dedup_parameter_tolerance
    }

    /// The coordinate-distance bifurcation-merge tolerance.
    pub fn dedup_coordinate_tolerance(&self) -> f64 {
        self.dedup_coordinate_tolerance
    }

    /// The deterministic parameter grid: `steps` points spanning `[min, max]`.
    ///
    /// The endpoints are reproduced exactly (`grid[0] == min`,
    /// `grid[steps-1] == max`); interior points are `min + k·(max−min)/(steps−1)`.
    /// The sequence is a pure function of `(min, max, steps)`.
    pub fn grid(&self) -> Vec<f64> {
        if self.steps == 1 {
            return vec![self.min];
        }
        let last = self.steps - 1;
        let span = self.max - self.min;
        (0..self.steps)
            .map(|k| {
                if k == 0 {
                    self.min
                } else if k == last {
                    self.max
                } else {
                    self.min + span * (k as f64) / (last as f64)
                }
            })
            .collect()
    }

    /// Validates the sweep, returning a typed error on the first fault.
    pub fn validate(&self) -> Result<(), BifurcationError> {
        if !self.min.is_finite() || !self.max.is_finite() {
            return Err(BifurcationError::InvalidSweep("min and max must be finite"));
        }
        if self.min > self.max {
            return Err(BifurcationError::InvalidSweep("min must be <= max"));
        }
        if self.steps < 2 {
            return Err(BifurcationError::InvalidSweep("steps must be >= 2"));
        }
        if self.localization_iterations == 0 {
            return Err(BifurcationError::InvalidSweep("localization_iterations must be >= 1"));
        }
        for (value, label) in [
            (self.match_tolerance, "match_tolerance"),
            (self.crossing_band, "crossing_band"),
            (self.imaginary_tolerance, "imaginary_tolerance"),
            (self.fold_eigenvalue_tolerance, "fold_eigenvalue_tolerance"),
            (self.dedup_parameter_tolerance, "dedup_parameter_tolerance"),
            (self.dedup_coordinate_tolerance, "dedup_coordinate_tolerance"),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(BifurcationError::InvalidSweep(match label {
                    "match_tolerance" => "match_tolerance must be finite and >= 0",
                    "crossing_band" => "crossing_band must be finite and >= 0",
                    "imaginary_tolerance" => "imaginary_tolerance must be finite and >= 0",
                    "fold_eigenvalue_tolerance" => {
                        "fold_eigenvalue_tolerance must be finite and >= 0"
                    }
                    "dedup_parameter_tolerance" => {
                        "dedup_parameter_tolerance must be finite and >= 0"
                    }
                    _ => "dedup_coordinate_tolerance must be finite and >= 0",
                }));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_reproduces_endpoints_exactly() {
        let grid = Sweep::new(-1.0, 1.0, 5).grid();
        assert_eq!(grid.len(), 5);
        assert_eq!(grid[0], -1.0);
        assert_eq!(grid[4], 1.0);
        assert_eq!(grid[2], 0.0);
    }

    #[test]
    fn grid_is_bit_identical_across_calls() {
        let sweep = Sweep::new(-2.0, 3.0, 17);
        let a = sweep.grid();
        let b = sweep.grid();
        assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()));
    }

    #[test]
    fn rejects_too_few_steps() {
        assert_eq!(
            Sweep::new(0.0, 1.0, 1).validate(),
            Err(BifurcationError::InvalidSweep("steps must be >= 2"))
        );
    }

    #[test]
    fn rejects_inverted_range() {
        assert_eq!(
            Sweep::new(1.0, 0.0, 4).validate(),
            Err(BifurcationError::InvalidSweep("min must be <= max"))
        );
    }

    #[test]
    fn rejects_negative_tolerance() {
        assert!(matches!(
            Sweep::new(0.0, 1.0, 4).with_match_tolerance(-1.0).validate(),
            Err(BifurcationError::InvalidSweep(_))
        ));
    }
}
