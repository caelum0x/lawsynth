/// Converts a domain error to the concise text shown by the PyO3 boundary.
pub fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}
