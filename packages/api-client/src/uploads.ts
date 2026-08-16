import type { ProjectId, UploadPart, UploadSession } from "./generated.js";
import { pathSegment, type Transport } from "./transport.js";
export interface CompleteUploadRequest { parts: readonly UploadPart[]; sha256: string; }
export class UploadsApi {
  constructor(private readonly transport: Transport) {}
  create(projectId: ProjectId, fileName: string, byteLength: number, idempotencyKey?: string, signal?: AbortSignal): Promise<UploadSession> {
    if (!fileName || fileName.length > 255 || /[\\/\0]/u.test(fileName) || !Number.isSafeInteger(byteLength) || byteLength < 0) throw new TypeError("Invalid upload metadata");
    return this.transport.request({ method: "POST", path: "/v1/uploads", body: { project_id: projectId, file_name: fileName, byte_length: byteLength }, idempotencyKey, signal });
  }
  putPart(uploadId: string, partNumber: number, bytes: Blob | ArrayBuffer | ArrayBufferView, signal?: AbortSignal): Promise<UploadPart> {
    if (!Number.isSafeInteger(partNumber) || partNumber < 1 || partNumber > 10_000) throw new RangeError("Part number must be in 1..=10000");
    return this.transport.request({ method: "PUT", path: `/v1/uploads/${pathSegment(uploadId)}/parts/${partNumber}`, body: bytes, signal });
  }
  complete(uploadId: string, request: CompleteUploadRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<UploadSession> {
    if (!/^[0-9a-f]{64}$/u.test(request.sha256)) throw new TypeError("Upload SHA-256 is invalid");
    return this.transport.request({ method: "POST", path: `/v1/uploads/${pathSegment(uploadId)}/complete`, body: request, idempotencyKey, signal });
  }
  abort(uploadId: string, idempotencyKey?: string, signal?: AbortSignal): Promise<void> { return this.transport.request({ method: "DELETE", path: `/v1/uploads/${pathSegment(uploadId)}`, idempotencyKey, response: "void", signal }); }
}
