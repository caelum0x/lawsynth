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
  /** Set once a discovery run has stored the world it produced. */
  world_id?: WorldId;
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

// --------------------------------------------------------------------------- //
// Discovery-as-a-service product surface                                       //
//                                                                              //
// These mirror the JSON contracts the Python service exposes for the discovery //
// run workflow (`POST /v1/runs` with a dataset, `GET /v1/runs/{id}/world`) and //
// the world product actions (`explain`, `forecast`, `report`, `compare`).      //
// --------------------------------------------------------------------------- //

/** Native discovery knobs accepted by the service (`discovery` object on a submit). */
export interface DiscoveryConfig {
  recipe?: string;
  preset?: string;
  polynomial_degree?: number;
  /** Friendly alias the service expands to `polynomial_degree`. */
  degree?: number;
  threshold?: number;
  solver?: string;
  include_trigonometric?: boolean;
  include_rational?: boolean;
  smoothing_radius?: number;
  derivative_method?: string;
  savgol_window?: number;
  tvreg_lambda?: number;
  tvreg_iterations?: number;
  symbolic_depth?: number;
}

/** An inline dataset given as structured `time`/`columns` numeric series. */
export interface InlineColumnsDataset {
  name?: string;
  time: readonly number[];
  columns: Readonly<Record<string, readonly number[]>>;
}

/** An inline dataset given as a raw CSV string. */
export interface InlineCsvDataset {
  name?: string;
  csv: string;
  time_column?: string;
}

export type InlineDataset = InlineColumnsDataset | InlineCsvDataset;

/**
 * Body for a discovery submit. Provide exactly one of `dataset_id` (an already
 * uploaded dataset) or `dataset` (inline observations). `states` are the
 * dataset columns whose dynamics to discover.
 */
export interface DiscoveryRunRequest {
  states: readonly string[];
  name?: string;
  world_name?: string;
  project_id?: ProjectId;
  discovery?: DiscoveryConfig;
  dataset_id?: DatasetId;
  dataset?: InlineDataset;
}

/** The declarative world record persisted by a completed discovery run. */
export interface WorldRecord {
  id: WorldId;
  organization_id: string;
  name: string;
  created_at: string;
  deleted_at: string | null;
  states: readonly string[];
  controls: readonly string[];
  parameters: Readonly<Record<string, number>>;
  equations: Readonly<Record<string, string>>;
  project_id?: ProjectId | null;
  dataset_id?: DatasetId | null;
  metadata?: JsonObject;
}

export interface RunWorldLinks {
  self: string;
  explain: string;
  report: string;
}

/** Response of `GET /v1/runs/{id}/world`: the world a run discovered. */
export interface RunWorld {
  run_id: RunId;
  world_id: WorldId;
  world: WorldRecord;
  links: RunWorldLinks;
}

/** One additive term of a read law (`coefficient * feature`). */
export interface LawTerm {
  coefficient: number;
  feature: string;
}

/** A single evolution law rendered in plain language, ordered by magnitude. */
export interface ReadLaw {
  target: string;
  expression: string;
  readable: string;
  terms: readonly LawTerm[];
  dominant_term: string | null;
}

export interface WorldComplexity {
  laws: number;
  parameters: number;
  controls: number;
  total_terms: number;
  terms_per_law: Readonly<Record<string, number>>;
}

/** Response of `GET /v1/worlds/{id}/explain`. */
export interface WorldExplanation {
  id: WorldId | null;
  name: string | null;
  variables: readonly string[];
  controls: readonly string[];
  parameters: Readonly<Record<string, number>>;
  laws: readonly ReadLaw[];
  dependencies: Readonly<Record<string, readonly string[]>>;
  complexity: WorldComplexity;
  assumptions: readonly string[];
}

export interface ForecastIntervention {
  at: number;
  parameters?: Readonly<Record<string, number>>;
  inputs?: Readonly<Record<string, number>>;
}

/** Body for `POST /v1/worlds/{id}/forecast`. */
export interface ForecastRequest {
  horizon: number;
  step: number;
  start?: number;
  initial: Readonly<Record<string, number>>;
  parameters?: Readonly<Record<string, number>>;
  inputs?: Readonly<Record<string, number>>;
  interventions?: readonly ForecastIntervention[];
}

export interface ForecastTrajectory {
  time: readonly number[];
  values: Readonly<Record<string, readonly number[]>>;
}

/** Response of `POST /v1/worlds/{id}/forecast`. */
export interface WorldForecast {
  id: WorldId | null;
  name: string | null;
  start: number;
  horizon: number;
  step: number;
  interventions: readonly ForecastIntervention[];
  trajectory: ForecastTrajectory;
}

/** Body for `POST /v1/worlds/compare`. */
export interface CompareRequest {
  left: WorldId;
  right: WorldId;
}

export interface WorldRef {
  id: WorldId | null;
  name: string | null;
}

export interface SetDiff {
  added: readonly string[];
  removed: readonly string[];
  common: readonly string[];
}

export interface ParameterChange {
  left: number;
  right: number;
  delta: number;
}

export interface ParameterDiff {
  added: Readonly<Record<string, number>>;
  removed: Readonly<Record<string, number>>;
  changed: Readonly<Record<string, ParameterChange>>;
  unchanged: readonly string[];
}

export interface LawChange {
  target: string;
  left: string;
  right: string;
}

export interface LawDiff {
  added: readonly string[];
  removed: readonly string[];
  changed: readonly LawChange[];
  unchanged: readonly string[];
}

export interface ComplexityDelta {
  laws: number;
  parameters: number;
  controls: number;
  total_terms: number;
}

/** Response of `POST /v1/worlds/compare`. */
export interface WorldComparison {
  left: WorldRef;
  right: WorldRef;
  variables: SetDiff;
  controls: SetDiff;
  parameters: ParameterDiff;
  laws: LawDiff;
  complexity_delta: ComplexityDelta;
}
