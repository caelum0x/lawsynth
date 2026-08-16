"""Loopback-only development entry point for an in-process WSGI gateway."""

from __future__ import annotations

import argparse
import importlib
import ipaddress
from collections.abc import Sequence
from wsgiref.simple_server import make_server

from .app import create_gateway


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="LawSynth in-process WSGI gateway")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", default=8081, type=int)
    parser.add_argument("--backend-module", default="lawsynth_api.main:application", help="local WSGI import target, e.g. lawsynth_api.main:application")
    options = parser.parse_args(argv)
    try:
        loopback = ipaddress.ip_address(options.host).is_loopback
    except ValueError:
        loopback = options.host == "localhost"
    if not loopback or not 1 <= options.port <= 65535:
        parser.error("the development server is loopback-only and port must be in 1..65535")
    module_name, separator, attribute = options.backend_module.partition(":")
    if not separator or not module_name or not attribute:
        parser.error("--backend-module must use module:attribute syntax")
    try:
        application = getattr(importlib.import_module(module_name), attribute)
    except (ImportError, AttributeError) as error:
        parser.error(f"cannot load local WSGI backend: {error}")
    if not callable(application):
        parser.error("--backend-module must resolve to a WSGI callable")
    gateway = create_gateway(application)
    with make_server(options.host, options.port, gateway) as server:
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            return 0
        finally:
            gateway.close()


if __name__ == "__main__":
    raise SystemExit(main())
