"""SSO admission-seam tests for the Python gateway (hosted-platform P10).

These drive the *real* API WSGI backend (two tenants) behind the gateway with an
:class:`SsoAuthenticator` configured, and assert the seam's honest contract:
verified assertions are exchanged for the right tenant's backend principal,
tampered/missing assertions are rejected at the edge (401), unprovisioned tenants
are refused (403), the machine bearer surface still works, and a client cannot
smuggle a foreign tenant's bearer past the exchange.
"""

from __future__ import annotations

import io
import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
for source in (ROOT / "services/gateway/src", ROOT / "services/api/src", ROOT / "python/lawsynth-server/src", ROOT / "python/lawsynth/src"):
    sys.path.insert(0, str(source))

from lawsynth_api import ApiSettings, create_wsgi_app  # noqa: E402
from lawsynth_gateway import GatewaySettings, SsoAuthenticator, create_gateway  # noqa: E402
from lawsynth_gateway.sso import ASSERTION_HEADER  # noqa: E402
from lawsynth_server.settings import Settings as ServerSettings  # noqa: E402

TOKEN_ACME = "0123456789abcdef0123456789abcdef"
TOKEN_GLOBEX = "fedcba9876543210fedcba9876543210"
SECRET = "sso-shared-hmac-secret-value"


def invoke(app, method: str, path: str, *, body: bytes = b"", headers: dict[str, str] | None = None, remote: str = "127.0.0.1") -> tuple[int, dict[str, str], Any]:
    environ: dict[str, object] = {"REQUEST_METHOD": method, "PATH_INFO": path, "QUERY_STRING": "", "CONTENT_LENGTH": str(len(body)), "wsgi.input": io.BytesIO(body), "REMOTE_ADDR": remote}
    for name, value in (headers or {}).items():
        key = "CONTENT_TYPE" if name.lower() == "content-type" else "HTTP_" + name.upper().replace("-", "_")
        environ[key] = value
    captured: dict[str, object] = {}
    payload = b"".join(app(environ, lambda status, response_headers: captured.update(status=status, headers=dict(response_headers))))
    decoded = json.loads(payload) if payload else None
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], decoded


class SsoSeamTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        directory = Path(self.temporary.name)
        domain_settings = ServerSettings(
            database_url=f"sqlite:///{directory / 'metadata.sqlite3'}",
            object_root=directory / "objects",
            tokens={
                TOKEN_ACME: ("acme", frozenset({"read", "write"})),
                TOKEN_GLOBEX: ("globex", frozenset({"read", "write"})),
            },
            max_upload_bytes=4096,
        )
        self.api = create_wsgi_app(ApiSettings(server=domain_settings, environment="test", max_request_bytes=4096))
        # The trust map is the provisioning boundary: SSO tenant -> backend token.
        self.sso = SsoAuthenticator(SECRET, {"acme": TOKEN_ACME, "globex": TOKEN_GLOBEX})
        self.gateway = create_gateway(self.api, GatewaySettings(max_body_bytes=4096, requests_per_window=100), sso=self.sso)

    def tearDown(self) -> None:
        self.gateway.close()
        self.temporary.cleanup()

    def _assertion(self, org: str, *, subject: str = "user@example", scopes=("read", "write")) -> str:
        return self.sso.issue(subject, org, scopes)

    def test_valid_assertion_is_exchanged_and_tenant_bound(self) -> None:
        # acme creates a project via an SSO assertion (no bearer supplied).
        raw = json.dumps({"name": "coastal"}).encode()
        status, _, created = invoke(
            self.gateway, "POST", "/v1/projects", body=raw,
            headers={ASSERTION_HEADER: self._assertion("acme"), "content-type": "application/json", "idempotency-key": "sso-create"},
        )
        self.assertEqual(status, 201)
        self.assertEqual(created["name"], "coastal")

        # The same acme assertion sees it; a globex assertion (different exchange)
        # sees nothing -- the exchange bound the resource to acme's tenant.
        status, _, acme_list = invoke(self.gateway, "GET", "/v1/projects", headers={ASSERTION_HEADER: self._assertion("acme")})
        self.assertEqual((status, acme_list["items"]), (200, [created]))
        status, _, globex_list = invoke(self.gateway, "GET", "/v1/projects", headers={ASSERTION_HEADER: self._assertion("globex")})
        self.assertEqual((status, globex_list["items"]), (200, []))

    def test_tampered_assertion_is_rejected_401(self) -> None:
        assertion = self._assertion("acme")
        # Flip the last signature character: HMAC verification must fail.
        tampered = assertion[:-1] + ("0" if assertion[-1] != "0" else "1")
        status, _, body = invoke(self.gateway, "GET", "/v1/projects", headers={ASSERTION_HEADER: tampered})
        self.assertEqual(status, 401)
        self.assertEqual(body["error"]["code"], "invalid_assertion")

    def test_forged_tenant_in_payload_is_rejected(self) -> None:
        # An attacker cannot re-sign: swapping the payload for a foreign tenant
        # invalidates the signature (401) rather than exchanging as that tenant.
        import base64

        forged_payload = base64.urlsafe_b64encode(json.dumps({"sub": "e", "org": "globex", "scopes": ["write"]}, sort_keys=True, separators=(",", ":")).encode()).rstrip(b"=").decode()
        acme = self._assertion("acme")
        forged = forged_payload + "." + acme.split(".", 1)[1]  # keep acme's signature
        status, _, body = invoke(self.gateway, "GET", "/v1/projects", headers={ASSERTION_HEADER: forged})
        self.assertEqual((status, body["error"]["code"]), (401, "invalid_assertion"))

    def test_unprovisioned_tenant_is_forbidden_403(self) -> None:
        status, _, body = invoke(self.gateway, "GET", "/v1/projects", headers={ASSERTION_HEADER: self._assertion("stranger")})
        self.assertEqual(status, 403)
        self.assertEqual(body["error"]["code"], "tenant_forbidden")

    def test_no_assertion_and_no_bearer_is_rejected_at_edge(self) -> None:
        # With SSO enabled, a request bearing neither credential is refused at the
        # edge (401) before it reaches the backend.
        status, _, body = invoke(self.gateway, "GET", "/v1/projects")
        self.assertEqual(status, 401)
        self.assertEqual(body["error"]["code"], "authentication_required")

    def test_machine_bearer_surface_still_works_without_an_assertion(self) -> None:
        # Bearer tokens remain the machine surface: a raw bearer with no assertion
        # is passed through and authenticated by the backend.
        status, _, listed = invoke(self.gateway, "GET", "/v1/projects", headers={"Authorization": f"Bearer {TOKEN_ACME}"})
        self.assertEqual(status, 200)
        self.assertEqual(listed["items"], [])

    def test_client_cannot_smuggle_a_foreign_bearer_past_the_exchange(self) -> None:
        # An acme assertion accompanied by a globex bearer: the seam strips the
        # client Authorization and injects acme's exchanged token, so the resource
        # is created under acme -- not globex.
        raw = json.dumps({"name": "bound-to-acme"}).encode()
        status, _, _ = invoke(
            self.gateway, "POST", "/v1/projects", body=raw,
            headers={ASSERTION_HEADER: self._assertion("acme"), "authorization": f"Bearer {TOKEN_GLOBEX}", "content-type": "application/json", "idempotency-key": "smuggle"},
        )
        self.assertEqual(status, 201)

        # Proof: globex sees nothing; acme owns it.
        _, _, globex_list = invoke(self.gateway, "GET", "/v1/projects", headers={"Authorization": f"Bearer {TOKEN_GLOBEX}"})
        self.assertEqual(globex_list["items"], [])
        _, _, acme_list = invoke(self.gateway, "GET", "/v1/projects", headers={ASSERTION_HEADER: self._assertion("acme")})
        self.assertEqual([item["name"] for item in acme_list["items"]], ["bound-to-acme"])


if __name__ == "__main__":
    unittest.main()
