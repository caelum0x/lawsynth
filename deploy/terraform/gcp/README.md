# LawSynth on Google Cloud (Terraform)

Provisions the GCP infrastructure for a **distributed** LawSynth deployment,
sized to be driven by the `deploy/helm/lawsynth` chart.

## What it creates

| File | Resources |
|---|---|
| `network.tf` | VPC-native network, node subnet with pod/service secondary ranges, Cloud NAT, Private Services Access peering |
| `cluster.tf` | GKE cluster (Workload Identity) + autoscaling worker node pool |
| `database.tf` | Cloud SQL Postgres (private IP), database, user |
| `storage.tf` | GCS bucket for content-addressed `.lsworld` artifacts (versioned) |

NATS (the job bus) runs in-cluster and is not provisioned here.

## Usage

```bash
cd deploy/terraform/gcp
terraform init
terraform plan  -var-file=example.tfvars
terraform apply -var-file=example.tfvars

# Then wire kubectl to the new cluster:
$(terraform output -raw kubeconfig_command)
```

Feed the outputs into the Helm chart:

```bash
terraform output helm_values_hint
```

## Notes

- **Required APIs** are enabled by `main.tf` (`google_project_service`). The
  identity running Terraform needs `roles/owner` or an equivalent custom role.
- **Private database.** Cloud SQL is created with a private IP over Private
  Services Access, reachable from GKE nodes only. There is no public endpoint.
- **Secrets.** The database password is generated with `random_password` and set
  on the Cloud SQL user. Retrieve it from state or wire it into a Secret Manager
  resource before production; treat Terraform state as sensitive.
- **Deletion protection** is enabled on both the GKE cluster and Cloud SQL
  instance. To tear down a disposable environment, set
  `db_deletion_protection = false`, flip the cluster's `deletion_protection` in
  `cluster.tf`, and set `artifact_force_destroy = true`.

## Requirements

- Terraform >= 1.5
- Google provider ~> 5.30
- A GCP project with billing enabled and quota for GKE, Cloud SQL, and GCS
