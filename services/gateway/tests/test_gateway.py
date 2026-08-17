from __future__ import annotations

import io
import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
for source in (ROOT / "services/gateway/src", ROOT / "services/api/src", ROOT / "python/lawsynth-server/src"):
    sys.path.insert(0, str(source))

from lawsynth_api import ApiSettings, create_wsgi_app  # noqa: E402
from lawsynth_gateway import GatewaySettings, create_gateway  # noqa: E402
from lawsynth_gateway.app import InProcessWsgiBackend, RemoteUpstreamUnavailable  # noqa: E402
from lawsynth_server.settings import Settings as ServerSettings  # noqa: E402

TOKEN = "0123456789abcdef0123456789abcdef"


def invoke(app, method: str, path: str, *, body: bytes = b"", headers: dict[str, str] | None = None, remote: str = "127.0.0.1") -> tuple[int, dict[str, str], Any]:
    environ: dict[str, object] = {"REQUEST_METHOD": method, "PATH_INFO": path, "QUERY_STRING": "", "CONTENT_LENGTH": str(len(body)), "wsgi.input": io.BytesIO(body), "REMOTE_ADDR": remote}
    for name, value in (headers or {}).items():
        key = "CONTENT_TYPE" if name.lower() == "content-type" else "HTTP_" + name.upper().replace("-", "_")
        environ[key] = value
    captured: dict[str, object] = {}
    payload = b"".join(app(environ, lambda status, response_headers: captured.update(status=status, headers=dict(response_headers))))
    decoded = json.loads(payload) if payload else None
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], decoded


class GatewayIntegrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        directory = Path(self.temporary.name)
        domain_settings = ServerSettings(database_url=f"sqlite:///{directory / 'metadata.sqlite3'}", object_root=directory / "objects", tokens={TOKEN: ("acme", frozenset({"read", "write"}))}, max_upload_bytes=1024)
        self.api = create_wsgi_app(ApiSettings(server=domain_settings, environment="test", max_request_bytes=1024))
        self.gateway = create_gateway(self.api, GatewaySettings(max_body_bytes=1024, requests_per_window=2, rate_window_seconds=60, allowed_origins=frozenset({"https://studio.example"})))

    def tearDown(self) -> None:
        self.gateway.close()
        self.temporary.cleanup()

    def test_actual_api_lifecycle_and_canonical_forwarding(self) -> None:
        document = {"name": "Coastal", "metadata": {"source": "real WSGI dispatch"}}
        raw = json.dumps(document).encode()
        status, headers, created = invoke(self.gateway, "POST", "/v1/projects", body=raw, headers={"authorization": f"Bearer {TOKEN}", "content-type": "application/json", "idempotency-key": "create-coastal", "x-request-id": "client.req-123", "x-forwarded-for": "forged"})
        self.assertEqual(status, 201)
        self.assertEqual(headers["X-Request-Id"], "client.req-123")
        self.assertEqual(created["name"], "Coastal")
        status, _, listed = invoke(self.gateway, "GET", "/v1/projects", headers={"Authorization": f"Bearer {TOKEN}"})
        self.assertEqual(status, 200)
        self.assertEqual(listed["items"], [created])

    def test_cors_preflight_and_disallowed_origin(self) -> None:
        status, headers, body = invoke(self.gateway, "OPTIONS", "/v1/projects", headers={"Origin": "https://studio.example", "Access-Control-Request-Method": "POST", "Access-Control-Request-Headers": "Authorization, Content-Type"})
        self.assertEqual((status, body), (204, None))
        self.assertEqual(headers["Access-Control-Allow-Origin"], "https://studio.example")
        status, _, body = invoke(self.gateway, "GET", "/v1/health", headers={"Origin": "https://attacker.example"})
        self.assertEqual(status, 403)
        self.assertEqual(body["error"]["code"], "origin_forbidden")

    def test_body_limit_rate_limit_and_safe_backend_error(self) -> None:
        oversized = b"x" * 1025
        status, _, body = invoke(self.gateway, "POST", "/v1/projects", body=oversized, headers={"Content-Type": "application/json"})
        self.assertEqual((status, body["error"]["code"]), (413, "payload_too_large"))
        # Admission is per source client and only complete API requests consume it.
        first = invoke(self.gateway, "GET", "/v1/health", headers={"Authorization": f"Bearer {TOKEN}"}, remote="192.0.2.1")
        second = invoke(self.gateway, "GET", "/v1/health", headers={"Authorization": f"Bearer {TOKEN}"}, remote="192.0.2.1")
        third = invoke(self.gateway, "GET", "/v1/health", headers={"Authorization": f"Bearer {TOKEN}"}, remote="192.0.2.1")
        self.assertEqual((first[0], second[0], third[0]), (200, 200, 429))

    def test_health_readiness_and_explicit_transport_boundary(self) -> None:
        status, _, health = invoke(self.gateway, "GET", "/healthz")
        self.assertEqual((status, health["status"]), (200, "ok"))
        with self.assertRaises(RemoteUpstreamUnavailable):
            InProcessWsgiBackend.remote("https://api.example")
        self.gateway.close()
        status, _, ready = invoke(self.gateway, "GET", "/readyz")
        self.assertEqual((status, ready["status"]), (503, "draining"))

    def test_forwarding_is_rebuilt_and_backend_failures_are_safe(self) -> None:
        observed: dict[str, object] = {}

        def inspecting_backend(environ, start_response):
            observed.update(environ)
            start_response("200 OK", [("Content-Type", "application/json")])
            return [b'{"ok":true}']

        gateway = create_gateway(inspecting_backend, GatewaySettings(requests_per_window=5))
        try:
            status, _, body = invoke(gateway, "GET", "/v1/health", headers={"X-Forwarded-For": "forged", "Forwarded": "for=forged"}, remote="192.0.2.9")
            self.assertEqual((status, body), (200, {"ok": True}))
            self.assertEqual(observed["HTTP_X_FORWARDED_FOR"], "192.0.2.9")
            self.assertNotIn("HTTP_FORWARDED", observed)
        finally:
            gateway.close()

        def failing_backend(environ, start_response):
            raise RuntimeError("sensitive upstream exception")

        broken = create_gateway(failing_backend, GatewaySettings(requests_per_window=5))
        try:
            status, _, body = invoke(broken, "GET", "/v1/health")
            self.assertEqual((status, body["error"]["code"]), (502, "backend_failure"))
            self.assertNotIn("sensitive", body["error"]["message"])
        finally:
            broken.close()

    def test_unauthenticated_request_is_forwarded_and_rejected_by_api(self) -> None:
        # The gateway is an admission layer, not an auth server: it forwards
        # the (missing) Authorization header and the non-internet-exposed API
        # makes the 401 decision.  This proves auth passthrough end to end.
        status, _, body = invoke(self.gateway, "GET", "/v1/projects")
        self.assertEqual(status, 401)
        self.assertEqual(body["error"]["code"], "authentication_required")

    def test_route_and_method_allowlist_reject_before_the_backend(self) -> None:
        observed: dict[str, object] = {}

        def counting_backend(environ, start_response):
            observed["called"] = True
            start_response("200 OK", [("Content-Type", "application/json")])
            return [b"{}"]

        gateway = create_gateway(counting_backend, GatewaySettings(requests_per_window=50))
        try:
            # Route outside the API prefix is rejected without reaching the backend.
            status, _, body = invoke(gateway, "GET", "/admin")
            self.assertEqual((status, body["error"]["code"]), (404, "route_not_found"))
            # Unsupported method is rejected during admission.
            status, _, body = invoke(gateway, "PUT", "/v1/projects")
            self.assertEqual((status, body["error"]["code"]), (405, "method_not_allowed"))
            self.assertNotIn("called", observed)
            # An allowed route and method reaches the backend.
            status, _, _ = invoke(gateway, "GET", "/v1/projects")
            self.assertEqual(status, 200)
            self.assertTrue(observed["called"])
        finally:
            gateway.close()


if __name__ == "__main__":
    unittest.main()
