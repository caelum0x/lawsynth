# syntax=docker/dockerfile:1.7
#
# LawSynth API image.
#
# The API is the Python WSGI facade (`lawsynth_api.main:application`) that
# fronts the `lawsynth-server` domain core. In server mode it talks to
# Postgres for metadata, an S3-compatible object store for artifacts, and a
# NATS bus for job/event publication. This image serves it under gunicorn.
#
# Build from the repository root:
#   docker build -f deploy/docker/images/api.Dockerfile -t ghcr.io/lawsynth/api:0.1.0 .

# ---- builder: resolve and install into an isolated venv ---------------------
FROM python:3.12-slim-bookworm AS build

ENV PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PYTHONDONTWRITEBYTECODE=1

WORKDIR /src

# Only the paths the API package depends on, to keep the layer cache warm.
COPY python/lawsynth-server ./python/lawsynth-server
COPY services/api ./services/api

# Build into a self-contained virtual environment we can copy wholesale.
RUN python -m venv /opt/venv \
    && /opt/venv/bin/pip install --upgrade pip \
    && /opt/venv/bin/pip install \
        ./python/lawsynth-server \
        "./services/api[server]"

# ---- runtime: minimal, non-root --------------------------------------------
FROM python:3.12-slim-bookworm AS runtime

# curl is used only by the container HEALTHCHECK.
RUN apt-get update \
    && apt-get install --no-install-recommends -y curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 65532 lawsynth \
    && useradd --uid 65532 --gid 65532 --home-dir /var/lib/lawsynth --create-home lawsynth \
    && mkdir -p /var/lib/lawsynth/objects \
    && chown -R 65532:65532 /var/lib/lawsynth

COPY --from=build /opt/venv /opt/venv

ENV PATH="/opt/venv/bin:${PATH}" \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    LAWSYNTH_API_ENV=production \
    GUNICORN_WORKERS=2 \
    GUNICORN_TIMEOUT=120 \
    LAWSYNTH_OBJECT_ROOT=/var/lib/lawsynth/objects

WORKDIR /var/lib/lawsynth
USER 65532:65532
EXPOSE 8080

# The domain core exposes GET /v1/health through the WSGI app.
HEALTHCHECK --interval=15s --timeout=5s --start-period=20s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8080/v1/health || exit 1

# Shell form so ${GUNICORN_*} expand from the environment / compose overrides.
CMD gunicorn \
      --bind 0.0.0.0:8080 \
      --workers "${GUNICORN_WORKERS}" \
      --timeout "${GUNICORN_TIMEOUT}" \
      --graceful-timeout 30 \
      --access-logfile - \
      --error-logfile - \
      lawsynth_api.main:application
