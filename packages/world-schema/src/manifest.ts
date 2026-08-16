import type { ArtifactReference } from "./provenance.js";
import type { Identifier, IsoTimestamp, JsonValue, Sha256 } from "./types.js";

/** Exact manifest accepted by `lawsynth-bundle` today. */
export const CURRENT_BUNDLE_VERSION = "0.1.0";
export const RUST_WORLD_ENCODING = "lawsynth-world-binary-v1";

export interface BundleEntry {
  path: string;
  mediaType: string;
  sha256: Sha256;
  bytes: number;
  required?: boolean;
}

export interface BundleSignature {
  algorithm: "ed25519";
  keyId: string;
  signature: string;
  signedHash: Sha256;
}

export interface WorldManifest {
  format: "lawsynth-world";
  /** The on-disk Rust manifest uses snake_case, not `formatVersion`. */
  format_version: typeof CURRENT_BUNDLE_VERSION;
  world_encoding: typeof RUST_WORLD_ENCODING;
}

/** Rich catalog metadata. It is not the on-disk `manifest.json` accepted by Rust. */
export interface BundleCatalog {
  worldId: Identifier;
  createdAt: IsoTimestamp;
  createdBy?: string;
  root: string;
  entries: readonly BundleEntry[];
  artifacts?: readonly ArtifactReference[];
  signature?: BundleSignature;
  metadata?: Readonly<Record<string, JsonValue>>;
}

export function entryByPath(catalog: BundleCatalog, path: string): BundleEntry | undefined {
  return catalog.entries.find((entry) => entry.path === path);
}

export function requiredEntries(manifest: BundleCatalog): readonly BundleEntry[] {
  return manifest.entries.filter((entry) => entry.required !== false);
}
