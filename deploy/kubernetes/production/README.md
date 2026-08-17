# LawSynth Kubernetes production overlay

Kustomize overlay that deploys the [base](../base) into the `lawsynth`
namespace as the production distributed service layer. Adds autoscaling,
hardened resource envelopes, PodDisruptionBudgets, deny-by-default networking, a
production TLS Ingress, and a metadata backup CronJob sized to the reliability
SLOs in architecture section 23.

## Contents

| File | Purpose |
|---|---|
| `kustomization.yaml` | Pins release image tags and wires resources/patches |
| `config.yaml` | Production `lawsynth-config` overrides (telemetry off) |
| `replicas.yaml` | HorizontalPodAutoscalers for api/gateway/worker/artifact |
| `resources.yaml` | Production CPU/memory envelopes |
| `disruption-budget.yaml` | PDBs for control and storage paths |
| `ingress.yaml` | Production TLS Ingress to the gateway |
| `network-policy.yaml` | Deny-by-default + explicit call-graph allows |
| `backup-cronjob.yaml` | 15-minute-RPO metadata database backup |
| `secrets.example.yaml` | Template for `lawsynth-secrets` (never applied) |

## Operational decisions

- **Autoscaling, not fixed replicas.** HPAs own replica counts for the
  stateless services and worker pool. The scheduler stays a singleton (`base`
  `replicas: 1`, `Recreate`) so jobs are never double-leased.
- **Disruption safety.** API and gateway keep `minAvailable: 2`; the worker pool
  is `maxUnavailable: 50%` because lost workers return jobs to the schedulable
  state after lease expiry. The scheduler has no PDB by design.
- **Backups.** `metadata-backup` dumps the metadata DB every 15 minutes to the
  object store. Restore is exercised through the documented runbook to meet the
  <2h RTO. The metadata DB holds references only -- never dataset content.
- **Secrets are external.** `secrets.example.yaml` is a template; supply the real
  `lawsynth-secrets` via a secret manager. Job envelopes stay secret-free.

## Prerequisites

- Postgres, an S3-compatible object store, and NATS running in-cluster and
  labelled `app.kubernetes.io/part-of: lawsynth-data`.
- `ingress-nginx`, `cert-manager` (with `letsencrypt-production`), and a
  Prometheus stack in a `monitoring` namespace.

## Deploy

```sh
kubectl apply --dry-run=client -k deploy/kubernetes/production   # validate
kubectl apply -k deploy/kubernetes/production                    # apply
kubectl -n lawsynth rollout status deploy/api deploy/gateway deploy/worker deploy/artifact deploy/scheduler
```
