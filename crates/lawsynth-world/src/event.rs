use lawsynth_core::Identifier;

/// Direction used when detecting a zero crossing in a scalar event signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventDirection {
    Any,
    Rising,
    Falling,
}

/// A detected event marker with a stable identifier and finite occurrence time.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub id: Identifier,
    pub time: f64,
    pub direction: EventDirection,
}

impl Event {
    pub fn new(id: Identifier, time: f64, direction: EventDirection) -> Option<Self> {
        time.is_finite().then_some(Self { id, time, direction })
    }
}

/// Returns whether consecutive finite event-function values cross zero in the
/// requested direction. A value exactly at the first endpoint does not trigger
/// again, avoiding duplicate marks over adjacent integration intervals.
pub fn crosses_zero(previous: f64, current: f64, direction: EventDirection) -> bool {
    if !previous.is_finite() || !current.is_finite() {
        return false;
    }
    match direction {
        EventDirection::Any => {
            (previous < 0.0 && current >= 0.0) || (previous > 0.0 && current <= 0.0)
        }
        EventDirection::Rising => previous < 0.0 && current >= 0.0,
        EventDirection::Falling => previous > 0.0 && current <= 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossing_detection_obeys_direction_and_avoids_duplicates() {
        assert!(crosses_zero(-1.0, 0.0, EventDirection::Rising));
        assert!(!crosses_zero(-1.0, 0.0, EventDirection::Falling));
        assert!(crosses_zero(1.0, -0.1, EventDirection::Falling));
        assert!(!crosses_zero(0.0, 1.0, EventDirection::Any));
    }
}
