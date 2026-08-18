//! End-to-end CLI tests for the alternative discovery engines wired in this
//! batch: `discover --method weak-form`, `koopman`, `sde`, and `pde`. Each drives
//! the real engine through `lawsynth_cli::run` on a small deterministic dataset,
//! with tolerances honest to the estimator. The default strong-form `discover`
//! path is asserted unchanged.

use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_cli::run;

/// A unique temp directory for one test's artifacts.
fn temp_dir(tag: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

/// Writes bytes to `dir/name` and returns the path string.
fn write_file(directory: &Path, name: &str, contents: &str) -> String {
    let path = directory.join(name);
    fs::write(&path, contents).unwrap();
    path.display().to_string()
}

/// Extracts the float that immediately follows `marker` in `text`, reading the
/// leading numeric token (sign, digits, `.`, `e`, `E`, `+`, `-`).
fn number_after(text: &str, marker: &str) -> f64 {
    let start = text.find(marker).unwrap_or_else(|| panic!("marker {marker:?} not in: {text}"));
    let rest = &text[start + marker.len()..];
    let token: String = rest
        .trim_start()
        .chars()
        .take_while(|character| {
            character.is_ascii_digit() || matches!(character, '.' | 'e' | 'E' | '+' | '-')
        })
        .collect();
    token.parse().unwrap_or_else(|_| panic!("could not parse number after {marker:?}: {rest}"))
}

/// The substring of `text` between the first `start` and the following `end`.
fn slice_between<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let from = text.find(start).unwrap_or_else(|| panic!("{start:?} not in: {text}"));
    let tail = &text[from..];
    let to = tail.find(end).map(|index| from + index).unwrap_or(text.len());
    &text[from..to]
}

/// A deterministic splitmix64 generator, used to seed the SDE sample path
/// without any external crate.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// A uniform float in the open interval `(0, 1)`.
    fn next_unit(&mut self) -> f64 {
        // 53-bit mantissa, shifted off zero so Box–Muller's log is finite.
        ((self.next_u64() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }

    /// A standard-normal sample via the Box–Muller transform.
    fn next_normal(&mut self) -> f64 {
        let u1 = self.next_unit();
        let u2 = self.next_unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

#[test]
fn discover_default_method_is_unchanged_strong_form() {
    // The same fixture run twice — once with no --method, once with
    // --method sindy — must produce byte-identical output, and must still write
    // the .lsworld world (the strong-form contract).
    let directory = temp_dir("discover-default");
    let mut csv = String::from("time,x\n");
    // x(t) = e^{-0.5 t} sampled on a fine grid: x' = -0.5 x.
    for step in 0..=400 {
        let t = step as f64 * 0.01;
        csv.push_str(&format!("{t:.6},{:.10}\n", (-0.5 * t).exp()));
    }
    let data = write_file(&directory, "decay.csv", &csv);
    let world_a = directory.join("a.lsworld").display().to_string();
    let world_b = directory.join("b.lsworld").display().to_string();

    let default = run(&[
        "discover".to_owned(),
        data.clone(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        world_a.clone(),
    ])
    .unwrap();
    let explicit = run(&[
        "discover".to_owned(),
        data,
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        world_b.clone(),
        "--method".to_owned(),
        "sindy".to_owned(),
    ])
    .unwrap();

    assert_eq!(default, explicit, "explicit --method sindy must equal the default path");
    assert!(default.contains("discovered world"), "output: {default}");
    assert!(Path::new(&world_a).exists(), "default discover must write the world");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn weak_form_recovers_a_linear_decay_coefficient() {
    let directory = temp_dir("weak-form");
    // x(t) = e^{-0.5 t}: the weak/integral form should recover d/dt x = -0.5 x
    // without ever differentiating the (noise-free here) data.
    let mut csv = String::from("time,x\n");
    for step in 0..=400 {
        let t = step as f64 * 0.01;
        csv.push_str(&format!("{t:.6},{:.12}\n", (-0.5 * t).exp()));
    }
    let data = write_file(&directory, "decay.csv", &csv);

    let text = run(&[
        "discover".to_owned(),
        data.clone(),
        "--method".to_owned(),
        "weak-form".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--degree".to_owned(),
        "1".to_owned(),
    ])
    .unwrap();
    assert!(text.contains("Weak-form discovery"), "text: {text}");
    assert!(text.contains("d/dt x ="), "text: {text}");
    assert!(text.contains("not differentiated"), "must surface the noise-robust caveat: {text}");

    let json = run(&[
        "discover".to_owned(),
        data,
        "--method".to_owned(),
        "weak-form".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--degree".to_owned(),
        "1".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    let coefficient = number_after(&json, "\"term\": \"x\", \"coefficient\": ");
    assert!(
        (coefficient - (-0.5)).abs() < 0.02,
        "weak-form should recover d/dt x ≈ -0.5 x, got {coefficient}: {json}"
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn koopman_recovers_a_decaying_oscillator_spectrum() {
    let directory = temp_dir("koopman");
    // A discrete decaying rotation: eigenvalues r·e^{±iθ} with r = 0.95 (inside
    // the unit circle) => an asymptotically stable, oscillatory linear system.
    let (r, theta, dt) = (0.95_f64, 0.3_f64, 0.1_f64);
    let (mut x, mut y) = (1.0_f64, 0.0_f64);
    let mut csv = String::from("time,x,y\n");
    for step in 0..40 {
        let t = step as f64 * dt;
        csv.push_str(&format!("{t:.4},{x:.12},{y:.12}\n"));
        let nx = r * (theta.cos() * x - theta.sin() * y);
        let ny = r * (theta.sin() * x + theta.cos() * y);
        x = nx;
        y = ny;
    }
    let data = write_file(&directory, "osc.csv", &csv);

    let text = run(&[
        "koopman".to_owned(),
        data.clone(),
        "--state".to_owned(),
        "x,y".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
    ])
    .unwrap();
    assert!(text.contains("Koopman/DMD discovery"), "text: {text}");
    assert!(text.contains("asymptotically stable: yes"), "text: {text}");
    assert!(text.contains("linear/lifted-linear approximation"), "honest caveat missing: {text}");

    let json = run(&[
        "koopman".to_owned(),
        data,
        "--state".to_owned(),
        "x,y".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    // Noise-free linear data => DMD recovers the operator (and its spectral
    // radius r = 0.95) essentially exactly.
    let radius = number_after(&json, "\"spectral_radius\": ");
    assert!((radius - 0.95).abs() < 1e-6, "spectral radius should be 0.95, got {radius}: {json}");
    assert!(json.contains("\"stable\": true"), "json: {json}");
    // A discrete eigenvalue modulus near 0.95 (|λ| < 1) and a negative
    // continuous-time real part (ln r / dt < 0) confirm the decay.
    let modulus = number_after(&json, "\"modulus\": ");
    assert!((modulus - 0.95).abs() < 1e-6, "eigenvalue modulus should be 0.95, got {modulus}");
    let continuous = slice_between(&json, "\"continuous_eigenvalues\"", "\"spectral_radius\"");
    let growth = number_after(continuous, "\"re\": ");
    assert!(growth < 0.0, "continuous eigenvalue growth rate should be negative, got {growth}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn sde_recovers_linear_ou_drift() {
    let directory = temp_dir("sde");
    // Ornstein–Uhlenbeck: dX = -θ X dt + σ dW, θ = 1, σ = 0.5. The drift is the
    // linear law a(x) = -x; a seeded Euler–Maruyama path recovers it.
    let (theta, sigma, dt) = (1.0_f64, 0.5_f64, 0.01_f64);
    let mut rng = SplitMix64(0x1234_5678_9ABC_DEF0);
    let mut position = 0.0_f64;
    let mut csv = String::from("time,x\n");
    for step in 0..40_000 {
        let t = step as f64 * dt;
        csv.push_str(&format!("{t:.5},{position:.12}\n"));
        position += -theta * position * dt + sigma * dt.sqrt() * rng.next_normal();
    }
    let data = write_file(&directory, "ou.csv", &csv);

    let text = run(&[
        "sde".to_owned(),
        data.clone(),
        "--state".to_owned(),
        "x".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
    ])
    .unwrap();
    assert!(text.contains("SDE discovery"), "text: {text}");
    assert!(text.contains("drift"), "text: {text}");
    assert!(text.contains("statistical"), "honest estimator caveat missing: {text}");

    let json = run(&[
        "sde".to_owned(),
        data,
        "--state".to_owned(),
        "x".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    // Scope to the drift object (not diffusion) and read the linear (power-1)
    // coefficient; the Kramers–Moyal estimator recovers -θ = -1 within a
    // path-length-honest tolerance.
    let drift = slice_between(&json, "\"drift\":", "\"diffusion\":");
    let slope = number_after(drift, "\"power\": 1, \"coefficient\": ");
    assert!(
        (slope - (-1.0)).abs() < 0.2,
        "SDE drift slope should be ≈ -1 (linear OU), got {slope}: {json}"
    );
    assert!(slope < 0.0, "OU drift must be mean-reverting (negative slope): {slope}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn pde_recovers_the_heat_equation_diffusivity() {
    let directory = temp_dir("pde");
    // Two-mode exact heat solution u_t = α u_xx, α = 0.2, on [0, 2π). Two modes
    // break the single-mode collinearity of u and u_xx.
    let alpha = 0.2_f64;
    let (nx, nt) = (48_usize, 24_usize);
    let dx = std::f64::consts::TAU / nx as f64;
    let dt = 0.02_f64;
    let mut grid = String::new();
    for ti in 0..nt {
        let t = ti as f64 * dt;
        let row: Vec<String> = (0..nx)
            .map(|xi| {
                let x = xi as f64 * dx;
                let u =
                    (-alpha * t).exp() * x.sin() + 0.5 * (-alpha * 4.0 * t).exp() * (2.0 * x).sin();
                format!("{u:.17e}")
            })
            .collect();
        grid.push_str(&row.join(","));
        grid.push('\n');
    }
    let field = write_file(&directory, "heat.csv", &grid);

    let text = run(&[
        "pde".to_owned(),
        field.clone(),
        "--dx".to_owned(),
        format!("{dx}"),
        "--dt".to_owned(),
        format!("{dt}"),
    ])
    .unwrap();
    assert!(text.contains("PDE discovery"), "text: {text}");
    assert!(text.contains("u_t ="), "text: {text}");
    assert!(text.contains("u_xx"), "should recover the diffusive term: {text}");
    assert!(text.contains("noise-sensitive"), "honest caveat missing: {text}");

    let json = run(&[
        "pde".to_owned(),
        field,
        "--dx".to_owned(),
        format!("{dx}"),
        "--dt".to_owned(),
        format!("{dt}"),
        "--json".to_owned(),
    ])
    .unwrap();
    // The u_xx term has u_power = 0, derivative_order = 2. Finite differences on
    // this grid recover α within a grid-honest tolerance.
    let uxx = slice_between(&json, "\"label\": \"u_xx\"", "}");
    let coefficient = number_after(uxx, "\"coefficient\": ");
    assert!(
        (coefficient - alpha).abs() < 0.05,
        "PDE should recover u_xx coefficient ≈ 0.2, got {coefficient}: {json}"
    );

    fs::remove_dir_all(directory).unwrap();
}
