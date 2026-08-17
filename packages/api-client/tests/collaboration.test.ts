import test from "node:test";
import assert from "node:assert/strict";
import { ApiError, InMemoryCollaborationTransport, LawSynthClient } from "../dist/index.js";

test("collaboration client drives membership, revisions, annotations, review and merge offline", async () => {
  const transport = new InMemoryCollaborationTransport({
    projectId: "project-1",
    worldId: "world-1",
    owner: "token:owner",
  });
  const client = new LawSynthClient(transport);

  // The creator is seeded as the sole owner; the owner binds an editor + viewer.
  assert.deepEqual(await client.collaboration.listMembers("project-1"), [{ principal: "token:owner", role: "owner" }]);
  await client.collaboration.addMember("project-1", "token:editor", "editor", "idem-m1");
  await client.collaboration.addMember("project-1", "token:viewer", "viewer", "idem-m2");
  const roles = Object.fromEntries((await client.collaboration.listMembers("project-1")).map((m) => [m.principal, m.role]));
  assert.deepEqual(roles, { "token:owner": "owner", "token:editor": "editor", "token:viewer": "viewer" });

  // The seeded world starts with one draft revision (imported derivation).
  const revisions = await client.collaboration.listRevisions("world-1");
  assert.equal(revisions.items.length, 1);
  assert.equal(revisions.items[0]?.review_state, "draft");
  assert.equal(revisions.items[0]?.derivation.kind, "imported");
  assert.equal(revisions.trusted, false);

  // Annotations are appended with a monotonic ordinal and readable back.
  const annotation = await client.collaboration.addAnnotation("world-1", "looks linear", { target: "law", ref: "x" }, "idem-a1");
  assert.equal(annotation.ordinal, 1);
  assert.equal(annotation.target, "law");
  assert.equal(annotation.ref, "x");
  assert.equal((await client.collaboration.listAnnotations("world-1")).length, 1);

  // Review state machine: draft -> in_review -> approved; the world becomes trusted.
  const inReview = await client.collaboration.reviewRevision("world-1", 1, "in_review", "idem-r1");
  assert.equal(inReview.review_state, "in_review");
  const approved = await client.collaboration.reviewRevision("world-1", 1, "approved", "idem-r2");
  assert.equal(approved.review_state, "approved");
  assert.equal(approved.trusted, true);
  assert.equal((await client.collaboration.getRevision("world-1", 1)).trusted, true);
  assert.equal((await client.collaboration.listRevisions("world-1")).trusted, true);

  // Approved is terminal: an illegal transition is a 409 conflict.
  await assert.rejects(
    () => client.collaboration.reviewRevision("world-1", 1, "in_review", "idem-r3"),
    (error: unknown) => error instanceof ApiError && error.status === 409,
  );

  // Deterministic merge: disjoint names union (sorted); same name + differing
  // content hash surfaces as a conflict rather than overwriting.
  const merge = await client.collaboration.merge(
    "project-1",
    [{ name: "decay", content_hash: "h1", revision: 1 }, { name: "osc", content_hash: "h2", revision: 2 }],
    [{ name: "decay", content_hash: "hX", revision: 1 }, { name: "growth", content_hash: "h3", revision: 1 }],
    "idem-mg",
  );
  assert.equal(merge.conflict_count, 1);
  assert.equal(merge.conflicts[0]?.name, "decay");
  assert.deepEqual(merge.merged.map((row) => row.name), ["growth", "osc"]);

  // Refusing to orphan a project: removing the last owner is a 409.
  await assert.rejects(
    () => client.collaboration.removeMember("project-1", "token:owner", "idem-d1"),
    (error: unknown) => error instanceof ApiError && error.status === 409,
  );
  await client.collaboration.removeMember("project-1", "token:viewer", "idem-d2");
  assert.equal((await client.collaboration.listMembers("project-1")).some((m) => m.principal === "token:viewer"), false);
});

test("collaboration role gate refuses a viewer's writes offline", async () => {
  const viewer = new LawSynthClient(new InMemoryCollaborationTransport({ projectId: "p", worldId: "w", actorRole: "viewer" }));
  // A viewer may read revisions/annotations...
  assert.equal((await viewer.collaboration.listRevisions("w")).items.length, 1);
  // ...but may not annotate, review, or manage membership.
  await assert.rejects(() => viewer.collaboration.addAnnotation("w", "nope", {}, "v1"), (e: unknown) => e instanceof ApiError && e.status === 403);
  await assert.rejects(() => viewer.collaboration.reviewRevision("w", 1, "in_review", "v2"), (e: unknown) => e instanceof ApiError && e.status === 403);
  await assert.rejects(() => viewer.collaboration.addMember("p", "x", "editor", "v3"), (e: unknown) => e instanceof ApiError && e.status === 403);

  // Only an owner may record `approved`, even for an editor.
  const editor = new LawSynthClient(new InMemoryCollaborationTransport({ projectId: "p", worldId: "w", actorRole: "editor" }));
  await editor.collaboration.reviewRevision("w", 1, "in_review", "e1");
  await assert.rejects(() => editor.collaboration.reviewRevision("w", 1, "approved", "e2"), (e: unknown) => e instanceof ApiError && e.status === 403);
});
