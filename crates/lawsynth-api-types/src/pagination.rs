use crate::ApiValidationError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub after: Option<String>,
    pub limit: u32,
}

impl PageRequest {
    pub fn new(
        after: Option<String>,
        limit: u32,
        maximum_limit: u32,
    ) -> Result<Self, ApiValidationError> {
        if limit == 0 || limit > maximum_limit {
            return Err(ApiValidationError::OutOfRange {
                field: "limit",
                minimum: 1,
                maximum: u64::from(maximum_limit),
            });
        }
        if let Some(cursor) = &after {
            if cursor.is_empty()
                || cursor.len() > 512
                || !cursor
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            {
                return Err(ApiValidationError::Invalid {
                    field: "after",
                    reason: "must be a URL-safe opaque cursor up to 512 bytes",
                });
            }
        }
        Ok(Self { after, limit })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next: Option<String>,
}

impl<T> Page<T> {
    pub fn new(
        items: Vec<T>,
        next: Option<String>,
        requested_limit: u32,
    ) -> Result<Self, ApiValidationError> {
        if items.len() > requested_limit as usize {
            return Err(ApiValidationError::Inconsistent {
                reason: "page has more items than its requested limit",
            });
        }
        if let Some(cursor) = &next {
            PageRequest::new(Some(cursor.clone()), 1, 1)?;
        }
        Ok(Self { items, next })
    }
}
