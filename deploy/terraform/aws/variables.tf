variable "region" {
  description = "AWS region to deploy LawSynth into."
  type        = string
  default     = "us-east-1"
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

variable "vpc_cidr" {
  description = "CIDR block for the LawSynth VPC."
  type        = string
  default     = "10.60.0.0/16"
}

variable "az_count" {
  description = "Number of availability zones to spread subnets across."
  type        = number
  default     = 3

  validation {
    condition     = var.az_count >= 2 && var.az_count <= 4
    error_message = "az_count must be between 2 and 4 for a highly available deployment."
  }
}

variable "single_nat_gateway" {
  description = "Use a single NAT gateway (cheaper) instead of one per AZ."
  type        = bool
  default     = true
}

# --- Cluster ----------------------------------------------------------------

variable "kubernetes_version" {
  description = "EKS control plane Kubernetes version."
  type        = string
  default     = "1.30"
}

variable "worker_instance_types" {
  description = "Instance types for the LawSynth worker node group (CPU-heavy)."
  type        = list(string)
  default     = ["m6i.2xlarge"]
}

variable "worker_min_size" {
  description = "Minimum number of worker nodes."
  type        = number
  default     = 2
}

variable "worker_max_size" {
  description = "Maximum number of worker nodes."
  type        = number
  default     = 10
}

variable "worker_desired_size" {
  description = "Initial desired number of worker nodes."
  type        = number
  default     = 3
}

variable "cluster_public_access_cidrs" {
  description = "CIDRs allowed to reach the EKS public API endpoint."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

# --- Database ---------------------------------------------------------------

variable "postgres_version" {
  description = "RDS Postgres engine version."
  type        = string
  default     = "16.3"
}

variable "db_instance_class" {
  description = "RDS instance class for the metadata database."
  type        = string
  default     = "db.m6g.large"
}

variable "db_allocated_storage" {
  description = "Initial allocated storage (GiB) for the metadata database."
  type        = number
  default     = 50
}

variable "db_max_allocated_storage" {
  description = "Storage autoscaling ceiling (GiB). Set equal to allocated to disable."
  type        = number
  default     = 200
}

variable "db_name" {
  description = "Initial Postgres database name."
  type        = string
  default     = "lawsynth"
}

variable "db_username" {
  description = "Master username for the metadata database."
  type        = string
  default     = "lawsynth"
}

variable "db_multi_az" {
  description = "Enable Multi-AZ standby for the metadata database."
  type        = bool
  default     = false
}

variable "db_backup_retention_days" {
  description = "Automated backup retention in days (RPO target is 15 min)."
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
  description = "Name of the S3 bucket for content-addressed .lsworld artifacts. Empty auto-generates a unique name."
  type        = string
  default     = ""
}

variable "artifact_force_destroy" {
  description = "Allow Terraform to delete a non-empty artifact bucket. Keep false in production."
  type        = bool
  default     = false
}

variable "artifact_noncurrent_expiration_days" {
  description = "Days after which noncurrent artifact object versions expire."
  type        = number
  default     = 90
}
