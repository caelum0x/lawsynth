# Error boundary

ApiValidationError reports invalid fields, empty required values, excessive
length, malformed identifiers, and invalid event sequencing. It is a local Rust
validation error, not an HTTP status or RFC-style error body.

A service MUST map validation, authentication, authorization, conflict, rate
limit, internal, and dependency failures into a documented transport error
format. It MUST NOT expose internal error strings as its sole machine-readable
contract, and no status-code mapping is defined by this repository.
