"""Public exceptions raised by the typed Python facade."""


class LawSynthError(Exception):
    """Base class for recoverable LawSynth errors."""


class ValidationError(LawSynthError, ValueError):
    """Raised when an input violates a public data contract."""


class NativeError(LawSynthError, RuntimeError):
    """Raised when the native executable-world engine rejects an operation."""


class ApiError(LawSynthError):
    """Raised when a LawSynth API service returns an error envelope.

    Carries the transport ``status`` alongside the service's typed error
    ``code``, human ``message``, and the ``request_id`` that correlates the
    failure with the server's logs (``{"error": {code, message, request_id}}``).
    """

    def __init__(
        self,
        message: str,
        *,
        status: int,
        code: str = "error",
        request_id: str | None = None,
    ) -> None:
        super().__init__(message)
        self.status = status
        self.code = code
        self.message = message
        self.request_id = request_id

    def __str__(self) -> str:
        suffix = f" (request_id={self.request_id})" if self.request_id else ""
        return f"[{self.status} {self.code}] {self.message}{suffix}"


class RunTimeout(LawSynthError):
    """Raised when a run does not reach a terminal status within the poll bound."""

    def __init__(self, run_id: str, *, status: str, attempts: int) -> None:
        super().__init__(
            f"run {run_id!r} did not reach a terminal status after {attempts} "
            f"poll attempts (last status: {status!r})"
        )
        self.run_id = run_id
        self.status = status
        self.attempts = attempts
