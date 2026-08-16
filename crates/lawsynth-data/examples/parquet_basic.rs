//! The native codec deliberately supports a safe subset: flat required numeric
//! columns in uncompressed, PLAIN-encoded Parquet pages.
use lawsynth_data::inspect_parquet;

fn main() {
    let invalid = b"not-a-parquet-file";
    match inspect_parquet(invalid) {
        Ok(envelope) => println!(
            "Parquet metadata begins at byte {} and is {} bytes long",
            envelope.metadata_offset, envelope.metadata_length
        ),
        Err(error) => println!("Input rejected before decoding: {error}"),
    }
    println!("Use read_parquet_numeric(bytes, \"time\") for supported numeric files.");
}
