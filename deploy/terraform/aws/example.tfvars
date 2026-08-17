# Example variables for a small production LawSynth deployment on AWS.
# Copy to terraform.tfvars and adjust, or pass with `-var-file=example.tfvars`.

region      = "us-east-1"
name_prefix = "lawsynth"
environment = "prod"

tags = {
  Owner   = "platform-eng"
  CostCtr = "research"
}

# Networking
vpc_cidr           = "10.60.0.0/16"
az_count           = 3
single_nat_gateway = false

# Cluster
kubernetes_version    = "1.30"
worker_instance_types = ["m6i.2xlarge"]
worker_min_size       = 3
worker_max_size       = 20
worker_desired_size   = 3

# Database
postgres_version         = "16.3"
db_instance_class        = "db.m6g.large"
db_allocated_storage     = 100
db_max_allocated_storage = 500
db_multi_az              = true
db_backup_retention_days = 14
db_deletion_protection   = true

# Storage
artifact_force_destroy              = false
artifact_noncurrent_expiration_days = 180
