use crate::HostError;
use lawsynth_plugin_api::PluginManifest;
use std::fs;
use std::path::Path;

/// Read immediate child plugin directories in deterministic filename order.
/// A directory contributes only when it contains a valid `plugin.manifest`.
pub fn discover_manifests(root: &Path) -> Result<Vec<PluginManifest>, HostError> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut manifests = Vec::new();
    for entry in entries {
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join("plugin.manifest");
        if path.is_file() {
            manifests.push(PluginManifest::parse(&fs::read_to_string(path)?)?);
        }
    }
    Ok(manifests)
}
