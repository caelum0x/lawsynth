//! Grammar-constrained candidate libraries — *template priors*.
//!
//! A [`TemplatePrior`] is a declarative, immutable description of which candidate
//! terms are admissible *before* the sparse solve. It is a **hard admissibility
//! filter over candidate terms**, applied deterministically to a materialised
//! feature library. It encodes a domain/physical prior (e.g. "rational of total
//! degree ≤ 2", "no cross terms", "only these variables") and shrinks the
//! candidate set to improve identifiability.
//!
//! ## What a prior IS (and is NOT)
//!
//! - It restricts the *candidate library* the solver may draw from. A term the
//!   filter drops can never appear in a discovered law.
//! - It is **not** a soft penalty, and **not** a proof that the true law satisfies
//!   the prior. If the prior excludes a term the true law needs, discovery cannot
//!   recover that law — that is the user's stated, and now enforced, assumption.
//! - Every drop is recorded with a reason in a [`TemplateFilterReport`], so the
//!   narrowing is always auditable — a prior never silently shrinks the search.
//!
//! ## Per-rule guarantees
//!
//! | Rule | Guarantee |
//! |---|---|
//! | `max_total_degree(d)` | No admitted term has total degree `> d`. |
//! | `allowed_variables(V)` | Every admitted term references only variables in `V`. |
//! | `allowed_kinds(K)` | Every admitted term's [`TermKind`] is in `K`. |
//! | `forbid_interactions` | No admitted term references two or more distinct variables. |
//! | `max_active_terms(n)` | At most `n` candidate terms survive (first `n` in library order), so a discovered law has at most `n` active terms. |
//! | `required_kinds(K)` | The *admitted candidate set* contains at least one term of every kind in `K`, or the prior is rejected as unsatisfiable. |
//!
//! The `required_kinds` rule constrains the **candidate set**: it guarantees the
//! solver is *offered* a term of that kind, not that the final selected law keeps
//! it (a sparse solve may still zero its coefficient). This is deliberately the
//! strongest honest guarantee a pre-solve candidate filter can make.

use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_features::FeatureTerm;

/// The structural family a candidate term belongs to.
///
/// Classification is priority-ordered and total: every term maps to exactly one
/// kind. A term is [`Rational`](TermKind::Rational) if it contains any division,
/// else [`Trigonometric`](TermKind::Trigonometric) if it contains a sine/cosine,
/// else [`Exponential`](TermKind::Exponential) if it contains an exp/log, else
/// [`Constant`](TermKind::Constant) when it has no variables, else
/// [`Polynomial`](TermKind::Polynomial).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TermKind {
    /// A constant (variable-free) term, e.g. the polynomial intercept `1`.
    Constant,
    /// A monomial in the variables, e.g. `x`, `x·x`, `x·y`.
    Polynomial,
    /// A term containing a division, e.g. the bounded rational `x / (1 + x²)`.
    Rational,
    /// A term containing a sine or cosine, e.g. `sin(x)`.
    Trigonometric,
    /// A term containing an exponential or logarithm, e.g. `exp(x)`.
    Exponential,
}

impl TermKind {
    /// Classifies a candidate expression into its single structural family.
    pub fn of(expression: &Expr) -> Self {
        if contains(expression, &|expr| {
            matches!(expr, Expr::Binary { operator: BinaryOperator::Divide, .. })
        }) {
            Self::Rational
        } else if contains(expression, &|expr| {
            matches!(expr, Expr::Unary { operator: UnaryOperator::Sin | UnaryOperator::Cos, .. })
        }) {
            Self::Trigonometric
        } else if contains(expression, &|expr| {
            matches!(expr, Expr::Unary { operator: UnaryOperator::Exp | UnaryOperator::Log, .. })
        }) {
            Self::Exponential
        } else if symbols(expression).is_empty() {
            Self::Constant
        } else {
            Self::Polynomial
        }
    }
}

/// The reason a single candidate term was rejected by a [`TemplatePrior`].
///
/// Exactly one reason is recorded per dropped term — the first rule it violates
/// in the fixed evaluation order (degree, variables, kind, interactions, then the
/// global active-term cap), so a report is deterministic and never double-counts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DropReason {
    /// The term's total degree exceeded `max`.
    DegreeExceeded { degree: usize, max: usize },
    /// The term referenced `variable`, which is not in the allowed set.
    DisallowedVariable { variable: Identifier },
    /// The term's structural [`TermKind`] is not in the allowed set.
    DisallowedKind { kind: TermKind },
    /// The term references two or more distinct variables while interactions are
    /// forbidden.
    InteractionForbidden { variables: usize },
    /// The term was admissible on its own merits but fell outside the first
    /// `limit` terms retained by `max_active_terms`.
    MaxActiveExceeded { limit: usize },
}

/// A candidate term dropped by the filter, paired with why it was dropped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DroppedTerm {
    /// The human-readable name of the dropped candidate term.
    pub name: String,
    /// The (single, first-violated) reason it was dropped.
    pub reason: DropReason,
}

/// An auditable tally of one application of a [`TemplatePrior`].
///
/// `considered` counts every candidate term the filter inspected, `admitted` the
/// subset retained, and `dropped` lists every rejected term with its reason. By
/// construction `considered == admitted + dropped.len()`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TemplateFilterReport {
    /// Total candidate terms inspected.
    pub considered: usize,
    /// Candidate terms retained as admissible.
    pub admitted: usize,
    /// Every dropped term with the reason it was rejected, in library order.
    pub dropped: Vec<DroppedTerm>,
}

impl TemplateFilterReport {
    /// Number of dropped terms; equal to `considered - admitted`.
    pub fn dropped_count(&self) -> usize {
        self.dropped.len()
    }
}

/// The outcome of filtering a candidate library through a [`TemplatePrior`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateSelection {
    /// Indices (ascending) into the input terms that survived the prior.
    pub admitted: Vec<usize>,
    /// The auditable drop report for this application.
    pub report: TemplateFilterReport,
}

/// Failure to satisfy a [`TemplatePrior`]'s hard requirements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplateError {
    /// A `required_kinds` entry could not be satisfied: no admissible candidate
    /// term of this kind survived (the base library had none, or every one was
    /// dropped by another rule). The prior is unsatisfiable against this library.
    UnsatisfiableRequiredKind(TermKind),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsatisfiableRequiredKind(kind) => write!(
                formatter,
                "template prior requires a {kind:?} term but the candidate library admits none",
            ),
        }
    }
}

impl std::error::Error for TemplateError {}

/// A declarative, immutable prior over admissible candidate terms.
///
/// Construct with [`TemplatePrior::unconstrained`] (a no-op prior that admits
/// everything) and refine with the chained `with_*` / `forbidding_*` /
/// `requiring_*` builders. All fields are private; equality and cloning are
/// structural.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TemplatePrior {
    max_total_degree: Option<usize>,
    allowed_variables: Option<BTreeSet<Identifier>>,
    allowed_kinds: Option<BTreeSet<TermKind>>,
    forbid_interactions: bool,
    required_kinds: BTreeSet<TermKind>,
    max_active_terms: Option<usize>,
}

impl TemplatePrior {
    /// A prior that constrains nothing: applying it admits every candidate term
    /// and drops none. Equivalent, in the pipeline, to supplying no prior at all.
    pub fn unconstrained() -> Self {
        Self::default()
    }

    /// Caps the total degree of admissible terms. Total degree is defined
    /// recursively: a constant is `0`, a variable is `1`, a product adds the
    /// degrees of its factors, a sum/quotient takes the max of its sides, and a
    /// transcendental (`sin`, `exp`, …) takes the degree of its argument. Thus
    /// `x·y` and `x / (1 + x²)` both have degree `2`, and `sin(x)` has degree `1`.
    pub fn with_max_total_degree(mut self, degree: usize) -> Self {
        self.max_total_degree = Some(degree);
        self
    }

    /// Restricts admissible terms to those referencing only `variables`. A term
    /// with no variables (a constant) trivially satisfies any whitelist.
    pub fn with_allowed_variables(
        mut self,
        variables: impl IntoIterator<Item = Identifier>,
    ) -> Self {
        self.allowed_variables = Some(variables.into_iter().collect());
        self
    }

    /// Restricts admissible terms to the given structural [`TermKind`]s.
    pub fn with_allowed_kinds(mut self, kinds: impl IntoIterator<Item = TermKind>) -> Self {
        self.allowed_kinds = Some(kinds.into_iter().collect());
        self
    }

    /// Drops every term that references two or more distinct variables (a
    /// cross / interaction term), keeping only single-variable and constant terms.
    pub fn forbidding_interactions(mut self) -> Self {
        self.forbid_interactions = true;
        self
    }

    /// Requires the admissible candidate set to contain at least one term of
    /// `kind`; otherwise [`admissible`](Self::admissible) fails with
    /// [`TemplateError::UnsatisfiableRequiredKind`]. This guarantees the solver is
    /// *offered* such a term — not that the final law retains it.
    pub fn requiring_kind(mut self, kind: TermKind) -> Self {
        self.required_kinds.insert(kind);
        self
    }

    /// Caps the number of admissible candidate terms at `limit`, keeping the first
    /// `limit` in the library's own (deterministic) order and dropping the rest.
    /// Because a discovered law's active terms are a subset of the candidates,
    /// this bounds the law to at most `limit` active terms. The standard
    /// polynomial library is ordered by ascending degree, so the retained terms
    /// are the simplest.
    pub fn with_max_active_terms(mut self, limit: usize) -> Self {
        self.max_active_terms = Some(limit);
        self
    }

    /// The configured total-degree cap, if any.
    pub fn max_total_degree(&self) -> Option<usize> {
        self.max_total_degree
    }

    /// The configured active-term cap, if any.
    pub fn max_active_terms(&self) -> Option<usize> {
        self.max_active_terms
    }

    /// Applies the prior to `terms` as a pure, deterministic filter.
    ///
    /// Returns the admissible term indices (ascending) plus an auditable drop
    /// report, or an error when a `required_kinds` entry cannot be satisfied. This
    /// function reads no clock, draws no randomness, and iterates only in stable
    /// slice order, so identical input yields identical output.
    pub fn admissible(&self, terms: &[FeatureTerm]) -> Result<TemplateSelection, TemplateError> {
        let mut admitted = Vec::new();
        let mut dropped = Vec::new();

        // Per-term rules, evaluated in a fixed priority order; the first violated
        // rule is the recorded reason.
        for (index, term) in terms.iter().enumerate() {
            match self.per_term_reason(&term.expression) {
                Some(reason) => {
                    dropped.push(DroppedTerm { name: term.name.clone(), reason });
                }
                None => admitted.push(index),
            }
        }

        // Global active-term cap: keep the first `limit` survivors in order, drop
        // the tail. Applied after per-term rules so the cap counts only otherwise
        // admissible terms.
        if let Some(limit) = self.max_active_terms {
            while admitted.len() > limit {
                let index = admitted.pop().expect("length checked above");
                dropped.push(DroppedTerm {
                    name: terms[index].name.clone(),
                    reason: DropReason::MaxActiveExceeded { limit },
                });
            }
        }

        // Enforce required kinds against the *final* admitted set so conflicts
        // with any rule (including the active-term cap) surface honestly.
        for &kind in &self.required_kinds {
            let satisfied =
                admitted.iter().any(|&index| TermKind::of(&terms[index].expression) == kind);
            if !satisfied {
                return Err(TemplateError::UnsatisfiableRequiredKind(kind));
            }
        }

        // `dropped` is assembled per-term then per-cap; sort by original library
        // index for a stable, order-independent report.
        dropped.sort_by(|left, right| left.name.cmp(&right.name));
        let report =
            TemplateFilterReport { considered: terms.len(), admitted: admitted.len(), dropped };
        Ok(TemplateSelection { admitted, report })
    }

    /// Evaluates the per-term rules in priority order, returning the first
    /// violation or `None` when the term is admissible.
    fn per_term_reason(&self, expression: &Expr) -> Option<DropReason> {
        if let Some(max) = self.max_total_degree {
            let degree = total_degree(expression);
            if degree > max {
                return Some(DropReason::DegreeExceeded { degree, max });
            }
        }
        if let Some(allowed) = &self.allowed_variables {
            // Deterministic: report the smallest disallowed variable.
            if let Some(variable) = symbols(expression).into_iter().find(|id| !allowed.contains(id))
            {
                return Some(DropReason::DisallowedVariable { variable });
            }
        }
        if let Some(allowed) = &self.allowed_kinds {
            let kind = TermKind::of(expression);
            if !allowed.contains(&kind) {
                return Some(DropReason::DisallowedKind { kind });
            }
        }
        if self.forbid_interactions {
            let variables = symbols(expression).len();
            if variables >= 2 {
                return Some(DropReason::InteractionForbidden { variables });
            }
        }
        None
    }
}

/// Total polynomial-style degree of an expression (see
/// [`TemplatePrior::with_max_total_degree`] for the exact definition).
fn total_degree(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) => 0,
        Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => total_degree(operand),
        Expr::Binary { operator, left, right } => match operator {
            BinaryOperator::Multiply => total_degree(left) + total_degree(right),
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Divide => {
                total_degree(left).max(total_degree(right))
            }
            BinaryOperator::Power => match right.as_ref() {
                Expr::Constant(exponent) if *exponent >= 0.0 && exponent.fract() == 0.0 => {
                    total_degree(left) * (*exponent as usize)
                }
                _ => total_degree(left),
            },
        },
    }
}

/// The set of distinct variables referenced by an expression.
fn symbols(expression: &Expr) -> BTreeSet<Identifier> {
    let mut found = BTreeSet::new();
    collect_symbols(expression, &mut found);
    found
}

fn collect_symbols(expression: &Expr, found: &mut BTreeSet<Identifier>) {
    match expression {
        Expr::Constant(_) => {}
        Expr::Symbol(symbol) => {
            found.insert(symbol.clone());
        }
        Expr::Unary { operand, .. } => collect_symbols(operand, found),
        Expr::Binary { left, right, .. } => {
            collect_symbols(left, found);
            collect_symbols(right, found);
        }
    }
}

/// Returns whether any node in the expression tree satisfies `predicate`.
fn contains(expression: &Expr, predicate: &impl Fn(&Expr) -> bool) -> bool {
    if predicate(expression) {
        return true;
    }
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => false,
        Expr::Unary { operand, .. } => contains(operand, predicate),
        Expr::Binary { left, right, .. } => contains(left, predicate) || contains(right, predicate),
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_features::FeatureLibrary;

    use super::*;

    fn ident(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    /// Polynomial(deg 2) + trig + rational library over `x, y`, matching the
    /// discovery pipeline's materialised candidate set.
    fn library() -> Vec<FeatureTerm> {
        let (x, y) = (ident("x"), ident("y"));
        let mut lib = FeatureLibrary::polynomial([x.clone(), y.clone()], 2, true).unwrap();
        lib.extend(FeatureLibrary::trigonometric([x.clone(), y.clone()]).unwrap());
        lib.extend(FeatureLibrary::bounded_rational([x, y]).unwrap());
        lib.terms().to_vec()
    }

    fn names(selection: &TemplateSelection, terms: &[FeatureTerm]) -> Vec<String> {
        selection.admitted.iter().map(|&i| terms[i].name.clone()).collect()
    }

    #[test]
    fn total_degree_treats_products_rationals_and_transcendentals() {
        let (x, y) = (ident("x"), ident("y"));
        assert_eq!(total_degree(&Expr::constant(1.0)), 0);
        assert_eq!(total_degree(&Expr::symbol(x.clone())), 1);
        assert_eq!(
            total_degree(&Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone()))),
            2
        );
        // x / (1 + x*x) -> max(1, max(0, 2)) = 2
        let rational = Expr::quotient(
            Expr::symbol(x.clone()),
            Expr::sum(
                Expr::constant(1.0),
                Expr::product(Expr::symbol(x.clone()), Expr::symbol(x.clone())),
            ),
        );
        assert_eq!(total_degree(&rational), 2);
        assert_eq!(total_degree(&Expr::unary(UnaryOperator::Sin, Expr::symbol(x))), 1);
    }

    #[test]
    fn classifies_every_term_kind() {
        let x = ident("x");
        assert_eq!(TermKind::of(&Expr::constant(1.0)), TermKind::Constant);
        assert_eq!(TermKind::of(&Expr::symbol(x.clone())), TermKind::Polynomial);
        assert_eq!(
            TermKind::of(&Expr::unary(UnaryOperator::Cos, Expr::symbol(x.clone()))),
            TermKind::Trigonometric
        );
        assert_eq!(
            TermKind::of(&Expr::unary(UnaryOperator::Exp, Expr::symbol(x.clone()))),
            TermKind::Exponential
        );
        assert_eq!(
            TermKind::of(&Expr::quotient(Expr::symbol(x.clone()), Expr::constant(2.0))),
            TermKind::Rational
        );
    }

    #[test]
    fn unconstrained_prior_admits_everything() {
        let terms = library();
        let selection = TemplatePrior::unconstrained().admissible(&terms).unwrap();
        assert_eq!(selection.admitted.len(), terms.len());
        assert_eq!(selection.report.dropped_count(), 0);
        assert_eq!(selection.report.considered, terms.len());
        assert_eq!(selection.report.admitted, terms.len());
    }

    #[test]
    fn degree_cap_drops_higher_degree_terms() {
        let terms = library();
        // Degree 1 keeps: 1, y, x, sin/cos(x), sin/cos(y); drops x*x, x*y, y*y and
        // both rationals (rational degree is 2).
        let prior = TemplatePrior::unconstrained().with_max_total_degree(1);
        let selection = prior.admissible(&terms).unwrap();
        for &index in &selection.admitted {
            assert!(total_degree(&terms[index].expression) <= 1);
        }
        assert!(
            selection
                .report
                .dropped
                .iter()
                .all(|d| matches!(d.reason, DropReason::DegreeExceeded { .. }))
        );
        assert_eq!(selection.report.admitted + selection.report.dropped_count(), terms.len());
    }

    #[test]
    fn variable_whitelist_drops_foreign_variables() {
        let terms = library();
        let prior = TemplatePrior::unconstrained().with_allowed_variables([ident("x")]);
        let selection = prior.admissible(&terms).unwrap();
        for &index in &selection.admitted {
            assert!(symbols(&terms[index].expression).iter().all(|id| id.as_str() == "x"));
        }
        // Every drop is a disallowed `y`.
        assert!(selection.report.dropped.iter().all(|d| matches!(
            &d.reason,
            DropReason::DisallowedVariable { variable } if variable.as_str() == "y"
        )));
    }

    #[test]
    fn kind_allowlist_keeps_only_requested_families() {
        let terms = library();
        let prior = TemplatePrior::unconstrained()
            .with_allowed_kinds([TermKind::Trigonometric, TermKind::Constant]);
        let selection = prior.admissible(&terms).unwrap();
        for &index in &selection.admitted {
            let kind = TermKind::of(&terms[index].expression);
            assert!(matches!(kind, TermKind::Trigonometric | TermKind::Constant));
        }
        assert!(names(&selection, &terms).iter().any(|n| n.contains("sin")));
    }

    #[test]
    fn forbidding_interactions_drops_cross_terms() {
        let terms = library();
        let prior = TemplatePrior::unconstrained().forbidding_interactions();
        let selection = prior.admissible(&terms).unwrap();
        for &index in &selection.admitted {
            assert!(symbols(&terms[index].expression).len() < 2);
        }
        // x*y is the only degree-2 cross term in this library.
        assert!(
            selection
                .report
                .dropped
                .iter()
                .any(|d| matches!(d.reason, DropReason::InteractionForbidden { variables: 2 }))
        );
    }

    #[test]
    fn max_active_terms_keeps_first_n_in_order() {
        let terms = library();
        let prior = TemplatePrior::unconstrained().with_max_active_terms(3);
        let selection = prior.admissible(&terms).unwrap();
        assert_eq!(selection.admitted, vec![0, 1, 2]);
        assert_eq!(selection.report.dropped_count(), terms.len() - 3);
        assert!(
            selection
                .report
                .dropped
                .iter()
                .all(|d| matches!(d.reason, DropReason::MaxActiveExceeded { limit: 3 }))
        );
    }

    #[test]
    fn required_kind_present_is_satisfied() {
        let terms = library();
        let prior = TemplatePrior::unconstrained().requiring_kind(TermKind::Trigonometric);
        assert!(prior.admissible(&terms).is_ok());
    }

    #[test]
    fn required_kind_absent_is_unsatisfiable() {
        let terms = library();
        // Allow only polynomials, then require a trig term -> conflict.
        let prior = TemplatePrior::unconstrained()
            .with_allowed_kinds([TermKind::Polynomial, TermKind::Constant])
            .requiring_kind(TermKind::Trigonometric);
        assert_eq!(
            prior.admissible(&terms),
            Err(TemplateError::UnsatisfiableRequiredKind(TermKind::Trigonometric))
        );
    }

    #[test]
    fn combined_rules_intersect() {
        let terms = library();
        // "Rational of total degree <= 2": rational kind AND degree <= 2.
        let prior = TemplatePrior::unconstrained()
            .with_allowed_kinds([TermKind::Rational])
            .with_max_total_degree(2);
        let selection = prior.admissible(&terms).unwrap();
        assert!(!selection.admitted.is_empty());
        for &index in &selection.admitted {
            assert_eq!(TermKind::of(&terms[index].expression), TermKind::Rational);
            assert!(total_degree(&terms[index].expression) <= 2);
        }
    }

    #[test]
    fn filter_is_deterministic() {
        let terms = library();
        let prior =
            TemplatePrior::unconstrained().with_max_total_degree(2).forbidding_interactions();
        let first = prior.admissible(&terms).unwrap();
        let second = prior.admissible(&terms).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn report_accounts_for_every_term_exactly_once() {
        let terms = library();
        let prior = TemplatePrior::unconstrained()
            .with_max_total_degree(1)
            .with_max_active_terms(2)
            .forbidding_interactions();
        let selection = prior.admissible(&terms).unwrap();
        assert_eq!(
            selection.report.admitted + selection.report.dropped_count(),
            selection.report.considered
        );
        assert_eq!(selection.report.considered, terms.len());
    }
}
