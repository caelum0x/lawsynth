import type { WorldDefinition } from "@lawsynth/world-schema";
import { createViewerBundle, encodeViewerBundle, exportWorldJson, type ExportDocument } from "@lawsynth/world-viewer";

export type StudioExportKind = "world-json" | "viewer-json" | "audit-json";

export interface AuditExport {
  readonly exportedAt: string;
  readonly worldId: string;
  readonly worldVersion: string;
  readonly provenancePresent: boolean;
  readonly uncertaintyEntries: number;
  readonly regimeCount: number;
  readonly lawCount: number;
}

export function createStudioExport(world: WorldDefinition, kind: StudioExportKind, clock: () => Date = () => new Date()): ExportDocument {
  if (kind === "world-json") return exportWorldJson(world);
  if (kind === "viewer-json") {
    const bundle = createViewerBundle(world);
    return { filename: `${world.id}.viewer.json`, mediaType: "application/vnd.lawsynth.viewer+json", content: encodeViewerBundle(bundle, true) };
  }
  const audit: AuditExport = {
    exportedAt: clock().toISOString(), worldId: world.id, worldVersion: world.formatVersion,
    provenancePresent: world.provenance !== undefined, uncertaintyEntries: world.uncertainty?.entries.length ?? 0,
    regimeCount: world.regimes?.regimes.length ?? 0, lawCount: world.laws.length,
  };
  return { filename: `${world.id}.audit.json`, mediaType: "application/json", content: JSON.stringify(audit, null, 2) };
}
