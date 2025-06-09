terraform {
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

resource "latitudesh_ssh_key" "qtrade_ssh_key" {
  project    = var.project
  name       = "qtrade_ssh_key"
  # Set via TF_VAR_ssh_public_key
  public_key = var.ssh_public_key
}
