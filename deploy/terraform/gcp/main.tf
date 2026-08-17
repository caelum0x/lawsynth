# -----------------------------------------------------------------------------
# LawSynth distributed deployment on Google Cloud.
#
#   - VPC and subnet (VPC-native) (network.tf)
#   - GKE cluster + node pool      (cluster.tf)
#   - Cloud SQL Postgres           (database.tf)
#   - GCS artifact bucket          (storage.tf)
#
# NATS (job bus) runs in-cluster and is not provisioned here.
# -----------------------------------------------------------------------------

provider "google" {
  project = var.project_id
  region  = var.region
}

locals {
  name = "${var.name_prefix}-${var.environment}"

  labels = merge(
    {
      part-of     = "lawsynth"
      project     = "lawsynth"
      environment = var.environment
      managed-by  = "terraform"
    },
    var.labels,
  )
}

# Enable the APIs this stack depends on. Safe to keep enabled.
resource "google_project_service" "services" {
  for_each = toset([
    "compute.googleapis.com",
    "container.googleapis.com",
    "sqladmin.googleapis.com",
    "servicenetworking.googleapis.com",
    "storage.googleapis.com",
  ])

  service            = each.value
  disable_on_destroy = false
}
