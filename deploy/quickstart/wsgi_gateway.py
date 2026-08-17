"""Gunicorn entrypoint: the LawSynth admission gateway wrapping the API.

The gateway (:mod:`lawsynth_gateway`) is an in-process WSGI admission layer —
it enforces body/header limits, a per-client rate window, and CORS origin
policy, then calls a backend WSGI application in-process. Here the backend is
the co-located API (``lawsynth_api.main:application``). Point it elsewhere with
``LAWSYNTH_GATEWAY_BACKEND=module:attribute`` if you ever split the two.

Rate-limit, body-limit, and allowed-origin settings are read from the
``LAWSYNTH_GATEWAY_*`` environment variables by
``GatewaySettings.from_environment`` inside ``create_gateway``.
"""

from __future__ import annotations

import importlib
import os

from lawsynth_gateway.app import create_gateway

_target = os.environ.get("LAWSYNTH_GATEWAY_BACKEND", "lawsynth_api.main:application")
_module_name, _, _attribute = _target.partition(":")
if not _module_name or not _attribute:
    raise RuntimeError("LAWSYNTH_GATEWAY_BACKEND must use module:attribute syntax")

_backend = getattr(importlib.import_module(_module_name), _attribute)
if not callable(_backend):
    raise RuntimeError("LAWSYNTH_GATEWAY_BACKEND must resolve to a WSGI callable")

application = create_gateway(_backend)
