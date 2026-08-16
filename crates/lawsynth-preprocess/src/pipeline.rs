use lawsynth_data::{Dataset, TimeAxis};

use crate::{
    AppliedTransform, PreprocessError, PreprocessStep, detrend_linear, moving_average,
    resample_linear_with_report, standardize,
};

/// Ordered, deterministic preprocessing operations with step-by-step provenance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreprocessPipeline {
    steps: Vec<PreprocessStep>,
}

impl PreprocessPipeline {
    pub fn new(steps: impl IntoIterator<Item = PreprocessStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    pub fn steps(&self) -> &[PreprocessStep] {
        &self.steps
    }

    pub fn apply(
        &self,
        input: &Dataset,
    ) -> Result<(Dataset, Vec<AppliedTransform>), PreprocessError> {
        let mut dataset = input.clone();
        let mut reports = Vec::with_capacity(self.steps.len());
        for step in &self.steps {
            match step {
                PreprocessStep::MovingAverage { radius } => {
                    let (next, report) = moving_average(&dataset, *radius)?;
                    dataset = next;
                    reports.push(AppliedTransform::MovingAverage(report));
                }
                PreprocessStep::DetrendLinear => {
                    let (next, report) = detrend_linear(&dataset)?;
                    dataset = next;
                    reports.push(AppliedTransform::DetrendLinear(report));
                }
                PreprocessStep::ResampleLinear { target_time } => {
                    let time = TimeAxis::new(target_time.clone())
                        .map_err(|_| PreprocessError::InvalidTargetTime)?;
                    let (next, report) = resample_linear_with_report(&dataset, time)?;
                    dataset = next;
                    reports.push(AppliedTransform::ResampleLinear(report));
                }
                PreprocessStep::Standardize => {
                    let (next, report) = standardize(&dataset)?;
                    dataset = next;
                    reports.push(AppliedTransform::Standardize(report));
                }
            }
        }
        Ok((dataset, reports))
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::NumericColumn;

    use super::*;

    #[test]
    fn applies_transforms_in_order_with_chained_provenance() {
        let input = Dataset::new(
            TimeAxis::new(vec![0.0, 2.0]).unwrap(),
            [NumericColumn::new(
                Identifier::new("x").unwrap(),
                vec![0.0, 4.0],
            )],
        )
        .unwrap();
        let pipeline = PreprocessPipeline::new([
            PreprocessStep::ResampleLinear {
                target_time: vec![0.0, 1.0, 2.0],
            },
            PreprocessStep::Standardize,
        ]);
        let (output, reports) = pipeline.apply(&input).unwrap();
        assert_eq!(reports.len(), 2);
        assert_eq!(output.time().values(), &[0.0, 1.0, 2.0]);
        assert_eq!(
            output.columns()[&Identifier::new("x").unwrap()].values,
            vec![-1.224_744_871_391_589, 0.0, 1.224_744_871_391_589]
        );
        let AppliedTransform::ResampleLinear(resample) = &reports[0] else {
            panic!("expected resampling report");
        };
        let AppliedTransform::Standardize(scale) = &reports[1] else {
            panic!("expected standardization report");
        };
        assert_eq!(resample.output_fingerprint, scale.input_fingerprint);
    }
}
