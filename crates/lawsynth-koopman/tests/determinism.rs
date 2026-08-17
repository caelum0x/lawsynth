//! Asserts bit-for-bit reproducibility of every fit.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_koopman::{Matrix, PolynomialDictionary, dmd, dmdc, edmd};

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn operator_bits(matrix: &Matrix) -> Vec<u64> {
    let mut out = Vec::new();
    for row in 0..matrix.rows() {
        for col in 0..matrix.cols() {
            out.push(matrix.get(row, col).to_bits());
        }
    }
    out
}

fn rotation_snapshots() -> (Matrix, Matrix) {
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..30 {
        let x = *states.last().unwrap();
        states.push([a[0][0] * x[0] + a[0][1] * x[1], a[1][0] * x[0] + a[1][1] * x[1]]);
    }
    let m = states.len();
    let rows_x = vec![
        (0..m - 1).map(|k| states[k][0]).collect::<Vec<_>>(),
        (0..m - 1).map(|k| states[k][1]).collect::<Vec<_>>(),
    ];
    let rows_xp = vec![
        (1..m).map(|k| states[k][0]).collect::<Vec<_>>(),
        (1..m).map(|k| states[k][1]).collect::<Vec<_>>(),
    ];
    (Matrix::from_rows(rows_x).unwrap(), Matrix::from_rows(rows_xp).unwrap())
}

#[test]
fn dmd_is_bit_reproducible() {
    let (x, x_prime) = rotation_snapshots();
    let first = dmd(&x, &x_prime, 2).unwrap();
    let second = dmd(&x, &x_prime, 2).unwrap();
    assert_eq!(operator_bits(first.operator()), operator_bits(second.operator()));

    let first_eigs: Vec<u64> =
        first.eigenvalues().iter().flat_map(|v| [v.re.to_bits(), v.im.to_bits()]).collect();
    let second_eigs: Vec<u64> =
        second.eigenvalues().iter().flat_map(|v| [v.re.to_bits(), v.im.to_bits()]).collect();
    assert_eq!(first_eigs, second_eigs);

    let start = [2.0, -1.0];
    let first_prediction: Vec<u64> =
        first.predict(&start, 10).unwrap().into_iter().flat_map(|s| bits(&s)).collect();
    let second_prediction: Vec<u64> =
        second.predict(&start, 10).unwrap().into_iter().flat_map(|s| bits(&s)).collect();
    assert_eq!(first_prediction, second_prediction);
}

#[test]
fn dmdc_is_bit_reproducible() {
    let a = [[0.8, -0.2], [0.1, 0.95]];
    let b = [0.5, -0.3];
    let horizon = 40;
    let control: Vec<f64> = (0..horizon).map(|t| (0.3 * t as f64).sin()).collect();
    let mut states = vec![[0.0, 0.0]];
    for (t, &u) in control.iter().enumerate() {
        let x = states[t];
        states.push([
            a[0][0] * x[0] + a[0][1] * x[1] + b[0] * u,
            a[1][0] * x[0] + a[1][1] * x[1] + b[1] * u,
        ]);
    }
    let m = control.len();
    let x = Matrix::from_rows(vec![
        (0..m).map(|k| states[k][0]).collect(),
        (0..m).map(|k| states[k][1]).collect(),
    ])
    .unwrap();
    let x_prime = Matrix::from_rows(vec![
        (0..m).map(|k| states[k + 1][0]).collect(),
        (0..m).map(|k| states[k + 1][1]).collect(),
    ])
    .unwrap();
    let u = Matrix::from_rows(vec![control]).unwrap();

    let first = dmdc(&x, &x_prime, &u, 3).unwrap();
    let second = dmdc(&x, &x_prime, &u, 3).unwrap();
    assert_eq!(operator_bits(first.state_operator()), operator_bits(second.state_operator()));
    assert_eq!(operator_bits(first.control_operator()), operator_bits(second.control_operator()));
}

#[test]
fn edmd_is_bit_reproducible() {
    let mut values = vec![0.5];
    for _ in 0..30 {
        let x = *values.last().unwrap();
        values.push(0.9 * x + 0.05 * x * x);
    }
    let time = TimeAxis::new((0..values.len()).map(|i| i as f64).collect()).unwrap();
    let column = NumericColumn::new(Identifier::new("x").unwrap(), values);
    let dataset = Dataset::new(time, [column]).unwrap();
    let dictionary = PolynomialDictionary::new(1, 2).unwrap();

    let first = edmd(&dataset, &dictionary, 3).unwrap();
    let second = edmd(&dataset, &dictionary, 3).unwrap();
    assert_eq!(operator_bits(first.koopman_operator()), operator_bits(second.koopman_operator()));

    let first_prediction: Vec<u64> =
        first.predict(&[0.5], 15).unwrap().into_iter().flat_map(|s| bits(&s)).collect();
    let second_prediction: Vec<u64> =
        second.predict(&[0.5], 15).unwrap().into_iter().flat_map(|s| bits(&s)).collect();
    assert_eq!(first_prediction, second_prediction);
}
