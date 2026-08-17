output "location" {
  description = "Azure region of the deployment."
  value       = azurerm_resource_group.this.location
}

output "resource_group" {
  description = "Resource group name."
  value       = azurerm_resource_group.this.name
}

output "cluster_name" {
  description = "AKS cluster name."
  value       = azurerm_kubernetes_cluster.this.name
}

output "kubeconfig_command" {
  description = "Command to configure kubectl for this cluster."
  value       = "az aks get-credentials --resource-group ${azurerm_resource_group.this.name} --name ${azurerm_kubernetes_cluster.this.name}"
}

output "database_fqdn" {
  description = "Fully qualified domain name of the Postgres flexible server."
  value       = azurerm_postgresql_flexible_server.this.fqdn
}

output "database_name" {
  description = "Initial Postgres database name."
  value       = var.db_name
}

output "artifact_storage_account" {
  description = "Storage account name holding the artifact container."
  value       = azurerm_storage_account.artifacts.name
}

output "artifact_container" {
  description = "Blob container name for .lsworld artifacts."
  value       = azurerm_storage_container.artifacts.name
}

output "helm_values_hint" {
  description = "externalServices overrides to pass to the lawsynth Helm chart."
  value = {
    "externalServices.postgres.host"              = azurerm_postgresql_flexible_server.this.fqdn
    "externalServices.postgres.database"          = var.db_name
    "externalServices.objectStore.endpoint"       = "https://${azurerm_storage_account.artifacts.name}.blob.core.windows.net"
    "externalServices.objectStore.bucket"         = azurerm_storage_container.artifacts.name
    "externalServices.objectStore.forcePathStyle" = "false"
  }
}
