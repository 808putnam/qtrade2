terraform {
  backend "s3" {
        bucket="qtrade-terraform"
        key = "latitude/accountsdb.tfstate"
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

resource "latitudesh_server" "qtrade_accountsdb_node" {
  hostname         = "qtrade_accountsdb_node"
  operating_system = "ubuntu_20_04_x64_lts" # Matches what the required OS for Solana is
  plan             = var.plan_other
  project          = var.project
  billing          = "hourly"
  site             = var.region
  # ssh_keys         = [var.ssh_key_id]

  # lifecycle {
  #   # A static list is required, hence comment out ignore_changes line
  #   # if we change ssh key to access servers with.
  #   # This will not work: ignore_changes = var.enable_ssh_update == true ? [ssh_keys] : []
  #   # ignore_changes = [ssh_keys]
  #   # Same here, we can't use a variable
  #   # This worn't work: prevent_destroy = var.enable_postgres_destroy == true ? false : true
  #   # Flip to true once stabilized
  #   prevent_destroy = false
  # }
}

# resource "latitudesh_vlan_assignment" "qtrade_accountsdb_vlan_assignment" {
#     server_id          = latitudesh_server.qtrade_accountsdb_node.id
#     virtual_network_id = var.virtual_network_id
# }