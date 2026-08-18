/// A single detected conserved-quantity hypothesis.
///
/// The coefficients are ordered to match [`InvariantReport::basis_labels`], so
/// `H(x) = Σ_j coefficients[j] · φ_j(x)`. The vector is canonically normalized:
/// unit Euclidean norm with the largest-magnitude entry made positive.
#[derive(Clone, Debug, PartialEq)]
pub struct Invariant {
    /// Coefficient of each labelled basis function.
    pub coefficients: Vec<f64>,
    /// The residual `‖M c‖` of the Lie-derivative matrix acting on the
    /// coefficient vector over the sample grid — how nearly `L_f H` vanishes.
    pub residual: f64,
    /// The singular value of `M` associated with this nullspace direction. A
    /// value near zero is the numerical evidence of conservation.
    pub singular_value: f64,
}

impl Invariant {
    /// Returns the coefficient on the basis function with the given label.
    pub fn coefficient(&self, labels: &[String], label: &str) -> Option<f64> {
        labels.iter().position(|candidate| candidate == label).map(|index| self.coefficients[index])
    }
}

/// The result of a conserved-quantity search.
///
/// `basis_labels` names the shared library over which every invariant's
/// coefficient vector is expressed. An empty `invariants` list is a valid,
/// honest answer: it means no conserved quantity expressible in the chosen
/// library was found within tolerance.
#[derive(Clone, Debug, PartialEq)]
pub struct InvariantReport {
    /// Labels of the candidate library, in coefficient order.
    pub basis_labels: Vec<String>,
    /// Every detected invariant, ordered by ascending singular value (the most
    /// strongly conserved first).
    pub invariants: Vec<Invariant>,
}

impl InvariantReport {
    /// A flat, order-preserving digest of every floating-point field as raw
    /// `f64` bits. Two reports are bit-identical iff their digests are equal —
    /// the determinism contract used by the test suite.
    pub fn to_bits(&self) -> Vec<u64> {
        let mut bits = Vec::new();
        for invariant in &self.invariants {
            for &coefficient in &invariant.coefficients {
                bits.push(coefficient.to_bits());
            }
            bits.push(invariant.residual.to_bits());
            bits.push(invariant.singular_value.to_bits());
        }
        bits
    }
}
