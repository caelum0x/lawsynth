use lawsynth_uncertainty::{
    BootstrapConfig, IntervalConfig, Samples, bootstrap, confidence_interval, structural_score,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let measurements = Samples::new(vec![9.8, 10.1, 9.9, 10.2, 10.0, 9.7])?;
    let distribution =
        bootstrap(&measurements, BootstrapConfig { replicates: 2_000, seed: 7 }, |draw| {
            draw.iter().sum::<f64>() / draw.len() as f64
        })?;
    let (lower, upper) = confidence_interval(&distribution, IntervalConfig::default())?;
    let ambiguity = structural_score(&[120.2, 121.7, 132.4])?;
    println!(
        "mean={:.4}, 95% interval=[{lower:.4}, {upper:.4}], structural ambiguity={ambiguity:.4}",
        distribution.observed
    );
    Ok(())
}
