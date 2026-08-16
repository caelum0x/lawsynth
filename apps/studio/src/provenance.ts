import type { Provenance, WorldDefinition } from "@lawsynth/world-schema";
import { isSha256, provenanceView, type ProvenanceView } from "@lawsynth/world-viewer";

export interface ProvenanceAudit {
  readonly view: ProvenanceView;
  readonly score: number;
  readonly missing: readonly string[];
  readonly invalid: readonly string[];
}

export function auditProvenance(provenance: Provenance | undefined): ProvenanceAudit {
  const missing: string[] = [];
  const invalid: string[] = [];
  if (provenance === undefined) missing.push("provenance");
  else {
    if (!provenance.createdAt) missing.push("createdAt"); else if (!Number.isFinite(Date.parse(provenance.createdAt))) invalid.push("createdAt");
    if (provenance.seed === undefined) missing.push("seed"); else if (!Number.isSafeInteger(provenance.seed)) invalid.push("seed");
    if (provenance.worldHash === undefined) missing.push("worldHash"); else if (!isSha256(provenance.worldHash)) invalid.push("worldHash");
    if (provenance.dataHash !== undefined && !isSha256(provenance.dataHash)) invalid.push("dataHash");
    if (provenance.planHash !== undefined && !isSha256(provenance.planHash)) invalid.push("planHash");
    if ((provenance.algorithms?.length ?? 0) === 0) missing.push("algorithms");
    for (const artifact of provenance.artifacts ?? []) if (!isSha256(artifact.sha256)) invalid.push(`artifact:${artifact.path}`);
  }
  const total = 5;
  const score = Math.max(0, (total - missing.length - invalid.length) / total);
  return Object.freeze({ view: provenanceView(provenance), score, missing: Object.freeze(missing), invalid: Object.freeze(invalid) });
}

export function auditWorldProvenance(world: WorldDefinition): ProvenanceAudit { return auditProvenance(world.provenance); }
