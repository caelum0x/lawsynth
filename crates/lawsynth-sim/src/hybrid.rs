use crate::SimulationError;

/// Splits a requested integration interval at finite in-range event times.
/// Duplicate event times coalesce, so event dispatch cannot create zero steps.
pub fn split_at_events(
    start: f64,
    end: f64,
    events: &[f64],
) -> Result<Vec<(f64, f64)>, SimulationError> {
    if !start.is_finite()
        || !end.is_finite()
        || end <= start
        || events.iter().any(|time| !time.is_finite() || *time < start || *time > end)
    {
        return Err(SimulationError::InvalidTimeGrid);
    }
    let mut boundaries = events.to_vec();
    boundaries.push(start);
    boundaries.push(end);
    boundaries.sort_by(f64::total_cmp);
    boundaries.dedup();
    Ok(boundaries
        .windows(2)
        .filter_map(|pair| (pair[1] > pair[0]).then_some((pair[0], pair[1])))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_boundaries_create_no_zero_length_segments() {
        assert_eq!(split_at_events(0.0, 2.0, &[1.0, 1.0]).unwrap(), vec![(0.0, 1.0), (1.0, 2.0)]);
    }
}
