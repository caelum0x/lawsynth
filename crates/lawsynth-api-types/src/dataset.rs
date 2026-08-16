use crate::{ApiValidationError, ProjectId, project::validate_identifier};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatasetId(String);

impl DatasetId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ApiValidationError> {
        let value = value.into();
        validate_identifier("dataset_id", &value, 128)?;
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColumnType {
    Float64,
    Int64,
    Boolean,
    Utf8,
    TimestampMillis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetColumn {
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
}

impl DatasetColumn {
    pub fn new(
        name: impl Into<String>,
        column_type: ColumnType,
        nullable: bool,
    ) -> Result<Self, ApiValidationError> {
        let name = name.into();
        validate_identifier("column.name", &name, 128)?;
        Ok(Self {
            name,
            column_type,
            nullable,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetDescriptor {
    pub id: DatasetId,
    pub project_id: ProjectId,
    pub columns: Vec<DatasetColumn>,
    pub row_count: u64,
    pub content_sha256: String,
}

impl DatasetDescriptor {
    pub fn new(
        id: DatasetId,
        project_id: ProjectId,
        columns: Vec<DatasetColumn>,
        row_count: u64,
        content_sha256: impl Into<String>,
    ) -> Result<Self, ApiValidationError> {
        if columns.is_empty() {
            return Err(ApiValidationError::Empty { field: "columns" });
        }
        for (index, column) in columns.iter().enumerate() {
            if columns[..index]
                .iter()
                .any(|prior| prior.name == column.name)
            {
                return Err(ApiValidationError::Invalid {
                    field: "columns",
                    reason: "names must be unique",
                });
            }
        }
        let content_sha256 = content_sha256.into();
        let hex = content_sha256.len() == 64
            && content_sha256.bytes().all(|byte| byte.is_ascii_hexdigit());
        if !hex {
            return Err(ApiValidationError::Invalid {
                field: "content_sha256",
                reason: "must be a 64-character hexadecimal SHA-256 digest",
            });
        }
        Ok(Self {
            id,
            project_id,
            columns,
            row_count,
            content_sha256: content_sha256.to_ascii_lowercase(),
        })
    }
}
