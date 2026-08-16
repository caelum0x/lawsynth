import type { ArtifactDescriptor, ArtifactId, Page, ProjectId } from "./generated.js";
import { pageQuery, paginate, type PageRequest } from "./pagination.js";
import { pathSegment, type Transport } from "./transport.js";
export class ArtifactsApi {
  constructor(private readonly transport: Transport) {}
  get(artifactId: ArtifactId, signal?: AbortSignal): Promise<ArtifactDescriptor> { return this.transport.request({ path: `/v1/artifacts/${pathSegment(artifactId)}`, signal }); }
  list(projectId: ProjectId, request: PageRequest = {}): Promise<Page<ArtifactDescriptor>> { return this.transport.request({ path: "/v1/artifacts", query: { project_id: projectId, ...pageQuery(request) }, signal: request.signal }); }
  all(projectId: ProjectId, request: PageRequest = {}): AsyncGenerator<ArtifactDescriptor, void, void> { return paginate((page) => this.list(projectId, page), request); }
  remove(artifactId: ArtifactId, idempotencyKey?: string, signal?: AbortSignal): Promise<void> { return this.transport.request({ method: "DELETE", path: `/v1/artifacts/${pathSegment(artifactId)}`, idempotencyKey, response: "void", signal }); }
}
export function validateArtifactDescriptor(artifact: ArtifactDescriptor): ArtifactDescriptor {
  if (!artifact.id || !artifact.project_id || !/^[a-f0-9]{64}$/iu.test(artifact.sha256) || !Number.isSafeInteger(artifact.byte_len) || artifact.byte_len < 0) throw new TypeError("Invalid artifact descriptor");
  return artifact;
}
