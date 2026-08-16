# Service API boundary

This directory defines forward-compatible service contracts using the validated
types in lawsynth-api-types. It is intentionally a boundary specification, not
evidence of an operational HTTP, gRPC, queue, authentication, or streaming
service. The current CLI serve command explicitly reports that daemon mode is
not compiled into this distribution.

Implementers MAY build a service against these contracts, but MUST publish a
concrete transport schema and security policy before claiming interoperability.
