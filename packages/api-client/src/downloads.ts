import type { ArtifactId } from "./generated.js";
import { pathSegment, type Transport } from "./transport.js";
export interface DownloadDescriptor { filename: string | undefined; mediaType: string | undefined; byteLength: number | undefined; }
export class DownloadsApi {
  constructor(private readonly transport: Transport) {}
  blob(artifactId: ArtifactId, signal?: AbortSignal): Promise<Blob> { return this.transport.request({ path: `/v1/artifacts/${pathSegment(artifactId)}/content`, response: "blob", signal, timeoutMs: 120_000 }); }
  bytes(artifactId: ArtifactId, signal?: AbortSignal): Promise<ArrayBuffer> { return this.transport.request({ path: `/v1/artifacts/${pathSegment(artifactId)}/content`, response: "arrayBuffer", signal, timeoutMs: 120_000 }); }
}
export function describeDownload(headers: Headers): DownloadDescriptor {
  const length = headers.get("content-length");
  return { filename: parseContentDisposition(headers.get("content-disposition")), mediaType: headers.get("content-type") ?? undefined, byteLength: length === null ? undefined : parseContentLength(length) };
}
export function parseContentDisposition(value: string | null): string | undefined {
  if (value === null) return undefined; const extended = /(?:^|;)\s*filename\*=UTF-8''([^;]+)/iu.exec(value)?.[1];
  if (extended !== undefined) { try { return safeFilename(decodeURIComponent(extended)); } catch { return undefined; } }
  const regular = /(?:^|;)\s*filename="?([^";]+)"?/iu.exec(value)?.[1]; return regular === undefined ? undefined : safeFilename(regular);
}
function parseContentLength(value: string): number { if (!/^\d+$/u.test(value)) throw new TypeError("Invalid Content-Length header"); const parsed = Number(value); if (!Number.isSafeInteger(parsed)) throw new RangeError("Content-Length exceeds JavaScript safe integer range"); return parsed; }
function safeFilename(value: string): string | undefined { const filename = value.trim(); return filename === "" || /[\\/\0\r\n]/u.test(filename) ? undefined : filename; }
