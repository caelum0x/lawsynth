terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.30"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Configure a GCS backend for shared state before team use.
  #
  # backend "gcs" {
  #   bucket = "lawsynth-tfstate"
  #   prefix = "gcp"
  # }
}
