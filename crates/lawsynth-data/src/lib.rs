//! Typed, validated numerical time-series inputs for discovery pipelines.

mod batch;
mod column;
mod config;
mod dataset;
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
pub use error::DataError;
pub use fingerprint::DatasetFingerprint;
pub use parquet::{ParquetEnvelope, ParquetError, inspect_parquet, read_parquet_numeric};
pub use schema::DatasetSchema;
pub use time_axis::TimeAxis;
pub use window::WindowConfig;
