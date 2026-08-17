import type { JsonObject, JsonValue, ProjectId, WorldId } from "./generated.js";
import { pathSegment, type Transport } from "./transport.js";

// --------------------------------------------------------------------------- //
// P6 collaboration surface (specs/collaboration/README.md)                     //
//                                                                              //
// Shared, multi-user workspaces: membership/roles, immutable revision lineage, //
// annotations, the draft->in_review->approved|rejected review state machine,   //
// and the deterministic workspace merge. These mirror the JSON contracts the   //
// Python service exposes in `collaboration_routes.py`. Role enforcement is      //
// server-side against the caller's role for a specific project.                //
// --------------------------------------------------------------------------- //

export type ProjectRole = "owner" | "editor" | "viewer";
export type ReviewState = "draft" | "in_review" | "approved" | "rejected";
/** The target states a review transition may request (`draft` is only an origin). */
export type ReviewAction = "in_review" | "approved" | "rejected";
export type AnnotationTarget = "world" | "law" | "revision";
export type DerivationKind = "discovered" | "edited" | "composed" | "imported";

export interface ProjectMember {
  readonly principal: string;
  readonly role: ProjectRole;
}

/** A parent a revision derives from, linked by content hash once resolved. */
export interface RevisionParentRef {
  readonly world_id: WorldId;
  readonly number: number;
  readonly content_hash?: string;
}

/** How a revision was produced — one of the four derivation tags in the spec. */
export interface RevisionDerivation {
  readonly kind: DerivationKind;
  readonly parent?: RevisionParentRef;
  readonly parents?: readonly RevisionParentRef[];
  readonly ops?: readonly string[];
  readonly source_hash?: string;
  readonly dataset_hash?: string;
  readonly config?: JsonObject;
  readonly namespacing?: JsonObject;
}

export interface ReviewHistoryEntry {
  readonly from: ReviewState;
  readonly to: ReviewState;
  readonly actor: string;
  readonly at: string;
}

/** An immutable, per-world revision record with its review state. */
export interface RevisionRecord {
  readonly world_id: WorldId;
  readonly number: number;
  readonly content_hash: string;
  readonly derivation: RevisionDerivation;
  readonly parents: readonly RevisionParentRef[];
  readonly actor: string;
  readonly created_at: string;
  readonly review_state: ReviewState;
  readonly review_history: readonly ReviewHistoryEntry[];
  /** Present on single-revision reads: true once the revision is `approved`. */
  readonly trusted?: boolean;
}

/** Response of `GET /v1/worlds/{id}/revisions`. */
export interface RevisionList {
  readonly items: readonly RevisionRecord[];
  /** True once any revision of the world is `approved`. */
  readonly trusted: boolean;
}

export interface Annotation {
  readonly world_id: WorldId;
  readonly ordinal: number;
  readonly target: AnnotationTarget;
  readonly ref: string | number | null;
  readonly text: string;
  readonly actor: string;
  readonly created_at: string;
}

/** A `library.tsv`-style workspace row keyed by world name. */
export interface WorkspaceRow {
  readonly name: string;
  readonly content_hash: string;
  readonly revision: number;
  readonly [key: string]: JsonValue;
}

export interface MergeConflict {
  readonly name: string;
  readonly base: WorkspaceRow;
  readonly incoming: WorkspaceRow;
}

/** Response of `POST /v1/projects/{id}/merge`. */
export interface MergeResult {
  readonly merged: readonly WorkspaceRow[];
  readonly conflicts: readonly MergeConflict[];
  readonly merged_count: number;
  readonly conflict_count: number;
}

interface MemberList {
  readonly items: readonly ProjectMember[];
}

interface AnnotationList {
  readonly items: readonly Annotation[];
}

export interface AddAnnotationOptions {
  readonly target?: AnnotationTarget;
  /** Required for `law` (a law name) and `revision` (a positive integer) targets. */
  readonly ref?: string | number;
}

/**
 * Typed, transport-agnostic client for the nine P6 collaboration endpoints. Like
 * the other resource clients it takes a {@link Transport}, so it drives the real
 * `FetchTransport` in production and an in-memory fake offline.
 */
export class CollaborationApi {
  constructor(private readonly transport: Transport) {}

  /** Bind a principal to a role on a project (owner-only) — `POST /v1/projects/{id}/members`. */
  addMember(projectId: ProjectId, principal: string, role: ProjectRole, idempotencyKey?: string, signal?: AbortSignal): Promise<ProjectMember> {
    return this.transport.request({ method: "POST", path: `/v1/projects/${pathSegment(projectId)}/members`, body: { principal, role }, idempotencyKey, signal });
  }

  /** List a project's members and roles — `GET /v1/projects/{id}/members`. */
  async listMembers(projectId: ProjectId, signal?: AbortSignal): Promise<readonly ProjectMember[]> {
    const page = await this.transport.request<MemberList>({ path: `/v1/projects/${pathSegment(projectId)}/members`, signal });
    return page.items;
  }

  /** Remove a member (owner-only; the last owner cannot be removed) — `DELETE /v1/projects/{id}/members/{principal}`. */
  removeMember(projectId: ProjectId, principal: string, idempotencyKey?: string, signal?: AbortSignal): Promise<void> {
    return this.transport.request({ method: "DELETE", path: `/v1/projects/${pathSegment(projectId)}/members/${pathSegment(principal)}`, response: "void", idempotencyKey, signal });
  }

  /** A world's immutable revision chain — `GET /v1/worlds/{id}/revisions`. */
  listRevisions(worldId: WorldId, signal?: AbortSignal): Promise<RevisionList> {
    return this.transport.request({ path: `/v1/worlds/${pathSegment(worldId)}/revisions`, signal });
  }

  /** A single revision by its monotonic number — `GET /v1/worlds/{id}/revisions/{n}`. */
  getRevision(worldId: WorldId, n: number, signal?: AbortSignal): Promise<RevisionRecord> {
    return this.transport.request({ path: `/v1/worlds/${pathSegment(worldId)}/revisions/${encodeURIComponent(String(n))}`, signal });
  }

  /** Attach an annotation to a world, law, or revision (editor+) — `POST /v1/worlds/{id}/annotations`. */
  addAnnotation(worldId: WorldId, text: string, options: AddAnnotationOptions = {}, idempotencyKey?: string, signal?: AbortSignal): Promise<Annotation> {
    const body: Record<string, JsonValue> = { text, target: options.target ?? "world" };
    if (options.ref !== undefined) body["ref"] = options.ref;
    return this.transport.request({ method: "POST", path: `/v1/worlds/${pathSegment(worldId)}/annotations`, body, idempotencyKey, signal });
  }

  /** List a world's annotations, oldest first — `GET /v1/worlds/{id}/annotations`. */
  async listAnnotations(worldId: WorldId, signal?: AbortSignal): Promise<readonly Annotation[]> {
    const page = await this.transport.request<AnnotationList>({ path: `/v1/worlds/${pathSegment(worldId)}/annotations`, signal });
    return page.items;
  }

  /** Advance a revision's review state (owner-only for `approved`) — `POST /v1/worlds/{id}/revisions/{n}/review`. */
  reviewRevision(worldId: WorldId, n: number, action: ReviewAction, idempotencyKey?: string, signal?: AbortSignal): Promise<RevisionRecord> {
    return this.transport.request({ method: "POST", path: `/v1/worlds/${pathSegment(worldId)}/revisions/${encodeURIComponent(String(n))}/review`, body: { state: action }, idempotencyKey, signal });
  }

  /** Deterministically merge two workspace indexes (editor+) — `POST /v1/projects/{id}/merge`. */
  merge(projectId: ProjectId, base: readonly WorkspaceRow[], incoming: readonly WorkspaceRow[], idempotencyKey?: string, signal?: AbortSignal): Promise<MergeResult> {
    return this.transport.request({ method: "POST", path: `/v1/projects/${pathSegment(projectId)}/merge`, body: { base, incoming }, idempotencyKey, signal });
  }
}
