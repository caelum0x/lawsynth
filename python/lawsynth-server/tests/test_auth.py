import pytest

from lawsynth_server.auth import TokenAuthenticator, require_scope
from lawsynth_server.errors import AuthenticationError, AuthorizationError


def test_bearer_auth_and_scope():
    auth = TokenAuthenticator({"secret": ("org", frozenset({"read"}))})
    principal = auth.authenticate("Bearer secret")
    require_scope(principal, "read")
    with pytest.raises(AuthorizationError):
        require_scope(principal, "write")
    with pytest.raises(AuthenticationError):
        auth.authenticate("Basic secret")
