output "project_id" {
  description = "GCP project ID."
  value       = var.project_id
}

output "region" {
  description = "GCP region of the deployment."
  value       = var.region
}

output "cluster_name" {
  description = "GKE cluster name."
  value       = google_container_cluster.cluster.name
}

output "cluster_endpoint" {
  description = "GKE API server endpoint."
  value       = google_container_cluster.cluster.endpoint
  sensitive   = true
}

output "kubeconfig_command" {
  description = "Command to configure kubectl for this cluster."
  value       = "gcloud container clusters get-credentials ${google_container_cluster.cluster.name} --region ${var.region} --project ${var.project_id}"
}

output "database_instance" {
  description = "Cloud SQL instance name."
  value       = google_sql_database_instance.postgres.name
}

output "database_private_ip" {
  description = "Private IP of the Cloud SQL Postgres instance."
  value       = google_sql_database_instance.postgres.private_ip_address
}

output "database_name" {
  description = "Initial Postgres database name."
  value       = var.db_name
}

output "artifact_bucket" {
  description = "GCS bucket name for .lsworld artifacts."
  value       = google_storage_bucket.artifacts.name
}

output "helm_values_hint" {
  description = "externalServices overrides to pass to the lawsynth Helm chart."
  value = {
    "externalServices.postgres.host"              = google_sql_database_instance.postgres.private_ip_address
    "externalServices.postgres.database"          = var.db_name
    "externalServices.objectStore.endpoint"       = "https://storage.googleapis.com"
    "externalServices.objectStore.region"         = var.region
    "externalServices.objectStore.bucket"         = google_storage_bucket.artifacts.name
    "externalServices.objectStore.forcePathStyle" = "false"
  }
}
