terraform {
  required_version = ">= 1.5.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.110"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Configure an azurerm backend for shared state before team use.
  #
  # backend "azurerm" {
  #   resource_group_name  = "lawsynth-tfstate"
  #   storage_account_name = "lawsynthtfstate"
  #   container_name       = "tfstate"
  #   key                  = "azure.terraform.tfstate"
  # }
}
