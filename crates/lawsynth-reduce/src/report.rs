//! Public reporting types. A detected reduction is a **hypothesis**, carried
//! with the residuals that justify it — never asserted as proof.

/// Whether the input columns formed a Cartesian grid that detection could use.
#[derive(Clone, Debug, PartialEq)]
pub enum GridStatus {
    /// A full tensor grid was reconstructed; `axis_lengths` are per variable.
    Reconstructed { axis_lengths: Vec<usize> },
    /// No usable grid; detection is skipped and nothing is reported.
    NotReconstructed { reason: String },
}

impl GridStatus {
    pub fn is_reconstructed(&self) -> bool {
        matches!(self, GridStatus::Reconstructed { .. })
    }
}

/// The kind of separability detected across a variable partition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeparabilityKind {
    /// `f = g(A) + h(B)`.
    Additive,
    /// `f = g(A) · h(B)`.
    Multiplicative,
}

/// A detected (and verified) separability hypothesis.
#[derive(Clone, Debug, PartialEq)]
pub struct Separability {
    pub kind: SeparabilityKind,
    /// Variables in group `A` (sorted).
    pub group_a: Vec<String>,
    /// Variables in group `B` (sorted).
    pub group_b: Vec<String>,
    /// Normalized mixed-partial screening residual (`≈ 0` when separable).
    pub screening_residual: f64,
    /// Relative reconstruction residual (`1 − R²`) of the reduced form.
    pub reconstruction_residual: f64,
    /// `1 − reconstruction_residual`, clamped to `[0, 1]`.
    pub confidence: f64,
}

/// The kind of pairwise symmetry detected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymmetryKind {
    /// `f` depends only on `x − y`.
    Difference,
    /// `f` depends only on `x + y`.
    Sum,
    /// `f` depends only on `x · y`.
    Product,
    /// `f` depends only on `x / y`.
    Ratio,
}

impl SymmetryKind {
    /// A stable order index for deterministic sorting.
    pub(crate) fn order(self) -> u8 {
        match self {
            SymmetryKind::Difference => 0,
            SymmetryKind::Sum => 1,
            SymmetryKind::Product => 2,
            SymmetryKind::Ratio => 3,
        }
    }
}

/// A detected pairwise symmetry hypothesis.
#[derive(Clone, Debug, PartialEq)]
pub struct Symmetry {
    pub kind: SymmetryKind,
    /// The two variables involved (`x`, `y`), in schema order.
    pub variables: (String, String),
    /// Normalized first-derivative invariance residual (`≈ 0` when symmetric).
    pub residual: f64,
    /// `1 − residual`, clamped to `[0, 1]`.
    pub confidence: f64,
}

/// The full, honest structural-reduction report.
#[derive(Clone, Debug, PartialEq)]
pub struct ReductionReport {
    /// The column treated as the scalar target `f`.
    pub target: String,
    /// The input variables, in schema (sorted) order.
    pub variables: Vec<String>,
    /// Grid reconstruction status.
    pub grid: GridStatus,
    /// Detected separabilities, best (highest confidence) first.
    pub separabilities: Vec<Separability>,
    /// Detected symmetries, in deterministic order.
    pub symmetries: Vec<Symmetry>,
}

impl ReductionReport {
    /// True when no structural reduction was found above tolerance.
    pub fn is_empty(&self) -> bool {
        self.separabilities.is_empty() && self.symmetries.is_empty()
    }
}

/// Clamps a confidence-from-residual value to `[0, 1]`.
pub(crate) fn confidence_from_residual(residual: f64) -> f64 {
    (1.0 - residual).clamp(0.0, 1.0)
}
