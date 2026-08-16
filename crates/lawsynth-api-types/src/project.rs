use crate::ApiValidationError;

/// Stable, URL-safe project identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ApiValidationError> {
        let value = value.into();
        validate_identifier("project_id", &value, 128)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProjectId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Project metadata deliberately excludes user-supplied executable code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub display_name: String,
    pub created_at_ms: u64,
}

impl Project {
    pub fn new(
        id: ProjectId,
        display_name: impl Into<String>,
        created_at_ms: u64,
    ) -> Result<Self, ApiValidationError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(ApiValidationError::Empty {
                field: "display_name",
            });
        }
        if display_name.len() > 256 {
            return Err(ApiValidationError::TooLong {
                field: "display_name",
                maximum: 256,
            });
        }
        Ok(Self {
            id,
            display_name,
            created_at_ms,
        })
    }
}

pub(crate) fn validate_identifier(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), ApiValidationError> {
    if value.is_empty() {
        return Err(ApiValidationError::Empty { field });
    }
    if value.len() > maximum {
        return Err(ApiValidationError::TooLong { field, maximum });
    }
    let valid = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if !valid {
        return Err(ApiValidationError::Invalid {
            field,
            reason: "use ASCII letters, digits, hyphens, or underscores",
        });
    }
    Ok(())
}
