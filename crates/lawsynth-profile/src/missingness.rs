/// Complete missingness information retained before values enter the finite
/// numerical Dataset boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingnessProfile {
    pub total: usize,
    pub missing: usize,
    pub missing_indices: Vec<usize>,
    pub longest_missing_run: usize,
}

impl MissingnessProfile {
    pub fn observed(&self) -> usize {
        self.total - self.missing
    }

    pub fn fraction(&self) -> f64 {
        if self.total == 0 { 0.0 } else { self.missing as f64 / self.total as f64 }
    }
}

/// Profiles a nullable source column without choosing an imputation policy.
pub fn profile_missingness<T>(values: &[Option<T>]) -> MissingnessProfile {
    let mut missing_indices = Vec::new();
    let mut run = 0;
    let mut longest_missing_run = 0;
    for (index, value) in values.iter().enumerate() {
        if value.is_none() {
            missing_indices.push(index);
            run += 1;
            longest_missing_run = longest_missing_run.max(run);
        } else {
            run = 0;
        }
    }
    MissingnessProfile {
        total: values.len(),
        missing: missing_indices.len(),
        missing_indices,
        longest_missing_run,
    }
}

/// Treats non-finite floating values as missing in pre-ingestion sources.
pub fn profile_f64_missingness(values: &[f64]) -> MissingnessProfile {
    profile_missingness(
        &values.iter().map(|value| value.is_finite().then_some(())).collect::<Vec<_>>(),
    )
}
