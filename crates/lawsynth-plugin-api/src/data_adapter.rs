use crate::PluginError;
use std::collections::BTreeSet;

/// Scalar values supported by the stable plugin data boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarType {
    Float64,
    Int64,
    Boolean,
    Utf8,
    Binary,
}

impl ScalarType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Float64 => "float64",
            Self::Int64 => "int64",
            Self::Boolean => "boolean",
            Self::Utf8 => "utf8",
            Self::Binary => "binary",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Column {
    pub name: String,
    pub scalar_type: ScalarType,
    pub nullable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataSchema {
    pub columns: Vec<Column>,
}

impl DataSchema {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.columns.is_empty() {
            return Err(PluginError::InvalidData("schema has no columns".into()));
        }

        let mut names = BTreeSet::new();
        for column in &self.columns {
            validate_column_name(&column.name)?;
            if !names.insert(column.name.as_str()) {
                return Err(PluginError::InvalidData(format!(
                    "duplicate column {:?}",
                    column.name
                )));
            }
        }
        Ok(())
    }
}

/// An owned, typed column buffer. Nullable variants make nullability explicit
/// instead of using sentinel values such as NaN or an empty string.
#[derive(Clone, Debug, PartialEq)]
pub enum DataBatch {
    Float64(Vec<f64>),
    Int64(Vec<i64>),
    Boolean(Vec<bool>),
    Utf8(Vec<String>),
    Binary(Vec<Vec<u8>>),
    NullableFloat64(Vec<Option<f64>>),
    NullableInt64(Vec<Option<i64>>),
    NullableBoolean(Vec<Option<bool>>),
    NullableUtf8(Vec<Option<String>>),
    NullableBinary(Vec<Option<Vec<u8>>>),
}

impl DataBatch {
    pub fn len(&self) -> usize {
        match self {
            Self::Float64(values) => values.len(),
            Self::Int64(values) => values.len(),
            Self::Boolean(values) => values.len(),
            Self::Utf8(values) => values.len(),
            Self::Binary(values) => values.len(),
            Self::NullableFloat64(values) => values.len(),
            Self::NullableInt64(values) => values.len(),
            Self::NullableBoolean(values) => values.len(),
            Self::NullableUtf8(values) => values.len(),
            Self::NullableBinary(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn scalar_type(&self) -> ScalarType {
        match self {
            Self::Float64(_) | Self::NullableFloat64(_) => ScalarType::Float64,
            Self::Int64(_) | Self::NullableInt64(_) => ScalarType::Int64,
            Self::Boolean(_) | Self::NullableBoolean(_) => ScalarType::Boolean,
            Self::Utf8(_) | Self::NullableUtf8(_) => ScalarType::Utf8,
            Self::Binary(_) | Self::NullableBinary(_) => ScalarType::Binary,
        }
    }

    pub const fn is_nullable(&self) -> bool {
        matches!(
            self,
            Self::NullableFloat64(_)
                | Self::NullableInt64(_)
                | Self::NullableBoolean(_)
                | Self::NullableUtf8(_)
                | Self::NullableBinary(_)
        )
    }

    pub fn null_count(&self) -> usize {
        match self {
            Self::NullableFloat64(values) => values.iter().filter(|value| value.is_none()).count(),
            Self::NullableInt64(values) => values.iter().filter(|value| value.is_none()).count(),
            Self::NullableBoolean(values) => values.iter().filter(|value| value.is_none()).count(),
            Self::NullableUtf8(values) => values.iter().filter(|value| value.is_none()).count(),
            Self::NullableBinary(values) => values.iter().filter(|value| value.is_none()).count(),
            _ => 0,
        }
    }

    /// Conservative in-memory estimate used before accepting untrusted data.
    pub fn estimated_bytes(&self) -> usize {
        let validity_bytes = if self.is_nullable() {
            self.len().div_ceil(8)
        } else {
            0
        };
        validity_bytes.saturating_add(match self {
            Self::Float64(values) => values.len().saturating_mul(size_of::<f64>()),
            Self::Int64(values) => values.len().saturating_mul(size_of::<i64>()),
            Self::Boolean(values) => values.len().div_ceil(8),
            Self::Utf8(values) => values.iter().map(String::len).sum(),
            Self::Binary(values) => values.iter().map(Vec::len).sum(),
            Self::NullableFloat64(values) => values.len().saturating_mul(size_of::<f64>()),
            Self::NullableInt64(values) => values.len().saturating_mul(size_of::<i64>()),
            Self::NullableBoolean(values) => values.len().div_ceil(8),
            Self::NullableUtf8(values) => values
                .iter()
                .filter_map(Option::as_ref)
                .map(String::len)
                .sum(),
            Self::NullableBinary(values) => {
                values.iter().filter_map(Option::as_ref).map(Vec::len).sum()
            }
        })
    }

    fn validate_values(&self, column: &str) -> Result<(), PluginError> {
        let invalid_float = match self {
            Self::Float64(values) => values.iter().any(|value| !value.is_finite()),
            Self::NullableFloat64(values) => {
                values.iter().flatten().any(|value| !value.is_finite())
            }
            _ => false,
        };
        if invalid_float {
            return Err(PluginError::InvalidData(format!(
                "column {column:?} contains a non-finite value"
            )));
        }

        let invalid_text = match self {
            Self::Utf8(values) => values.iter().any(|value| value.contains('\0')),
            Self::NullableUtf8(values) => values.iter().flatten().any(|value| value.contains('\0')),
            _ => false,
        };
        if invalid_text {
            return Err(PluginError::InvalidData(format!(
                "column {column:?} contains a null byte"
            )));
        }
        Ok(())
    }
}

/// Pull-based streaming adapter. Returning `None` ends the stream.
pub trait DataAdapter: Send {
    fn schema(&self) -> &DataSchema;

    fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<DataBatch>>, PluginError>;
}

/// Validate a column group before it crosses the plugin boundary.
pub fn validate_row_group(
    schema: &DataSchema,
    batches: &[DataBatch],
    max_rows: usize,
) -> Result<usize, PluginError> {
    schema.validate()?;
    if max_rows == 0 {
        return Err(PluginError::InvalidData("max_rows must be positive".into()));
    }
    if batches.len() != schema.columns.len() {
        return Err(PluginError::InvalidData(format!(
            "batch has {} columns but schema declares {}",
            batches.len(),
            schema.columns.len()
        )));
    }

    let rows = batches.first().map_or(0, DataBatch::len);
    if rows > max_rows {
        return Err(PluginError::ResourceLimit(format!(
            "batch row count {rows} exceeds host limit {max_rows}"
        )));
    }

    for (column, batch) in schema.columns.iter().zip(batches) {
        if column.scalar_type != batch.scalar_type() {
            return Err(PluginError::InvalidData(format!(
                "column {:?} expects {} but received {}",
                column.name,
                column.scalar_type.as_str(),
                batch.scalar_type().as_str()
            )));
        }
        if batch.len() != rows {
            return Err(PluginError::InvalidData(format!(
                "column {:?} has {} rows, expected {rows}",
                column.name,
                batch.len()
            )));
        }
        if !column.nullable && batch.null_count() != 0 {
            return Err(PluginError::InvalidData(format!(
                "non-nullable column {:?} contains null values",
                column.name
            )));
        }
        batch.validate_values(&column.name)?;
    }
    Ok(rows)
}

fn validate_column_name(name: &str) -> Result<(), PluginError> {
    if name.is_empty() || name.len() > 255 {
        return Err(PluginError::InvalidData(
            "column names must contain 1..=255 bytes".into(),
        ));
    }
    if name.contains('\0') || name.chars().any(char::is_control) {
        return Err(PluginError::InvalidData(format!(
            "invalid column name {name:?}"
        )));
    }
    Ok(())
}
