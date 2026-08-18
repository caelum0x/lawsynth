//! Deterministic preset registry: a fixed-order enum of the shipped presets plus
//! lookup-by-name with an explicit error for unknown names.

use crate::catalog;
use crate::preset::DomainPreset;

/// One of the curated domain presets LawSynth ships.
///
/// The variant order defines the canonical, deterministic ordering used by
/// [`all`](Self::all) and [`names`]; it never depends on hashing or iteration
/// nondeterminism.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DomainPresetKind {
    /// Damped linear harmonic oscillator (mechanics).
    DampedOscillator,
    /// Lotka-Volterra predator-prey population dynamics.
    LotkaVolterra,
    /// Brusselator autocatalytic chemical kinetics.
    Brusselator,
}

impl DomainPresetKind {
    /// Every preset kind, in canonical (deterministic) order.
    pub const ALL: [DomainPresetKind; 3] = [
        DomainPresetKind::DampedOscillator,
        DomainPresetKind::LotkaVolterra,
        DomainPresetKind::Brusselator,
    ];

    /// The stable lowercase lookup name of the kind.
    pub fn name(self) -> &'static str {
        match self {
            DomainPresetKind::DampedOscillator => "damped-oscillator",
            DomainPresetKind::LotkaVolterra => "lotka-volterra",
            DomainPresetKind::Brusselator => "brusselator",
        }
    }

    /// Builds the fully-formed preset for this kind.
    pub fn build(self) -> DomainPreset {
        match self {
            DomainPresetKind::DampedOscillator => catalog::damped_oscillator(),
            DomainPresetKind::LotkaVolterra => catalog::lotka_volterra(),
            DomainPresetKind::Brusselator => catalog::brusselator(),
        }
    }
}

/// The error returned when a preset name does not resolve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PresetError {
    /// No preset is registered under the requested name.
    Unknown {
        /// The name that failed to resolve.
        requested: String,
        /// The available preset names, in canonical order.
        available: Vec<&'static str>,
    },
}

impl std::fmt::Display for PresetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetError::Unknown { requested, available } => write!(
                formatter,
                "unknown domain preset '{requested}'; available: {}",
                available.join(", ")
            ),
        }
    }
}

impl std::error::Error for PresetError {}

/// Every shipped preset, in canonical order.
pub fn all() -> Vec<DomainPreset> {
    DomainPresetKind::ALL.iter().map(|kind| kind.build()).collect()
}

/// The canonical, ordered list of preset names.
pub fn names() -> Vec<&'static str> {
    DomainPresetKind::ALL.iter().map(|kind| kind.name()).collect()
}

/// Looks a preset up by its stable name, or returns [`PresetError::Unknown`].
///
/// Deterministic: the match is a fixed slice scan in canonical order, never a
/// hashed lookup.
pub fn preset(name: &str) -> Result<DomainPreset, PresetError> {
    DomainPresetKind::ALL
        .iter()
        .find(|kind| kind.name() == name)
        .map(|kind| kind.build())
        .ok_or_else(|| PresetError::Unknown { requested: name.to_owned(), available: names() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_canonically_ordered_and_unique() {
        assert_eq!(names(), vec!["damped-oscillator", "lotka-volterra", "brusselator"]);
        let mut sorted = names();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names().len(), "preset names must be unique");
    }

    #[test]
    fn lookup_resolves_every_registered_name() {
        for kind in DomainPresetKind::ALL {
            let found = preset(kind.name()).expect("registered name resolves");
            assert_eq!(found.name(), kind.name());
        }
    }

    #[test]
    fn unknown_name_reports_available_presets() {
        let error = preset("navier-stokes").unwrap_err();
        match &error {
            PresetError::Unknown { requested, available } => {
                assert_eq!(requested, "navier-stokes");
                assert_eq!(available, &names());
            }
        }
        assert!(error.to_string().contains("navier-stokes"));
        assert!(error.to_string().contains("lotka-volterra"));
    }

    #[test]
    fn all_returns_one_preset_per_kind_in_order() {
        let all = all();
        assert_eq!(all.len(), DomainPresetKind::ALL.len());
        for (preset, kind) in all.iter().zip(DomainPresetKind::ALL) {
            assert_eq!(preset.name(), kind.name());
        }
    }
}
