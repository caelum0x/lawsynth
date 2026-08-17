# LawSynth Kubernetes staging overlay

Kustomize overlay that deploys the [base](../base) into the
`lawsynth-staging` namespace as a pre-production mirror of the distributed
service layer. Staging runs minimal replica counts on a shared node pool while
still exercising multi-replica behaviour, deny-by-default networking, and the
full alerting stack.

## Contents

| File | Purpose |
|---|---|
| `kustomization.yaml` | Sets namespace, `staging` image tags, and wires patches |
| `config.yaml` | Strategic-merge overrides onto `lawsynth-config` |
| `replicas.yaml` | Reduced replica counts (scheduler stays a singleton) |
| `resources.yaml` | Smaller CPU/memory envelopes |
| `ingress.yaml` | TLS Ingress routing to the gateway |
| `network-policy.yaml` | Deny-by-default + explicit call-graph allows |
| `alerts.yaml` | PrometheusRule mapping SLOs to warning alerts |
| `smoke-job.yaml` | Post-deploy health/version check |
| `secrets.example.yaml` | Template for `lawsynth-secrets` (never applied) |

## Deploy

```sh
# 1. Provision the lawsynth-secrets Secret via your secret manager first.
# 2. Render and validate:
kubectl apply --dry-run=client -k deploy/kubernetes/staging
# 3. Apply:
kubectl apply -k deploy/kubernetes/staging
# 4. Confirm the smoke Job succeeded:
kubectl -n lawsynth-staging wait --for=condition=complete job/lawsynth-smoke --timeout=300s
```

## Notes

- **Secrets are external.** `secrets.example.yaml` is a template only; it is not
  in the kustomization `resources` list. Deliver the real Secret through
  External Secrets, Sealed Secrets, or Vault.
- **Backing services** (Postgres, object store, NATS) are expected to already
  exist in-cluster, labelled `app.kubernetes.io/part-of: lawsynth-data` so the
  NetworkPolicies permit egress to them.
- **Image channel.** Staging tracks the `staging` tag; promotion to production
  is an explicit tag change in the production overlay.
