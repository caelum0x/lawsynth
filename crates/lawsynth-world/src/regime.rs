use std::fmt;

use lawsynth_core::Identifier;

/// One half-open temporal interval `[start, end)` labelled with an active regime.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeInterval {
    pub regime: Identifier,
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegimeError {
    InvalidInterval,
    OverlappingIntervals,
}

impl fmt::Display for RegimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInterval => write!(
                formatter,
                "regime interval bounds must be finite and increasing"
            ),
            Self::OverlappingIntervals => write!(formatter, "regime intervals must not overlap"),
        }
    }
}

impl std::error::Error for RegimeError {}

/// A validated, chronologically sorted timeline of mutually exclusive regimes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegimeSchedule {
    intervals: Vec<RegimeInterval>,
}

impl RegimeSchedule {
    pub fn new(mut intervals: Vec<RegimeInterval>) -> Result<Self, RegimeError> {
        if intervals.iter().any(|interval| {
            !interval.start.is_finite()
                || !interval.end.is_finite()
                || interval.end <= interval.start
        }) {
            return Err(RegimeError::InvalidInterval);
        }
        intervals.sort_by(|left, right| {
            left.start
                .total_cmp(&right.start)
                .then_with(|| left.end.total_cmp(&right.end))
        });
        if intervals.windows(2).any(|pair| pair[0].end > pair[1].start) {
            return Err(RegimeError::OverlappingIntervals);
        }
        Ok(Self { intervals })
    }

    pub fn intervals(&self) -> &[RegimeInterval] {
        &self.intervals
    }

    pub fn active_at(&self, time: f64) -> Option<&RegimeInterval> {
        time.is_finite()
            .then(|| {
                self.intervals
                    .iter()
                    .find(|interval| interval.start <= time && time < interval.end)
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    #[test]
    fn regime_timeline_sorts_and_selects_half_open_intervals() {
        let id = |value| Identifier::new(value).unwrap();
        let schedule = RegimeSchedule::new(vec![
            RegimeInterval {
                regime: id("late"),
                start: 2.0,
                end: 3.0,
            },
            RegimeInterval {
                regime: id("early"),
                start: 0.0,
                end: 2.0,
            },
        ])
        .unwrap();
        assert_eq!(schedule.active_at(1.0).unwrap().regime, id("early"));
        assert_eq!(schedule.active_at(2.0).unwrap().regime, id("late"));
        assert!(schedule.active_at(3.0).is_none());
    }
}
