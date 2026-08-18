//! `lawsynth basins` — basin-of-attraction mapping of a multistable world.
//!
//! Loads a continuous world, reads its laws as an autonomous vector field
//! `ẋ = f(x)`, and runs the deterministic mapper of
//! [`lawsynth_basins::map_basins`] over a caller-supplied search box. It reports
//! the stable attractors located by the reused stability engine, the fraction of
//! the *settled* initial conditions that reached each one, and the honest counts
//! of trajectories that escaped the box or never settled within `max-time`.
//!
//! The search box is required: it fixes both the initial-condition grid and the
//! region past which a trajectory is deemed to have escaped. Only fixed-point
//! attractors (stable nodes/spirals) are recognized — a bounded limit cycle or
//! strange attractor reads as `Undetermined`, reported plainly rather than forced
//! into a basin.

use std::fmt::Write as _;

use lawsynth_basins::{BasinConfig, BasinReport, Label, map_basins};
use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_report::format_number;

use crate::analysis::{
    autonomous_fields, classification_label, json_string, parse_positive, parse_search_box,
    parse_usize, render_coordinates,
};

/// Help text for `lawsynth basins`.
pub fn help() -> String {
    "lawsynth basins WORLD.lsworld --box LOW:HIGH[,LOW:HIGH...] [--resolution N] \
[--dt DT] [--max-time T] [--tolerance V] [--json]\n\n\
Maps the basins of attraction of a world's autonomous vector field over the given \
search box (one LOW:HIGH interval per state). Locates the stable attractors, lays \
a deterministic grid of initial conditions, integrates each forward with fixed-step \
RK4, and classifies its fate. Reports the attractors, the per-attractor share of \
the settled initial conditions, and the counts that escaped the box or never \
settled. Only fixed-point attractors are recognized: a limit cycle or strange \
attractor reads as undetermined. --json emits a stable machine-readable report \
including the per-cell grid labels."
        .to_owned()
}

/// Runs the `basins` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }

    let mut search_box = None;
    let mut resolution = None;
    let mut dt = None;
    let mut max_time = None;
    let mut tolerance = None;
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--json" {
            as_json = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--box" => search_box = Some(parse_search_box(value)?),
            "--resolution" => resolution = Some(parse_usize(value, "--resolution")?),
            "--dt" => dt = Some(parse_positive(value, "--dt")?),
            "--max-time" => max_time = Some(parse_positive(value, "--max-time")?),
            "--tolerance" => tolerance = Some(parse_positive(value, "--tolerance")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let fields = autonomous_fields(&world);

    let search_box = search_box.ok_or_else(|| {
        format!("--box is required (one LOW:HIGH interval per state, {} state(s))", states.len())
    })?;
    let mut config = BasinConfig::new(search_box);
    if let Some(resolution) = resolution {
        config = config.with_grid_resolution(resolution);
    }
    if let Some(dt) = dt {
        config = config.with_dt(dt);
    }
    if let Some(max_time) = max_time {
        config = config.with_max_time(max_time);
    }
    if let Some(tolerance) = tolerance {
        config = config.with_convergence_tolerance(tolerance);
    }

    let report = map_basins(&fields, &states, &config).map_err(|error| error.to_string())?;
    if as_json { Ok(render_json(bundle, &report)) } else { Ok(render_text(bundle, &report)) }
}

/// Human-facing report.
fn render_text(bundle: &str, report: &BasinReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Basin mapping of {bundle}");
    let states: Vec<&str> = report.states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:     {}", states.join(", "));
    let _ = writeln!(
        out,
        "  grid:       {} sample(s) per axis, {} initial condition(s)",
        report.resolution,
        report.total()
    );
    let _ = writeln!(
        out,
        "  settled:    {}, escaped: {}, undetermined: {}",
        report.settled(),
        report.escaped,
        report.undetermined
    );
    out.push('\n');

    if report.is_empty() {
        let _ = writeln!(
            out,
            "No stable fixed-point attractor found inside the box. Every trajectory \
escaped or never settled \u{2014} widen --box, or note that a limit cycle / strange \
attractor is not recognized as a fixed point."
        );
        return out;
    }

    let _ = writeln!(out, "Attractor(s): {}", report.len());
    for (number, attractor) in report.attractors.iter().enumerate() {
        let coordinates = render_coordinates(&report.states, &attractor.coordinates);
        let _ = writeln!(out, "  #{}  ({})", number + 1, coordinates);
        let _ = writeln!(out, "      class:  {}", classification_label(attractor.classification));
        let _ = writeln!(
            out,
            "      basin:  {} of the settled initial conditions",
            format_percentage(report.fractions.get(number).copied().unwrap_or(0.0))
        );
    }
    out
}

/// Renders a fraction in `[0, 1]` as a percentage with the shared number format.
fn format_percentage(fraction: f64) -> String {
    format!("{}%", format_number(fraction * 100.0))
}

/// A stable JSON token for one grid label: `"a{index}"`, `"escaped"`, or
/// `"undetermined"`.
fn label_token(label: Label) -> String {
    match label {
        Label::Attractor(index) => format!("a{index}"),
        Label::Escaped => "escaped".to_owned(),
        Label::Undetermined => "undetermined".to_owned(),
    }
}

/// Stable, machine-readable report.
fn render_json(bundle: &str, report: &BasinReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let states: Vec<String> =
        report.states.iter().map(|state| json_string(state.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let _ = writeln!(out, "  \"resolution\": {},", report.resolution);
    let _ = writeln!(out, "  \"total\": {},", report.total());
    let _ = writeln!(out, "  \"settled\": {},", report.settled());
    let _ = writeln!(out, "  \"escaped\": {},", report.escaped);
    let _ = writeln!(out, "  \"undetermined\": {},", report.undetermined);
    let _ = writeln!(out, "  \"attractors\": [");
    for (number, attractor) in report.attractors.iter().enumerate() {
        let coordinates: Vec<String> =
            attractor.coordinates.iter().map(|value| format!("{value:.17e}")).collect();
        let fraction = report.fractions.get(number).copied().unwrap_or(0.0);
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"coordinates\": [{}],", coordinates.join(", "));
        let _ = writeln!(
            out,
            "      \"classification\": {},",
            json_string(classification_label(attractor.classification))
        );
        let _ = writeln!(out, "      \"basin_fraction\": {fraction:.17e}");
        let terminator = if number + 1 == report.attractors.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ],");
    let labels: Vec<String> =
        report.grid_labels.iter().map(|label| json_string(&label_token(*label))).collect();
    let _ = writeln!(out, "  \"grid_labels\": [{}]", labels.join(", "));
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_required_box() {
        let help = help();
        assert!(help.contains("--box"));
        assert!(help.contains("attractor"));
    }

    #[test]
    fn label_tokens_are_distinct_and_stable() {
        assert_eq!(label_token(Label::Attractor(0)), "a0");
        assert_eq!(label_token(Label::Attractor(2)), "a2");
        assert_eq!(label_token(Label::Escaped), "escaped");
        assert_eq!(label_token(Label::Undetermined), "undetermined");
    }

    #[test]
    fn percentage_uses_the_shared_formatter() {
        assert_eq!(format_percentage(0.5), "50%");
        assert_eq!(format_percentage(0.0), "0%");
    }
}
