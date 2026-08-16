# Migrations

The crate exposes `BundleFormatVersion::{V0_1}` and `migration_path`. For equal supported versions it returns the singleton path `[V0_1]`; there are currently no other versions or transformation steps. This API is a compatibility planning hook, not an implemented migration engine.

Do not relabel, edit, or decode a newer bundle as 0.1. A future version must define a new exact manifest contract, accepted reader behavior, and an explicit lossless or documented-lossy transform before it is added to the migration graph.
