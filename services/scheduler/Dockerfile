# Multi-stage build for the local scheduler control-plane service.
# Mirrors the root Dockerfile style: pinned toolchain, locked build, non-root
# runtime on debian-slim. The image ships the `lawsynth-scheduler` binary whose
# only serving command is `serve <addr> [root]`; the HTTP surface is the
# scheduler's serializable control plane (see docs/security.md).
FROM rust:1.94-bookworm AS build
WORKDIR /workspace
COPY . .
RUN rm -f .cargo/config.toml \
    && cargo build --locked --release -p lawsynth-scheduler

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 lawsynth \
    && mkdir -p /var/lib/lawsynth/scheduler \
    && chown lawsynth:lawsynth /var/lib/lawsynth/scheduler
COPY --from=build /workspace/target/release/lawsynth-scheduler /usr/local/bin/lawsynth-scheduler
USER lawsynth
# Default root is the persistent volume; override the address as needed. Bind
# behind an authenticating, TLS-terminating front door — the control plane has
# no principals of its own.
VOLUME ["/var/lib/lawsynth/scheduler"]
EXPOSE 8082
ENTRYPOINT ["lawsynth-scheduler"]
CMD ["serve", "0.0.0.0:8082", "/var/lib/lawsynth/scheduler"]
