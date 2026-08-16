import type { ArtifactReference, Provenance } from "@lawsynth/world-schema";

export type ProvenanceTone = "neutral" | "positive" | "warning";

export interface ProvenanceRow {
  readonly label: string;
  readonly value: string;
  readonly tone: ProvenanceTone;
}

export interface ProvenanceView {
  readonly rows: readonly ProvenanceRow[];
  readonly assumptions: readonly string[];
  readonly artifacts: readonly ArtifactReference[];
  readonly reproducible: boolean;
}

function row(label: string, value: string, tone: ProvenanceTone = "neutral"): ProvenanceRow {
  return Object.freeze({ label, value, tone });
}

export function provenanceView(provenance: Provenance | undefined): ProvenanceView {
  if (provenance === undefined) return Object.freeze({
    rows: Object.freeze([row("Record", "No provenance supplied", "warning")]),
    assumptions: Object.freeze([]), artifacts: Object.freeze([]), reproducible: false,
  });
  const rows: ProvenanceRow[] = [row("Created", provenance.createdAt)];
  if (provenance.runId !== undefined) rows.push(row("Run", provenance.runId));
  if (provenance.seed !== undefined) rows.push(row("Random seed", String(provenance.seed), "positive"));
  if (provenance.dataHash !== undefined) rows.push(row("Data SHA-256", provenance.dataHash, "positive"));
  if (provenance.planHash !== undefined) rows.push(row("Plan SHA-256", provenance.planHash, "positive"));
  if (provenance.worldHash !== undefined) rows.push(row("World SHA-256", provenance.worldHash, "positive"));
  if (provenance.environment !== undefined) {
    rows.push(row("LawSynth", provenance.environment.lawsynthVersion));
    if (provenance.environment.runtime !== undefined) rows.push(row("Runtime", provenance.environment.runtime));
  }
  for (const algorithm of provenance.algorithms ?? []) {
    rows.push(row("Algorithm", `${algorithm.name} ${algorithm.version}${algorithm.deterministic === false ? " · nondeterministic" : ""}`, algorithm.deterministic === false ? "warning" : "neutral"));
  }
  const reproducible = provenance.seed !== undefined && provenance.worldHash !== undefined && (provenance.algorithms?.length ?? 0) > 0;
  return Object.freeze({
    rows: Object.freeze(rows),
    assumptions: Object.freeze([...(provenance.assumptions ?? [])]),
    artifacts: Object.freeze([...(provenance.artifacts ?? [])]),
    reproducible,
  });
}

export function isSha256(value: string): boolean {
  return /^[a-f0-9]{64}$/iu.test(value);
}

export async function sha256Hex(data: ArrayBuffer | Uint8Array | string): Promise<string> {
  if (globalThis.crypto?.subtle === undefined) throw new Error("SHA-256 requires the Web Crypto API");
  const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data instanceof Uint8Array ? data : new Uint8Array(data);
  const copy = new Uint8Array(bytes.byteLength);
  copy.set(bytes);
  const digest = await globalThis.crypto.subtle.digest("SHA-256", copy.buffer);
  return [...new Uint8Array(digest)].map((value) => value.toString(16).padStart(2, "0")).join("");
}

export async function verifyArtifact(data: ArrayBuffer | Uint8Array | string, artifact: ArtifactReference): Promise<boolean> {
  if (!isSha256(artifact.sha256)) throw new RangeError(`artifact ${artifact.path} has an invalid SHA-256`);
  return (await sha256Hex(data)).toLowerCase() === artifact.sha256.toLowerCase();
}
