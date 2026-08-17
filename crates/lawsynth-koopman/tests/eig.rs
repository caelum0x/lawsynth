//! Verifies the Hessenberg + complex-QR eigensolver on known matrices.

use lawsynth_koopman::{Matrix, eigen};

fn assert_close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "expected {b}, got {a} (|Δ|={})", (a - b).abs());
}

#[test]
fn diagonal_matrix_reports_diagonal_eigenvalues() {
    let matrix =
        Matrix::from_rows(vec![vec![2.0, 0.0, 0.0], vec![0.0, -1.0, 0.0], vec![0.0, 0.0, 0.5]])
            .unwrap();
    let decomposition = eigen(&matrix).unwrap();
    // Sorted by descending modulus: 2, -1, 0.5.
    assert_close(decomposition.values[0].re, 2.0, 1e-12);
    assert_close(decomposition.values[1].re, -1.0, 1e-12);
    assert_close(decomposition.values[2].re, 0.5, 1e-12);
    for value in &decomposition.values {
        assert_close(value.im, 0.0, 1e-12);
    }
}

#[test]
fn rotation_matrix_has_complex_conjugate_pair() {
    // [[0.9,-0.3],[0.3,0.9]] has eigenvalues 0.9 ± 0.3 i.
    let matrix = Matrix::from_rows(vec![vec![0.9, -0.3], vec![0.3, 0.9]]).unwrap();
    let decomposition = eigen(&matrix).unwrap();
    let plus = decomposition.values.iter().find(|v| v.im > 0.0).unwrap();
    let minus = decomposition.values.iter().find(|v| v.im < 0.0).unwrap();
    assert_close(plus.re, 0.9, 1e-12);
    assert_close(plus.im, 0.3, 1e-12);
    assert_close(minus.re, 0.9, 1e-12);
    assert_close(minus.im, -0.3, 1e-12);
}

#[test]
fn eigenvectors_satisfy_the_eigen_relation() {
    let matrix = Matrix::from_rows(vec![vec![2.0, 1.0], vec![1.0, 2.0]]).unwrap();
    let decomposition = eigen(&matrix).unwrap();
    // Symmetric: real eigenvalues 3 and 1.
    assert_close(decomposition.values[0].re, 3.0, 1e-10);
    assert_close(decomposition.values[1].re, 1.0, 1e-10);
    for (lambda, vector) in decomposition.values.iter().zip(&decomposition.vectors) {
        // Check ‖A v − λ v‖ ≈ 0 in complex arithmetic.
        let mut residual = 0.0;
        for row in 0..matrix.rows() {
            let mut av_re = 0.0;
            let mut av_im = 0.0;
            for (col, component) in vector.iter().enumerate() {
                av_re += matrix.get(row, col) * component.re;
                av_im += matrix.get(row, col) * component.im;
            }
            let lv = lambda.mul(vector[row]);
            residual += (av_re - lv.re).powi(2) + (av_im - lv.im).powi(2);
        }
        assert!(residual.sqrt() < 1e-9, "eigen relation residual {}", residual.sqrt());
    }
}

#[test]
fn upper_triangular_eigenvalues_are_the_diagonal() {
    let matrix =
        Matrix::from_rows(vec![vec![1.0, 7.0, -2.0], vec![0.0, 4.0, 3.0], vec![0.0, 0.0, 2.0]])
            .unwrap();
    let decomposition = eigen(&matrix).unwrap();
    let mut reals: Vec<f64> = decomposition.values.iter().map(|value| value.re).collect();
    reals.sort_by(|a, b| a.total_cmp(b));
    assert_close(reals[0], 1.0, 1e-10);
    assert_close(reals[1], 2.0, 1e-10);
    assert_close(reals[2], 4.0, 1e-10);
}
