# LawSynth operations runbook

Response procedures for the alerts defined in [`alerts.yaml`](./alerts.yaml).
Each section is linked from the alert's `runbook` annotation. Alerts map to the
SLOs and failure modes in the production architecture (section 23).

General principle: **stabilize first, root-cause second.** Protect accepted-run
durability and artifact integrity over raw throughput.

Quick references:

```bash
# Compose (single-node reference):
docker compose -f deploy/compose/production/compose.yaml ps
docker compose -f deploy/compose/production/compose.yaml logs -f <service>

# Health:
curl -fsS https://$LAWSYNTH_DOMAIN/v1/health
```

---

## api-unavailable
**Alert:** `LawSynthApiProbeDown` — the external `/v1/health` probe fails.

1. Confirm scope: is it the probe/network or the service?
   `curl -v https://$LAWSYNTH_DOMAIN/v1/health` and from inside the network
   `docker compose exec gateway wget -qO- http://api:8080/v1/health`.
2. If the proxy is down, check certificate/ACME issues in the `proxy` logs.
3. If the API is down, check `api` logs for crash loops and its dependencies
   (Postgres, object store, NATS) — a hard dependency failure surfaces here.
4. Restart the affected service; scale API replicas if it is resource-bound.
5. Verify recovery: probe returns 200 and error ratio falls.

## api-error-ratio
**Alerts:** `LawSynthApiHighErrorRatioFast` / `...Slow`.

1. Break down `lawsynth_api_requests_total` by route and status to find the
   failing endpoint.
2. Correlate with a recent deploy (`LAWSYNTH_VERSION`), a dependency alert, or a
   saturation alert.
3. If a deploy regressed, roll back to the previous pinned tag.
4. If a dependency is degraded, follow its section below.
5. Fast-burn is budget-critical: mitigate within minutes even if root cause is
   still open.

## api-latency
**Alert:** `LawSynthApiLatencyHigh` (p95 > 1s).

1. Check Postgres saturation and slow-query logs
   (`log_min_duration_statement=1000` is enabled).
2. Check object-store latency for artifact reads.
3. Look for GC/CPU throttling on the API containers; raise CPU limits or
   replicas.

## run-persistence
**Alert:** `LawSynthAcceptedRunPersistenceLoss` — SLO-critical (99.99%).

1. This means the API acknowledged a run but the outbox/transaction did not
   commit. Inspect API logs for transaction/outbox errors around the spike.
2. Verify Postgres write health and disk space.
3. Reconcile: the scheduler is reconstructable from DB state — confirm no
   accepted run is missing an outbox row; requeue if a durable record exists.
4. If data was lost, restore from the most recent backup set and replay.
5. Open an incident; this alert must never be silenced without root cause.

## queue-delay
**Alert:** `LawSynthQueueDelayHigh` (p95 wait > 5m).

1. Check worker replica count and `lawsynth_job_lease_age_seconds`.
2. Scale workers: `docker compose up -d --scale worker=N`.
3. Confirm the scheduler is running (exactly one) and draining the outbox.
4. Inspect for a poison job repeatedly failing and blocking a pool.

## stale-lease
**Alert:** `LawSynthStaleJobLease`.

1. A worker likely died holding a lease. Confirm with `worker` logs / restarts.
2. The scheduler returns the job to schedulable state after lease expiry —
   verify this happens; if not, check scheduler recovery logic.
3. Checkpoint-compatible jobs resume; others restart explicitly (expected).

## cancellation
**Alert:** `LawSynthCancelAckSlow` (p95 > 2s).

1. Cancellation is cooperative plus hard timeout. Check whether workers are
   blocked in a long native call ignoring the cancel signal.
2. Verify the hard-timeout path fires and releases resources.

## artifact-checksum
**Alert:** `LawSynthArtifactChecksumFailure` — SLO-critical (100% detection).

1. Detection working as designed: an artifact is corrupt or tampered.
2. Quarantine the object; do NOT serve it. Identify referencing runs/worlds.
3. Restore the object from the latest verified backup; re-verify on read.
4. If corruption is on the object-store volume, run a storage integrity check
   and consider replacing the disk.

## event-ordering
**Alert:** `LawSynthEventOrderingViolation`.

1. Per-run-attempt event sequence must be monotonic. Check for duplicate
   producers or a NATS redelivery not handled idempotently.
2. Confirm consumers order by `sequence`, not arrival time.
3. Inspect the run's `run_events` for the offending attempt.

## backup-rpo
**Alert:** `LawSynthBackupRpoBreached` (no success > 20m; RPO 15m).

1. Check the backup schedule (cron/systemd timer) and `backup.sh` logs.
2. Common causes: disk full at `BACKUP_ROOT`, DB unreachable, object mirror
   failing. Fix and run `./backup.sh` manually.
3. Confirm a fresh set with `sha256sum -c checksums.sha256`.

## db-saturation
**Alert:** `LawSynthPostgresConnectionsSaturated` (> 90%).

1. Identify connection sources; check for leaks or an over-provisioned pool.
2. Raise `POSTGRES_MAX_CONNECTIONS` or lower per-service pool sizes.
3. Watch for latency knock-on effects (see api-latency).

## object-capacity
**Alert:** `LawSynthObjectStoreCapacityLow` (< 10% free).

1. Uploads will soon fail. Expand the object-store volume.
2. Run artifact garbage collection (mark live references, wait the safety
   window, sweep) to reclaim unreferenced objects.

## scientific-warnings
**Alert:** `LawSynthUnstableCandidateRateHigh` (informational).

1. Not an outage. Elevated unstable-candidate or unsupported-extrapolation
   warnings suggest dataset quality or method-setting issues.
2. Review recent datasets/plans; feed back to the science team. No paging.
