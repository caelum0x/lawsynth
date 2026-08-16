import type { CreateDatasetRequest, DatasetDescriptor, DatasetId, Page, ProjectId } from "./generated.js";
import { pageQuery, paginate, type PageRequest } from "./pagination.js";
import { pathSegment, type Transport } from "./transport.js";
export class DatasetsApi {
  constructor(private readonly transport: Transport) {}
  create(request: CreateDatasetRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<DatasetDescriptor> { return this.transport.request({ method: "POST", path: "/v1/datasets", body: request, idempotencyKey, signal }); }
  get(datasetId: DatasetId, signal?: AbortSignal): Promise<DatasetDescriptor> { return this.transport.request({ path: `/v1/datasets/${pathSegment(datasetId)}`, signal }); }
  list(projectId: ProjectId, request: PageRequest = {}): Promise<Page<DatasetDescriptor>> { return this.transport.request({ path: "/v1/datasets", query: { project_id: projectId, ...pageQuery(request) }, signal: request.signal }); }
  all(projectId: ProjectId, request: PageRequest = {}): AsyncGenerator<DatasetDescriptor, void, void> { return paginate((page) => this.list(projectId, page), request); }
}
