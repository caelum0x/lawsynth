import type { WorldDefinition } from "@lawsynth/world-schema";
import type { CreateWorldRequest, ForecastRequest, Page, ProjectId, SimulationRequest, SimulationSummary, WorldComparison, WorldExplanation, WorldForecast, WorldId, WorldRevision } from "./generated.js";
import { pageQuery, paginate, type PageRequest } from "./pagination.js";
import { pathSegment, type Transport } from "./transport.js";
export class WorldsApi {
  constructor(private readonly transport: Transport) {}
  /** Structured, plain-language explanation of a world (`GET /v1/worlds/{id}/explain`). */
  explain(id: WorldId, signal?: AbortSignal): Promise<WorldExplanation> { return this.transport.request({ path: `/v1/worlds/${pathSegment(id)}/explain`, signal }); }
  /** Native-backed forecast honouring scheduled interventions (`POST /v1/worlds/{id}/forecast`). */
  forecast(id: WorldId, request: ForecastRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<WorldForecast> { return this.transport.request({ method: "POST", path: `/v1/worlds/${pathSegment(id)}/forecast`, body: request, idempotencyKey, signal }); }
  /** Self-contained HTML report for a world (`GET /v1/worlds/{id}/report`). */
  report(id: WorldId, signal?: AbortSignal): Promise<string> { return this.transport.request({ path: `/v1/worlds/${pathSegment(id)}/report`, response: "text", signal }); }
  /** Structured diff of two worlds (`POST /v1/worlds/compare`). */
  compare(left: WorldId, right: WorldId, idempotencyKey?: string, signal?: AbortSignal): Promise<WorldComparison> { return this.transport.request({ method: "POST", path: "/v1/worlds/compare", body: { left, right }, idempotencyKey, signal }); }
  create(request: CreateWorldRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<WorldRevision> { return this.transport.request({ method: "POST", path: "/v1/worlds", body: request, idempotencyKey, signal }); }
  get(id: WorldId, revision?: number, signal?: AbortSignal): Promise<WorldRevision> { return this.transport.request({ path: `/v1/worlds/${pathSegment(id)}`, query: { revision }, signal }); }
  list(projectId: ProjectId, request: PageRequest = {}): Promise<Page<WorldRevision>> { return this.transport.request({ path: "/v1/worlds", query: { project_id: projectId, ...pageQuery(request) }, signal: request.signal }); }
  all(projectId: ProjectId, request: PageRequest = {}): AsyncGenerator<WorldRevision, void, void> { return paginate((page) => this.list(projectId, page), request); }
  simulate(id: WorldId, request: SimulationRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<SimulationSummary> { return this.transport.request({ method: "POST", path: `/v1/worlds/${pathSegment(id)}/simulate`, body: request, idempotencyKey, signal }); }
  intervene(id: WorldId, request: SimulationRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<SimulationSummary> { return this.transport.request({ method: "POST", path: `/v1/worlds/${pathSegment(id)}/intervene`, body: request, idempotencyKey, signal }); }
  bundle(id: WorldId, revision?: number, signal?: AbortSignal): Promise<ArrayBuffer> { return this.transport.request({ path: `/v1/worlds/${pathSegment(id)}/bundle`, query: { revision }, response: "arrayBuffer", signal }); }
  import(world: WorldDefinition, projectId: ProjectId, idempotencyKey?: string, signal?: AbortSignal): Promise<WorldRevision> { return this.transport.request({ method: "POST", path: "/v1/bundles/import", body: { project_id: projectId, world }, idempotencyKey, signal }); }
}
