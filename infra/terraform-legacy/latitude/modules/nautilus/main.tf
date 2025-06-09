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

resource "latitudesh_server" "qtrade_nautilus_node" {
  
  hostname         = "qtrade_nautilus_node"
  operating_system = "ubuntu_20_04_x64_lts" # Matches what the required OS for Solana is
  plan             = var.plan
  project          = var.project
  billing          = "hourly"
  site             = var.region
  ssh_keys         = [var.ssh_public_key_id]

  lifecycle {
    # A static list is required, hence comment out ignore_changes line
    # if we change ssh key to access servers with.
    # This will not work: ignore_changes = var.enable_ssh_update == true ? [ssh_keys] : []
    ignore_changes = [ssh_keys]
  }
}

resource "latitudesh_vlan_assignment" "qtrade_nautilus_vlan_assignment" {
    server_id          = latitudesh_server.qtrade_nautilus_node.id
    virtual_network_id = var.network
}