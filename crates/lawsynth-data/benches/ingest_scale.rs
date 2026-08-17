//! Large-dataset ingest + profile throughput benchmark.
//!
//! Generates a deterministic multi-column CSV entirely in memory, then measures
//! CSV decode throughput (rows/sec, MB/sec) and end-to-end load + profile
//! throughput. Run with:
//!
//! ```text
//! cargo run --release -p lawsynth-data --bench ingest_scale
//! ```
//!
//! Override the row/column counts with `LAWSYNTH_BENCH_ROWS` and
//! `LAWSYNTH_BENCH_COLS` environment variables so regressions stay reproducible.

use std::{hint::black_box, time::Instant};

use lawsynth_data::read_csv_numeric;

/// Deterministically renders a CSV with a strictly increasing time column and
/// `cols` numeric feature columns. Uses a fixed LCG so bytes are reproducible.
fn generate_csv(rows: usize, cols: usize) -> Vec<u8> {
    let mut text = String::with_capacity(rows * (cols + 1) * 10);
    text.push_str("time");
    for column in 0..cols {
        text.push_str(&format!(",x{column}"));
    }
    text.push('\n');
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut buffer = ryu_like();
    for row in 0..rows {
        text.push_str(itoa_like(row, &mut buffer));
        for _ in 0..cols {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            let value = unit * 200.0 - 100.0;
            text.push(',');
            text.push_str(&format!("{value:.6}"));
        }
        text.push('\n');
    }
    text.into_bytes()
}

// Minimal reusable integer formatting to keep generation cheap; correctness of
// the generator itself is not under test, only its determinism.
fn ryu_like() -> String {
    String::with_capacity(24)
}

fn itoa_like(value: usize, buffer: &mut String) -> &str {
    buffer.clear();
    buffer.push_str(&value.to_string());
    buffer.as_str()
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|raw| raw.parse().ok()).unwrap_or(default)
}

fn main() {
    let rows = env_usize("LAWSYNTH_BENCH_ROWS", 500_000);
    let cols = env_usize("LAWSYNTH_BENCH_COLS", 4);

    let generation_start = Instant::now();
    let csv = generate_csv(rows, cols);
    let megabytes = csv.len() as f64 / (1024.0 * 1024.0);
    println!(
        "generated {rows} rows x {cols} cols = {:.1} MiB CSV in {:?}",
        megabytes,
        generation_start.elapsed()
    );

    // Warm one decode so allocator caches and file-independent effects settle.
    let warm = read_csv_numeric(&csv, "time").expect("ingest must succeed");
    let fingerprint = warm.fingerprint();
    drop(warm);

    let ingest_iterations = 5;
    let ingest_start = Instant::now();
    let mut last = None;
    for _ in 0..ingest_iterations {
        last = Some(black_box(read_csv_numeric(black_box(&csv), "time").expect("ingest")));
    }
    let ingest_elapsed = ingest_start.elapsed();
    let dataset = last.unwrap();
    assert_eq!(dataset.fingerprint(), fingerprint, "ingest must be deterministic");

    let per_ingest = ingest_elapsed / ingest_iterations;
    let rows_per_sec = rows as f64 / per_ingest.as_secs_f64();
    let mb_per_sec = megabytes / per_ingest.as_secs_f64();
    println!("ingest:  {per_ingest:?}/run  {rows_per_sec:>12.0} rows/s  {mb_per_sec:>8.1} MiB/s");

    let profile_iterations = 5;
    let profile_start = Instant::now();
    for _ in 0..profile_iterations {
        black_box(lawsynth_profile::profile(black_box(&dataset)).expect("profile"));
    }
    let profile_elapsed = profile_start.elapsed();
    let per_profile = profile_elapsed / profile_iterations;
    let profile_rows_per_sec = rows as f64 / per_profile.as_secs_f64();
    println!("profile: {per_profile:?}/run  {profile_rows_per_sec:>12.0} rows/s",);

    let combined = per_ingest + per_profile;
    println!(
        "load+profile: {combined:?}/run  {:>12.0} rows/s",
        rows as f64 / combined.as_secs_f64()
    );
    println!("fingerprint: {fingerprint:#018x}");
}
