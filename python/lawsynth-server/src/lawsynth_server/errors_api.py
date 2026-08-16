from .errors import ServerError


def error_body(error: ServerError, request_id: str) -> dict[str, object]:
    return {"error": {"code": error.code, "message": error.message, "details": error.details, "request_id": request_id}}
