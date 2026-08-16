# Build the implemented local CLI. The image does not expose a server because
# LawSynth currently ships no hosted API runtime.
FROM rust:1.94-bookworm AS build
WORKDIR /workspace
COPY . .
RUN rm -f .cargo/config.toml && cargo build --locked --release -p lawsynth-cli

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 lawsynth
COPY --from=build /workspace/target/release/lawsynth /usr/local/bin/lawsynth
USER lawsynth
ENTRYPOINT ["lawsynth"]
