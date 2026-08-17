# syntax=docker/dockerfile:1.7
#
# LawSynth scheduler image.
#
# The scheduler drains the transactional outbox, assigns pending jobs to
# compatible worker pools, tracks leases and heartbeats, and returns lost jobs
# to a schedulable state after lease expiry. Its full state is reconstructable
# from Postgres. It is a native Rust binary (`lawsynth-scheduler`).
#
# Build from the repository root:
#   docker build -f deploy/docker/images/scheduler.Dockerfile -t ghcr.io/lawsynth/scheduler:0.1.0 .

# ---- builder ---------------------------------------------------------------
FROM rust:1.94-bookworm AS build
WORKDIR /workspace
COPY . .
RUN rm -f .cargo/config.toml \
    && cargo build --locked --release -p lawsynth-scheduler

# ---- runtime ---------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates procps \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 65532 lawsynth \
    && useradd --uid 65532 --gid 65532 --home-dir /var/lib/lawsynth --create-home lawsynth

COPY --from=build /workspace/target/release/lawsynth-scheduler /usr/local/bin/lawsynth-scheduler

ENV LAWSYNTH_LOG_LEVEL=info

WORKDIR /var/lib/lawsynth
USER 65532:65532

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD pgrep -x lawsynth-scheduler >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/lawsynth-scheduler"]
