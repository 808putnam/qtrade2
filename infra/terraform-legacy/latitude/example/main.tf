terraform {
  backend "s3" {
        bucket="qtrade-terraform"
        key = "latitude/example.tfstate"
        region = "us-east-1"
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

# Configure the provider
provider "latitudesh" {
  auth_token = "<auth_tokenh>"
}
resource "latitudesh_server" "example_node" {
  hostname         = "example_node"
  operating_system = "ubuntu_20_04_x64_lts"
  plan             = "m4-metal-medium"
  project          = "<project_id>"
  billing          = "hourly"
  site             = "CHI"
  ssh_keys         = ["<ssh_key"]
}

