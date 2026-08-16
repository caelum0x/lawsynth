import type { Provenance } from "../src/provenance.js";

export const provenanceExample: Provenance = {
  runId: "run-42",
  createdAt: "2026-08-16T12:00:00.000Z",
  seed: 42,
  algorithms: [{ name: "sindy", version: "0.1.0", deterministic: true }],
  environment: { lawsynthVersion: "0.1.0", runtime: "rust" },
};
