/// Bounds for safely accepting bundle payloads from untrusted sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BundleConfig {
    pub maximum_entries: usize,
    pub maximum_entry_bytes: usize,
    pub maximum_total_bytes: usize,
}
impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            maximum_entries: 64,
            maximum_entry_bytes: 64 * 1024 * 1024,
            maximum_total_bytes: 256 * 1024 * 1024,
        }
    }
}
impl BundleConfig {
    pub fn accepts(self, entries: usize, largest_entry: usize, total_bytes: usize) -> bool {
        self.maximum_entries > 0
            && entries <= self.maximum_entries
            && largest_entry <= self.maximum_entry_bytes
            && total_bytes <= self.maximum_total_bytes
    }
}
