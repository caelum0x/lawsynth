import type { Annotation, ProjectMember, ProjectRole, ReviewState, RevisionRecord } from "@lawsynth/api-client";
import type {
  ActionButton,
  ControlField,
  ControlOption,
  Metric,
  Notice,
  NoticeTone,
  ScreenModel,
  ScreenSection,
  TableColumn,
  TableRow,
} from "./types.js";

export type { ProjectRole } from "@lawsynth/api-client";
export type AnnotationTarget = "world" | "law" | "revision";

export const PROJECT_ROLES: readonly ProjectRole[] = Object.freeze(["owner", "editor", "viewer"]);
export const ANNOTATION_TARGETS: readonly AnnotationTarget[] = Object.freeze(["world", "law", "revision"]);

/**
 * Everything the collaboration screen renders. Membership, the world's revision
 * lineage, annotations, and the review state are supplied by the controller
 * (loaded through `@lawsynth/api-client`); the *acting role* and the *selected
 * revision* are read back from the shared store so role-gated actions and the
 * highlighted revision stay in sync with the rest of Studio.
 */
export interface CollaborationInput {
  readonly projectId?: string;
  readonly worldId?: string;
  /** The role the current user is acting as; drives which actions are enabled. */
  readonly actingRole: ProjectRole;
  readonly actingPrincipal?: string;
  readonly members: readonly ProjectMember[];
  readonly revisions: readonly RevisionRecord[];
  readonly annotations: readonly Annotation[];
  /** True once any revision of the world is approved. */
  readonly trusted: boolean;
  readonly selectedRevision?: number;
  readonly annotationDraft: string;
  readonly annotationTarget: AnnotationTarget;
  readonly annotationRef: string;
  readonly memberDraft: string;
  readonly memberRole: ProjectRole;
  /** True when no collaboration backend is configured (offline, additive). */
  readonly offline: boolean;
}

/** The membership + lineage + annotations a collaboration screen renders. */
export interface CollaborationData {
  readonly members: readonly ProjectMember[];
  readonly revisions: readonly RevisionRecord[];
  readonly annotations: readonly Annotation[];
  readonly trusted: boolean;
}

/**
 * Seed collaboration data so the screen is populated and demonstrable offline,
 * exactly as the other screens seed a fixture world when no live service is
 * present. A two-person project with an approved base revision and a draft edit.
 */
export function fixtureCollaboration(): CollaborationData {
  const worldId = "world-demo";
  const revisions: readonly RevisionRecord[] = [
    { world_id: worldId, number: 1, content_hash: "h1", derivation: { kind: "imported", source_hash: "h1" }, parents: [], actor: "token:owner", created_at: "2026-01-01T00:00:00.000Z", review_state: "approved", review_history: [{ from: "in_review", to: "approved", actor: "token:owner", at: "2026-01-01T00:01:00.000Z" }] },
    { world_id: worldId, number: 2, content_hash: "h2", derivation: { kind: "edited", parent: { world_id: worldId, number: 1, content_hash: "h1" }, ops: ["retune rate"] }, parents: [{ world_id: worldId, number: 1, content_hash: "h1" }], actor: "token:editor", created_at: "2026-01-01T00:02:00.000Z", review_state: "draft", review_history: [] },
  ];
  return {
    members: [
      { principal: "token:owner", role: "owner" },
      { principal: "token:editor", role: "editor" },
    ],
    revisions,
    annotations: [
      { world_id: worldId, ordinal: 1, target: "law", ref: "x", text: "looks linear in x", actor: "token:editor", created_at: "2026-01-01T00:03:00.000Z" },
    ],
    trusted: true,
  };
}

const REVIEW_TONE: Record<ReviewState, NoticeTone> = {
  approved: "success",
  in_review: "warning",
  rejected: "error",
  draft: "info",
};

function roleOptions(): readonly ControlOption[] {
  return PROJECT_ROLES.map((role) => ({ value: role, label: role[0]!.toUpperCase() + role.slice(1) }));
}

function derivationLabel(revision: RevisionRecord): string {
  const parents = revision.parents.length;
  return parents === 0 ? revision.derivation.kind : `${revision.derivation.kind} (${parents} parent${parents === 1 ? "" : "s"})`;
}

function refLabel(annotation: Annotation): string {
  return annotation.ref === null || annotation.ref === "" ? "—" : String(annotation.ref);
}

/**
 * Collaboration / Review — the "who can do what, and what's approved?" screen.
 * It shows a shared project's members and roles, a world's revision lineage and
 * review state, and its annotations, with review/annotate/membership actions
 * gated by the acting role: a viewer sees everything but can mutate nothing;
 * an editor can annotate and request review; only an owner may approve or manage
 * membership. All gating mirrors the server-side P6 rules so the UI never offers
 * an action the service would reject.
 */
export function collaborationModel(input: CollaborationInput): ScreenModel {
  const canManage = input.actingRole === "owner";
  const canWrite = input.actingRole !== "viewer";
  const canApprove = input.actingRole === "owner";
  const selected = input.selectedRevision === undefined
    ? undefined
    : input.revisions.find((revision) => revision.number === input.selectedRevision);

  const sections: ScreenSection[] = [];

  // ── Role banner + offline notice ──────────────────────────────────────────
  const notices: Notice[] = [
    { tone: "info", message: `Acting as ${input.actingRole}${input.actingPrincipal === undefined ? "" : ` (${input.actingPrincipal})`} — actions are gated by this role.` },
  ];
  if (input.offline) notices.push({ tone: "warning", message: "No collaboration backend configured — showing seeded demo data. Sharing, roles, and review activate when a backend is connected." });
  if (input.worldId === undefined) notices.push({ tone: "info", message: "Open a workspace world to see its revision lineage, review state, and annotations." });
  sections.push({ kind: "notices", id: "collab-notices", notices });

  // ── Health metrics ────────────────────────────────────────────────────────
  const metrics: readonly Metric[] = [
    { label: "Acting role", value: input.actingRole, tone: canManage ? "success" : canWrite ? "warning" : "info" },
    { label: "Members", value: String(input.members.length) },
    { label: "Revisions", value: String(input.revisions.length) },
    { label: "World trusted", value: input.trusted ? "yes" : "no", tone: input.trusted ? "success" : "info" },
    { label: "Selected revision", value: selected === undefined ? "—" : `#${selected.number} · ${selected.review_state}`, tone: selected === undefined ? "info" : REVIEW_TONE[selected.review_state] },
  ];
  sections.push({ kind: "metrics", id: "collab-metrics", title: "Collaboration", metrics });

  // ── "View as" role selector (persisted through the shared store) ──────────
  sections.push({
    kind: "controls",
    id: "collab-role",
    title: "View as role",
    fields: [{ id: "collab:role", label: "Acting role", kind: "select", value: input.actingRole, options: roleOptions(), help: "Switch roles to preview what each collaborator can do." }],
  });

  // ── Members + roles ───────────────────────────────────────────────────────
  const memberColumns: readonly TableColumn[] = [
    { key: "principal", label: "Principal" },
    { key: "role", label: "Role" },
    { key: "you", label: "You", align: "end" },
  ];
  const memberRows: readonly TableRow[] = input.members.map((member) => ({
    id: member.principal,
    selected: false,
    emphasis: member.role === "owner",
    cells: [member.principal, member.role, member.principal === input.actingPrincipal ? "●" : ""],
  }));
  sections.push({ kind: "table", id: "collab-members", title: "Members & roles", columns: memberColumns, rows: memberRows, empty: "No members yet — this project is not shared." });

  // ── Membership management (owner-only) ────────────────────────────────────
  const memberFields: readonly ControlField[] = [
    { id: "collab:member-principal", label: "New member principal", kind: "text", value: input.memberDraft, disabled: !canManage, help: canManage ? "An opaque principal id (e.g. token:abcd1234)." : "Only an owner may manage membership." },
    { id: "collab:member-role", label: "Role", kind: "select", value: input.memberRole, options: roleOptions(), disabled: !canManage },
  ];
  sections.push({ kind: "controls", id: "collab-member-form", title: "Manage membership", fields: memberFields });
  const memberButtons: readonly ActionButton[] = [
    { id: "collab:add-member", label: "Add / update member", disabled: !canManage || input.memberDraft.trim() === "" },
  ];
  sections.push({ kind: "actions", id: "collab-member-actions", buttons: memberButtons });

  // ── Revision lineage ──────────────────────────────────────────────────────
  const revisionColumns: readonly TableColumn[] = [
    { key: "number", label: "#", align: "end" },
    { key: "derivation", label: "Derivation" },
    { key: "state", label: "Review" },
    { key: "actor", label: "Actor" },
    { key: "trusted", label: "Trusted", align: "end" },
  ];
  const revisionRows: readonly TableRow[] = input.revisions.map((revision) => ({
    id: String(revision.number),
    selected: revision.number === input.selectedRevision,
    emphasis: revision.review_state === "approved",
    cells: [String(revision.number), derivationLabel(revision), revision.review_state, revision.actor, revision.review_state === "approved" ? "✓" : ""],
  }));
  sections.push({ kind: "table", id: "collab-revisions", title: "Revision lineage", columns: revisionColumns, rows: revisionRows, empty: "No revisions — submit or edit a world to record one." });

  // ── Review actions (gated) ────────────────────────────────────────────────
  const state = selected?.review_state;
  const reviewButtons: readonly ActionButton[] = [
    { id: "collab:request-review", label: "Request review", disabled: !canWrite || state !== "draft" },
    { id: "collab:approve", label: "Approve", tone: "success", disabled: !canApprove || state !== "in_review" },
    { id: "collab:reject", label: "Reject", tone: "error", disabled: !canWrite || state !== "in_review" },
  ];
  sections.push({ kind: "actions", id: "collab-review", title: selected === undefined ? "Review (select a revision)" : `Review revision #${selected.number} (${selected.review_state})`, buttons: reviewButtons });

  // ── Annotations ───────────────────────────────────────────────────────────
  const annotationColumns: readonly TableColumn[] = [
    { key: "ordinal", label: "#", align: "end" },
    { key: "target", label: "Target" },
    { key: "ref", label: "Ref" },
    { key: "text", label: "Note" },
    { key: "actor", label: "Actor" },
  ];
  const annotationRows: readonly TableRow[] = input.annotations.map((annotation) => ({
    id: String(annotation.ordinal),
    selected: false,
    cells: [String(annotation.ordinal), annotation.target, refLabel(annotation), annotation.text, annotation.actor],
  }));
  sections.push({ kind: "table", id: "collab-annotations", title: "Annotations", columns: annotationColumns, rows: annotationRows, empty: "No annotations yet." });

  const needsRef = input.annotationTarget === "law" || input.annotationTarget === "revision";
  const annotationFields: readonly ControlField[] = [
    { id: "collab:annotation-text", label: "Note", kind: "text", value: input.annotationDraft, disabled: !canWrite, help: canWrite ? "Attach a note to the world, a law, or a revision." : "Only an editor or owner may annotate." },
    { id: "collab:annotation-target", label: "Target", kind: "select", value: input.annotationTarget, options: ANNOTATION_TARGETS.map((target) => ({ value: target, label: target })), disabled: !canWrite },
    { id: "collab:annotation-ref", label: input.annotationTarget === "revision" ? "Revision number" : "Law name", kind: "text", value: input.annotationRef, disabled: !canWrite || !needsRef, help: needsRef ? "Required for law/revision targets." : "Not used for a world annotation." },
  ];
  sections.push({ kind: "controls", id: "collab-annotation-form", title: "Add annotation", fields: annotationFields });
  const annotationButtons: readonly ActionButton[] = [
    { id: "collab:add-annotation", label: "Add annotation", disabled: !canWrite || input.annotationDraft.trim() === "" || (needsRef && input.annotationRef.trim() === "") },
  ];
  sections.push({ kind: "actions", id: "collab-annotation-actions", buttons: annotationButtons });

  return { id: "collaboration", title: "Collaboration & Review", subtitle: "Members, roles, revision lineage, annotations, and approval", sections };
}
