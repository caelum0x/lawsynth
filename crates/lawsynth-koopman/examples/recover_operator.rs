//! Demonstrates recovering a known 2×2 linear operator's eigenvalues to ~1e-10
//! and rolling a trajectory forward that matches the true linear system.

use lawsynth_koopman::{Matrix, dmd};

fn step(a: &[[f64; 2]; 2], x: [f64; 2]) -> [f64; 2] {
    [a[0][0] * x[0] + a[0][1] * x[1], a[1][0] * x[0] + a[1][1] * x[1]]
}

fn main() {
    // A decaying rotation: eigenvalues 0.9 ± 0.3 i, modulus √0.9 ≈ 0.948683.
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..40 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let m = states.len();
    let x = Matrix::from_rows(vec![
        (0..m - 1).map(|k| states[k][0]).collect(),
        (0..m - 1).map(|k| states[k][1]).collect(),
    ])
    .unwrap();
    let x_prime = Matrix::from_rows(vec![
        (1..m).map(|k| states[k][0]).collect(),
        (1..m).map(|k| states[k][1]).collect(),
    ])
    .unwrap();

    let model = dmd(&x, &x_prime, 2).unwrap();

    println!("recovered operator A:");
    for row in 0..2 {
        println!(
            "  [{:+.15}, {:+.15}]",
            model.operator().get(row, 0),
            model.operator().get(row, 1)
        );
    }

    println!("DMD eigenvalues (truth 0.9 ± 0.3i):");
    for value in model.eigenvalues() {
        println!("  {value}   (|λ|={:.15})", value.abs());
    }

    let start = [2.0, -1.0];
    let horizon = 6;
    let predicted = model.predict(&start, horizon).unwrap();
    let mut truth = start;
    println!("forward roll-out from {start:?} (predicted vs true):");
    for state in predicted.iter().take(horizon) {
        truth = step(&a, truth);
        println!(
            "  pred [{:+.12}, {:+.12}]  true [{:+.12}, {:+.12}]  err {:.2e}",
            state[0],
            state[1],
            truth[0],
            truth[1],
            ((state[0] - truth[0]).powi(2) + (state[1] - truth[1]).powi(2)).sqrt()
        );
    }
}
