terraform {
    backend "s3" {
        bucket="qtrade-terraform"
        key = "latitude/network.tfstate"
  }

  required_providers {
    latitudesh = {
      source  = "latitudesh/latitudesh"

      # Make sure you are using the latest version published to the Terraform registry
      # https://registry.terraform.io/providers/latitudesh/latitudesh/latest/docs
      version = "~> 1.2.0" # Make sure you are using the latest version published to the Terraform registry

      # auth_token set via env. variable LATITUDESH_API_TOKEN
    }
  }
}

resource "latitudesh_virtual_network" "qtrade_virtual_network" {
  description      = "qtrade_virtual_network"
  site             = var.region
  project          = var.project
}
