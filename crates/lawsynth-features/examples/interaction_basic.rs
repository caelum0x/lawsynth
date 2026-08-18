use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::FeatureLibrary;

fn main() {
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
    let matrix = FeatureLibrary::interactions([x, y]).unwrap().evaluate(&data).unwrap();
    println!("interaction terms: {:?}; rows: {:?}", matrix.terms, matrix.rows);
}
