//! Discovers the Michaelis-Menten rational law from a simulated trajectory and
//! prints the implicit relation, the explicit `ẋ = P(x)/Q(x)`, and the errors.
//!
//! Run with: `cargo run -p lawsynth-implicit --example recover_rational`.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_implicit::{ImplicitConfig, MonomialTerm, implicit_discover};

fn integrate(f: impl Fn(f64) -> f64, x0: f64, dt: f64, steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut time = Vec::new();
    let mut xs = Vec::new();
    let mut x = x0;
    for step in 0..=steps {
        time.push(step as f64 * dt);
        xs.push(x);
        let k1 = f(x);
        let k2 = f(x + 0.5 * dt * k1);
        let k3 = f(x + 0.5 * dt * k2);
        let k4 = f(x + dt * k3);
        x += dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
    }
    (time, xs)
}

fn coefficient(terms: &[MonomialTerm], name: &str) -> f64 {
    terms.iter().find(|t| t.name == name).map(|t| t.coefficient).unwrap_or(0.0)
}

fn main() {
    let vmax = 1.5;
    let km = 0.3;
    let (time, xs) = integrate(|x| -vmax * x / (km + x), 2.0, 0.01, 400);

    let x = Identifier::new("x").unwrap();
    let dataset = Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x, xs)]).unwrap();

    let config = ImplicitConfig { degree: 1, ..Default::default() };
    let result = implicit_discover(&dataset, &config).unwrap();

    println!("implicit relation (normalised on `{}`):", result.relation.lhs_name);
    let relation = result
        .relation
        .terms
        .iter()
        .map(|term| format!("{:+.4}·{}", term.coefficient, term.term.name))
        .collect::<Vec<_>>()
        .join(" ");
    println!("  0 = {relation}");
    println!("  relative residual = {:.3e}", result.relation.relative_residual);

    if let Some(law) = result.rational_law {
        let recovered_vmax = -coefficient(&law.numerator.terms, "x");
        let recovered_km = coefficient(&law.denominator.terms, "1");
        println!("explicit rational law:");
        println!("  ẋ = -{recovered_vmax:.4}·x / ({recovered_km:.4} + x)");
        println!("true law:");
        println!("  ẋ = -{vmax:.4}·x / ({km:.4} + x)");
        println!(
            "coefficient errors: Vmax {:.2e}, Km {:.2e}",
            (recovered_vmax - vmax).abs(),
            (recovered_km - km).abs()
        );
        println!("min |Q(x)| over samples = {:.4}", law.min_abs_denominator);
    }
}
