use lawsynth_data::TimeAxis;

/// Sampling properties used by the discovery planner.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeProfile {
    pub start: f64,
    pub end: f64,
    pub nominal_step: f64,
    pub is_regular: bool,
}

impl TimeProfile {
    pub fn from_time_axis(time: &TimeAxis) -> Self {
        Self::from_time_axis_with_tolerance(time, 1e-9)
    }

    pub fn from_time_axis_with_tolerance(time: &TimeAxis, regularity_tolerance: f64) -> Self {
        let values = time.values();
        let nominal_step = if values.len() < 2 {
            0.0
        } else {
            (values[values.len() - 1] - values[0]) / (values.len() - 1) as f64
        };
        Self {
            start: values[0],
            end: values[values.len() - 1],
            nominal_step,
            is_regular: time.is_regular(regularity_tolerance),
        }
    }
}
