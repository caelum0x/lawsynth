use lawsynth_profile::DatasetProfile;
/// Emits a concise stable profile summary for command-line callers.
pub fn profile_summary(profile: &DatasetProfile) -> String {
    format!(
        "samples={}, columns={}\n",
        profile.samples,
        profile.columns.len()
    )
}
