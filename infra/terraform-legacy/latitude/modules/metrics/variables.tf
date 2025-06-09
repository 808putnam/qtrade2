# Determines the server configuration we are going to deploy
# https://www.latitude.sh/pricing
# Set via a TF_VAR_project environment variable
variable "plan" {
  description = "Latitude.sh server plan"
}
 
# Determines the location we are going to deploy to
# https://www.latitude.sh/locations
# Set via a TF_VAR_project environment variable
variable "region" {
  description = "Latitude.sh server region slug"
}

# Set via a ssh module output
variable "ssh_public_key_id" {
  description = "Latitude.sh SSH public key id"
}

# Set via a TF_VAR_project environment variable
variable "project" {
  description = "Latitude.sh project id"
}

# Set via network module output
variable "network" {
  description = "Latitude.sh virtual network id"
}

# Set via a TF_VAR_enable_ssh_update environment variable
variable "enable_ssh_update" {
  description = "Allow creation of the ssh key to cause the servers to update"
}

# Set via a TF_VAR_enable_postgres_destroy environment variable
variable "enable_postgres_destroy" {
  description = "Allow for destruction of the postgres server"
}


