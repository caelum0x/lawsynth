# syntax=docker/dockerfile:1.7
#
# LawSynth worker image.
#
# The worker leases discovery / simulation jobs, executes them under CPU,
# memory, and wall-clock quotas, streams progress events, and uploads
# content-addressed artifacts. It is a native Rust binary (`lawsynth-worker`).
#
# Build from the repository root:
#   docker build -f deploy/docker/images/worker.Dockerfile -t ghcr.io/lawsynth/worker:0.1.0 .

# ---- builder ---------------------------------------------------------------
FROM rust:1.94-bookworm AS build
WORKDIR /workspace

# The workspace crates the binary links against are path dependencies, so the
# whole tree is required for a --locked build.
COPY . .
RUN rm -f .cargo/config.toml \
    && cargo build --locked --release -p lawsynth-worker

# ---- runtime ---------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# ca-certificates for TLS to Postgres/NATS/S3; procps for the liveness probe.
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates procps \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 65532 lawsynth \
    && useradd --uid 65532 --gid 65532 --home-dir /var/lib/lawsynth --create-home lawsynth \
    && mkdir -p /var/lib/lawsynth/scratch \
    && chown -R 65532:65532 /var/lib/lawsynth

COPY --from=build /workspace/target/release/lawsynth-worker /usr/local/bin/lawsynth-worker

ENV LAWSYNTH_WORKER_SCRATCH=/var/lib/lawsynth/scratch \
    LAWSYNTH_LOG_LEVEL=info

WORKDIR /var/lib/lawsynth
USER 65532:65532

# The worker has no HTTP surface; liveness is the running process.
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD pgrep -x lawsynth-worker >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/lawsynth-worker"]
