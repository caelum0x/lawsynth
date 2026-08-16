use crate::PluginError;
use std::collections::BTreeSet;
use std::str::FromStr;

/// An operation that is denied until both the manifest and host policy allow it.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    ReadDataset,
    WriteArtifact,
    FilesystemRead,
    FilesystemWrite,
    Network,
    ExecuteProcess,
    WorldValidate,
    Algorithm,
    DataAdapter,
    Simulator,
}

impl Capability {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadDataset => "dataset.read",
            Self::WriteArtifact => "artifact.write",
            Self::FilesystemRead => "filesystem.read",
            Self::FilesystemWrite => "filesystem.write",
            Self::Network => "network",
            Self::ExecuteProcess => "process.execute",
            Self::WorldValidate => "world.validate",
            Self::Algorithm => "algorithm",
            Self::DataAdapter => "data.adapter",
            Self::Simulator => "simulator",
        }
    }
}

impl FromStr for Capability {
    type Err = PluginError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "dataset.read" => Ok(Self::ReadDataset),
            "artifact.write" => Ok(Self::WriteArtifact),
            "filesystem.read" => Ok(Self::FilesystemRead),
            "filesystem.write" => Ok(Self::FilesystemWrite),
            "network" => Ok(Self::Network),
            "process.execute" => Ok(Self::ExecuteProcess),
            "world.validate" => Ok(Self::WorldValidate),
            "algorithm" => Ok(Self::Algorithm),
            "data.adapter" => Ok(Self::DataAdapter),
            "simulator" => Ok(Self::Simulator),
            other => Err(PluginError::InvalidCapability(other.to_owned())),
        }
    }
}

/// Deterministically ordered declared or granted capabilities.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilitySet(BTreeSet<Capability>);

impl CapabilitySet {
    pub fn new(values: impl IntoIterator<Item = Capability>) -> Self {
        Self(values.into_iter().collect())
    }
    pub fn contains(&self, capability: Capability) -> bool {
        self.0.contains(&capability)
    }
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
    }
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.0.iter().copied()
    }
    pub fn insert(&mut self, capability: Capability) {
        self.0.insert(capability);
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
