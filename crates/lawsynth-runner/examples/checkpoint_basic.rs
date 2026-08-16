use lawsynth_runner::{Checkpoint, ProcessRecord};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint = Checkpoint::new(1, 1_700_000_000_000, b"integrator state".to_vec(), 1024)?;
    let mut record = ProcessRecord::new("simulation-42");
    record.record_checkpoint(checkpoint)?;
    println!(
        "saved checkpoint {} for {}",
        record.latest_checkpoint().unwrap().sequence,
        record.work_id
    );
    Ok(())
}
