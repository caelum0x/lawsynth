# Rust error model

Crates expose domain errors such as `BundleError`, `WorldError`, simulation and discovery errors. Errors carry validation, semantic, input/output, cancellation, or unsupported-capability context and implement `std::error::Error` where appropriate.

Treat variants and structured fields as the programmatic interface. Human-readable `Display` messages identify the failing contract but are not a versioned serialization format.
