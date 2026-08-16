# Bundle roundtrip

The Python reference fixture writer emits the documented stored-ZIP/binary-v1
wire format. Rust's CLI verifies checksums, decodes it, and runs the trajectory.
