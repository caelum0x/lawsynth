use lawsynth_sim::SimulationConfig;
/// Validates and constructs continuous simulation configuration from CLI values.
pub fn simulation_config(start: f64, end: f64, step: f64) -> Result<SimulationConfig, String> {
    SimulationConfig::new(start, end, step).map_err(|error| error.to_string())
}
