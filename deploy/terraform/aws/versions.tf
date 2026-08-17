terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.40"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Configure a remote backend for shared state before using in a team.
  # Left as a partial config so it can be supplied via `-backend-config`.
  #
  # backend "s3" {
  #   bucket         = "lawsynth-tfstate"
  #   key            = "aws/terraform.tfstate"
  #   region         = "us-east-1"
  #   dynamodb_table = "lawsynth-tflock"
  #   encrypt        = true
  # }
}
