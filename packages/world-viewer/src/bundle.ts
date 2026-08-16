import type { BundleCatalog, WorldDefinition, WorldManifest } from "@lawsynth/world-schema";
import type { TrajectoryInput } from "@lawsynth/chart-core";

export const VIEWER_BUNDLE_FORMAT = "lawsynth-viewer";
export const VIEWER_BUNDLE_VERSION = 1;
export const DEFAULT_MAX_BUNDLE_BYTES = 32 * 1024 * 1024;

export interface ViewerBundle {
  readonly format: typeof VIEWER_BUNDLE_FORMAT;
  readonly version: typeof VIEWER_BUNDLE_VERSION;
  readonly world: WorldDefinition;
  readonly trajectory?: TrajectoryInput;
  readonly manifest?: WorldManifest;
  readonly catalog?: BundleCatalog;
}

export interface BundleDecodeOptions {
  readonly maxBytes?: number;
}

export class ViewerBundleError extends Error {
  constructor(message: string, override readonly cause?: unknown) {
    super(message, cause === undefined ? undefined : { cause });
    this.name = "ViewerBundleError";
  }
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function assertWorldShape(value: unknown): asserts value is WorldDefinition {
  if (!record(value)) throw new ViewerBundleError("world must be an object");
  if (typeof value.id !== "string" || value.id.length === 0) throw new ViewerBundleError("world.id must be non-empty");
  if (!record(value.time) || (value.time.kind !== "continuous" && value.time.kind !== "discrete")) throw new ViewerBundleError("world.time.kind must be continuous or discrete");
  if (!Array.isArray(value.variables) || value.variables.length === 0) throw new ViewerBundleError("world.variables must be non-empty");
  if (!Array.isArray(value.laws)) throw new ViewerBundleError("world.laws must be an array");
}

function assertTrajectoryShape(value: unknown): asserts value is TrajectoryInput {
  if (!record(value) || !Array.isArray(value.variables) || !Array.isArray(value.times) || !Array.isArray(value.values)) {
    throw new ViewerBundleError("trajectory must contain variables, times, and values arrays");
  }
}

export function parseViewerBundle(value: unknown): ViewerBundle {
  if (!record(value)) throw new ViewerBundleError("viewer bundle must be an object");
  if (value.format !== VIEWER_BUNDLE_FORMAT) throw new ViewerBundleError(`unsupported viewer bundle format: ${String(value.format)}`);
  if (value.version !== VIEWER_BUNDLE_VERSION) throw new ViewerBundleError(`unsupported viewer bundle version: ${String(value.version)}`);
  assertWorldShape(value.world);
  if (value.trajectory !== undefined) assertTrajectoryShape(value.trajectory);
  return Object.freeze({
    format: VIEWER_BUNDLE_FORMAT,
    version: VIEWER_BUNDLE_VERSION,
    world: value.world,
    ...(value.trajectory === undefined ? {} : { trajectory: value.trajectory }),
    ...(value.manifest === undefined ? {} : { manifest: value.manifest as WorldManifest }),
    ...(value.catalog === undefined ? {} : { catalog: value.catalog as BundleCatalog }),
  });
}

export function decodeViewerBundle(input: string | ArrayBuffer | Uint8Array, options: BundleDecodeOptions = {}): ViewerBundle {
  const maxBytes = options.maxBytes ?? DEFAULT_MAX_BUNDLE_BYTES;
  if (!Number.isSafeInteger(maxBytes) || maxBytes <= 0) throw new RangeError("maxBytes must be a positive safe integer");
  let json: string;
  if (typeof input === "string") {
    if (new TextEncoder().encode(input).byteLength > maxBytes) throw new ViewerBundleError(`bundle exceeds ${maxBytes} bytes`);
    json = input;
  } else {
    const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
    if (bytes.byteLength > maxBytes) throw new ViewerBundleError(`bundle exceeds ${maxBytes} bytes`);
    if (bytes[0] === 0x50 && bytes[1] === 0x4b) {
      throw new ViewerBundleError("Native .lsworld ZIP archives must be decoded by the LawSynth bundle service before browser viewing");
    }
    try { json = new TextDecoder("utf-8", { fatal: true }).decode(bytes); }
    catch (error) { throw new ViewerBundleError("bundle is not valid UTF-8", error); }
  }
  try { return parseViewerBundle(JSON.parse(json) as unknown); }
  catch (error) {
    if (error instanceof ViewerBundleError) throw error;
    throw new ViewerBundleError("bundle is not valid JSON", error);
  }
}

export function encodeViewerBundle(bundle: ViewerBundle, pretty = false): string {
  const checked = parseViewerBundle(bundle);
  return JSON.stringify(checked, null, pretty ? 2 : undefined);
}

export function createViewerBundle(world: WorldDefinition, trajectory?: TrajectoryInput): ViewerBundle {
  return parseViewerBundle({ format: VIEWER_BUNDLE_FORMAT, version: VIEWER_BUNDLE_VERSION, world, ...(trajectory === undefined ? {} : { trajectory }) });
}
