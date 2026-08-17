# S3 bucket for content-addressed .lsworld artifacts.
# Versioning + checksum integrity supports the "100% detected on read" SLO.

resource "random_id" "bucket_suffix" {
  byte_length = 4
}

locals {
  artifact_bucket_name = coalesce(
    var.artifact_bucket_name,
    "${local.name}-artifacts-${random_id.bucket_suffix.hex}",
  )
}

module "artifact_bucket" {
  source  = "terraform-aws-modules/s3-bucket/aws"
  version = "~> 4.1"

  bucket        = local.artifact_bucket_name
  force_destroy = var.artifact_force_destroy

  # Artifacts are content-addressed and immutable; block all public access.
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true

  versioning = {
    status = "Enabled"
  }

  server_side_encryption_configuration = {
    rule = {
      apply_server_side_encryption_by_default = {
        sse_algorithm = "aws:kms"
      }
      bucket_key_enabled = true
    }
  }

  lifecycle_rule = [
    {
      id      = "expire-noncurrent"
      enabled = true
      noncurrent_version_expiration = {
        days = var.artifact_noncurrent_expiration_days
      }
      abort_incomplete_multipart_upload_days = 7
    },
  ]

  tags = local.tags
}
