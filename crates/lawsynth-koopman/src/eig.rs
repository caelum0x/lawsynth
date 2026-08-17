//! Deterministic eigendecomposition of a general real square matrix.
//!
//! The pipeline is textbook (Golub & Van Loan, Ch. 7): reduce to upper
//! Hessenberg form with Householder reflectors, then run a Wilkinson-shifted QR
//! iteration in *complex* arithmetic so that complex-conjugate eigenpairs are
//! handled uniformly (no real 2×2-block bookkeeping). Eigenvectors are then
//! recovered by inverse iteration from a fixed starting vector. Every step has a
//! fixed order and fixed tolerances, so the result is reproducible.

use crate::{Complex, KoopmanError, Matrix};

/// An eigendecomposition: paired eigenvalues and eigenvectors.
#[derive(Clone, Debug)]
pub struct Eigen {
    /// Eigenvalues, ordered by descending modulus (ties broken deterministically).
    pub values: Vec<Complex>,
    /// `vectors[k]` is the unit eigenvector for `values[k]`, length `n`.
    pub vectors: Vec<Vec<Complex>>,
}

/// Computes eigenvalues and eigenvectors of a square real matrix.
pub fn eigen(matrix: &Matrix) -> Result<Eigen, KoopmanError> {
    let n = matrix.rows();
    if n == 0 {
        return Err(KoopmanError::EmptyMatrix);
    }
    if matrix.cols() != n {
        return Err(KoopmanError::ShapeMismatch);
    }
    if n == 1 {
        return Ok(Eigen {
            values: vec![Complex::real(matrix.get(0, 0))],
            vectors: vec![vec![Complex::ONE]],
        });
    }

    let hessenberg = to_hessenberg(matrix);
    let mut values = qr_eigenvalues(hessenberg)?;
    // Canonical, deterministic order: descending modulus, then real, then imag.
    values.sort_by(|a, b| {
        b.abs()
            .total_cmp(&a.abs())
            .then_with(|| b.re.total_cmp(&a.re))
            .then_with(|| b.im.total_cmp(&a.im))
    });

    let vectors =
        values.iter().map(|&lambda| eigenvector(matrix, lambda)).collect::<Result<Vec<_>, _>>()?;

    Ok(Eigen { values, vectors })
}

/// Householder reduction of a real matrix to upper Hessenberg form.
#[allow(clippy::needless_range_loop)]
fn to_hessenberg(matrix: &Matrix) -> Vec<Vec<f64>> {
    let n = matrix.rows();
    let mut h: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| matrix.get(i, j)).collect()).collect();

    for k in 0..n.saturating_sub(2) {
        let norm = (k + 1..n).map(|i| h[i][k] * h[i][k]).sum::<f64>().sqrt();
        if norm == 0.0 {
            continue;
        }
        let alpha = if h[k + 1][k] >= 0.0 { -norm } else { norm };
        let mut v = vec![0.0; n];
        for i in k + 1..n {
            v[i] = h[i][k];
        }
        v[k + 1] -= alpha;
        let v_norm_sq = v[k + 1..n].iter().map(|x| x * x).sum::<f64>();
        if v_norm_sq == 0.0 {
            continue;
        }

        // Left application: H ← (I - 2 v vᵀ / ‖v‖²) H.
        for j in 0..n {
            let dot = (k + 1..n).map(|i| v[i] * h[i][j]).sum::<f64>();
            let factor = 2.0 * dot / v_norm_sq;
            for i in k + 1..n {
                h[i][j] -= factor * v[i];
            }
        }
        // Right application: H ← H (I - 2 v vᵀ / ‖v‖²).
        for row in h.iter_mut() {
            let dot = row[k + 1..n].iter().zip(&v[k + 1..n]).map(|(a, b)| a * b).sum::<f64>();
            let factor = 2.0 * dot / v_norm_sq;
            for (entry, &component) in row[k + 1..n].iter_mut().zip(&v[k + 1..n]) {
                *entry -= factor * component;
            }
        }
    }
    h
}

const QR_ITER_TOLERANCE: f64 = f64::EPSILON;

/// Wilkinson-shifted complex QR iteration returning the eigenvalues.
fn qr_eigenvalues(hessenberg: Vec<Vec<f64>>) -> Result<Vec<Complex>, KoopmanError> {
    let n = hessenberg.len();
    let mut h: Vec<Vec<Complex>> = hessenberg
        .iter()
        .map(|row| row.iter().map(|&value| Complex::real(value)).collect())
        .collect();
    let mut eigenvalues = vec![Complex::ZERO; n];

    let mut active = n;
    let mut budget = 200 * n + 200;
    let mut stagnant = 0usize;

    while active > 0 {
        // Locate the start `p` of the trailing unreduced Hessenberg block.
        let mut p = active - 1;
        while p > 0 {
            let sub = h[p][p - 1].abs();
            let scale = h[p - 1][p - 1].abs() + h[p][p].abs();
            if sub <= QR_ITER_TOLERANCE * scale {
                h[p][p - 1] = Complex::ZERO;
                break;
            }
            p -= 1;
        }

        if p == active - 1 {
            // A 1×1 block has converged; deflate one eigenvalue.
            eigenvalues[active - 1] = h[active - 1][active - 1];
            active -= 1;
            stagnant = 0;
            continue;
        }

        let shift = if stagnant > 0 && stagnant % 12 == 0 {
            // Exceptional shift to dislodge a stalled block.
            h[active - 1][active - 1].add(Complex::real(h[active - 1][active - 2].abs()))
        } else {
            wilkinson_shift(&h, active)
        };
        qr_step(&mut h, p, active, shift);

        stagnant += 1;
        budget -= 1;
        if budget == 0 {
            return Err(KoopmanError::NoConvergence);
        }
    }

    Ok(eigenvalues)
}

/// The eigenvalue of the trailing 2×2 block closest to its bottom-right entry.
fn wilkinson_shift(h: &[Vec<Complex>], active: usize) -> Complex {
    let a = h[active - 2][active - 2];
    let b = h[active - 2][active - 1];
    let c = h[active - 1][active - 2];
    let d = h[active - 1][active - 1];
    let trace = a.add(d);
    let det = a.mul(d).sub(b.mul(c));
    let disc = trace.mul(trace).sub(det.scale(4.0)).sqrt();
    let first = trace.add(disc).scale(0.5);
    let second = trace.sub(disc).scale(0.5);
    if first.sub(d).abs() <= second.sub(d).abs() { first } else { second }
}

/// One explicit shifted-QR sweep on the block `[p, active)`.
#[allow(clippy::needless_range_loop)]
fn qr_step(h: &mut [Vec<Complex>], p: usize, active: usize, shift: Complex) {
    for i in p..active {
        h[i][i] = h[i][i].sub(shift);
    }

    let mut rotations = Vec::with_capacity(active - p - 1);
    for i in p..active - 1 {
        let (cos, sin) = givens(h[i][i], h[i + 1][i]);
        apply_left(h, i, cos, sin);
        rotations.push((cos, sin));
    }
    for (offset, &(cos, sin)) in rotations.iter().enumerate() {
        apply_right(h, p + offset, cos, sin);
    }

    for i in p..active {
        h[i][i] = h[i][i].add(shift);
    }
}

/// A complex Givens rotation `(cos, sin)` that zeros `g` against `f`.
fn givens(f: Complex, g: Complex) -> (f64, Complex) {
    if g.is_zero() {
        return (1.0, Complex::ZERO);
    }
    if f.is_zero() {
        return (0.0, g.conj().scale(1.0 / g.abs()));
    }
    let f_abs = f.abs();
    let denom = (f_abs * f_abs + g.norm_sqr()).sqrt();
    let cos = f_abs / denom;
    let sin = f.scale(1.0 / f_abs).mul(g.conj()).scale(1.0 / denom);
    (cos, sin)
}

/// Applies a Givens rotation to rows `i` and `i + 1` across all columns.
fn apply_left(h: &mut [Vec<Complex>], i: usize, cos: f64, sin: Complex) {
    let (top, bottom) = h.split_at_mut(i + 1);
    let row_i = &mut top[i];
    let row_j = &mut bottom[0];
    for (a_ref, b_ref) in row_i.iter_mut().zip(row_j.iter_mut()) {
        let a = *a_ref;
        let b = *b_ref;
        *a_ref = a.scale(cos).add(sin.mul(b));
        *b_ref = b.scale(cos).sub(sin.conj().mul(a));
    }
}

/// Applies the conjugate rotation to columns `i` and `i + 1` across all rows.
fn apply_right(h: &mut [Vec<Complex>], i: usize, cos: f64, sin: Complex) {
    for row in h.iter_mut() {
        let a = row[i];
        let b = row[i + 1];
        row[i] = a.scale(cos).add(b.mul(sin.conj()));
        row[i + 1] = b.scale(cos).sub(a.mul(sin));
    }
}

const INVERSE_ITERATIONS: usize = 10;

/// Recovers a unit eigenvector for `lambda` by inverse iteration.
#[allow(clippy::needless_range_loop)]
fn eigenvector(matrix: &Matrix, lambda: Complex) -> Result<Vec<Complex>, KoopmanError> {
    let n = matrix.rows();
    // A tiny shift keeps `A - σI` non-singular while staying close to λ.
    let perturbation = 1e-11 * (1.0 + lambda.abs());
    let sigma = lambda.sub(Complex::real(perturbation));
    let shifted: Vec<Vec<Complex>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    let value = Complex::real(matrix.get(i, j));
                    if i == j { value.sub(sigma) } else { value }
                })
                .collect()
        })
        .collect();

    let mut vector = vec![Complex::ONE; n];
    normalize(&mut vector);
    for _ in 0..INVERSE_ITERATIONS {
        vector = solve_complex(&shifted, &vector)?;
        normalize(&mut vector);
    }
    Ok(vector)
}

/// Normalises a complex vector to a canonical unit phase and scale.
fn normalize(vector: &mut [Complex]) {
    let mut pivot_index = 0;
    let mut best = 0.0;
    for (index, component) in vector.iter().enumerate() {
        let magnitude = component.abs();
        if magnitude > best {
            best = magnitude;
            pivot_index = index;
        }
    }
    if best == 0.0 {
        return;
    }
    let pivot = vector[pivot_index];
    for component in vector.iter_mut() {
        *component = component.div(pivot);
    }
}

/// Solves the complex linear system `matrix · x = rhs` by partial-pivot LU.
#[allow(clippy::needless_range_loop)]
fn solve_complex(matrix: &[Vec<Complex>], rhs: &[Complex]) -> Result<Vec<Complex>, KoopmanError> {
    let n = matrix.len();
    let mut work: Vec<Vec<Complex>> = matrix.to_vec();
    let mut vector = rhs.to_vec();

    for col in 0..n {
        let mut pivot = col;
        let mut best = work[col][col].abs();
        for row in col + 1..n {
            let candidate = work[row][col].abs();
            if candidate > best {
                best = candidate;
                pivot = row;
            }
        }
        if best == 0.0 {
            return Err(KoopmanError::SingularSystem);
        }
        work.swap(col, pivot);
        vector.swap(col, pivot);

        let diagonal = work[col][col];
        for row in col + 1..n {
            let factor = work[row][col].div(diagonal);
            if factor.is_zero() {
                continue;
            }
            for c in col..n {
                let update = factor.mul(work[col][c]);
                work[row][c] = work[row][c].sub(update);
            }
            let update = factor.mul(vector[col]);
            vector[row] = vector[row].sub(update);
        }
    }

    let mut solution = vec![Complex::ZERO; n];
    for i in (0..n).rev() {
        let mut sum = vector[i];
        for j in i + 1..n {
            sum = sum.sub(work[i][j].mul(solution[j]));
        }
        solution[i] = sum.div(work[i][i]);
    }
    Ok(solution)
}
