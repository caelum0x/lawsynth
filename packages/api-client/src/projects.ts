import type { CreateProjectRequest, Page, Project, ProjectId } from "./generated.js";
import { pageQuery, paginate, type PageRequest } from "./pagination.js";
import { pathSegment, type Transport } from "./transport.js";
export class ProjectsApi {
  constructor(private readonly transport: Transport) {}
  create(request: CreateProjectRequest, idempotencyKey?: string, signal?: AbortSignal): Promise<Project> { return this.transport.request({ method: "POST", path: "/v1/projects", body: request, idempotencyKey, signal }); }
  get(projectId: ProjectId, signal?: AbortSignal): Promise<Project> { return this.transport.request({ path: `/v1/projects/${pathSegment(projectId)}`, signal }); }
  list(request: PageRequest = {}): Promise<Page<Project>> { return this.transport.request({ path: "/v1/projects", query: pageQuery(request), signal: request.signal }); }
  all(request: PageRequest = {}): AsyncGenerator<Project, void, void> { return paginate((page) => this.list(page), request); }
}
