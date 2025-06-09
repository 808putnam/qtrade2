terraform {
    backend "s3" {
        bucket="qtrade-terraform"
        key = "latitude/terraform.tfstate"
  }
  required_providers {
    # IMPORTANT: If you modify this provider, update the required_providers entry in the modules as well
    latitudesh = {
      source  = "latitudesh/latitudesh"

      # Make sure you are using the latest version published to the Terraform registry
      # https://registry.terraform.io/providers/latitudesh/latitudesh/latest/docs
      version = "~> 1.2.0" # Make sure you are using the latest version published to the Terraform registry

      # auth_token set via env. variable LATITUDESH_API_TOKEN
    }
  }
}

module "ssh" {
  source = "./modules/ssh"
  project = var.project
  ssh_public_key = var.ssh_public_key
}

module "network" {
  source = "./modules/network"
  region = var.region
  project = var.project
}

module "accountsdb" {
  source = "./modules/accountsdb"
  plan = var.plan_other
  region = var.region
  ssh_public_key_id = module.ssh.qtrade_ssh_key_id
  project = var.project
  network = module.network.qtrade_virtual_network_id
  enable_ssh_update = var.enable_ssh_update
  enable_postgres_destroy = var.enable_accountsdb_destroy
}

module "metrics" {
  source = "./modules/metrics"
  plan = var.plan_other
  region = var.region
  ssh_public_key_id = module.ssh.qtrade_ssh_key_id
  project = var.project
  network = module.network.qtrade_virtual_network_id
  enable_ssh_update = var.enable_ssh_update
  enable_postgres_destroy = var.enable_metrics_destroy
}

module "nautilus" {
  source = "./modules/nautilus"
  plan = var.plan_other
  region = var.region
  ssh_public_key_id = module.ssh.qtrade_ssh_key_id
  project = var.project
  network = module.network.qtrade_virtual_network_id
  enable_ssh_update = var.enable_ssh_update
}

module "rpc" {
  source = "./modules/rpc"
  plan = var.plan_rpc
  region = var.region
  ssh_public_key_id = module.ssh.qtrade_ssh_key_id
  project = var.project
  network = module.network.qtrade_virtual_network_id
  enable_ssh_update = var.enable_ssh_update
}

module "vixen" {
  source = "./modules/vixen"
  plan = var.plan_other
  region = var.region
  ssh_public_key_id = module.ssh.qtrade_ssh_key_id
  project = var.project
  network = module.network.qtrade_virtual_network_id
  enable_ssh_update = var.enable_ssh_update
}
