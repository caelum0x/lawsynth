from lawsynth_server.errors import NotFoundError
from lawsynth_server.errors_api import error_body


def test_error_body_has_machine_code_and_request_id():
    assert error_body(NotFoundError("gone"), "r")["error"] == {"code": "not_found", "message": "gone", "details": {}, "request_id": "r"}
