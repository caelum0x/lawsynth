# Example variables for a LawSynth deployment on Microsoft Azure.
# Copy to terraform.tfvars and adjust, or pass with `-var-file=example.tfvars`.

location    = "eastus"
name_prefix = "lawsynth"
environment = "prod"

tags = {
  owner   = "platform-eng"
  costctr = "research"
}

# Networking
vnet_cidr       = "10.80.0.0/16"
aks_subnet_cidr = "10.80.0.0/20"
db_subnet_cidr  = "10.80.16.0/24"

# Cluster
worker_vm_size    = "Standard_D8s_v5"
worker_min_count  = 3
worker_max_count  = 20
system_node_count = 2

# Database
postgres_version         = "16"
db_sku_name              = "GP_Standard_D4ds_v5"
db_storage_mb            = 131072
db_high_availability     = true
db_backup_retention_days = 14

# Storage
artifact_container_name = "artifacts"
artifact_retention_days = 180
