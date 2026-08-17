variable "project_id" {
  description = "GCP project ID to deploy LawSynth into."
  type        = string
}

variable "region" {
  description = "GCP region for regional resources."
  type        = string
  default     = "us-central1"
}

variable "name_prefix" {
  description = "Prefix applied to all resource names."
  type        = string
  default     = "lawsynth"
}

variable "environment" {
  description = "Deployment environment (e.g. dev, staging, prod)."
  type        = string
  default     = "dev"
}

variable "labels" {
  description = "Additional labels merged onto every resource."
  type        = map(string)
  default     = {}
}

# --- Networking -------------------------------------------------------------

variable "subnet_cidr" {
  description = "Primary CIDR for the node subnet."
  type        = string
  default     = "10.70.0.0/20"
}

variable "pods_cidr" {
  description = "Secondary CIDR range for GKE pods (VPC-native)."
  type        = string
  default     = "10.72.0.0/14"
}

variable "services_cidr" {
  description = "Secondary CIDR range for GKE services (VPC-native)."
  type        = string
  default     = "10.76.0.0/20"
}

# --- Cluster ----------------------------------------------------------------

variable "kubernetes_version" {
  description = "GKE release channel version prefix, or empty to use the channel default."
  type        = string
  default     = ""
}

variable "release_channel" {
  description = "GKE release channel: RAPID, REGULAR, or STABLE."
  type        = string
  default     = "REGULAR"
}

variable "worker_machine_type" {
  description = "Machine type for the LawSynth worker node pool (CPU-heavy)."
  type        = string
  default     = "e2-standard-8"
}

variable "worker_min_nodes" {
  description = "Minimum nodes per zone in the worker node pool."
  type        = number
  default     = 1
}

variable "worker_max_nodes" {
  description = "Maximum nodes per zone in the worker node pool."
  type        = number
  default     = 5
}

variable "worker_initial_nodes" {
  description = "Initial nodes per zone in the worker node pool."
  type        = number
  default     = 1
}

# --- Database ---------------------------------------------------------------

variable "postgres_version" {
  description = "Cloud SQL Postgres version (e.g. POSTGRES_16)."
  type        = string
  default     = "POSTGRES_16"
}

variable "db_tier" {
  description = "Cloud SQL machine tier for the metadata database."
  type        = string
  default     = "db-custom-2-8192"
}

variable "db_disk_size" {
  description = "Cloud SQL data disk size in GiB."
  type        = number
  default     = 50
}

variable "db_name" {
  description = "Initial Postgres database name."
  type        = string
  default     = "lawsynth"
}

variable "db_username" {
  description = "Metadata database user."
  type        = string
  default     = "lawsynth"
}

variable "db_availability_type" {
  description = "Cloud SQL availability: ZONAL or REGIONAL (HA)."
  type        = string
  default     = "ZONAL"
}

variable "db_backup_retention" {
  description = "Number of automated backups to retain."
  type        = number
  default     = 7
}

variable "db_deletion_protection" {
  description = "Prevent accidental deletion of the metadata database."
  type        = bool
  default     = true
}

# --- Storage ----------------------------------------------------------------

variable "artifact_bucket_name" {
  description = "GCS bucket for .lsworld artifacts. Empty auto-generates a unique name."
  type        = string
  default     = ""
}

variable "artifact_force_destroy" {
  description = "Allow Terraform to delete a non-empty artifact bucket."
  type        = bool
  default     = false
}

variable "artifact_noncurrent_age_days" {
  description = "Days after which noncurrent artifact object versions are deleted."
  type        = number
  default     = 90
}
