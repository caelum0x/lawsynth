# Model governance boundary (P9)

This directory specifies lineage, model cards, approval, and audit — what makes a
discovered model auditable and accountable. It is a **boundary specification**
extending the provenance already captured by `library`/`runs`, the trust surfaces
(`validate`, `backtest`, `discover_ensemble`), and the event bus.

## Lineage record

A governed model MUST carry a queryable, exportable lineage chain:

```
dataset(hash, columns) → preparation(ops) → discovery(config, engine version)
  → world(revision hash) → evaluation(validate/backtest/ensemble) → report(hash)
  → decision(actor, action, timestamp-free ordinal)
```

Every link is content-addressed and immutable. Given a world revision, an
implementation MUST reconstruct the full chain back to the source dataset hash.
Lineage references collaboration revisions (`specs/collaboration/`).

## Model card

A model card is a standardized, self-contained document (renderable as the
existing HTML report) that MUST include: the recovered law system; the
assumptions it is contingent on (continuity, feature library, causal caveats);
fit quality; **out-of-sample** skill (holdout `validate` and rolling-origin
`backtest`); ensemble term-stability (robust vs unstable terms); and an explicit
"known limitations / not validated" section (e.g. extrapolation beyond the
observed window). A model card MUST NOT overstate confidence; unmeasured fields
are marked absent, never fabricated.

## Approval states

A world revision moves through `draft → in_review → approved | rejected`
(aligned with `specs/collaboration/`). A `trusted` designation MUST reference an
`approved` revision and a model card. Only an authorized approver (owner role)
may record `approved`; the action MUST be attributable.

## Audit log

Every governance-relevant action (submission, evaluation, approval, edit,
export, share) MUST append an immutable audit event to the event log
(`specs/service-api/streaming.md` semantics: strictly increasing sequence, scope
preserved). The audit log MUST be append-only and tamper-evident (each event
carries the prior event's digest); a consumer MUST be able to detect a gap or
alteration.

## Determinism

Because discovery, evaluation, and reporting are deterministic and offline, a
lineage chain MUST be independently reproducible: re-running the recorded
dataset + config MUST reproduce the same world hash. A governance implementation
MUST expose this reproducibility check.
