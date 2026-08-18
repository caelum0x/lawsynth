use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::ControlError;

/// Designates which dataset columns are STATES and which are CONTROLS.
///
/// The distinction is the heart of SINDYc: states `x` have their derivatives
/// `ẋ` estimated and predicted, whereas controls `u` are exogenous measured
/// signals that enter the candidate library but are **never differentiated and
/// never predicted**. Any dataset column not named as a state or a control is
/// simply ignored.
///
/// Ordering is preserved exactly as given. The combined variable order
/// `[states.., controls..]` fixes the deterministic column order of the
/// augmented library `Θ(x, u)`, so identical specs yield identical term layouts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlSpec {
    states: Vec<Identifier>,
    controls: Vec<Identifier>,
}

impl ControlSpec {
    /// Builds a spec from ordered state and control identifiers.
    ///
    /// Fails when either group is empty, an identifier repeats within a group,
    /// or an identifier is claimed by both groups. Validation against a concrete
    /// dataset happens later in [`discover_controlled`](crate::discover_controlled).
    pub fn new(
        states: impl IntoIterator<Item = Identifier>,
        controls: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, ControlError> {
        let states = states.into_iter().collect::<Vec<_>>();
        let controls = controls.into_iter().collect::<Vec<_>>();
        if states.is_empty() {
            return Err(ControlError::NoStates);
        }
        if controls.is_empty() {
            return Err(ControlError::NoControls);
        }
        let mut seen = BTreeSet::new();
        for identifier in states.iter().chain(&controls) {
            if !seen.insert(identifier.clone()) {
                // A duplicate inside one group or an overlap across groups both
                // land here; distinguish them for a precise message.
                if states.contains(identifier) && controls.contains(identifier) {
                    return Err(ControlError::StateControlOverlap(identifier.to_string()));
                }
                return Err(ControlError::DuplicateIdentifier(identifier.to_string()));
            }
        }
        Ok(Self { states, controls })
    }

    /// State identifiers in caller-supplied order.
    pub fn states(&self) -> &[Identifier] {
        &self.states
    }

    /// Control identifiers in caller-supplied order.
    pub fn controls(&self) -> &[Identifier] {
        &self.controls
    }

    /// The combined library variable order `[states.., controls..]`.
    ///
    /// This is the single source of truth for augmented-library column ordering,
    /// which makes the whole pipeline deterministic.
    pub fn variables(&self) -> Vec<Identifier> {
        self.states.iter().chain(&self.controls).cloned().collect()
    }

    /// Confirms every designated identifier is present in `dataset`.
    ///
    /// Extra dataset columns not named by the spec are permitted and ignored.
    pub(crate) fn validate_against(&self, dataset: &Dataset) -> Result<(), ControlError> {
        let columns = dataset.columns();
        for identifier in self.states.iter().chain(&self.controls) {
            if !columns.contains_key(identifier) {
                return Err(ControlError::UnknownIdentifier(identifier.to_string()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn rejects_empty_states() {
        assert_eq!(ControlSpec::new([], [id("u")]), Err(ControlError::NoStates));
    }

    #[test]
    fn rejects_empty_controls() {
        assert_eq!(ControlSpec::new([id("x")], []), Err(ControlError::NoControls));
    }

    #[test]
    fn rejects_state_control_overlap() {
        assert_eq!(
            ControlSpec::new([id("x")], [id("x")]),
            Err(ControlError::StateControlOverlap("x".into()))
        );
    }

    #[test]
    fn rejects_duplicate_state() {
        assert_eq!(
            ControlSpec::new([id("x"), id("x")], [id("u")]),
            Err(ControlError::DuplicateIdentifier("x".into()))
        );
    }

    #[test]
    fn rejects_duplicate_control() {
        assert_eq!(
            ControlSpec::new([id("x")], [id("u"), id("u")]),
            Err(ControlError::DuplicateIdentifier("u".into()))
        );
    }

    #[test]
    fn preserves_variable_order_states_then_controls() {
        let spec = ControlSpec::new([id("y"), id("x")], [id("u")]).unwrap();
        let names = spec.variables().iter().map(|id| id.as_str().to_string()).collect::<Vec<_>>();
        assert_eq!(names, vec!["y", "x", "u"]);
    }
}
