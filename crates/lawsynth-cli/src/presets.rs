//! `lawsynth presets` and the `discover --preset` flag.
//!
//! A preset is a curated, documented bundle of discovery settings tuned for a
//! problem domain. Presets set *real* discovery configuration (the same knobs a
//! user would reach for by hand) so a newcomer can get a sensible result without
//! learning every flag. Explicit CLI flags always override a preset: the preset
//! only supplies the starting values for the flags the user did not set.

use std::fmt::Write as _;

use lawsynth_discovery::SparseMethod;

/// The discovery knobs a preset can seed. These mirror the `discover` defaults
/// so `PresetSettings::default()` reproduces the plain (no-preset) behaviour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresetSettings {
    pub degree: usize,
    pub threshold: f64,
    pub include_trigonometric: bool,
    pub include_rational: bool,
    pub enable_refine: bool,
    pub sparse_method: SparseMethod,
}

impl Default for PresetSettings {
    /// The stock `discover` defaults — used when no preset is selected.
    fn default() -> Self {
        Self {
            degree: 2,
            threshold: 0.05,
            include_trigonometric: false,
            include_rational: false,
            enable_refine: false,
            sparse_method: SparseMethod::Stlsq,
        }
    }
}

/// A named, documented domain recipe.
struct Preset {
    /// Canonical name plus any aliases (all selectable with `--preset`).
    names: &'static [&'static str],
    /// One-line summary of the domain the preset targets.
    description: &'static str,
    /// Human-readable list of what the preset tunes (shown by `presets`).
    tunes: &'static [&'static str],
    /// Template systems the preset is a good starting point for.
    suits: &'static [&'static str],
    /// The concrete settings the preset seeds.
    settings: PresetSettings,
}

const PRESETS: &[Preset] = &[
    Preset {
        names: &["physics", "mechanics"],
        description: "Oscillatory & mechanical systems (polynomial + trig features)",
        tunes: &[
            "polynomial degree 3",
            "trigonometric features on (sin/cos)",
            "sparse threshold 0.05",
        ],
        suits: &["pendulum", "van-der-pol"],
        settings: PresetSettings {
            degree: 3,
            threshold: 0.05,
            include_trigonometric: true,
            include_rational: false,
            enable_refine: false,
            sparse_method: SparseMethod::Stlsq,
        },
    },
    Preset {
        names: &["ecology"],
        description: "Predator-prey & logistic interactions (quadratic cross terms)",
        tunes: &[
            "polynomial degree 2 (bilinear x*y interactions)",
            "sparse threshold 0.02 (keeps small interaction coefficients)",
        ],
        suits: &["lotka-volterra"],
        settings: PresetSettings {
            degree: 2,
            threshold: 0.02,
            include_trigonometric: false,
            include_rational: false,
            enable_refine: false,
            sparse_method: SparseMethod::Stlsq,
        },
    },
    Preset {
        names: &["epidemiology"],
        description: "Compartmental models (bilinear infection terms)",
        tunes: &[
            "polynomial degree 2 (S*I compartment coupling)",
            "sparse threshold 0.02 (keeps small transmission rates)",
        ],
        suits: &["sir"],
        settings: PresetSettings {
            degree: 2,
            threshold: 0.02,
            include_trigonometric: false,
            include_rational: false,
            enable_refine: false,
            sparse_method: SparseMethod::Stlsq,
        },
    },
    Preset {
        names: &["finance"],
        description: "Rate & pricing dynamics (rational / higher-degree, refined)",
        tunes: &[
            "polynomial degree 3",
            "rational features on",
            "joint parameter refinement on",
            "sparse threshold 0.05",
        ],
        suits: &["van-der-pol"],
        settings: PresetSettings {
            degree: 3,
            threshold: 0.05,
            include_trigonometric: false,
            include_rational: true,
            enable_refine: true,
            sparse_method: SparseMethod::Stlsq,
        },
    },
    Preset {
        names: &["general"],
        description: "Balanced default for an unknown system",
        tunes: &["polynomial degree 2", "sparse threshold 0.05", "no extra feature families"],
        suits: &["lorenz", "lotka-volterra", "sir"],
        settings: PresetSettings {
            degree: 2,
            threshold: 0.05,
            include_trigonometric: false,
            include_rational: false,
            enable_refine: false,
            sparse_method: SparseMethod::Stlsq,
        },
    },
];

fn find(name: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|preset| preset.names.contains(&name))
}

/// A sorted, comma-joined list of every selectable preset name (incl. aliases).
fn known_names() -> String {
    let mut names: Vec<&str> =
        PRESETS.iter().flat_map(|preset| preset.names.iter().copied()).collect();
    names.sort_unstable();
    names.join(", ")
}

/// Extracts a `--preset NAME` pair from `arguments`, returning the resolved
/// settings (if any) plus the remaining arguments with the pair removed.
///
/// Explicit flags in the remaining arguments override the preset because the
/// caller seeds its defaults from these settings and then parses those flags.
pub fn extract(arguments: &[String]) -> Result<(Option<PresetSettings>, Vec<String>), String> {
    let mut settings = None;
    let mut rest = Vec::with_capacity(arguments.len());
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--preset" {
            let name = arguments
                .get(index + 1)
                .ok_or_else(|| "missing value for --preset (try `lawsynth presets`)".to_owned())?;
            let preset = find(name)
                .ok_or_else(|| format!("unknown preset '{name}'; available: {}", known_names()))?;
            settings = Some(preset.settings);
            index += 2;
        } else {
            rest.push(arguments[index].clone());
            index += 1;
        }
    }
    Ok((settings, rest))
}

/// Help text for `lawsynth presets`.
pub fn help() -> String {
    "lawsynth presets\n\n\
Lists the discovery presets usable with `discover --preset <name>`. Each preset \
seeds a bundle of discovery settings tuned for a domain; explicit flags override it."
        .to_owned()
}

/// Runs the `presets` command: prints the catalog with what each tunes.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    if !arguments.is_empty() {
        return Err(help());
    }
    let mut out = String::from("Discovery presets (use with `discover --preset <name>`):\n\n");
    for preset in PRESETS {
        let (primary, aliases) = preset.names.split_first().expect("preset has a name");
        let _ = write!(out, "  {primary}");
        if !aliases.is_empty() {
            let _ = write!(out, " (alias: {})", aliases.join(", "));
        }
        out.push('\n');
        let _ = writeln!(out, "    {}", preset.description);
        let _ = writeln!(out, "    tunes:  {}", preset.tunes.join("; "));
        let _ = writeln!(out, "    suits:  {}", preset.suits.join(", "));
        out.push('\n');
    }
    let _ =
        writeln!(out, "Explicit flags (e.g. --degree, --threshold) always override the preset.");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_preset_and_leaves_other_args() {
        let args = vec![
            "data.csv".to_owned(),
            "--preset".to_owned(),
            "ecology".to_owned(),
            "--time".to_owned(),
            "t".to_owned(),
        ];
        let (settings, rest) = extract(&args).unwrap();
        assert_eq!(settings.unwrap().degree, 2);
        assert_eq!(settings.unwrap().threshold, 0.02);
        assert_eq!(rest, vec!["data.csv", "--time", "t"]);
    }

    #[test]
    fn aliases_resolve_to_same_settings() {
        let physics = find("physics").unwrap().settings;
        let mechanics = find("mechanics").unwrap().settings;
        assert_eq!(physics, mechanics);
        assert!(physics.include_trigonometric);
    }

    #[test]
    fn unknown_preset_is_rejected() {
        let args = vec!["--preset".to_owned(), "nope".to_owned()];
        let error = extract(&args).unwrap_err();
        assert!(error.contains("unknown preset"));
    }

    #[test]
    fn listing_mentions_every_preset() {
        let listing = run(&[]).unwrap();
        for preset in PRESETS {
            assert!(listing.contains(preset.names[0]));
        }
    }
}
