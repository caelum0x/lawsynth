# AKS cluster hosting the LawSynth services (api, scheduler, worker, artifact).

resource "azurerm_kubernetes_cluster" "this" {
  name                = "${local.name}-aks"
  location            = azurerm_resource_group.this.location
  resource_group_name = azurerm_resource_group.this.name
  dns_prefix          = "${local.name}-aks"
  kubernetes_version  = var.kubernetes_version != "" ? var.kubernetes_version : null

  # System node pool runs cluster add-ons; user workloads target the worker pool.
  default_node_pool {
    name                         = "system"
    node_count                   = var.system_node_count
    vm_size                      = "Standard_D4s_v5"
    vnet_subnet_id               = azurerm_subnet.aks.id
    orchestrator_version         = var.kubernetes_version != "" ? var.kubernetes_version : null
    only_critical_addons_enabled = true
  }

  identity {
    type = "SystemAssigned"
  }

  network_profile {
    network_plugin     = "azure"
    network_policy     = "cilium"
    network_data_plane = "cilium"
    load_balancer_sku  = "standard"
  }

  oidc_issuer_enabled       = true
  workload_identity_enabled = true

  tags = local.tags
}

resource "azurerm_kubernetes_cluster_node_pool" "workers" {
  name                  = "workers"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.this.id
  vm_size               = var.worker_vm_size
  vnet_subnet_id        = azurerm_subnet.aks.id

  auto_scaling_enabled = true
  min_count            = var.worker_min_count
  max_count            = var.worker_max_count

  os_disk_size_gb = 128
  os_disk_type    = "Ephemeral"

  node_labels = {
    "lawsynth.io/pool" = "workers"
  }

  tags = local.tags
}
