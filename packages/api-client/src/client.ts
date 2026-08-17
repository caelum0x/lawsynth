import type { AuthProvider } from "./auth.js";
import { ArtifactsApi } from "./artifacts.js";
import { CollaborationApi } from "./collaboration.js";
import { DatasetsApi } from "./datasets.js";
import { DownloadsApi } from "./downloads.js";
import { streamRunEvents, type EventStreamOptions } from "./events.js";
import type { ApiEvent, RunId } from "./generated.js";
import { ProjectsApi } from "./projects.js";
import { RunsApi } from "./runs.js";
import { SimulationsApi } from "./simulations.js";
import { FetchTransport, type Transport } from "./transport.js";
import { UploadsApi } from "./uploads.js";
import { WorldsApi } from "./worlds.js";

export interface LawSynthClientOptions {
  baseUrl: string | URL;
  auth?: AuthProvider | undefined;
  fetch?: typeof globalThis.fetch | undefined;
  timeoutMs?: number | undefined;
  maxRetries?: number | undefined;
}

/** High-level composition of the HTTP resource clients. */
export class LawSynthClient {
  readonly transport: Transport;
  readonly projects: ProjectsApi;
  readonly datasets: DatasetsApi;
  readonly runs: RunsApi;
  readonly worlds: WorldsApi;
  readonly simulations: SimulationsApi;
  readonly artifacts: ArtifactsApi;
  readonly uploads: UploadsApi;
  readonly downloads: DownloadsApi;
  readonly collaboration: CollaborationApi;

  constructor(options: LawSynthClientOptions | Transport) {
    this.transport = isTransport(options) ? options : new FetchTransport({
      baseUrl: options.baseUrl, auth: options.auth, fetch: options.fetch,
      defaultTimeoutMs: options.timeoutMs, maxRetries: options.maxRetries, userAgent: "@lawsynth/api-client/0.1.0",
    });
    this.projects = new ProjectsApi(this.transport);
    this.datasets = new DatasetsApi(this.transport);
    this.runs = new RunsApi(this.transport);
    this.worlds = new WorldsApi(this.transport);
    this.simulations = new SimulationsApi(this.transport);
    this.artifacts = new ArtifactsApi(this.transport);
    this.uploads = new UploadsApi(this.transport);
    this.downloads = new DownloadsApi(this.transport);
    this.collaboration = new CollaborationApi(this.transport);
  }

  health(signal?: AbortSignal): Promise<{ status: string }> { return this.transport.request({ path: "/v1/health", signal }); }
  version(signal?: AbortSignal): Promise<{ version: string; api_version: string }> { return this.transport.request({ path: "/v1/version", signal }); }
  events(runId: RunId, options?: EventStreamOptions): AsyncGenerator<ApiEvent, void, void> { return streamRunEvents(this.transport, runId, options); }
}

function isTransport(value: LawSynthClientOptions | Transport): value is Transport { return "request" in value && typeof value.request === "function"; }
