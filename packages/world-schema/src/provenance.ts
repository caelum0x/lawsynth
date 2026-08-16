import type { Identifier, IsoTimestamp, JsonValue, Sha256 } from "./types.js";

export interface AlgorithmRecord {
  name: string;
  version: string;
  configurationHash?: Sha256;
  deterministic?: boolean;
}

export interface EnvironmentRecord {
  lawsynthVersion: string;
  operatingSystem?: string;
  architecture?: string;
  runtime?: string;
  dependencies?: Readonly<Record<string, string>>;
  hardwareClass?: string;
}

export interface ArtifactReference {
  path: string;
  mediaType: string;
  sha256: Sha256;
  bytes?: number;
}

export interface Provenance {
  runId?: Identifier;
  createdAt: IsoTimestamp;
  seed?: number;
  dataHash?: Sha256;
  planHash?: Sha256;
  worldHash?: Sha256;
  algorithms?: readonly AlgorithmRecord[];
  environment?: EnvironmentRecord;
  assumptions?: readonly string[];
  artifacts?: readonly ArtifactReference[];
  parentWorld?: Identifier;
  metadata?: Readonly<Record<string, JsonValue>>;
}
