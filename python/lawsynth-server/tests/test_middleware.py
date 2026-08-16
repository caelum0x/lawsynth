from lawsynth_server.errors import ValidationError
from lawsynth_server.middleware import invoke


def test_middleware_formats_public_errors():
    response = invoke(lambda _: (_ for _ in ()).throw(ValidationError("bad")), {})
    assert response["status"] == 422
    assert response["body"]["error"]["code"] == "validation_error"
