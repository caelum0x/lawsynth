# Streaming & online discovery boundary (P7)

This directory specifies discovering and maintaining models on continuously
arriving data. It is a **boundary specification**. It builds on the streaming,
bounded-memory `Read` loaders already in `lawsynth-data` and the `monitor`
drift-detection surface; it does not assert an online engine is built.

## Windowing

An online discovery MUST operate over an explicit, documented window policy:
either a fixed-size sliding window or a growing window with a stated cap. The
window is defined in the time column's units, not wall-clock. Ingestion MUST be
bounded-memory: peak memory tracks the window and the resulting columns, never
the full stream.

## Determinism under replay

Replaying the identical byte stream through the identical window/config MUST
produce the identical sequence of models — same worlds, same change records,
byte-for-byte. No wall clock, no ambient randomness: any resampling/bootstrap
MUST be seeded from content. This is the streaming analog of the batch engine's
reproducibility guarantee.

## Update triggers

A conforming implementation MUST distinguish an **outlier** (a transient
excursion, handled by `monitor`) from a **regime/law change** (a sustained shift
in the governing dynamics). Re-discovery is triggered only by the latter, under
a documented rule (e.g. sustained standardized residual drift over K windows).
The trigger rule MUST be explicit and deterministic.

## Change records

Every model update MUST emit an immutable **change record** documenting the
transition: the prior world revision, the new world revision, the window that
triggered it, and a per-law diff (which terms/coefficients changed). A consumer
reading the change-record stream MUST be able to reconstruct the full model
history. Change records reference revisions per `specs/collaboration/` lineage.

## Service surface

A streaming run is a long-lived service run. It MUST emit `Progress` events and a
`ModelUpdated` event (carrying the change record's revision ids) over the
existing event stream (`specs/service-api/streaming.md`), preserving strictly
increasing sequence and project scope. It MUST expose the current model and the
change-record history for retrieval. Cancellation MUST cleanly stop ingestion and
finalize the last model.

## Honesty

If an implementation cannot yet perform true incremental discovery and instead
re-discovers over each window from scratch, it MUST document that (it is still
conforming, but the "online efficiency" claim is not made). It MUST NOT present a
batched re-run as incremental learning.
