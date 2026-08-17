# -----------------------------------------------------------------------------
# LawSynth distributed deployment on Microsoft Azure.
#
#   - Resource group + VNet/subnet    (network.tf)
#   - AKS cluster + node pool          (cluster.tf)
#   - PostgreSQL Flexible Server       (database.tf)
#   - Storage account + blob container (storage.tf)
#
# NATS (job bus) runs in-cluster and is not provisioned here.
# -----------------------------------------------------------------------------

provider "azurerm" {
  features {
    resource_group {
      prevent_deletion_if_contains_resources = true
    }
  }
}

locals {
  name = "${var.name_prefix}-${var.environment}"

  tags = merge(
    {
      "app.kubernetes.io/part-of" = "lawsynth"
      project                     = "lawsynth"
      environment                 = var.environment
      managedBy                   = "terraform"
    },
    var.tags,
  )
}

resource "azurerm_resource_group" "this" {
  name     = "${local.name}-rg"
  location = var.location
  tags     = local.tags
}
