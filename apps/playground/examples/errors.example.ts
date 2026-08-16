import { PlaygroundError, normalizePlaygroundError, userErrorMessage } from "../src/errors.js";

/** Convert an operation failure into user-facing text without discarding its typed cause. */
export function presentPlaygroundFailure(error: unknown): { readonly code: PlaygroundError["code"]; readonly message: string } {
  const normalized = normalizePlaygroundError(error);
  return Object.freeze({ code: normalized.code, message: userErrorMessage(normalized) });
}

/** Represent deployment absence honestly; callers must not replace this with a successful simulation. */
export function wasmArtifactUnavailable(): PlaygroundError {
  return new PlaygroundError("wasm-unavailable", "The generated lawsynth-wasm bindings are not available in this deployment.");
}
