# Collaboration boundary (P6)

This directory specifies shared, multi-user workspaces. It is a **boundary
specification**: it states what a conforming implementation MUST do, not that one
is built. The local single-user surfaces it extends (`Project`, CLI `workspace`,
service `/v1/projects`) already exist and remain fully functional offline.

## Membership & roles

A shared project MUST bind every actor to exactly one role per project:

- **owner** — full control, including membership and deletion.
- **editor** — add/update/remove worlds, submit runs, annotate, request approval.
- **viewer** — read worlds/reports/annotations only.

Authorization MUST be evaluated server-side against the actor's role for the
specific project (identifiers are opaque references, never grants — see
`specs/service-api/authorization.md`). A viewer MUST NOT mutate any resource.

## Revision lineage

Every world stored in a shared project MUST carry an immutable revision record:

- the content hash of the `.lsworld` bytes;
- the **derivation**: `discovered` (dataset hash + discovery config), `edited`
  (parent revision + edit ops), `composed` (parent revision hashes + namespacing),
  or `imported` (source archive hash);
- the actor and a monotonically increasing revision number within the world.

A revision is append-only. Editing/composing a world produces a NEW revision
referencing its parents; the prior revision MUST remain retrievable.

## Annotations & review

An implementation MUST support annotations attached to a world, a specific law,
or a revision, each carrying an actor, an ordinal, and bounded UTF-8 text.
Approval is a state on a revision (`draft → in_review → approved | rejected`);
only an owner may record `approved`. A world marked `trusted` MUST reference an
`approved` revision. Approval transitions MUST emit an audit event
(see `specs/model-governance/`).

## Workspace merge

Two workspace indexes (the `library.tsv` provenance format) MUST merge
deterministically: by world name, then by revision lineage. A merge MUST NOT
silently overwrite a name whose content hash differs; such a case is a
**conflict** the implementation MUST surface (with both revisions) rather than
resolve arbitrarily. Merge MUST be associative and commutative over disjoint
names so that offline replicas converge.

## Offline guarantee

Collaboration is additive. A conforming implementation MUST allow a single user
to operate a project fully offline (the local `Project`/`workspace` behavior),
with sharing/roles/review activating only when a collaboration backend is
configured. No collaboration feature may make the local core require a network.
