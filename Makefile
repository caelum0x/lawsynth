.DEFAULT_GOAL := help
.PHONY: help fmt check test lint python-test ts verify

help:
	@printf '%s\n' 'Targets: fmt check test lint python-test ts verify'

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

ts:
	pnpm install --frozen-lockfile
	pnpm run build
	pnpm run typecheck
	pnpm run test

verify: fmt check test lint python-test ts
