# Example variables for a LawSynth deployment on Google Cloud.
# Copy to terraform.tfvars and adjust, or pass with `-var-file=example.tfvars`.

project_id  = "my-lawsynth-project"
region      = "us-central1"
name_prefix = "lawsynth"
environment = "prod"

labels = {
  owner   = "platform-eng"
  costctr = "research"
}

# Networking
subnet_cidr   = "10.70.0.0/20"
pods_cidr     = "10.72.0.0/14"
services_cidr = "10.76.0.0/20"

# Cluster
release_channel      = "REGULAR"
worker_machine_type  = "e2-standard-8"
worker_min_nodes     = 1
worker_max_nodes     = 8
worker_initial_nodes = 1

# Database
postgres_version       = "POSTGRES_16"
db_tier                = "db-custom-4-16384"
db_disk_size           = 100
db_availability_type   = "REGIONAL"
db_backup_retention    = 14
db_deletion_protection = true

# Storage
artifact_force_destroy       = false
artifact_noncurrent_age_days = 180
