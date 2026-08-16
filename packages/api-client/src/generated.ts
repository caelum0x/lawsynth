/** JSON values accepted by the dependency-free server dispatcher. */
export type JsonValue = null | boolean | number | string | readonly JsonValue[] | { readonly [key: string]: JsonValue };
export type JsonObject = Readonly<Record<string, JsonValue>>;

export type ProjectId = string;
export type DatasetId = string;
export type RunId = string;
export type WorldId = string;
export type ArtifactId = string;
export type SimulationId = string;

/**
 * Common persisted record returned by the currently implemented Python
 * service.  `organization_id` comes from the authenticated principal and is
 * not accepted on create requests.
 */
export interface RepositoryRecord {
  id: string;
  organization_id: string;
  name: string;
  created_at: string;
  deleted_at: string | null;
  metadata?: JsonObject;
}

export interface Project extends RepositoryRecord {}

export interface CreateProjectRequest {
  name: string;
  metadata?: JsonObject;
}

export type ColumnType = "float64" | "int64" | "boolean" | "utf8" | "timestamp_millis";

/** The service currently validates dataset schemas as unique non-empty names. */
export interface DatasetDescriptor extends RepositoryRecord {
  schema: readonly string[];
}

export interface CreateDatasetRequest {
  name: string;
  schema: readonly string[];
  metadata?: JsonObject;
}

export type RunStatus = "queued" | "running" | "succeeded" | "failed" | "cancelled";

export interface RunSummary {
  id: RunId;
  organization_id: string;
  name: string;
  status: RunStatus;
  created_at: string;
  deleted_at: string | null;
  metadata?: JsonObject;
}

export interface CreateRunRequest {
  name: string;
  status?: RunStatus;
  dataset_id?: DatasetId;
  world_id?: WorldId;
  metadata?: JsonObject;
}

export interface CandidateSummary {
  id: string;
  run_id: RunId;
  score: number;
  equation?: string;
  world_id?: WorldId;
}

export interface WorldRevision {
  id: WorldId;
  organization_id: string;
  name: string;
  world_id: WorldId;
  equations: readonly string[];
  created_at: string;
  deleted_at: string | null;
  metadata?: JsonObject;
}

export interface CreateWorldRequest {
  name: string;
  equations: readonly string[];
  metadata?: JsonObject;
}

export interface TimeRange {
  start: number;
  end: number;
  step: number;
}

export interface SimulationRequest {
  horizon: number;
  step: number;
  method?: "rk4" | string;
}

export interface CreateSimulationRequest {
  name: string;
  simulation: SimulationRequest;
  world_id?: WorldId;
  dataset_id?: DatasetId;
  metadata?: JsonObject;
}

export interface SimulationSummary extends RepositoryRecord {
  world_id: WorldId;
  status: RunStatus;
  artifact_id?: ArtifactId;
}

export type ArtifactMediaType = "json" | "csv" | "parquet" | "zip" | "text";

export interface ArtifactDescriptor {
  id: ArtifactId;
  project_id: ProjectId;
  run_id: RunId | null;
  media_type: ArtifactMediaType;
  byte_len: number;
  sha256: string;
}

export type EventKind =
  | "run_queued"
  | "run_started"
  | "progress"
  | "run_succeeded"
  | "run_failed"
  | "run_cancelled"
  | "artifact_created";

export interface ApiEvent<T = unknown> {
  event_id: string;
  organization_id: string;
  topic: string;
  occurred_at: string;
  payload: T;
}

export interface Page<T> {
  items: readonly T[];
  next: string | null;
}

export interface UploadSession {
  id: string;
  project_id: ProjectId;
  part_size: number;
  expires_at: string;
}

export interface UploadPart {
  part_number: number;
  etag: string;
  bytes: number;
}

export interface ProblemDetails {
  type?: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
  code?: string;
  errors?: Readonly<Record<string, readonly string[]>>;
  request_id?: string;
}
