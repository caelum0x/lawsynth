"""Typed errors with HTTP-safe status codes."""

from __future__ import annotations


class ServerError(Exception):
    status_code = 500
    code = "internal_error"

    def __init__(self, message: str, *, details: dict[str, object] | None = None) -> None:
        super().__init__(message)
        self.message, self.details = message, details or {}


class ValidationError(ServerError):
    status_code, code = 422, "validation_error"


class AuthenticationError(ServerError):
    status_code, code = 401, "authentication_required"


class AuthorizationError(ServerError):
    status_code, code = 403, "forbidden"


class NotFoundError(ServerError):
    status_code, code = 404, "not_found"


class ConflictError(ServerError):
    status_code, code = 409, "conflict"


class IdempotencyConflict(ConflictError):
    code = "idempotency_conflict"


class NativeUnavailableError(ServerError):
    """The optional executable LawSynth runtime is not installed for this server."""

    status_code, code = 503, "native_unavailable"
