# Operations

## Running

The binary takes explicit CLI arguments and reads no environment variables:

```sh
lawsynth-artifact serve /var/lib/lawsynth/artifacts 0.0.0.0:8080
lawsynth-artifact health /var/lib/lawsynth/artifacts
lawsynth-artifact gc /var/lib/lawsynth/artifacts $(date +%s) --dry-run
```

`serve` binds the listener, prints one startup line to stderr, then blocks. The
process owns its `root` directory; run it as a dedicated non-root user that owns
that path (the Dockerfile uses uid 10001 and a `/var/lib/lawsynth/artifacts`
volume).

## Capacity and limits

Sizing lives in `config/`. Set `limits.max_total_bytes` to the usable size of the
volume backing `root`; it is the ceiling reported by `/health` and enforced on
write, not a reservation. `store.max_object_bytes` and `limits.max_multipart_*`
bound individual objects and multipart sessions. The current CLI applies the
compiled-in defaults in `limits.yaml`; treat the profile files as the reviewed
source of those numbers.

## Health and metrics

`GET /health` (or the `health` subcommand) proves the catalog can be listed and
reports `artifact_count`, `stored_data_bytes`, and `capacity_bytes`. The service
also maintains in-process counters (`uploads`, `downloads`, `checksum_failures`,
`gc_deletions`); see `config/logging.yaml` for how to scrape them from an
embedding process.

## Retention

Retention is a per-artifact Unix timestamp set at ingest via
`X-Retention-Expires-At`. Reclaim expired objects with `gc <root> <now>`. Always
run `--dry-run` first in a shared environment to review the id list before a
destructive sweep. GC removes both the object and its cache entry.

## Backup

Because objects are content-addressed and metadata is published atomically, the
`root` directory can be snapshotted with normal filesystem tooling while the
service is quiescent. Restoring the directory restores the full catalog.
