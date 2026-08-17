# syntax=docker/dockerfile:1.7
#
# LawSynth admission gateway image.
#
# `lawsynth-gateway` is an in-process WSGI admission layer: it enforces body /
# header limits, a per-client rate window, and CORS origin policy, then hands
# the request to a backend WSGI application. This image bundles the gateway, the
# API, and the compiled `lawsynth` engine, and serves the admission-wrapped API
# through a tiny WSGI shim so the gateway can run as its own replica in front of
# the API.
#
# Because it runs the API in-process, it needs the same backing as the API:
# SQLite metadata (`LAWSYNTH_DATABASE_URL`, SQLite only), a filesystem object
# store (`LAWSYNTH_OBJECT_ROOT`), and the compiled `lawsynth._native` engine
# (built in the first stage). The gateway serves its OWN liveness at /healthz
# and readiness at /readyz; only /v1/* is proxied to the API.
#
# Build from the repository root (the Rust build needs the whole workspace):
#   docker build -f deploy/docker/images/gateway.Dockerfile -t ghcr.io/lawsynth/gateway:0.1.0 .

# ---- native: compile lawsynth._native against CPython 3.12 ------------------
FROM python:3.12-slim-bookworm AS native

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        curl build-essential pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain 1.94.0 --profile minimal

WORKDIR /src
COPY . .

RUN rm -f .cargo/config.toml \
    && cargo build -p lawsynth-python --release \
    && SUFFIX="$(python3 -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')" \
    && cp target/release/lib_native.so "python/lawsynth/src/lawsynth/_native${SUFFIX}" \
    && PYTHONPATH=python/lawsynth/src python3 -c "import lawsynth; _ = lawsynth.World; print('native ok', lawsynth.__version__)"

# ---- builder: install core + API + gateway ----------------------------------
FROM python:3.12-slim-bookworm AS build

ENV PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PYTHONDONTWRITEBYTECODE=1

WORKDIR /src

COPY python/lawsynth-server ./python/lawsynth-server
COPY services/api ./services/api
COPY services/gateway ./services/gateway

RUN python -m venv /opt/venv \
    && /opt/venv/bin/pip install --upgrade pip \
    && /opt/venv/bin/pip install \
        ./python/lawsynth-server \
        "./services/api[server]" \
        ./services/gateway \
        "gunicorn>=22,<24"

# ---- runtime ----------------------------------------------------------------
FROM python:3.12-slim-bookworm AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends -y curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 65532 lawsynth \
    && useradd --uid 65532 --gid 65532 --home-dir /var/lib/lawsynth --create-home lawsynth \
    && mkdir -p /var/lib/lawsynth/objects \
    && chown -R 65532:65532 /var/lib/lawsynth

COPY --from=build /opt/venv /opt/venv
COPY --from=native /src/python/lawsynth/src /opt/lawsynth/pysrc

# WSGI shim: wrap the configured backend WSGI callable in the gateway admission
# layer. The backend defaults to the co-located API but can be repointed via
# LAWSYNTH_GATEWAY_BACKEND (module:attribute).
RUN set -eu; \
    mkdir -p /opt/lawsynth; \
    printf '%s\n' \
      '"""Gunicorn entrypoint: gateway admission wrapping a backend WSGI app."""' \
      'from __future__ import annotations' \
      'import importlib' \
      'import os' \
      'from lawsynth_gateway.app import create_gateway' \
      '_target = os.environ.get("LAWSYNTH_GATEWAY_BACKEND", "lawsynth_api.main:application")' \
      '_module_name, _, _attribute = _target.partition(":")' \
      'if not _module_name or not _attribute:' \
      '    raise RuntimeError("LAWSYNTH_GATEWAY_BACKEND must use module:attribute syntax")' \
      '_backend = getattr(importlib.import_module(_module_name), _attribute)' \
      'if not callable(_backend):' \
      '    raise RuntimeError("LAWSYNTH_GATEWAY_BACKEND must resolve to a WSGI callable")' \
      'application = create_gateway(_backend)' \
      > /opt/lawsynth/wsgi.py; \
    chown -R 65532:65532 /opt/lawsynth

ENV PATH="/opt/venv/bin:${PATH}" \
    PYTHONPATH="/opt/lawsynth:/opt/lawsynth/pysrc" \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    GUNICORN_WORKERS=1 \
    GUNICORN_TIMEOUT=180 \
    LAWSYNTH_API_ENV=production \
    LAWSYNTH_DATABASE_URL=sqlite:////var/lib/lawsynth/metadata.sqlite3 \
    LAWSYNTH_OBJECT_ROOT=/var/lib/lawsynth/objects \
    LAWSYNTH_GATEWAY_BACKEND=lawsynth_api.main:application

WORKDIR /opt/lawsynth
USER 65532:65532
EXPOSE 8081

# The gateway's own liveness endpoint (not proxied to the backend).
HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8081/healthz || exit 1

CMD gunicorn \
      --bind 0.0.0.0:8081 \
      --workers "${GUNICORN_WORKERS}" \
      --timeout "${GUNICORN_TIMEOUT}" \
      --graceful-timeout 30 \
      --access-logfile - \
      --error-logfile - \
      wsgi:application
