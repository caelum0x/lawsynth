# syntax=docker/dockerfile:1.7
#
# LawSynth artifact service image.
#
# The artifact service is the content-addressed object lifecycle boundary:
# upload with checksum verification, download, retention, and garbage
# collection over a live-reference set. It is a native Rust binary
# (`lawsynth-artifact`) that exposes an HTTP surface via `serve <root> <addr>`.
#
# Build from the repository root:
#   docker build -f deploy/docker/images/artifact.Dockerfile -t ghcr.io/lawsynth/artifact:0.1.0 .

# ---- builder ---------------------------------------------------------------
FROM rust:1.94-bookworm AS build
WORKDIR /workspace
COPY . .
RUN rm -f .cargo/config.toml \
    && cargo build --locked --release -p lawsynth-artifact-service

# ---- runtime ---------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 65532 lawsynth \
    && useradd --uid 65532 --gid 65532 --home-dir /var/lib/lawsynth --create-home lawsynth \
    && mkdir -p /var/lib/lawsynth/artifacts \
    && chown -R 65532:65532 /var/lib/lawsynth

COPY --from=build /workspace/target/release/lawsynth-artifact /usr/local/bin/lawsynth-artifact

ENV LAWSYNTH_ARTIFACT_ROOT=/var/lib/lawsynth/artifacts \
    LAWSYNTH_ARTIFACT_ADDR=0.0.0.0:8082 \
    LAWSYNTH_LOG_LEVEL=info

WORKDIR /var/lib/lawsynth
USER 65532:65532
EXPOSE 8082

# `health <root>` opens the store and reports capacity without a network call.
HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=5 \
    CMD ["/usr/local/bin/lawsynth-artifact", "health", "/var/lib/lawsynth/artifacts"]

# Shell form so the configured root/addr expand at start.
ENTRYPOINT []
CMD ["sh", "-c", "exec /usr/local/bin/lawsynth-artifact serve \"$LAWSYNTH_ARTIFACT_ROOT\" \"$LAWSYNTH_ARTIFACT_ADDR\""]
