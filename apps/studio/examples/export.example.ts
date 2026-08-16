import { createStudioExport } from "../src/export.js";
import type { WorldDefinition } from "@lawsynth/world-schema";

/** Create an auditable export without relying on browser download APIs. */
export function createAuditDocument(world: WorldDefinition) {
  return createStudioExport(world, "audit-json", () => new Date("2026-01-01T00:00:00.000Z"));
}
