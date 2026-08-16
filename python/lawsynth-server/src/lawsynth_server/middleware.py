from __future__ import annotations

from uuid import uuid4

from .errors import ServerError
from .errors_api import error_body


def invoke(handler, request: dict[str, object]) -> dict[str, object]:
    request_id = str(uuid4())
    try:
        response = handler(request)
    except ServerError as error:
        return {"status": error.status_code, "headers": {"X-Request-ID": request_id}, "body": error_body(error, request_id)}
    except Exception:
        return {"status": 500, "headers": {"X-Request-ID": request_id}, "body": {"error": {"code": "internal_error", "message": "internal server error", "request_id": request_id}}}
    response.setdefault("headers", {})["X-Request-ID"] = request_id
    return response
