import assert from "node:assert/strict";
import test from "node:test";
import { buildProvenanceModel } from "../src/index.js";
test("marks complete deterministic provenance reproducible", () => { const model = buildProvenanceModel({ seed: 1, dataHash: "d", planHash: "p", worldHash: "w", algorithms: [{ name: "fit", version: "1", deterministic: true }] }); assert.equal(model.reproducible, true); });
