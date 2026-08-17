# LawSynth Helm Chart

Deploys the distributed LawSynth discovery engine onto Kubernetes: the API
gateway, scheduler, worker pool, and content-addressed artifact service.

> Kubernetes is the **distributed** deployment mode. For notebooks, a single
> developer, or a small lab, prefer the embedded library, Local Studio, or the
> single-node Docker Compose stack under `deploy/compose/`. Reach for this chart
> only once discovery jobs exceed one machine.

## Architecture

```
        Ingress ──▶ api (gateway)  ──┐
                        │            │ job envelopes (NATS)
                        ▼            ▼
                   Postgres     scheduler ──▶ worker pool
                        ▲            │             │
                        └── run events, artifact refs
                                     ▼             ▼
                                 artifact ──▶ object store (S3/MinIO)
```

- **api** — validates and persists `RunSpec`s, publishes job envelopes, streams
  SSE/WebSocket progress to Studio, serves the HTTP API.
- **scheduler** — assigns compatible worker pools, tracks leases; fully
  reconstructable from database state (single active replica by default).
- **worker** — leases jobs, runs discovery pipelines, uploads content-addressed
  `.lsworld` artifacts. CPU/memory/time quotas are enforced per job.
- **artifact** — serves and verifies content-addressed bundles; finalizes
  uploads only after checksum verification.

## Dependencies (provided externally)

Postgres, an S3-compatible object store, and NATS are **not** deployed by this
chart. Provision them with a managed offering or the Terraform modules under
`deploy/terraform/`, then point the chart at them via `externalServices`.

## Install

```bash
helm install lawsynth deploy/helm/lawsynth \
  --namespace lawsynth --create-namespace \
  --set externalServices.postgres.host=pg.internal \
  --set externalServices.postgres.password=$PGPASSWORD \
  --set externalServices.objectStore.endpoint=https://s3.internal \
  --set externalServices.objectStore.accessKey=$S3_KEY \
  --set externalServices.objectStore.secretKey=$S3_SECRET
```

For production, supply secrets via `existingSecret` references instead of
inline `--set`:

```bash
kubectl create secret generic lawsynth-db \
  --from-literal=LAWSYNTH_PG_USER=lawsynth \
  --from-literal=LAWSYNTH_PG_PASSWORD=... 
helm upgrade --install lawsynth deploy/helm/lawsynth \
  --set externalServices.postgres.existingSecret=lawsynth-db
```

## Key values

| Key | Default | Description |
|---|---|---|
| `global.imageRegistry` | `ghcr.io` | Registry for all service images |
| `global.imageNamespace` | `lawsynth` | Image org/project namespace |
| `api.replicaCount` | `2` | API gateway replicas |
| `scheduler.replicaCount` | `1` | Scheduler replicas (keep at 1 without lease election) |
| `worker.replicaCount` | `3` | Worker pool size |
| `worker.quotas.*` | see values | Per-job CPU/memory/time limits |
| `artifact.cache.enabled` | `true` | Local hot-artifact cache PVC |
| `migration.enabled` | `true` | Run schema migration as a pre-upgrade hook |
| `ingress.enabled` | `false` | Expose the API via Ingress |
| `networkPolicy.enabled` | `false` | Restrict pod-to-pod traffic |
| `podDisruptionBudget.enabled` | `false` | PDB for the API service |

See [`values.yaml`](./values.yaml) for the full, inline-documented list, and
[`values.schema.json`](./values.schema.json) for the validation schema.

## Reliability notes

- Schema migrations run as a `pre-install,pre-upgrade` hook (weight `-5`); a
  failed migration aborts the rollout before new app replicas start.
- The scheduler uses a `Recreate` strategy and coordination leases so at most
  one instance assigns jobs at a time.
- Workers use a 120s termination grace period; a worker lost mid-job returns the
  job to schedulable state once its lease expires.
- Dataset names, column values, and equations must not appear in telemetry —
  keep `observability.logLevel` at `info` or higher in production.

## Template layout

Because the repository manifest tracks templates as flat `templates-*.yaml`
files, this chart keeps them at the chart root rather than in a `templates/`
subdirectory:

| File | Contents |
|---|---|
| `templates-api.yaml` | Shared helpers, ConfigMap, Secret, api Deployment/Service/HPA |
| `templates-worker.yaml` | scheduler + worker Deployments, worker HPA |
| `templates-storage.yaml` | artifact Deployment/Service + cache PVC |
| `templates-rbac.yaml` | ServiceAccount, Role, RoleBinding |
| `templates-ingress.yaml` | Ingress, PodDisruptionBudget, NetworkPolicy |
| `templates-migration.yaml` | Schema migration Job (Helm hook) |

To render or lint with `helm`, move these into a `templates/` directory:

```bash
mkdir -p templates && for f in templates-*.yaml; do mv "$f" "templates/${f#templates-}"; done
helm lint . && helm template . | kubectl apply --dry-run=client -f -
```
