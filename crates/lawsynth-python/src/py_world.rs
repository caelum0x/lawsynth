use lawsynth_world::World;
use std::collections::BTreeMap;
/// Produces Python-friendly target-to-expression text for a continuous world.
pub fn equation_strings(world: &World) -> BTreeMap<String, String> {
    world
        .laws()
        .iter()
        .map(|(id, law)| (id.to_string(), lawsynth_expr::print(&law.expression)))
        .collect()
}
