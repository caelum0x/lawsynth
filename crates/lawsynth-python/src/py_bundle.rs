use lawsynth_bundle::{read_world, write_world};
use lawsynth_world::World;
/// Saves a continuous world bundle from a Python-facing path value.
pub fn save_continuous_world(path: &str, world: &World) -> Result<(), String> {
    write_world(path, world).map_err(|error| error.to_string())
}
/// Loads a continuous world bundle from a Python-facing path value.
pub fn load_continuous_world(path: &str) -> Result<World, String> {
    read_world(path).map_err(|error| error.to_string())
}
