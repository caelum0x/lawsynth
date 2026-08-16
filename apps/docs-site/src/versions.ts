export interface DocumentationVersion {
  readonly version: string;
  readonly label: string;
  readonly path: string;
  readonly stable: boolean;
  readonly deprecated?: boolean;
}

const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z.-]+))?$/u;

function compareVersions(left: string, right: string): number {
  const a = SEMVER.exec(left)!;
  const b = SEMVER.exec(right)!;
  for (let index = 1; index <= 3; index += 1) {
    const difference = Number(a[index]) - Number(b[index]);
    if (difference !== 0) return difference;
  }
  return (a[4] ?? "").localeCompare(b[4] ?? "");
}

export class VersionCatalog {
  readonly versions: readonly DocumentationVersion[];

  constructor(values: readonly DocumentationVersion[]) {
    const seen = new Set<string>();
    this.versions = Object.freeze(values.map((value) => {
      if (!SEMVER.test(value.version) || !value.path.startsWith("/") || !value.label.trim() || seen.has(value.version)) {
        throw new RangeError(`invalid documentation version: ${value.version}`);
      }
      seen.add(value.version);
      return Object.freeze(value);
    }).sort((left, right) => compareVersions(right.version, left.version)));
    if (this.versions.filter((value) => value.stable).length !== 1) {
      throw new RangeError("exactly one documentation version must be stable");
    }
  }

  get current(): DocumentationVersion { return this.versions.find((value) => value.stable)!; }
  get(version: string): DocumentationVersion | undefined { return this.versions.find((value) => value.version === version); }
}
