variable "location" {
  description = "Azure region to deploy LawSynth into."
  type        = string
  default     = "eastus"
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

variable "tags" {
  description = "Additional tags merged onto every resource."
  type        = map(string)
  default     = {}
}

# --- Networking -------------------------------------------------------------

variable "vnet_cidr" {
  description = "Address space for the LawSynth virtual network."
  type        = string
  default     = "10.80.0.0/16"
}

variable "aks_subnet_cidr" {
  description = "Subnet CIDR for AKS nodes."
  type        = string
  default     = "10.80.0.0/20"
}

variable "db_subnet_cidr" {
  description = "Delegated subnet CIDR for the PostgreSQL Flexible Server."
  type        = string
  default     = "10.80.16.0/24"
}

# --- Cluster ----------------------------------------------------------------

variable "kubernetes_version" {
  description = "AKS Kubernetes version, or empty to use the region default."
  type        = string
  default     = ""
}

variable "worker_vm_size" {
  description = "VM size for the LawSynth worker node pool (CPU-heavy)."
  type        = string
  default     = "Standard_D8s_v5"
}

variable "worker_min_count" {
  description = "Minimum nodes in the worker node pool."
  type        = number
  default     = 2
}

variable "worker_max_count" {
  description = "Maximum nodes in the worker node pool."
  type        = number
  default     = 10
}

variable "system_node_count" {
  description = "Node count for the AKS system node pool."
  type        = number
  default     = 2
}

# --- Database ---------------------------------------------------------------

variable "postgres_version" {
  description = "PostgreSQL Flexible Server major version."
  type        = string
  default     = "16"
}

variable "db_sku_name" {
  description = "SKU for the PostgreSQL Flexible Server (e.g. GP_Standard_D2ds_v5)."
  type        = string
  default     = "GP_Standard_D2ds_v5"
}

variable "db_storage_mb" {
  description = "Storage for the PostgreSQL Flexible Server in MB."
  type        = number
  default     = 65536
}

variable "db_name" {
  description = "Initial Postgres database name."
  type        = string
  default     = "lawsynth"
}

variable "db_username" {
  description = "Administrator login for the metadata database."
  type        = string
  default     = "lawsynth"
}

variable "db_high_availability" {
  description = "Enable zone-redundant high availability for the database."
  type        = bool
  default     = false
}

variable "db_backup_retention_days" {
  description = "Automated backup retention in days."
  type        = number
  default     = 7
}

# --- Storage ----------------------------------------------------------------

variable "storage_account_name" {
  description = "Storage account name for artifacts (3-24 lowercase alphanumeric). Empty auto-generates."
  type        = string
  default     = ""
}

variable "artifact_container_name" {
  description = "Blob container name for content-addressed .lsworld artifacts."
  type        = string
  default     = "artifacts"
}

variable "artifact_retention_days" {
  description = "Days to retain soft-deleted blobs and previous versions."
  type        = number
  default     = 90
}
