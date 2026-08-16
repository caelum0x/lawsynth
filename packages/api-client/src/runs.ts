import type { ApiEvent, CandidateSummary, CreateRunRequest, Page, ProjectId, RunId, RunSummary } from "./generated.js";
import { pageQuery, paginate, type PageRequest } from "./pagination.js";
import { pathSegment, type Transport } from "./transport.js";

export class RunsApi {
  constructor(private readonly transport: Transport) {}
  create(request: CreateRunRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<RunSummary> { return this.transport.request({ method: "POST", path: "/v1/runs", body: request, idempotencyKey, signal }); }
  get(runId: RunId, signal?: AbortSignal): Promise<RunSummary> { return this.transport.request({ path: `/v1/runs/${pathSegment(runId)}`, signal }); }
  list(projectId: ProjectId, request: PageRequest = {}): Promise<Page<RunSummary>> { return this.transport.request({ path: "/v1/runs", query: { project_id: projectId, ...pageQuery(request) }, signal: request.signal }); }
  all(projectId: ProjectId, request: PageRequest = {}): AsyncGenerator<RunSummary, void, void> { return paginate((page) => this.list(projectId, page), request); }
  cancel(runId: RunId, signal?: AbortSignal): Promise<RunSummary> { return this.transport.request({ method: "POST", path: `/v1/runs/${pathSegment(runId)}/cancel`, idempotencyKey: `cancel-${runId}`, signal }); }
  events(runId: RunId, after?: number, signal?: AbortSignal): Promise<readonly ApiEvent[]> { return this.transport.request({ path: `/v1/runs/${pathSegment(runId)}/events`, query: { after }, signal }); }
  candidates(runId: RunId, request: PageRequest = {}): Promise<Page<CandidateSummary>> { return this.transport.request({ path: `/v1/runs/${pathSegment(runId)}/candidates`, query: pageQuery(request), signal: request.signal }); }
}
