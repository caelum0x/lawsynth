/// Versioned bundle formats accepted by the initial on-disk compatibility contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BundleFormatVersion {
    V0_1,
}

/// Returns each required forward migration. No migration is fabricated for an
/// unsupported format: callers must reject it rather than guessing semantics.
pub fn migration_path(
    from: BundleFormatVersion,
    to: BundleFormatVersion,
) -> Vec<BundleFormatVersion> {
    if from == to { vec![from] } else { Vec::new() }
}
