/// Hard limits for callers orchestrating repeated simulations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimulationLimits {
    pub maximum_steps: usize,
}
impl Default for SimulationLimits {
    fn default() -> Self {
        Self {
            maximum_steps: 1_000_000,
        }
    }
}
impl SimulationLimits {
    pub fn accepts(self, steps: usize) -> bool {
        self.maximum_steps > 0 && steps <= self.maximum_steps
    }
}
