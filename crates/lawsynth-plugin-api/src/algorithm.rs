use crate::{DataBatch, DataSchema, PluginError};

const MAX_TARGET_BYTES: usize = 255;
const MAX_EQUATION_BYTES: usize = 1 << 20;
const MAX_DIAGNOSTICS: usize = 1_024;
const MAX_DIAGNOSTIC_BYTES: usize = 16 * 1024;

/// Typed discovery invocation passed to an algorithm plugin.
#[derive(Clone, Debug, PartialEq)]
pub struct AlgorithmRequest {
    pub schema: DataSchema,
    pub columns: Vec<DataBatch>,
    pub target: String,
}

impl AlgorithmRequest {
    pub fn validate(&self) -> Result<(), PluginError> {
        self.validate_with_row_limit(usize::MAX)
    }

    pub fn validate_with_row_limit(&self, max_rows: usize) -> Result<(), PluginError> {
        crate::data_adapter::validate_row_group(&self.schema, &self.columns, max_rows)?;
        if self.target.is_empty()
            || self.target.len() > MAX_TARGET_BYTES
            || self.target.contains('\0')
        {
            return Err(PluginError::InvalidData(
                "algorithm target is empty, too large, or contains a null byte".into(),
            ));
        }

        let target = self
            .schema
            .columns
            .iter()
            .find(|column| column.name == self.target)
            .ok_or_else(|| {
                PluginError::InvalidData(format!(
                    "algorithm target {:?} is absent from schema",
                    self.target
                ))
            })?;
        if !matches!(
            target.scalar_type,
            crate::ScalarType::Float64 | crate::ScalarType::Int64
        ) {
            return Err(PluginError::Unsupported(format!(
                "algorithm target {:?} has non-numeric type {}",
                self.target,
                target.scalar_type.as_str()
            )));
        }
        Ok(())
    }

    pub fn row_count(&self) -> usize {
        self.columns.first().map_or(0, DataBatch::len)
    }

    pub fn estimated_bytes(&self) -> usize {
        self.columns
            .iter()
            .map(DataBatch::estimated_bytes)
            .fold(0usize, usize::saturating_add)
    }
}

/// A discovered candidate produced by a plugin. Scores are plugin-defined but
/// must be finite; the host records their interpretation in run provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct AlgorithmResponse {
    pub equation: String,
    pub score: f64,
    pub diagnostics: Vec<String>,
}

impl AlgorithmResponse {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.equation.is_empty()
            || self.equation.len() > MAX_EQUATION_BYTES
            || self.equation.contains('\0')
        {
            return Err(PluginError::InvalidData(format!(
                "algorithm equation must contain 1..={MAX_EQUATION_BYTES} bytes and no null byte"
            )));
        }
        if !self.score.is_finite() {
            return Err(PluginError::InvalidData(
                "algorithm score must be finite".into(),
            ));
        }
        if self.diagnostics.len() > MAX_DIAGNOSTICS {
            return Err(PluginError::ResourceLimit(format!(
                "algorithm returned {} diagnostics, limit is {MAX_DIAGNOSTICS}",
                self.diagnostics.len()
            )));
        }
        if self
            .diagnostics
            .iter()
            .any(|message| message.len() > MAX_DIAGNOSTIC_BYTES || message.contains('\0'))
        {
            return Err(PluginError::InvalidData(format!(
                "diagnostics must be at most {MAX_DIAGNOSTIC_BYTES} bytes and contain no null byte"
            )));
        }
        Ok(())
    }
}

/// Object-safe discovery plugin contract. Implementations must be thread-safe
/// because hosts may schedule independent requests concurrently.
pub trait AlgorithmPlugin: Send + Sync {
    fn discover(&self, request: AlgorithmRequest) -> Result<AlgorithmResponse, PluginError>;
}
