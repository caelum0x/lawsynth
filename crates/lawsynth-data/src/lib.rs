//! Typed, validated numerical time-series inputs for discovery pipelines.

mod batch;
mod column;
mod config;
mod dataset;
mod delimited;
mod error;
mod fingerprint;
mod parquet;
mod schema;
mod time_axis;
mod window;

pub use batch::DatasetBatch;
pub use column::NumericColumn;
pub use config::DatasetConfig;
pub use dataset::Dataset;
pub use delimited::{
    load_csv_numeric, load_csv_numeric_with_progress, load_delimited_numeric,
    load_delimited_numeric_with_progress, read_csv_numeric, read_csv_numeric_with_progress,
    read_delimited_numeric, read_delimited_numeric_with_progress, read_tsv_numeric,
    read_tsv_numeric_with_progress,
};
pub use error::DataError;
pub use fingerprint::DatasetFingerprint;
pub use parquet::{ParquetEnvelope, ParquetError, inspect_parquet, read_parquet_numeric};
pub use schema::DatasetSchema;
pub use time_axis::TimeAxis;
pub use window::WindowConfig;
