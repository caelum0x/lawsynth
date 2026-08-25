# LawSynth: Next Phases (P6–P10)

> Direction update, 23 August 2026: LawSynth remains an open-source local and
> self-hosted project. The P10 managed-platform section is retained as historical
> boundary research, not an active hosted-service plan. `lawsynth.dev` is a
> static project website.

The original blueprint (`LawSynth_Production_Architecture.md`, `LawSynth_3161_Repository_Manifest.md`)
defined P0–P5: the engine, bindings, discovery depth, Studio, services, and
deployment. Those are built. The product loop —
`observe → prepare → discover → understand → use → validate → monitor → share → organize`
— ships across CLI, SDK, Studio, and services.

This document plans what comes next as a **product and company**: the phases that
turn a complete single-user tool into a collaborative, extensible, governed
platform. Each phase names a goal, the user problem it solves, the shipped
surfaces it extends, and the new spec(s) that bound it. New specs are written as
**boundary specifications** in the house style of `specs/` — they state what a
conforming implementation MUST do, not that it exists yet.

## Principles carried forward

- **Interpretable first** — every new capability must keep results readable and
  reproducible. No black boxes, no hidden state.
- **Local-first, opt-in cloud** — collaboration and hosting are additive; the
  local, offline, deterministic core never regresses.
- **Honest boundaries** — a phase ships only what its spec's conformance suite
  can verify; capability gaps are documented, never faked.

---

## P6 — Collaboration & workspaces

**Goal:** turn `.lsworkspace` + `Project` from a single-user file into a shared,
multi-user space where a team discovers, reviews, and reuses models together.

**User problem:** "My colleague and I both work on the same systems; today we
email `.lsworld` files. We need one shared, versioned, permissioned space."

**Extends:** SDK `Project`, CLI `workspace`, services `/v1/projects`, Studio
workspace.

**New capabilities:**
- Shared project membership with roles (owner / editor / viewer).
- World *revisions* with lineage (which data + config produced this world, and
  what it was derived/edited/composed from).
- Annotations & review: comment on a world/law, request/grant approval before a
  world is marked "trusted".
- Deterministic 3-way merge of workspace indexes (names/tags/provenance).

**Spec:** [`specs/collaboration/`](../../specs/collaboration/) — membership,
roles, revision lineage, annotation model, and the merge contract.

## P7 — Streaming & online discovery

**Goal:** discover and *maintain* models on data that arrives continuously,
instead of one-shot batch discovery.

**User problem:** "My sensor emits forever. I want a model that updates as new
data lands and tells me when the system's law has *changed* (not just an
outlier)."

**Extends:** `monitor` (drift → re-discovery), `discover`, services runs.

**New capabilities:**
- Windowed/incremental ingestion (builds on the streaming `Read` loaders already
  in `lawsynth-data`) with bounded memory.
- Online re-discovery triggered by sustained drift, with a *change record*
  documenting the old law → new law transition.
- A stream run type in the service: a long-lived run emitting `Progress` +
  `ModelUpdated` events over SSE.

**Spec:** [`specs/streaming-discovery/`](../../specs/streaming-discovery/) —
windowing, update triggers, determinism under replay, and the change-record
contract.

## P8 — Extensibility & a plugin marketplace

**Goal:** let the community add connectors, feature libraries, operators,
exporters — and distribute them safely.

**User problem:** "LawSynth doesn't read my database / my domain needs a custom
feature library. I want to add one and share it, and trust what I install."

**Extends:** the existing `lawsynth-plugin-api`/`-host`, the 10 example plugins,
connectors, export targets.

**New capabilities:**
- A signed, versioned plugin package format + a local index (`lawsynth plugin
  install/list/verify`).
- Capability-scoped, sandboxed execution (already an honest seam in the worker)
  hardened into a real permission grant model.
- A registry contract (offline-first: a directory/index that can be mirrored),
  not a mandatory hosted store.

**Spec:** [`specs/plugin-marketplace/`](../../specs/plugin-marketplace/) —
package format, signing/trust, capability grants, and registry mirroring.

## P9 — Governance, lineage & trust

**Goal:** make a discovered model auditable and accountable — required for
science, finance, and regulated use.

**User problem:** "Before I act on this model I must show exactly what data and
settings produced it, who approved it, and how confident we are."

**Extends:** provenance in `library`/`runs`, `validate`/`backtest`/ensemble,
reports.

**New capabilities:**
- End-to-end lineage: dataset hash → discovery config → world → report →
  decision, as a queryable, exportable record.
- Model cards: a standardized report bundling assumptions, fit, backtest skill,
  ensemble stability, and known limitations.
- Approval workflow + immutable audit log (the event bus already exists).

**Spec:** [`specs/model-governance/`](../../specs/model-governance/) — lineage
record, model-card schema, approval states, and audit-log immutability.

## P10 — Hosted platform & scale

**Goal:** a managed LawSynth that teams can adopt without operating it — and that
scales discovery across large data and many tenants.

**User problem:** "I don't want to run Postgres and a worker fleet. Give me a
hosted endpoint with SSO, and make discovery fast on big data."

**Extends:** the discovery-as-a-service backend, gateway/scheduler/worker,
deployment (compose/k8s/terraform), performance work.

**New capabilities:**
- Multi-tenant hosting with SSO/OIDC at the gateway, quota + rate policy per
  tenant (the scheduler already has quota/fairness modules).
- Distributed discovery for large datasets (partitioned feature evaluation),
  building on the 3.19× ingest work and the worker fleet.
- Usage metering + billing hooks; managed backups/DR (compose/k8s already
  scaffold these).

**Spec:** [`specs/hosted-platform/`](../../specs/hosted-platform/) — tenancy
isolation, SSO contract, quota/metering, and the distributed-discovery
determinism guarantee.

---

## Sequencing & goals

| Phase | Headline goal | Primary surface | Depends on |
|---|---|---|---|
| P6 | Teams share & review models | services + SDK/CLI + Studio | provenance, `Project` |
| P7 | Models that update on live data | monitor + services | streaming loaders, monitor |
| P8 | Safe community extensions | plugin host | plugin API, connectors |
| P9 | Auditable, accountable models | reports + services | provenance, validate/backtest |
| P10 | Managed, multi-tenant, at scale | services + deploy | P6/P9 + perf + fleet |

**Definition of done for each phase:** a boundary spec with a conformance suite,
the capability shipped on at least the CLI/SDK and the service, docs + a cookbook
recipe, and honest notes on anything the conformance suite cannot yet verify.

See the per-phase specs under [`specs/`](../../specs/) for the binding contracts.
