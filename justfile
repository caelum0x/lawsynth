set shell := ["sh", "-eu", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

check:
    cargo check --workspace --all-targets

test:
    cargo test --workspace

lint:
    cargo clippy --workspace --all-targets -- -D warnings

python-test:
    PYTHONPATH=python/lawsynth/src python3 -m pytest -q python/lawsynth/tests

# Build, typecheck, and test the whole pnpm TypeScript workspace (packages + apps).
ts:
    pnpm install --frozen-lockfile
    pnpm run build
    pnpm run typecheck
    pnpm run test

verify: fmt check test lint python-test ts
