# Multi-stage build for the local-first artifact service.
# Mirrors the root Dockerfile style: pinned toolchain, locked build, non-root
# runtime on debian-slim. The image ships the `lawsynth-artifact` binary whose
# supported commands are `serve`, `health`, and `gc`.
FROM rust:1.94-bookworm AS build
WORKDIR /workspace
COPY . .
RUN rm -f .cargo/config.toml \
    && cargo build --locked --release -p lawsynth-artifact-service

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 lawsynth \
    && mkdir -p /var/lib/lawsynth/artifacts \
    && chown lawsynth:lawsynth /var/lib/lawsynth/artifacts
COPY --from=build /workspace/target/release/lawsynth-artifact /usr/local/bin/lawsynth-artifact
USER lawsynth
# Default root is the persistent volume; override the address as needed.
VOLUME ["/var/lib/lawsynth/artifacts"]
EXPOSE 8080
ENTRYPOINT ["lawsynth-artifact"]
CMD ["serve", "/var/lib/lawsynth/artifacts", "0.0.0.0:8080"]
