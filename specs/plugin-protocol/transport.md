# Transport framing

Frame is a complete length-delimited binary message: a four-byte big-endian body
length, two-byte big-endian protocol version, one-byte kind, one reserved zero
byte, eight-byte big-endian request id, then payload bytes. Protocol version is
1; total body bytes are capped at 16 MiB.

Kinds are Hello=1, Request=2, Response=3, Error=4, and Shutdown=5. Decoders
MUST reject truncation, length mismatches, oversized frames, a nonzero reserved
byte, unknown kind, and unsupported version. Payload encoding and I/O transport
are deliberately not specified by this crate.
