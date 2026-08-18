use crate::{Capability, CapabilitySet, PluginError, ResourceLimits};
use std::str::FromStr;

/// Execution isolation selected by the plugin author.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginKind {
    Wasi,
    Process,
    TrustedNative,
}

impl PluginKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wasi => "wasi",
            Self::Process => "process",
            Self::TrustedNative => "trusted-native",
        }
    }
}

impl FromStr for PluginKind {
    type Err = PluginError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "wasi" => Ok(Self::Wasi),
            "process" => Ok(Self::Process),
            "trusted-native" => Ok(Self::TrustedNative),
            v => Err(PluginError::InvalidManifest(format!("unknown plugin kind {v:?}"))),
        }
    }
}

/// Validated portable metadata, parsed from a deliberately small key-value manifest format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginManifest {
    pub id: String,
    pub version: String,
    pub kind: PluginKind,
    pub entrypoint: String,
    pub capabilities: CapabilitySet,
    pub limits: ResourceLimits,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), PluginError> {
        valid_id(&self.id)?;
        if !valid_version(&self.version) {
            return Err(PluginError::InvalidManifest("version must be major.minor.patch".into()));
        }
        if self.entrypoint.is_empty()
            || self.entrypoint.contains('\0')
            || self.entrypoint.contains("..")
        {
            return Err(PluginError::InvalidManifest("entrypoint is empty or unsafe".into()));
        }
        if self.kind == PluginKind::TrustedNative
            && !self.capabilities.contains(Capability::ExecuteProcess)
        {
            return Err(PluginError::InvalidManifest(
                "trusted-native plugins must explicitly declare process.execute".into(),
            ));
        }
        self.limits.validate()?;
        Ok(())
    }

    /// Parse a manifest without a general-purpose TOML dependency. The accepted
    /// grammar is one `key = value` per line; unknown and duplicate keys reject.
    pub fn parse(text: &str) -> Result<Self, PluginError> {
        let mut id = None;
        let mut version = None;
        let mut kind = None;
        let mut entrypoint = None;
        let mut capabilities = None;
        let mut limits = ResourceLimits::default();
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| {
                PluginError::InvalidManifest(format!("expected key = value: {line}"))
            })?;
            let key = key.trim();
            let value = unquote(value.trim())?;
            match key {
                "id" if id.is_none() => id = Some(value.to_owned()),
                "version" if version.is_none() => version = Some(value.to_owned()),
                "kind" if kind.is_none() => kind = Some(value.parse()?),
                "entrypoint" if entrypoint.is_none() => entrypoint = Some(value.to_owned()),
                "capabilities" if capabilities.is_none() => {
                    let parsed = value
                        .split(',')
                        .filter(|s| !s.trim().is_empty())
                        .map(str::parse)
                        .collect::<Result<Vec<Capability>, PluginError>>()?;
                    capabilities = Some(CapabilitySet::new(parsed));
                }
                "max_cpu_millis" => limits.max_cpu_millis = parse_num(value, key)?,
                "max_memory_bytes" => limits.max_memory_bytes = parse_num(value, key)?,
                "max_output_bytes" => limits.max_output_bytes = parse_num(value, key)?,
                "max_requests" => limits.max_requests = parse_num(value, key)?,
                _ => {
                    return Err(PluginError::InvalidManifest(format!(
                        "unknown or duplicate key {key:?}"
                    )));
                }
            }
        }
        let manifest = Self {
            id: id.ok_or_else(|| PluginError::InvalidManifest("missing id".into()))?,
            version: version
                .ok_or_else(|| PluginError::InvalidManifest("missing version".into()))?,
            kind: kind.ok_or_else(|| PluginError::InvalidManifest("missing kind".into()))?,
            entrypoint: entrypoint
                .ok_or_else(|| PluginError::InvalidManifest("missing entrypoint".into()))?,
            capabilities: capabilities.unwrap_or_default(),
            limits,
        };
        manifest.validate()?;
        Ok(manifest)
    }
}

fn unquote(value: &str) -> Result<&str, PluginError> {
    if value.starts_with('"') || value.ends_with('"') {
        if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            return Err(PluginError::InvalidManifest("unterminated quoted value".into()));
        }
        Ok(&value[1..value.len() - 1])
    } else {
        Ok(value)
    }
}
fn parse_num<T: FromStr>(value: &str, key: &str) -> Result<T, PluginError> {
    value
        .parse()
        .map_err(|_| PluginError::InvalidManifest(format!("{key} must be an unsigned integer")))
}
fn valid_id(id: &str) -> Result<(), PluginError> {
    if id.is_empty() || id.len() > 96 {
        return Err(PluginError::InvalidManifest("id must contain 1..=96 characters".into()));
    }
    if id.starts_with('-')
        || id.ends_with('-')
        || !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(PluginError::InvalidManifest(
            "id must use lowercase ASCII letters, digits, and internal hyphens".into(),
        ));
    }
    Ok(())
}
fn valid_version(version: &str) -> bool {
    let mut pieces = version.split('.');
    matches!((pieces.next(), pieces.next(), pieces.next(), pieces.next()), (Some(a), Some(b), Some(c), None) if a.parse::<u32>().is_ok() && b.parse::<u32>().is_ok() && c.parse::<u32>().is_ok())
}
