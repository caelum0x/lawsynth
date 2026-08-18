//! `lawsynth analyze` — one-shot dynamics report combining `stability`,
//! `lyapunov`, and `invariants` on a single world.
//!
//! Loads a continuous world, reads its laws as an autonomous vector field
//! `ẋ = f(x)` (pinning every declared parameter at its stored value, exactly as
//! the sibling commands do), and runs all three deterministic engines in one
//! pass:
//!
//! - **Stability** locates the fixed points inside a required `--box` and
//!   classifies each by its Jacobian eigenvalues.
//! - **Lyapunov** estimates the spectrum from `--initial` (defaulting to the
//!   origin when omitted) and distils a plain three-way verdict — *chaotic*,
//!   *dissipative*, or *neutral/conservative* — around a small neutral band.
//! - **Invariants** searches a degree-`D` monomial library (optionally with
//!   `sin`/`cos`) for conserved quantities.
//!
//! Each part is rendered by reusing the exact renderer of the corresponding
//! standalone command, so the `--json` sub-objects are byte-for-byte the shape
//! those commands already emit and existing parsers compose over them. A part
//! that cannot run (for example a non-autonomous world, or a Lyapunov run that
//! diverges) is reported as an honest skip rather than failing the whole command;
//! only an unloadable world or a missing `--box` is a hard error.

use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_invariants::{InvariantConfig, detect_invariants};
use lawsynth_lyapunov::{LyapunovConfig, LyapunovReport, lyapunov_spectrum};
use lawsynth_report::format_number;
use lawsynth_stability::{StabilityConfig, analyze_stability};

use crate::analysis::{
    autonomous_fields, json_string, parse_positive, parse_search_box, parse_state_vector,
    parse_usize,
};

/// Half-width of the band around zero within which an exponent (or the exponent
/// sum) is treated as neither growing nor shrinking. A finite time-averaged
/// estimate never lands exactly on zero — fixed-step RK4 alone imparts a tiny
/// systematic drift to a conservative flow — so a symmetric band avoids reading
/// that numerical noise as chaos or dissipation.
const NEUTRAL_BAND: f64 = 0.05;

/// Help text for `lawsynth analyze`.
pub fn help() -> String {
    "lawsynth analyze WORLD.lsworld --box LOW:HIGH[,LOW:HIGH...] [--grid N] \
[--initial NAME=VALUE[,NAME=VALUE...]] [--dt DT] [--steps N] [--degree D] [--trig] [--json]\n\n\
Runs the three dynamics analyses on one world and prints a combined report: \
stability (fixed points and their linear classification inside the required \
--box), the Lyapunov spectrum (from --initial, defaulting to the origin) with a \
plain chaotic / dissipative / neutral-conservative verdict, and a search for \
conserved quantities over a degree-D monomial library (add --trig for sin/cos \
terms). The --box (one LOW:HIGH interval per state, in state order) is required \
and drives the stability search. If a single part cannot run — a non-autonomous \
world, or a Lyapunov run that fails — that part is skipped with a note and the \
rest still report. --json emits one object with the world, states, and a \
stability/lyapunov/invariants sub-object, each mirroring the shape the matching \
standalone command's --json already emits.\n\n\
Verdict: chaotic when the largest exponent exceeds the neutral band (0.05), \
dissipative when the exponent sum falls below that band, otherwise \
neutral/conservative."
        .to_owned()
}

/// A finished sub-analysis: its human text and machine JSON, or a skip note.
enum Section {
    /// The analysis ran; carries its text block and its `--json` object block.
    Ran { text: String, json: String },
    /// The analysis could not run; carries an honest, human-readable reason.
    Skipped { reason: String },
}

impl Section {
    /// The text to print under the section header.
    fn text(&self) -> String {
        match self {
            Section::Ran { text, .. } => text.clone(),
            Section::Skipped { reason } => format!("skipped: {reason}\n"),
        }
    }

    /// The JSON value for this section: the standalone object, or a skip note.
    fn json(&self) -> String {
        match self {
            Section::Ran { json, .. } => json.trim_end().to_owned(),
            Section::Skipped { reason } => {
                format!("{{\"skipped\": true, \"note\": {}}}", json_string(reason))
            }
        }
    }
}

/// The three-way dynamics verdict distilled from a Lyapunov spectrum.
///
/// The order is deliberate: exponential separation (chaos) is decided first, then
/// volume contraction (dissipation), leaving the neutral/conservative case for a
/// flow that neither separates nor contracts within the estimate.
fn lyapunov_verdict(report: &LyapunovReport) -> &'static str {
    if report.largest() > NEUTRAL_BAND {
        "chaotic"
    } else if report.sum() < -NEUTRAL_BAND {
        "dissipative"
    } else {
        "neutral/conservative"
    }
}

/// A one-line justification for the verdict, quoting the deciding quantities.
fn verdict_explanation(report: &LyapunovReport) -> String {
    match lyapunov_verdict(report) {
        "chaotic" => format!(
            "largest exponent {} exceeds the neutral band {}: nearby trajectories \
separate exponentially (chaos).",
            format_number(report.largest()),
            format_number(NEUTRAL_BAND)
        ),
        "dissipative" => format!(
            "exponent sum {} is below the neutral band -{}: phase-space volume \
contracts (dissipative).",
            format_number(report.sum()),
            format_number(NEUTRAL_BAND)
        ),
        _ => format!(
            "largest exponent {} within the neutral band {} and sum {} not below it: \
neither chaotic nor volume-contracting (neutral/conservative).",
            format_number(report.largest()),
            format_number(NEUTRAL_BAND),
            format_number(report.sum())
        ),
    }
}

/// Runs the `analyze` command.
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
    let mut grid = None;
    let mut initial_text = None;
    let mut dt = None;
    let mut steps = None;
    let mut degree = None;
    let mut trig = false;
    let mut as_json = false;

    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--trig" {
            trig = true;
            index += 1;
            continue;
        }
        if option == "--json" {
            as_json = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--box" => search_box = Some(parse_search_box(value)?),
            "--grid" => grid = Some(parse_usize(value, "--grid")?),
            "--initial" => initial_text = Some(value.clone()),
            "--dt" => dt = Some(parse_positive(value, "--dt")?),
            "--steps" => steps = Some(parse_usize(value, "--steps")?),
            "--degree" => degree = Some(parse_usize(value, "--degree")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    // A genuinely unloadable world is a hard error, as is a missing --box: both
    // are the caller's responsibility, not something to skip past.
    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let fields = autonomous_fields(&world);

    let search_box = search_box.ok_or_else(|| {
        format!("--box is required (one LOW:HIGH interval per state, {} state(s))", states.len())
    })?;

    // The Lyapunov initial condition defaults to the origin when omitted; the
    // origin is a documented, deterministic starting point (often a fixed point,
    // where the exponents recover the linearization's eigenvalue real parts).
    let (initial, initial_defaulted) = match &initial_text {
        Some(text) => (parse_state_vector(text, &states, "--initial")?, false),
        None => (vec![0.0; states.len()], true),
    };

    let stability = run_stability(bundle, &fields, &states, search_box, grid);
    let (lyapunov, verdict_line) =
        run_lyapunov(bundle, &fields, &states, &initial, initial_defaulted, dt, steps);
    let invariants = run_invariants(bundle, &fields, &states, degree, trig);

    if as_json {
        Ok(render_json(bundle, &states, &stability, &lyapunov, &invariants))
    } else {
        Ok(render_text(
            bundle,
            &states,
            &stability,
            &lyapunov,
            verdict_line.as_deref(),
            &invariants,
        ))
    }
}

/// Runs the stability sub-analysis, reusing the standalone renderers.
fn run_stability(
    bundle: &str,
    fields: &[(Identifier, lawsynth_expr::Expr)],
    states: &[Identifier],
    search_box: Vec<(f64, f64)>,
    grid: Option<usize>,
) -> Section {
    let mut config = StabilityConfig::new(search_box);
    if let Some(grid) = grid {
        config = config.with_grid_resolution(grid);
    }
    match analyze_stability(fields, states, &config) {
        Ok(report) => Section::Ran {
            text: crate::stability::render_text(bundle, &report),
            json: crate::stability::render_json(bundle, &report),
        },
        Err(error) => Section::Skipped { reason: error.to_string() },
    }
}

/// Runs the Lyapunov sub-analysis, returning the section plus the verdict line
/// for the text report (the JSON sub-object keeps the standalone shape exactly, so
/// the verdict lives only in the human report).
fn run_lyapunov(
    bundle: &str,
    fields: &[(Identifier, lawsynth_expr::Expr)],
    states: &[Identifier],
    initial: &[f64],
    initial_defaulted: bool,
    dt: Option<f64>,
    steps: Option<usize>,
) -> (Section, Option<String>) {
    let mut config = LyapunovConfig::default();
    if let Some(dt) = dt {
        config = config.with_step(dt);
    }
    if let Some(steps) = steps {
        config = config.with_steps(steps);
    }
    match lyapunov_spectrum(fields, states, initial, &config) {
        Ok(report) => {
            let mut text = crate::lyapunov::render_text(bundle, states, initial, &report);
            if initial_defaulted {
                let _ =
                    writeln!(text, "note: no --initial given \u{2014} defaulted to the origin.");
            }
            let verdict = lyapunov_verdict(&report);
            let line = format!("verdict: {verdict} \u{2014} {}", verdict_explanation(&report));
            (
                Section::Ran { text, json: crate::lyapunov::render_json(bundle, states, &report) },
                Some(line),
            )
        }
        Err(error) => (Section::Skipped { reason: error.to_string() }, None),
    }
}

/// Runs the invariants sub-analysis, reusing the standalone renderers.
fn run_invariants(
    bundle: &str,
    fields: &[(Identifier, lawsynth_expr::Expr)],
    states: &[Identifier],
    degree: Option<usize>,
    trig: bool,
) -> Section {
    let mut config = InvariantConfig::default();
    if let Some(degree) = degree {
        config.degree = degree;
    }
    config.include_trigonometric = trig;
    match detect_invariants(fields, states, &config) {
        Ok(report) => Section::Ran {
            text: crate::invariants::render_text(bundle, &config, &report),
            json: crate::invariants::render_json(bundle, &config, &report),
        },
        Err(error) => Section::Skipped { reason: error.to_string() },
    }
}

/// The combined human-facing report: three labelled parts plus consolidated
/// honest caveats.
fn render_text(
    bundle: &str,
    states: &[Identifier],
    stability: &Section,
    lyapunov: &Section,
    verdict_line: Option<&str>,
    invariants: &Section,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Dynamics analysis of {bundle}");
    let names: Vec<&str> = states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states: {}", names.join(", "));
    out.push('\n');

    let _ = writeln!(out, "== Stability ==");
    out.push_str(&stability.text());
    out.push('\n');

    let _ = writeln!(out, "== Lyapunov spectrum ==");
    out.push_str(&lyapunov.text());
    if let Some(line) = verdict_line {
        let _ = writeln!(out, "{line}");
    }
    out.push('\n');

    let _ = writeln!(out, "== Invariants ==");
    out.push_str(&invariants.text());
    out.push('\n');

    let _ = writeln!(out, "Caveats:");
    let _ = writeln!(
        out,
        "  - Stability: center/marginal fixed points are non-hyperbolic \u{2014} the \
linearization cannot decide them."
    );
    let _ = writeln!(
        out,
        "  - Lyapunov: a time-averaged estimate \u{2014} lengthen --steps or shrink --dt \
to sharpen it; the sum is the tightest quantity."
    );
    let _ = writeln!(
        out,
        "  - Invariants: library-bounded \u{2014} an empty result means none was found in \
the chosen library, not that none exists."
    );
    out
}

/// The combined machine-readable report. Each sub-object is the exact JSON the
/// matching standalone command emits (or a skip note), so SDK parsers written for
/// those commands compose over the sub-objects unchanged.
fn render_json(
    bundle: &str,
    states: &[Identifier],
    stability: &Section,
    lyapunov: &Section,
    invariants: &Section,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let names: Vec<String> = states.iter().map(|state| json_string(state.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", names.join(", "));
    let _ = writeln!(out, "  \"stability\": {},", stability.json());
    let _ = writeln!(out, "  \"lyapunov\": {},", lyapunov.json());
    let _ = writeln!(out, "  \"invariants\": {}", invariants.json());
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_parts_and_verdict() {
        let help = help();
        assert!(help.contains("--box"));
        assert!(help.contains("--initial"));
        assert!(help.contains("--degree"));
        assert!(help.contains("chaotic"));
        assert!(help.contains("dissipative"));
    }

    #[test]
    fn skip_section_renders_a_note_in_text_and_json() {
        let section = Section::Skipped { reason: "no field".to_owned() };
        assert!(section.text().contains("skipped: no field"));
        assert!(section.json().contains("\"skipped\": true"));
        assert!(section.json().contains("\"note\": \"no field\""));
    }
}
