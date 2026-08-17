# GCS bucket for content-addressed .lsworld artifacts.

resource "random_id" "bucket_suffix" {
  byte_length = 4
}

locals {
  artifact_bucket_name = coalesce(
    var.artifact_bucket_name,
    "${local.name}-artifacts-${random_id.bucket_suffix.hex}",
  )
}

resource "google_storage_bucket" "artifacts" {
  name          = local.artifact_bucket_name
  location      = var.region
  storage_class = "STANDARD"
  force_destroy = var.artifact_force_destroy

  # Content-addressed and immutable; no public access, uniform IAM only.
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"

  versioning {
    enabled = true
  }

  lifecycle_rule {
    condition {
      num_newer_versions         = 3
      days_since_noncurrent_time = var.artifact_noncurrent_age_days
    }
    action {
      type = "Delete"
    }
  }

  lifecycle_rule {
    condition {
      age = 7
    }
    action {
      type = "AbortIncompleteMultipartUpload"
    }
  }

  labels = local.labels

  depends_on = [google_project_service.services]
}
