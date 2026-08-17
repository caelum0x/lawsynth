#!/usr/bin/env bash
# Build the LawSynth native extension without maturin.
#
# `maturin develop` is the supported path (see README), but it is not always
# available (air-gapped CI, minimal toolchains). The `lawsynth-python` crate is a
# plain cdylib named `_native`, so a direct `cargo build` produces exactly the
# artifact maturin would; this script builds it and copies it next to the pure
# -Python package under the interpreter's expected extension suffix. The result
# is import-compatible with `from lawsynth import _native`.
#
# Usage:
#   python/lawsynth/scripts/build-native.sh [--debug]
#
# Env:
#   PYTHON  interpreter to target (default: python3)
set -euo pipefail

PYTHON="${PYTHON:-python3}"
PROFILE="release"
CARGO_FLAG="--release"
if [[ "${1:-}" == "--debug" ]]; then
  PROFILE="debug"
  CARGO_FLAG=""
fi

# Resolve repo root from this script's location (…/python/lawsynth/scripts).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
PACKAGE_DIR="${REPO_ROOT}/python/lawsynth/src/lawsynth"

# The interpreter dictates the ABI-specific extension suffix (e.g.
# .cpython-314-darwin.so); mismatching it makes the module unimportable.
SUFFIX="$("${PYTHON}" -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"

# On macOS a PyO3 extension-module leaves CPython symbols to be resolved from the
# running interpreter; the crate's build config already supplies the required
# `-undefined dynamic_lookup`, so a plain cargo build links cleanly.
echo "building lawsynth-python (${PROFILE}) …"
( cd "${REPO_ROOT}" && cargo build -p lawsynth-python ${CARGO_FLAG} )

# The cdylib output is platform-named: lib_native.{dylib,so} (or _native.dll).
BUILD_DIR="${REPO_ROOT}/target/${PROFILE}"
ARTIFACT=""
for candidate in "lib_native.dylib" "lib_native.so" "_native.dll"; do
  if [[ -f "${BUILD_DIR}/${candidate}" ]]; then
    ARTIFACT="${BUILD_DIR}/${candidate}"
    break
  fi
done
if [[ -z "${ARTIFACT}" ]]; then
  echo "error: could not find the built _native cdylib under ${BUILD_DIR}" >&2
  exit 1
fi

DEST="${PACKAGE_DIR}/_native${SUFFIX}"
cp "${ARTIFACT}" "${DEST}"
echo "installed ${DEST}"

# Prove the module imports and exposes the public surface.
PYTHONPATH="${REPO_ROOT}/python/lawsynth/src" "${PYTHON}" -c \
  'from lawsynth import _native; print("native ok:", sorted(n for n in dir(_native) if not n.startswith("__")))'
