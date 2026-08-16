use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_expr::{Environment, evaluate};

use crate::{FeatureConstraint, FeatureError, FeatureTerm, constraints, interaction, polynomial};

/// A deterministic collection of candidate expression terms.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureLibrary {
    terms: Vec<FeatureTerm>,
}

/// A row-major feature design matrix paired with its expression columns.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureMatrix {
    pub terms: Vec<FeatureTerm>,
    pub rows: Vec<Vec<f64>>,
}

impl FeatureLibrary {
    pub fn polynomial(
        variables: impl IntoIterator<Item = Identifier>,
        degree: usize,
        include_constant: bool,
    ) -> Result<Self, FeatureError> {
        let variables = variables.into_iter().collect::<Vec<_>>();
        if variables.is_empty() {
            return Err(FeatureError::EmptyVariables);
        }
        Ok(Self {
            terms: polynomial::terms(&variables, degree, include_constant),
        })
    }

    /// Builds a deterministic sine/cosine library for dimensionless signals.
    pub fn trigonometric(
        variables: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, FeatureError> {
        let variables = variables.into_iter().collect::<Vec<_>>();
        if variables.is_empty() {
            return Err(FeatureError::EmptyVariables);
        }
        Ok(Self {
            terms: crate::trigonometric::terms(&variables),
        })
    }

    /// Builds bounded rational terms `x / (1 + x²)` for each variable.
    ///
    /// The protected denominator is always positive, making these terms safe
    /// for real-valued datasets without silently dropping rows at poles.
    pub fn bounded_rational(
        variables: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, FeatureError> {
        let variables = variables.into_iter().collect::<Vec<_>>();
        if variables.is_empty() {
            return Err(FeatureError::EmptyVariables);
        }
        Ok(Self {
            terms: crate::rational::bounded_terms(&variables),
        })
    }

    /// Builds pairwise interaction terms `xᵢ * xⱼ` in input order.
    pub fn interactions(
        variables: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, FeatureError> {
        let variables = variables.into_iter().collect::<Vec<_>>();
        if variables.is_empty() {
            return Err(FeatureError::EmptyVariables);
        }
        Ok(Self {
            terms: interaction::terms(&variables)?,
        })
    }

    pub fn terms(&self) -> &[FeatureTerm] {
        &self.terms
    }

    /// Adds terms from another library while preserving deterministic order.
    pub fn extend(&mut self, other: Self) {
        self.terms.extend(other.terms);
    }

    /// Returns a copy retaining only terms accepted by every constraint.
    ///
    /// Constraints are evaluated structurally against the expression tree, so
    /// this cannot be bypassed by a misleading human-readable term name.
    pub fn constrained(&self, rules: &[FeatureConstraint]) -> Self {
        Self {
            terms: self
                .terms
                .iter()
                .filter(|term| rules.iter().all(|rule| constraints::allows(rule, term)))
                .cloned()
                .collect(),
        }
    }

    pub fn evaluate(&self, dataset: &Dataset) -> Result<FeatureMatrix, FeatureError> {
        let values = dataset.columns();
        let rows = (0..dataset.time().len())
            .map(|row| {
                let environment: Environment = values
                    .iter()
                    .map(|(id, column)| (id.clone(), column.values[row]))
                    .collect::<BTreeMap<_, _>>();
                self.terms
                    .iter()
                    .map(|term| {
                        evaluate(&term.expression, &environment)
                            .map_err(|error| FeatureError::Evaluation(error.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FeatureMatrix {
            terms: self.terms.clone(),
            rows,
        })
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn polynomial_library_has_deterministic_terms_and_values() {
        let x = Identifier::new("x").unwrap();
        let y = Identifier::new("y").unwrap();
        let library = FeatureLibrary::polynomial([x.clone(), y.clone()], 2, true).unwrap();
        assert_eq!(library.terms().len(), 6);
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [
                NumericColumn::new(x, vec![2.0, 3.0]),
                NumericColumn::new(y, vec![5.0, 7.0]),
            ],
        )
        .unwrap();
        let matrix = library.evaluate(&data).unwrap();
        assert_eq!(matrix.rows[0], vec![1.0, 5.0, 2.0, 25.0, 10.0, 4.0]);
    }

    #[test]
    fn trigonometric_library_evaluates_sine_and_cosine() {
        let x = Identifier::new("x").unwrap();
        let library = FeatureLibrary::trigonometric([x.clone()]).unwrap();
        let data = Dataset::new(
            TimeAxis::new(vec![0.0]).unwrap(),
            [NumericColumn::new(x, vec![0.0])],
        )
        .unwrap();
        assert_eq!(library.evaluate(&data).unwrap().rows[0], vec![0.0, 1.0]);
    }

    #[test]
    fn bounded_rational_library_has_no_zero_denominator() {
        let x = Identifier::new("x").unwrap();
        let library = FeatureLibrary::bounded_rational([x.clone()]).unwrap();
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(x, vec![0.0, 2.0])],
        )
        .unwrap();
        assert_eq!(
            library.evaluate(&data).unwrap().rows,
            vec![vec![0.0], vec![0.4]]
        );
    }

    #[test]
    fn interaction_library_is_ordered_and_evaluates_products() {
        let x = Identifier::new("x").unwrap();
        let y = Identifier::new("y").unwrap();
        let z = Identifier::new("z").unwrap();
        let library = FeatureLibrary::interactions([x.clone(), y.clone(), z.clone()]).unwrap();
        assert_eq!(library.terms().len(), 3);
        let data = Dataset::new(
            TimeAxis::new(vec![0.0]).unwrap(),
            [
                NumericColumn::new(x, vec![2.0]),
                NumericColumn::new(y, vec![3.0]),
                NumericColumn::new(z, vec![5.0]),
            ],
        )
        .unwrap();
        assert_eq!(
            library.evaluate(&data).unwrap().rows,
            vec![vec![6.0, 10.0, 15.0]]
        );
    }

    #[test]
    fn constraints_apply_to_expression_structure() {
        let x = Identifier::new("x").unwrap();
        let y = Identifier::new("y").unwrap();
        let library = FeatureLibrary::polynomial([x.clone(), y.clone()], 2, true).unwrap();
        let restricted = library.constrained(&[
            FeatureConstraint::AllowedSymbols([x].into_iter().collect()),
            FeatureConstraint::MaximumNodes(3),
        ]);
        assert_eq!(restricted.terms().len(), 3);
        assert_eq!(restricted.terms()[1].name, "x");
        assert!(matches!(
            restricted.terms()[0].expression,
            lawsynth_expr::Expr::Constant(1.0)
        ));
        assert!(matches!(
            restricted.terms()[2].expression,
            lawsynth_expr::Expr::Binary {
                operator: lawsynth_expr::BinaryOperator::Multiply,
                ..
            }
        ));
    }
}
