# Versioning boundary

lawsynth-api-types validates values but does not define a service version
negotiation mechanism, media-type parameter, route version, compatibility
matrix, or deprecation process. Its Rust public types are not a network schema.

Every service implementation MUST publish an explicit protocol version and
backward-compatibility policy. It MUST reject or negotiate unsupported client
versions rather than accepting data under ambiguous semantics. Versioning cannot
be inferred from a WorldRevision, which identifies world content rather than an
API protocol.
