"""Public exceptions raised by the typed Python facade."""


class LawSynthError(Exception):
    """Base class for recoverable LawSynth errors."""


class ValidationError(LawSynthError, ValueError):
    """Raised when an input violates a public data contract."""


class NativeError(LawSynthError, RuntimeError):
    """Raised when the native executable-world engine rejects an operation."""
