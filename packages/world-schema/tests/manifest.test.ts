import { entryByPath, requiredEntries } from "../src/manifest.js";
import { validateManifest } from "../src/validators.js";
import { equal, ok } from "./test-support.js";
export function runManifestTests(): void { ok(validateManifest({ format: "lawsynth-world", format_version: "0.1.0", world_encoding: "lawsynth-world-binary-v1" }).ok); equal(validateManifest({ format: "lawsynth-world", format_version: "1.0.0", world_encoding: "lawsynth-world-binary-v1" }).ok, false); const catalog = { worldId: "x", createdAt: "2026-08-16T00:00:00Z", root: "world/world.bin", entries: [{ path: "world/world.bin", mediaType: "application/octet-stream", sha256: "0".repeat(64), bytes: 1 }] }; equal(entryByPath(catalog, "world/world.bin")?.bytes, 1); equal(requiredEntries(catalog).length, 1); }
