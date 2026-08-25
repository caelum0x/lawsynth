# Hosted platform & scale boundary (P10)

> Archived direction, 23 August 2026: this specification is retained as design
> history and is not an active managed-service plan. LawSynth is distributed for
> local and user-operated self-hosting; `lawsynth.dev` is a static website.

This directory specifies a managed, multi-tenant LawSynth and distributed
discovery for large data. It is a **boundary specification** extending the
discovery-as-a-service backend (gateway/scheduler/worker/artifact/api), the
deployment scaffolding (compose/k8s/terraform), and the ingest performance work.

## Tenancy isolation

A hosted deployment MUST isolate tenants completely. Every resource (project,
dataset, world, run, artifact, event) is owned by exactly one tenant; a request
MUST be authorized against the caller's tenant server-side (identifiers are never
grants). No query, event stream, or artifact download may cross a tenant
boundary. Storage MUST be partitioned or scoped so a tenant cannot enumerate
another's content.

## Authentication (SSO)

The gateway (the public entry; the API is never internet-exposed) MUST accept a
documented SSO/OIDC flow, exchange it for a tenant-scoped principal, and pass an
authenticated principal to the API. Bearer tokens remain the machine surface. The
gateway MUST reject unauthenticated or cross-tenant requests before they reach
the backend (`specs/service-api/authentication.md`).

## Quota, rate & metering

Per-tenant policy MUST bound concurrent runs, queued jobs, dataset size, and
request rate (the scheduler already carries quota/fairness/priority modules; the
gateway carries rate limiting). Every billable action (run submitted, CPU-time
consumed, bytes stored) MUST be recorded to an immutable metering log suitable
for usage reporting. Exceeding quota MUST return a documented error, never
silently drop work.

## Distributed discovery

For datasets beyond a single worker's budget, discovery MAY be partitioned
(e.g. feature-library evaluation split across workers) provided the result is
**identical** to the single-node result for the same inputs and config. The
distributed path MUST preserve the engine's determinism guarantee: same inputs →
same world, independent of worker count or placement. If exact partitioning is
not achievable for a step, the implementation MUST fall back to single-node for
that step rather than return an approximate result unmarked.

## Reliability

The platform MUST define and honor the SLOs in `ARCHITECTURE.md`/§23: run
durability (a submitted run is never silently lost), artifact-checksum
integrity, event ordering, and cancel acknowledgement latency. Backups/DR
(scaffolded in compose/k8s) MUST meet a stated RPO. A worker or scheduler
failure MUST be recoverable from durable checkpoints without duplicating work
(the scheduler's lease fencing already provides this in-process).

## Local-first guarantee

Hosting is opt-in. The single-node, offline, deterministic product (CLI, SDK,
local Studio, self-hosted compose) MUST remain fully functional and is the
reference for correctness; the hosted platform MUST NOT introduce behavior the
local engine cannot reproduce.
