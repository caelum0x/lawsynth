# LawSynth Kubernetes base

Environment-neutral Kustomize base for the LawSynth distributed service layer.
It renders the workloads described in the production architecture (sections 7
and 10): a request-admission gateway in front of a stateless HTTP API, a single
active scheduler, a horizontally scalable worker pool, and a content-addressed
artifact service, all backed by Postgres, an object store, and a NATS event
bus that live outside this base.

## Contents

| File | Purpose |
|---|---|
| `namespace.yaml` | `lawsynth` namespace pinned to the restricted Pod Security profile |
| `configmap.yaml` | Non-secret shared configuration (`lawsynth-config`) |
| `rbac.yaml` | One ServiceAccount per service + read-only config Role |
| `api.yaml` | HTTP API Deployment + Service (`:8080`) with pre-start migration init container |
| `gateway.yaml` | Request-admission gateway Deployment + Service (`:8081`) |
| `scheduler.yaml` | Single active scheduler Deployment + Service (`:8083`) |
| `worker.yaml` | Scalable worker Deployment + headless metrics Service |
| `artifact.yaml` | Artifact store Deployment + Service (`:8082`) |
| `kustomization.yaml` | Aggregates resources, common labels, and image tags |

## Design decisions

- **Not directly deployable.** The base has no Secret, Ingress, HPA, or
  NetworkPolicy. Apply an overlay (`staging/`, `production/`) that supplies the
  `lawsynth-secrets` Secret and edge configuration.
- **Security posture.** Every Pod runs non-root (uid/gid 65532), with
  `readOnlyRootFilesystem`, all Linux capabilities dropped,
  `allowPrivilegeEscalation: false`, and the runtime-default seccomp profile.
  ServiceAccount tokens are not automounted; services coordinate through
  Postgres/NATS, never the Kubernetes API.
- **Scheduler singleton.** `replicas: 1` with a `Recreate` strategy prevents two
  schedulers from leasing the same job. The scheduler is reconstructable from
  the database, so this is availability-safe.
- **Ephemeral storage in base.** API objects, worker scratch, and artifact
  cache use `emptyDir`. Overlays must back durable paths with real volumes or
  route object writes through the artifact service.

## Rendering

```sh
kubectl kustomize deploy/kubernetes/base
# validate without a cluster:
kubectl apply --dry-run=client -k deploy/kubernetes/base
```

Do not `kubectl apply -k deploy/kubernetes/base` against a real cluster; use an
overlay.
