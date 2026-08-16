# Python errors

`LawSynthError` is the recoverable SDK base exception. `ValidationError` is also a `ValueError` and reports a violated Python-facing input contract. `NativeError` is also a `RuntimeError` and is used when the native executable layer is unavailable or rejects an operation.

Callers should validate models before execution and catch `LawSynthError` at API boundaries. Native extension errors may preserve additional implementation-specific messages; do not parse message text as a stable protocol.
