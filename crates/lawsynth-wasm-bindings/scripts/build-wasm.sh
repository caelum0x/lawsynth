#!/usr/bin/env bash
#
# Build the deployable LawSynth WASM module and stage it next to the JS glue.
#
# This crate uses NO wasm-bindgen and NO external crates: the output is a plain
# `wasm32-unknown-unknown` cdylib with a hand-rolled C-ABI (see the crate README
# and the `ffi` module). The steps below are the exact, documented build.
#
# NETWORK NOTE: `rustup target add wasm32-unknown-unknown` downloads the target's
# std once. In a fully offline environment this single step cannot run, so the
# `.wasm` is produced on a networked machine / CI and the artifact committed or
# published. Everything else (compile, tests) runs offline.
#
# Usage:
#   scripts/build-wasm.sh            # release build + stage into web/
#   PROFILE=debug scripts/build-wasm.sh
set -euo pipefail

CRATE="lawsynth-wasm-bindings"
PROFILE="${PROFILE:-release}"
TARGET="wasm32-unknown-unknown"

# Resolve paths relative to this script so it works from any CWD.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_ROOT="$(cd "${CRATE_DIR}/../.." && pwd)"

echo "==> Ensuring the ${TARGET} target is installed (needs network, one-time)"
if ! rustup target list --installed 2>/dev/null | grep -qx "${TARGET}"; then
  rustup target add "${TARGET}"
fi

echo "==> Building ${CRATE} (${PROFILE}) for ${TARGET}"
if [ "${PROFILE}" = "release" ]; then
  cargo build -p "${CRATE}" --release --target "${TARGET}"
  BUILD_SUBDIR="release"
else
  cargo build -p "${CRATE}" --target "${TARGET}"
  BUILD_SUBDIR="debug"
fi

# cdylib output lands here (underscores replace dashes in the file name).
WASM_SRC="${WORKSPACE_ROOT}/target/${TARGET}/${BUILD_SUBDIR}/lawsynth_wasm_bindings.wasm"
WASM_DEST="${CRATE_DIR}/web/lawsynth_wasm_bindings.wasm"

if [ ! -f "${WASM_SRC}" ]; then
  echo "error: expected artifact not found at ${WASM_SRC}" >&2
  exit 1
fi

echo "==> Staging artifact next to the JS glue"
cp "${WASM_SRC}" "${WASM_DEST}"
echo "    ${WASM_DEST}"

# Optional: copy into the playground's public assets so the app can fetch it.
# The playground consumes the glue via `createBindings(fetch('/lawsynth_wasm_bindings.wasm'))`.
PLAYGROUND_PUBLIC="${WORKSPACE_ROOT}/apps/playground/public"
if [ -d "${PLAYGROUND_PUBLIC}" ]; then
  cp "${WASM_SRC}" "${PLAYGROUND_PUBLIC}/lawsynth_wasm_bindings.wasm"
  cp "${CRATE_DIR}/web/lawsynth_wasm.mjs" "${PLAYGROUND_PUBLIC}/lawsynth_wasm.mjs" 2>/dev/null || true
  echo "    also copied into ${PLAYGROUND_PUBLIC}/"
fi

echo "==> Done. Load it with:"
echo "    import { createBindings } from './lawsynth_wasm.mjs';"
echo "    const bindings = await createBindings(fetch('./lawsynth_wasm_bindings.wasm'));"
