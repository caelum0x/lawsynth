import { auditWorldProvenance } from "../src/provenance.js";
import type { WorldDefinition } from "@lawsynth/world-schema";

/** Gate publication on traceable inputs, algorithms, and checksums. */
export function provenancePublicationCheck(world: WorldDefinition): { readonly accepted: boolean; readonly missing: readonly string[]; readonly invalid: readonly string[] } {
  const audit = auditWorldProvenance(world);
  return Object.freeze({ accepted: audit.score === 1, missing: audit.missing, invalid: audit.invalid });
}
