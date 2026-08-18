//! The report produced by basin mapping.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_stability::Classification;

/// A stable attractor whose basin is being mapped.
///
/// Attractors are exactly the STABLE fixed points located by
/// [`lawsynth_stability::analyze_stability`] — stable nodes and stable spirals.
/// Saddles, unstable points, and non-hyperbolic (`Center`/`Marginal`) points are
/// not attractors and are never listed here.
#[derive(Clone, Debug, PartialEq)]
pub struct Attractor {
    /// The fixed-point coordinates `x*`, in `states` order.
    pub coordinates: Vec<f64>,
    /// The linear-stability class (`StableNode` or `StableSpiral`).
    pub classification: Classification,
}

/// The fate of one initial condition under the forward flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    /// Converged to the attractor at this index in [`BasinReport::attractors`].
    Attractor(usize),
    /// Left the search box (padded by the escape margin) or diverged. Note that
    /// a bounded non-fixed-point attractor — a limit cycle or strange attractor —
    /// is not recognized and, if it stays bounded, reads as `Undetermined`.
    Escaped,
    /// Neither converged nor escaped within `max_time`. An honest "did not settle
    /// in the time given" — never coerced into a basin.
    Undetermined,
}

/// The result of [`crate::map_basins`].
///
/// The report is honest about the search: `escaped` and `undetermined` count the
/// initial conditions that did not settle onto a recognized attractor, and the
/// `fractions` are taken over the *settled* population only, so they read as
/// "of the trajectories that reached an attractor, this share reached each one".
#[derive(Clone, Debug, PartialEq)]
pub struct BasinReport {
    /// The state ordering that indexes every coordinate vector.
    pub states: Vec<Identifier>,
    /// The stable attractors whose basins were mapped, in the order
    /// `lawsynth-stability` reports them (lexicographic by coordinate).
    pub attractors: Vec<Attractor>,
    /// One label per initial condition, in the grid's row-major order.
    pub grid_labels: Vec<Label>,
    /// Per-attractor basin fraction: `count(attractor i) / settled_total`, where
    /// `settled_total` excludes escaped and undetermined trajectories. If nothing
    /// settled, every fraction is `0.0`.
    pub fractions: Vec<f64>,
    /// How many initial conditions escaped the box / diverged.
    pub escaped: usize,
    /// How many initial conditions never settled within `max_time`.
    pub undetermined: usize,
    /// The number of initial-condition samples per axis.
    pub resolution: usize,
    /// The search box over which the grid was laid and escape was judged.
    pub search_box: Vec<(f64, f64)>,
}

impl BasinReport {
    /// The number of stable attractors whose basins were mapped.
    pub fn len(&self) -> usize {
        self.attractors.len()
    }

    /// Whether no stable attractor was found (so every trajectory is escaped or
    /// undetermined).
    pub fn is_empty(&self) -> bool {
        self.attractors.is_empty()
    }

    /// The total number of classified initial conditions (grid size).
    pub fn total(&self) -> usize {
        self.grid_labels.len()
    }

    /// How many initial conditions settled onto some attractor.
    pub fn settled(&self) -> usize {
        self.grid_labels.iter().filter(|label| matches!(label, Label::Attractor(_))).count()
    }

    /// A stable textual fingerprint of the whole report, encoding every float by
    /// its `f64` bit pattern. Two runs on identical input MUST produce identical
    /// strings; this is the basis of the determinism guarantee.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        output.push_str("states:");
        for state in &self.states {
            output.push_str(state.as_str());
            output.push(',');
        }
        let _ = write!(output, "\nresolution={} box:", self.resolution);
        for &(lower, upper) in &self.search_box {
            let _ = write!(output, "[{:016x},{:016x}]", lower.to_bits(), upper.to_bits());
        }
        let _ = writeln!(output, "\nescaped={} undetermined={}", self.escaped, self.undetermined);
        for attractor in &self.attractors {
            output.push_str("attractor coords:");
            for coordinate in &attractor.coordinates {
                let _ = write!(output, "{:016x},", coordinate.to_bits());
            }
            let _ = writeln!(output, " class={:?}", attractor.classification);
        }
        output.push_str("fractions:");
        for fraction in &self.fractions {
            let _ = write!(output, "{:016x},", fraction.to_bits());
        }
        output.push_str("\nlabels:");
        for label in &self.grid_labels {
            match label {
                Label::Attractor(index) => {
                    let _ = write!(output, "A{index},");
                }
                Label::Escaped => output.push_str("E,"),
                Label::Undetermined => output.push_str("U,"),
            }
        }
        output.push('\n');
        output
    }
}
