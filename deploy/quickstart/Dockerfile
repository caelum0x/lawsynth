# syntax=docker/dockerfile:1.7
#
# LawSynth quickstart image — the smallest thing that actually serves a
# discovery over HTTP.
#
# Unlike the stock `deploy/docker/images/api.Dockerfile`, this image bundles the
# *compiled* `lawsynth` engine. That matters: the API's product surface imports
# `lawsynth.report` at module load, and `POST /v1/runs` runs native discovery
# in-process, so without the engine the API cannot import and every run returns
# `503 native_unavailable`. This image therefore compiles the PyO3 extension
# (`lawsynth._native`) from the Rust workspace, then installs the pure-Python
# domain core, API, and gateway on top of it.
#
# A single image runs BOTH services in the quickstart compose profile:
#   * the API      — `gunicorn lawsynth_api.main:application`      (:8080)
#   * the gateway  — `gunicorn wsgi_gateway:application`           (:8081)
#
# Build (from the repository root — the Rust build needs the whole workspace):
#   docker build -f deploy/quickstart/Dockerfile -t lawsynth-quickstart:local .
#
# The first build compiles the numeric Rust crates and takes several minutes.

# ---- native: compile the lawsynth._native extension against CPython 3.12 ----
#
# We build in a python:3.12 base (not a rust base) so pyo3-build-config resolves
# the *same* CPython ABI the runtime stage uses; a mismatch would make the
# extension unimportable. This mirrors python/lawsynth/scripts/build-native.sh:
# a plain `cargo build -p lawsynth-python` yields exactly the cdylib maturin
# would, which we drop next to the pure-Python `lawsynth` package.
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

# `.cargo/config.toml` may pin registry mirrors that are unavailable offline;
# the Rust service images drop it too. `lawsynth-python` is a cdylib named
# `_native`, so the release artifact is target/release/lib_native.so.
RUN rm -f .cargo/config.toml \
    && cargo build -p lawsynth-python --release \
    && SUFFIX="$(python3 -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')" \
    && cp target/release/lib_native.so "python/lawsynth/src/lawsynth/_native${SUFFIX}" \
    && PYTHONPATH=python/lawsynth/src python3 -c "import lawsynth; _ = lawsynth.World; print('native ok', lawsynth.__version__)"

# ---- runtime: minimal, non-root --------------------------------------------
FROM python:3.12-slim-bookworm AS runtime

# curl backs the container HEALTHCHECK.
RUN apt-get update \
    && apt-get install --no-install-recommends -y curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 65532 lawsynth \
    && useradd --uid 65532 --gid 65532 --home-dir /var/lib/lawsynth --create-home lawsynth \
    && mkdir -p /data/objects \
    && chown -R 65532:65532 /data

# Pure-Python packages: domain core, WSGI API (with gunicorn), admission gateway.
COPY python/lawsynth-server /opt/lawsynth/python/lawsynth-server
COPY services/api /opt/lawsynth/services/api
COPY services/gateway /opt/lawsynth/services/gateway
RUN pip install --no-cache-dir \
        /opt/lawsynth/python/lawsynth-server \
        "/opt/lawsynth/services/api[server]" \
        /opt/lawsynth/services/gateway

# The compiled `lawsynth` package (pure-Python tree + the _native.so built
# above) is provided on PYTHONPATH rather than pip-installed, so no maturin is
# needed at image-build time.
COPY --from=native /src/python/lawsynth/src /opt/lawsynth/pysrc

# Gateway WSGI shim: wrap the co-located API in the admission layer.
COPY deploy/quickstart/wsgi_gateway.py /opt/lawsynth/wsgi_gateway.py

ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONPATH=/opt/lawsynth:/opt/lawsynth/pysrc \
    LAWSYNTH_API_ENV=production \
    LAWSYNTH_DATABASE_URL=sqlite:////data/metadata.sqlite3 \
    LAWSYNTH_OBJECT_ROOT=/data/objects

WORKDIR /opt/lawsynth
USER 65532:65532
EXPOSE 8080 8081

# Default to the API; the gateway service overrides the command in compose.
# GET /v1/health is unauthenticated and reports database + storage readiness.
HEALTHCHECK --interval=15s --timeout=5s --start-period=20s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8080/v1/health || exit 1

# One gunicorn worker: the alpha domain repositories and the in-process
# discovery/event bus are process-local (see services/api/README.md), so a
# single worker keeps runs, worlds, and their SSE stream coherent.
CMD ["gunicorn", "--bind", "0.0.0.0:8080", "--workers", "1", "--timeout", "180", \
     "--graceful-timeout", "30", "--access-logfile", "-", "--error-logfile", "-", \
     "lawsynth_api.main:application"]
