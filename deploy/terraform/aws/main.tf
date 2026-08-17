# -----------------------------------------------------------------------------
# LawSynth distributed deployment on AWS.
#
# Provisions the infrastructure the deploy/helm/lawsynth chart depends on:
#   - VPC and subnets            (network.tf)
#   - EKS cluster and node group (cluster.tf)
#   - RDS Postgres metadata DB   (database.tf)
#   - S3 artifact object store   (storage.tf)
#
# NATS (job bus) is expected to run in-cluster or as a managed add-on and is
# not provisioned here.
# -----------------------------------------------------------------------------

provider "aws" {
  region = var.region

  default_tags {
    tags = local.tags
  }
}

locals {
  name = "${var.name_prefix}-${var.environment}"

  tags = merge(
    {
      "app.kubernetes.io/part-of" = "lawsynth"
      Project                     = "lawsynth"
      Environment                 = var.environment
      ManagedBy                   = "terraform"
    },
    var.tags,
  )
}

data "aws_availability_zones" "available" {
  state = "available"
}
