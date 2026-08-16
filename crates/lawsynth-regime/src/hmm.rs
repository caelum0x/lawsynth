use crate::{RegimeError, Result};
#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteHmm {
    pub initial: Vec<f64>,
    pub transition: Vec<Vec<f64>>,
    pub emission: Vec<Vec<f64>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViterbiPath {
    pub states: Vec<usize>,
}
impl DiscreteHmm {
    pub fn validate(&self) -> Result<()> {
        let states = self.initial.len();
        if states == 0 || self.transition.len() != states || self.emission.len() != states {
            return Err(RegimeError::DimensionMismatch {
                expected: states,
                actual: self.transition.len(),
            });
        }
        validate_distribution(&self.initial)?;
        let symbols = self.emission[0].len();
        if symbols == 0 {
            return Err(RegimeError::InvalidParameter("emission"));
        }
        for row in &self.transition {
            if row.len() != states {
                return Err(RegimeError::DimensionMismatch {
                    expected: states,
                    actual: row.len(),
                });
            }
            validate_distribution(row)?;
        }
        for row in &self.emission {
            if row.len() != symbols {
                return Err(RegimeError::DimensionMismatch {
                    expected: symbols,
                    actual: row.len(),
                });
            }
            validate_distribution(row)?;
        }
        Ok(())
    }
    pub fn viterbi(&self, observations: &[usize]) -> Result<ViterbiPath> {
        self.validate()?;
        if observations.is_empty() {
            return Err(RegimeError::EmptySeries);
        }
        let states = self.initial.len();
        for &o in observations {
            if o >= self.emission[0].len() {
                return Err(RegimeError::InvalidParameter("observation"));
            }
        }
        let log = |p: f64| if p == 0.0 { f64::NEG_INFINITY } else { p.ln() };
        let mut score: Vec<f64> = (0..states)
            .map(|s| log(self.initial[s]) + log(self.emission[s][observations[0]]))
            .collect();
        if score.iter().all(|v| !v.is_finite()) {
            return Err(RegimeError::ImpossibleObservation { index: 0 });
        }
        let mut back = Vec::new();
        for (index, &obs) in observations.iter().enumerate().skip(1) {
            let mut next = vec![f64::NEG_INFINITY; states];
            let mut predecessor = vec![0; states];
            for to in 0..states {
                for (from, &previous_score) in score.iter().enumerate() {
                    let v = previous_score
                        + log(self.transition[from][to])
                        + log(self.emission[to][obs]);
                    if v > next[to] {
                        next[to] = v;
                        predecessor[to] = from;
                    }
                }
            }
            if next.iter().all(|v| !v.is_finite()) {
                return Err(RegimeError::ImpossibleObservation { index });
            }
            score = next;
            back.push(predecessor);
        }
        let mut state = score
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let mut path = vec![state];
        for predecessor in back.iter().rev() {
            state = predecessor[state];
            path.push(state);
        }
        path.reverse();
        Ok(ViterbiPath { states: path })
    }
}
fn validate_distribution(values: &[f64]) -> Result<()> {
    if values.iter().any(|p| !p.is_finite() || *p < 0.0)
        || (values.iter().sum::<f64>() - 1.0).abs() > 1e-9
    {
        Err(RegimeError::InvalidProbability)
    } else {
        Ok(())
    }
}
