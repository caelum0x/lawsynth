use lawsynth_wasm::{Expression, MemoryBudget, WasmConfig, World, simulate_rk4};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut memory = MemoryBudget::new(1024 * 1024)?;
    memory.reserve(128)?;
    let world = World::new(vec!["x".into()], vec![1.0], vec![Expression::parse("-x")?])?;
    let trajectory = simulate_rk4(&world, 0.0, 1.0, 0.1, &WasmConfig::default())?;
    println!("{} samples; {} bytes remain", trajectory.len(), memory.available());
    Ok(())
}
