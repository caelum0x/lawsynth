FROM rust:1.85 AS build
WORKDIR /src
COPY . .
RUN cargo build --locked --release -p lawsynth-worker

FROM debian:bookworm-slim
COPY --from=build /src/target/release/lawsynth-worker /usr/local/bin/lawsynth-worker
ENTRYPOINT ["/usr/local/bin/lawsynth-worker"]
