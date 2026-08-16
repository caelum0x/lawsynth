/**
 * Schema generation boundary. The aliases remain stable even when the JSON
 * Schema generator changes its internal output layout.
 */
export { CURRENT_BUNDLE_VERSION, type WorldManifest as GeneratedWorldManifest } from "./manifest.js";
export { CURRENT_WORLD_VERSION, type WorldDefinition as GeneratedWorldDefinition } from "./world.js";

export const GENERATED_SCHEMA_REVISION = "2026-08-16";
