# LawSynth on AWS (Terraform)

Provisions the AWS infrastructure for a **distributed** LawSynth deployment,
sized to be driven by the `deploy/helm/lawsynth` chart.

## What it creates

| File | Resources |
|---|---|
| `network.tf` | VPC, public/private subnets, NAT, IGW (via `terraform-aws-modules/vpc`) |
| `cluster.tf` | EKS control plane + managed worker node group |
| `database.tf` | RDS Postgres (metadata), security group, Secrets Manager credentials |
| `storage.tf` | S3 bucket for content-addressed `.lsworld` artifacts (versioned, KMS-encrypted) |

NATS (the job bus) runs in-cluster and is not provisioned here.

## Usage

```bash
cd deploy/terraform/aws
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

- **State backend.** `versions.tf` includes a commented S3 + DynamoDB backend.
  Configure it (`terraform init -backend-config=...`) before team use.
- **Secrets.** The database master password is generated with `random_password`
  and stored in AWS Secrets Manager (`<name>/postgres`). Nothing sensitive is
  written to the Terraform code; state should still be treated as sensitive.
- **Cost.** Defaults favor availability (Multi-AZ optional, autoscaling node
  group). For a lab, set `single_nat_gateway = true`, `db_multi_az = false`, and
  a smaller `db_instance_class`.
- **Deletion protection** is on by default for the database. Set
  `db_deletion_protection = false` and `artifact_force_destroy = true` only in
  disposable environments.

## Requirements

- Terraform >= 1.5
- AWS provider ~> 5.40
- Credentials with permissions for VPC, EKS, RDS, S3, IAM, and Secrets Manager
