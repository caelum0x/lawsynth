"""WSGI import target and a loopback-only development server."""

from __future__ import annotations

import argparse
import ipaddress
import os
import sys
from collections.abc import Sequence
from wsgiref.simple_server import make_server

from lawsynth_server.errors import ValidationError

from .app import WsgiApplication, create_wsgi_app
from .settings import ApiSettings

# Production WSGI servers should import ``lawsynth_api.main:application``.
application = create_wsgi_app()


def _loopback(host: str) -> bool:
    try:
        return ipaddress.ip_address(host).is_loopback
    except ValueError:
        return host == "localhost"


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run LawSynth's loopback WSGI development server")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--environment", choices=("development", "test", "staging", "production"))
    arguments = parser.parse_args(argv)
    if not 1 <= arguments.port <= 65535:
        parser.error("--port must be in 1..65535")
    if not _loopback(arguments.host):
        parser.error("the stdlib server is loopback-only; deploy with a production WSGI server and TLS proxy")
    values = dict(os.environ)
    if arguments.environment:
        values["LAWSYNTH_API_ENV"] = arguments.environment
    try:
        app: WsgiApplication = create_wsgi_app(ApiSettings.from_environment(values))
    except ValidationError as error:
        parser.error(error.message)
    with make_server(arguments.host, arguments.port, app) as server:
        print(f"LawSynth API development server listening on http://{arguments.host}:{arguments.port}", file=sys.stderr)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            return 0
        finally:
            app.close()


if __name__ == "__main__":
    raise SystemExit(main())
