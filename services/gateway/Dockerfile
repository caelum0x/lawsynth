# Multi-stage build for the public WSGI admission gateway.
#
# The gateway is a Python package (lawsynth_gateway) that wraps the in-process
# lawsynth-api application in the SAME process (see docs/architecture.md), so the
# runtime image installs lawsynth-server, lawsynth-api, and lawsynth-gateway and
# serves the composed callable with gunicorn behind an upstream TLS terminator.
# Non-root runtime on python-slim, mirroring services/api/Dockerfile.

FROM python:3.12-slim AS build
WORKDIR /opt/lawsynth
# A dedicated venv keeps the runtime copy self-contained and easy to move.
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"
COPY python/lawsynth-server /opt/lawsynth/python/lawsynth-server
COPY services/api /opt/lawsynth/services/api
COPY services/gateway /opt/lawsynth/services/gateway
RUN pip install --no-cache-dir \
    /opt/lawsynth/python/lawsynth-server \
    "/opt/lawsynth/services/api[server]" \
    /opt/lawsynth/services/gateway

FROM python:3.12-slim AS runtime
# Composition entrypoint: the gateway admits requests before an in-process API
# call, so the served application is create_gateway(create_wsgi_app()). This
# shim is a deployment artifact, not part of any package.
RUN printf '%s\n' \
    'from lawsynth_api import create_wsgi_app' \
    'from lawsynth_gateway import create_gateway' \
    '' \
    'application = create_gateway(create_wsgi_app())' \
    > /opt/lawsynth/wsgi.py \
    && useradd --create-home --uid 65532 lawsynth \
    && mkdir -p /var/lib/lawsynth/objects \
    && chown -R 65532:65532 /var/lib/lawsynth /opt/lawsynth
COPY --from=build /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH" \
    PYTHONUNBUFFERED=1 \
    PYTHONPATH=/opt/lawsynth \
    LAWSYNTH_API_ENV=production \
    LAWSYNTH_DATABASE_URL=sqlite:////var/lib/lawsynth/metadata.sqlite3 \
    LAWSYNTH_OBJECT_ROOT=/var/lib/lawsynth/objects
USER 65532:65532
EXPOSE 8080
# One worker: the API's domain repositories are process-local (see the API
# operations doc). Scale with replicas, not workers.
CMD ["gunicorn", "--bind", "0.0.0.0:8080", "--workers", "1", "--access-logfile", "-", "--error-logfile", "-", "wsgi:application"]
