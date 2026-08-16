use lawsynth_stats::{HistogramConfig, histogram_mutual_information};

fn main() {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.0, 1.0, 2.0, 3.0, 4.0];
    println!(
        "I(x;y) = {:.4} nats",
        histogram_mutual_information(&x, &y, HistogramConfig { bins: 3 }).unwrap()
    );
}
