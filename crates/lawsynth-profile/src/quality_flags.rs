use crate::{ProfileError, distribution};

/// Robust data-quality diagnostics for one finite numeric column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnQuality {
    pub is_constant: bool,
    pub outlier_indices: Vec<usize>,
}

/// Identifies constant series and Tukey-IQR outliers deterministically.
pub fn quality_flags(values: &[f64]) -> Result<ColumnQuality, ProfileError> {
    let summary = distribution(values)?;
    let is_constant = summary.minimum == summary.maximum;
    let iqr = summary.third_quartile - summary.first_quartile;
    let outlier_indices = if iqr <= f64::EPSILON {
        Vec::new()
    } else {
        let lower = summary.first_quartile - 1.5 * iqr;
        let upper = summary.third_quartile + 1.5 * iqr;
        values
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value < lower || *value > upper).then_some(index))
            .collect()
    };
    Ok(ColumnQuality { is_constant, outlier_indices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_constant_columns_and_tukey_outliers() {
        assert!(quality_flags(&[2.0, 2.0, 2.0]).unwrap().is_constant);
        assert_eq!(quality_flags(&[1.0, 1.0, 2.0, 2.0, 100.0]).unwrap().outlier_indices, vec![4]);
    }
}
