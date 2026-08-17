//! Verifies the one-sided Jacobi SVD against known decompositions.

use lawsynth_koopman::{Matrix, svd};

fn reconstruct(decomposition: &lawsynth_koopman::Svd) -> Matrix {
    // U · diag(s) · Vᵀ
    let mut scaled = decomposition.u.clone();
    for (column, &sigma) in decomposition.s.iter().enumerate() {
        scaled.scale_column(column, sigma);
    }
    scaled.matmul(&decomposition.v.transpose()).unwrap()
}

fn assert_close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "expected {b}, got {a} (|Δ|={})", (a - b).abs());
}

#[test]
fn singular_values_of_a_known_2x2() {
    // [[1,1],[0,1]] has singular values (√5±1)/2 = 1.6180339887…, 0.6180339887…
    let matrix = Matrix::from_rows(vec![vec![1.0, 1.0], vec![0.0, 1.0]]).unwrap();
    let decomposition = svd(&matrix).unwrap();
    assert_close(decomposition.s[0], 1.618_033_988_749_895, 1e-10);
    assert_close(decomposition.s[1], 0.618_033_988_749_895, 1e-10);
}

#[test]
fn diagonal_singular_values_are_absolute_values() {
    let matrix = Matrix::from_rows(vec![vec![2.0, 0.0], vec![0.0, -3.0]]).unwrap();
    let decomposition = svd(&matrix).unwrap();
    // Descending order: 3 then 2.
    assert_close(decomposition.s[0], 3.0, 1e-12);
    assert_close(decomposition.s[1], 2.0, 1e-12);
}

#[test]
fn reconstructs_a_tall_matrix() {
    let matrix = Matrix::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]]).unwrap();
    let decomposition = svd(&matrix).unwrap();
    let reconstructed = reconstruct(&decomposition);
    for row in 0..matrix.rows() {
        for col in 0..matrix.cols() {
            assert_close(reconstructed.get(row, col), matrix.get(row, col), 1e-10);
        }
    }
}

#[test]
fn reconstructs_a_wide_matrix() {
    // Fewer rows than columns exercises the transpose path.
    let matrix = Matrix::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
    let decomposition = svd(&matrix).unwrap();
    let reconstructed = reconstruct(&decomposition);
    for row in 0..matrix.rows() {
        for col in 0..matrix.cols() {
            assert_close(reconstructed.get(row, col), matrix.get(row, col), 1e-10);
        }
    }
}

#[test]
fn left_and_right_factors_are_orthonormal() {
    let matrix = Matrix::from_rows(vec![vec![2.0, -1.0], vec![1.0, 3.0], vec![0.0, 1.0]]).unwrap();
    let decomposition = svd(&matrix).unwrap();
    // Columns of U are orthonormal.
    let gram_u = decomposition.u.transpose().matmul(&decomposition.u).unwrap();
    for i in 0..gram_u.rows() {
        for j in 0..gram_u.cols() {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_close(gram_u.get(i, j), expected, 1e-10);
        }
    }
    // Columns of V are orthonormal.
    let gram_v = decomposition.v.transpose().matmul(&decomposition.v).unwrap();
    for i in 0..gram_v.rows() {
        for j in 0..gram_v.cols() {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_close(gram_v.get(i, j), expected, 1e-10);
        }
    }
}

#[test]
fn singular_values_are_non_increasing() {
    let matrix =
        Matrix::from_rows(vec![vec![4.0, 1.0, 0.0], vec![1.0, 3.0, 1.0], vec![0.0, 1.0, 2.0]])
            .unwrap();
    let decomposition = svd(&matrix).unwrap();
    for pair in decomposition.s.windows(2) {
        assert!(pair[0] >= pair[1], "singular values must be sorted descending");
    }
}
