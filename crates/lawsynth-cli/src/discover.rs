use lawsynth_discovery::DiscoveryConfig;
/// Renders the stable one-line summary returned by successful discovery.
pub fn discovery_summary(config: &DiscoveryConfig, mse: f64, complexity: usize) -> String {
    format!(
        "discovered {} state laws: mse={mse:.6e}, complexity={complexity}\n",
        config.state.len()
    )
}
