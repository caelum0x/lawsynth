use crate::{ArtifactError, ArtifactId, Retention, UploadOptions};

/// Durable metadata that publishes an object to readers after its bytes are committed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactMetadata {
    pub id: ArtifactId,
    pub sha256: String,
    pub size_bytes: u64,
    pub created_at_unix_seconds: u64,
    pub content_type: Option<String>,
    pub retention: Retention,
}

impl ArtifactMetadata {
    pub fn new(
        id: ArtifactId,
        size_bytes: u64,
        created_at_unix_seconds: u64,
        options: UploadOptions,
    ) -> Result<Self, ArtifactError> {
        options.validate()?;
        Ok(Self {
            sha256: id.as_str().to_owned(),
            id,
            size_bytes,
            created_at_unix_seconds,
            content_type: options.content_type,
            retention: options.retention,
        })
    }

    pub fn is_expired(&self, now_unix_seconds: u64) -> bool {
        self.retention.is_expired(now_unix_seconds)
    }

    /// Line-oriented durable encoding with a version marker and ignorable future fields.
    pub fn encode(&self) -> String {
        format!(
            "version=1\nid={}\nsha256={}\nsize_bytes={}\ncreated_at={}\ncontent_type={}\nexpires_at={}\n",
            self.id,
            self.sha256,
            self.size_bytes,
            self.created_at_unix_seconds,
            self.content_type.as_deref().unwrap_or(""),
            self.retention
                .expires_at_unix_seconds
                .map(|value| value.to_string())
                .unwrap_or_default()
        )
    }

    pub fn decode(input: &[u8]) -> Result<Self, ArtifactError> {
        let input = std::str::from_utf8(input)
            .map_err(|_| ArtifactError::CorruptMetadata("metadata is not UTF-8".into()))?;
        let mut fields = std::collections::BTreeMap::new();
        for line in input.lines() {
            let (name, value) = line
                .split_once('=')
                .ok_or_else(|| ArtifactError::CorruptMetadata("metadata line has no '='".into()))?;
            if fields.insert(name, value).is_some() {
                return Err(ArtifactError::CorruptMetadata(format!("metadata repeats '{name}'")));
            }
        }
        if fields.remove("version") != Some("1") {
            return Err(ArtifactError::CorruptMetadata(
                "unsupported or missing metadata version".into(),
            ));
        }
        let id = ArtifactId::new(required(&fields, "id")?.to_owned())?;
        let sha256 = required(&fields, "sha256")?.to_owned();
        if sha256 != id.as_str() {
            return Err(ArtifactError::CorruptMetadata("id and SHA-256 disagree".into()));
        }
        let size_bytes = parse_u64(&fields, "size_bytes")?;
        let created_at_unix_seconds = parse_u64(&fields, "created_at")?;
        let content_type = fields
            .get("content_type")
            .filter(|value| !value.is_empty())
            .map(|value| (*value).to_owned());
        let expires_at_unix_seconds = match required(&fields, "expires_at")? {
            "" => None,
            value => Some(value.parse::<u64>().map_err(|_| {
                ArtifactError::CorruptMetadata("expires_at is not an unsigned integer".into())
            })?),
        };
        let options =
            UploadOptions { content_type, retention: Retention { expires_at_unix_seconds } };
        options.validate()?;
        Ok(Self {
            id,
            sha256,
            size_bytes,
            created_at_unix_seconds,
            content_type: options.content_type,
            retention: options.retention,
        })
    }
}

fn required<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    field: &str,
) -> Result<&'a str, ArtifactError> {
    fields
        .get(field)
        .copied()
        .ok_or_else(|| ArtifactError::CorruptMetadata(format!("metadata omits '{field}'")))
}

fn parse_u64(
    fields: &std::collections::BTreeMap<&str, &str>,
    field: &str,
) -> Result<u64, ArtifactError> {
    required(fields, field)?
        .parse::<u64>()
        .map_err(|_| ArtifactError::CorruptMetadata(format!("{field} is not an unsigned integer")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum::sha256;

    #[test]
    fn durable_metadata_round_trips_and_rejects_address_changes() {
        let id = ArtifactId::new(sha256(b"bytes")).unwrap();
        let metadata = ArtifactMetadata::new(
            id,
            5,
            10,
            UploadOptions {
                content_type: Some("application/octet-stream".into()),
                retention: Retention::until(11),
            },
        )
        .unwrap();
        assert_eq!(ArtifactMetadata::decode(metadata.encode().as_bytes()).unwrap(), metadata);
        assert!(ArtifactMetadata::decode(b"version=1\nid=x\n").is_err());
    }
}
