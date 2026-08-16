import { VersionCatalog } from "../src/versions.js";

/** The production catalog has one stable release and keeps prior documentation available. */
export const documentationVersions = new VersionCatalog([
  { version: "0.2.0-beta.1", label: "0.2 beta", path: "/v0.2", stable: false },
  { version: "0.1.0", label: "0.1", path: "/v0.1", stable: true },
]);

export function documentationVersionPath(version: string): string | undefined {
  return documentationVersions.get(version)?.path;
}
