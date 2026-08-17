# syntax=docker/dockerfile:1.7
#
# LawSynth API image.
#
# The API is the Python WSGI facade (`lawsynth_api.main:application`) that
# fronts the `lawsynth-server` domain core. Its real backing services are
# **SQLite** metadata (`LAWSYNTH_DATABASE_URL`, SQLite URLs only) and a
# **content-addressed filesystem** object store (`LAWSYNTH_OBJECT_ROOT`);
# discovery runs on the compiled `lawsynth` engine **in-process**. It does not
# use Postgres, S3, or NATS — those back the separate Rust scheduler/worker/
# artifact plane. This image serves the API under gunicorn.
#
# The engine matters: the API imports `lawsynth.report` at module load and runs
# native discovery for `POST /v1/runs`, so the compiled extension
# (`lawsynth._native`) MUST be present or the process cannot import and every
# run returns `503 native_unavailable`. The first stage compiles it from the
# Rust workspace (a several-minute first build).
#
# Build from the repository root (the Rust build needs the whole workspace):
#   docker build -f deploy/docker/images/api.Dockerfile -t ghcr.io/lawsynth/api:0.1.0 .

# ---- native: compile lawsynth._native against CPython 3.12 ------------------
# Built in a python:3.12 base so pyo3-build-config resolves the same ABI the
# runtime uses. Mirrors python/lawsynth/scripts/build-native.sh.
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

# ---- builder: resolve and install the pure-Python packages ------------------
FROM python:3.12-slim-bookworm AS build

ENV PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PYTHONDONTWRITEBYTECODE=1

WORKDIR /src

COPY python/lawsynth-server ./python/lawsynth-server
COPY services/api ./services/api

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
# The compiled `lawsynth` package (pure-Python tree + _native.so) is provided on
# PYTHONPATH rather than pip-installed, so no maturin is needed here.
COPY --from=native /src/python/lawsynth/src /opt/lawsynth/pysrc

ENV PATH="/opt/venv/bin:${PATH}" \
    PYTHONPATH="/opt/lawsynth/pysrc" \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    LAWSYNTH_API_ENV=production \
    GUNICORN_WORKERS=1 \
    GUNICORN_TIMEOUT=180 \
    LAWSYNTH_DATABASE_URL=sqlite:////var/lib/lawsynth/metadata.sqlite3 \
    LAWSYNTH_OBJECT_ROOT=/var/lib/lawsynth/objects

WORKDIR /var/lib/lawsynth
USER 65532:65532
EXPOSE 8080

# The domain core exposes GET /v1/health (unauthenticated) through the WSGI app.
HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8080/v1/health || exit 1

# One gunicorn worker: the alpha domain repositories and the in-process
# discovery/event bus are process-local (services/api/README.md). Shell form so
# ${GUNICORN_*} expand from the environment / compose overrides.
CMD gunicorn \
      --bind 0.0.0.0:8080 \
      --workers "${GUNICORN_WORKERS}" \
      --timeout "${GUNICORN_TIMEOUT}" \
      --graceful-timeout 30 \
      --access-logfile - \
      --error-logfile - \
      lawsynth_api.main:application
