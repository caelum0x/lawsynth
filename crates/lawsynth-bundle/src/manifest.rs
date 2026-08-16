pub(crate) const MANIFEST_PATH: &str = "manifest.json";
pub(crate) const WORLD_PATH: &str = "world/world.bin";
pub(crate) const CHECKSUM_PATH: &str = "provenance/checksums.sha256";

pub(crate) fn contents() -> &'static [u8] {
    br#"{
  "format": "lawsynth-world",
  "format_version": "0.1.0",
  "world_encoding": "lawsynth-world-binary-v1"
}
"#
}
