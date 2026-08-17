use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::DiscoveryCheckpoint;
fn main() {
    let data = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0]).unwrap(),
        [NumericColumn::new(Identifier::new("x").unwrap(), vec![0.0, 1.0])],
    )
    .unwrap();
    let checkpoint = DiscoveryCheckpoint::new(data.fingerprint());
    println!("new checkpoint: {} completed states", checkpoint.completed_states().count());
}
