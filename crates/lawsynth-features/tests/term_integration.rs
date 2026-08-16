use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::{FeatureConstraint, FeatureLibrary};

#[test]
fn constrained_extended_library_builds_a_design_matrix_from_dataset_rows() {
    let x = Identifier::new("x").unwrap();
    let y = Identifier::new("y").unwrap();
    let data = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0]).unwrap(),
        [
            NumericColumn::new(x.clone(), vec![2.0, 3.0]),
            NumericColumn::new(y.clone(), vec![5.0, 7.0]),
        ],
    )
    .unwrap();
    let mut library = FeatureLibrary::polynomial([x.clone(), y.clone()], 1, true).unwrap();
    library.extend(FeatureLibrary::interactions([x.clone(), y.clone()]).unwrap());
    let constrained = library.constrained(&[
        FeatureConstraint::AllowedSymbols([x, y].into_iter().collect()),
        FeatureConstraint::RequireSymbol,
    ]);
    let matrix = constrained.evaluate(&data).unwrap();
    assert_eq!(matrix.terms.len(), 3);
    assert_eq!(
        matrix.rows,
        vec![vec![5.0, 2.0, 10.0], vec![7.0, 3.0, 21.0]]
    );
}
