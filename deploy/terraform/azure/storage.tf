# Storage account + blob container for content-addressed .lsworld artifacts.

resource "random_string" "sa_suffix" {
  length  = 6
  upper   = false
  special = false
}

locals {
  # Storage account names: 3-24 chars, lowercase alphanumeric only.
  storage_account_name = coalesce(
    var.storage_account_name,
    substr("${replace(local.name, "-", "")}sa${random_string.sa_suffix.result}", 0, 24),
  )
}

resource "azurerm_storage_account" "artifacts" {
  name                = local.storage_account_name
  resource_group_name = azurerm_resource_group.this.name
  location            = azurerm_resource_group.this.location

  account_tier             = "Standard"
  account_replication_type = "ZRS"
  account_kind             = "StorageV2"

  min_tls_version                 = "TLS1_2"
  https_traffic_only_enabled      = true
  allow_nested_items_to_be_public = false

  blob_properties {
    versioning_enabled = true

    delete_retention_policy {
      days = var.artifact_retention_days
    }
    container_delete_retention_policy {
      days = var.artifact_retention_days
    }
  }

  tags = local.tags
}

resource "azurerm_storage_container" "artifacts" {
  name                  = var.artifact_container_name
  storage_account_name  = azurerm_storage_account.artifacts.name
  container_access_type = "private"
}
