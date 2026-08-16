use lawsynth_core::{ProgressStage, ProgressTracker};

fn main() {
    let mut tracker = ProgressTracker::default();
    for (fraction, message) in [
        (0.0, "accepted input"),
        (0.5, "building features"),
        (1.0, "done"),
    ] {
        let event = tracker
            .report(ProgressStage::Features, fraction, message)
            .expect("valid progress");
        println!(
            "#{:03} {:?}: {:.0}% — {}",
            event.sequence,
            event.stage,
            event.fraction * 100.0,
            event.message
        );
    }
}
