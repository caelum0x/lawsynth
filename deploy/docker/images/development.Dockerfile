# syntax=docker/dockerfile:1.7
#
# LawSynth development image.
#
# A single multi-language toolchain image (Rust + Python + Node/pnpm) for use
# as a dev container or CI base. It is deliberately NOT a runtime image: it is
# large, runs as a non-root developer user, and ships no service entrypoint.
#
# Build from the repository root:
#   docker build -f deploy/docker/images/development.Dockerfile -t ghcr.io/lawsynth/development:0.1.0 .

FROM rust:1.94-bookworm

# System build dependencies shared by the Rust, Python, and Node toolchains.
RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        git \
        jq \
        pkg-config \
        python3 \
        python3-dev \
        python3-venv \
        python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Rust components used across the workspace.
RUN rustup component add clippy rustfmt

# Node 22 via NodeSource + pnpm via corepack.
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install --no-install-recommends -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && corepack enable \
    && corepack prepare pnpm@10.18.2 --activate

# Non-root developer user with a cargo/registry cache directory it owns.
RUN groupadd --gid 1000 dev \
    && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash dev \
    && mkdir -p /home/dev/.cargo /workspace \
    && chown -R 1000:1000 /home/dev /workspace

ENV CARGO_HOME=/home/dev/.cargo \
    PATH=/home/dev/.cargo/bin:/usr/local/cargo/bin:$PATH \
    PIP_DISABLE_PIP_VERSION_CHECK=1

USER 1000:1000
WORKDIR /workspace
CMD ["bash"]
