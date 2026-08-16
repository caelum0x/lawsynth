import { GENERATED_SCHEMA_REVISION, type GeneratedWorldDefinition } from "../src/generated.js";

/** Stable generated alias used by clients that do not depend on source layout. */
export const generatedWorldExample: Pick<GeneratedWorldDefinition, "id" | "formatVersion"> = {
  id: "generated-decay",
  formatVersion: "0.1.0",
};

export const generatedRevision = GENERATED_SCHEMA_REVISION;
