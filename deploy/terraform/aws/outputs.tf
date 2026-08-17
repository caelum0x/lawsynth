output "region" {
  description = "AWS region of the deployment."
  value       = var.region
}

output "cluster_name" {
  description = "EKS cluster name."
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "EKS API server endpoint."
  value       = module.eks.cluster_endpoint
}

output "cluster_certificate_authority_data" {
  description = "Base64-encoded EKS cluster CA certificate."
  value       = module.eks.cluster_certificate_authority_data
  sensitive   = true
}

output "kubeconfig_command" {
  description = "Command to configure kubectl for this cluster."
  value       = "aws eks update-kubeconfig --region ${var.region} --name ${module.eks.cluster_name}"
}

output "database_endpoint" {
  description = "Postgres connection endpoint (host:port)."
  value       = module.db.db_instance_endpoint
}

output "database_name" {
  description = "Initial Postgres database name."
  value       = var.db_name
}

output "database_secret_arn" {
  description = "Secrets Manager ARN holding the database master credentials."
  value       = aws_secretsmanager_secret.db.arn
}

output "artifact_bucket" {
  description = "S3 bucket name for .lsworld artifacts."
  value       = module.artifact_bucket.s3_bucket_id
}

output "artifact_bucket_arn" {
  description = "ARN of the artifact S3 bucket."
  value       = module.artifact_bucket.s3_bucket_arn
}

output "helm_values_hint" {
  description = "externalServices overrides to pass to the lawsynth Helm chart."
  value = {
    "externalServices.postgres.host"              = module.db.db_instance_address
    "externalServices.postgres.database"          = var.db_name
    "externalServices.objectStore.endpoint"       = "https://s3.${var.region}.amazonaws.com"
    "externalServices.objectStore.region"         = var.region
    "externalServices.objectStore.bucket"         = module.artifact_bucket.s3_bucket_id
    "externalServices.objectStore.forcePathStyle" = "false"
  }
}
