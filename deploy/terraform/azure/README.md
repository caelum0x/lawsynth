# LawSynth on Azure (Terraform)

Provisions the Azure infrastructure for a **distributed** LawSynth deployment,
sized to be driven by the `deploy/helm/lawsynth` chart.

## What it creates

| File | Resources |
|---|---|
| `network.tf` | VNet, AKS subnet, delegated Postgres subnet, private DNS zone |
| `cluster.tf` | AKS cluster (Workload Identity, Cilium) + autoscaling worker node pool |
| `database.tf` | PostgreSQL Flexible Server (private VNet), database, TLS enforcement |
| `storage.tf` | Storage account + private blob container for `.lsworld` artifacts |

NATS (the job bus) runs in-cluster and is not provisioned here.

## Usage

```bash
cd deploy/terraform/azure
az login
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

- **Private database.** PostgreSQL Flexible Server is deployed with VNet
  integration into a delegated subnet and a private DNS zone; there is no public
  endpoint. `require_secure_transport` is enforced.
- **Secrets.** The database administrator password is generated with
  `random_password`. Wire it into Azure Key Vault before production and treat
  Terraform state as sensitive.
- **Artifacts.** The storage account uses ZRS replication with blob versioning
  and soft-delete retention to support the "checksum integrity detected on read"
  reliability goal. Public access is disabled.
- **Resource protection.** The provider is configured with
  `prevent_deletion_if_contains_resources`. For a disposable environment, relax
  it and lower the database SKU / disable HA.

## Requirements

- Terraform >= 1.5
- azurerm provider ~> 3.110
- An Azure subscription with quota for AKS, PostgreSQL Flexible Server, and
  Storage, plus permission to create resource groups and role assignments
