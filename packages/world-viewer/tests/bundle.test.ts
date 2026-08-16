import assert from "node:assert/strict";
import test from "node:test";
import { parseViewerBundle, viewBundle } from "../src/index.js";
const bundle = { format: "lawsynth-world" as const, format_version: "0.1.0", world_encoding: "lawsynth-world-binary-v1", world: { formatVersion: "0.1.0", id: "bundle", time: { kind: "continuous" as const }, variables: [{ id: "x", role: "state" }], laws: [{ id: "dx", kind: "continuous", target: "x", expression: { kind: "constant", value: 0 } }] }, entries: [{ path: "world/world.json", mediaType: "application/json", sha256: "a".repeat(64), bytes: 1 }] };
test("parses metadata-bearing JSON bundle envelopes", () => { assert.equal(parseViewerBundle(JSON.stringify(bundle)).world.id, "bundle"); assert.equal(viewBundle(bundle).title, "bundle"); });
test("rejects invalid bundle checksums", () => { assert.throws(() => parseViewerBundle({ ...bundle, entries: [{ ...bundle.entries[0], sha256: "bad" }] })); });
