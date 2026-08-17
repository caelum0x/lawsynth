# PostgreSQL Flexible Server backing LawSynth metadata: run specs, artifact
# references, and append-only run events. Private VNet access only.

resource "random_password" "db" {
  length  = 32
  special = false
}

resource "azurerm_postgresql_flexible_server" "this" {
  name                = "${local.name}-pg"
  resource_group_name = azurerm_resource_group.this.name
  location            = azurerm_resource_group.this.location
  version             = var.postgres_version

  administrator_login    = var.db_username
  administrator_password = random_password.db.result

  sku_name   = var.db_sku_name
  storage_mb = var.db_storage_mb

  backup_retention_days = var.db_backup_retention_days

  # Private access: integrate into the delegated subnet, resolve via private DNS.
  delegated_subnet_id = azurerm_subnet.database.id
  private_dns_zone_id = azurerm_private_dns_zone.postgres.id

  dynamic "high_availability" {
    for_each = var.db_high_availability ? [1] : []
    content {
      mode = "ZoneRedundant"
    }
  }

  tags = local.tags

  depends_on = [azurerm_private_dns_zone_virtual_network_link.postgres]

  lifecycle {
    # Zone assignment can be picked by Azure; avoid perpetual diffs.
    ignore_changes = [zone]
  }
}

resource "azurerm_postgresql_flexible_server_database" "lawsynth" {
  name      = var.db_name
  server_id = azurerm_postgresql_flexible_server.this.id
  charset   = "UTF8"
  collation = "en_US.utf8"
}

# Require TLS for all client connections.
resource "azurerm_postgresql_flexible_server_configuration" "require_ssl" {
  name      = "require_secure_transport"
  server_id = azurerm_postgresql_flexible_server.this.id
  value     = "ON"
}
