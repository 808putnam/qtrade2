# Determines the server configuration we are going to deploy
# https://www.latitude.sh/pricing
# Set via a TF_VAR_plan_rpc environment variable
variable "plan_rpc" {
  description = "Latitude.sh server plan for rpc server"
}
# Set via a TF_VAR_plan_other environment variable
variable "plan_other" {
  description = "Latitude.sh server plan for other servers"
}
 
# Determines the location we are going to deploy to
# https://www.latitude.sh/locations
# Set via a TF_VAR_project environment variable
variable "region" {
  description = "Latitude.sh server region slug"
}

# Set via a TF_VAR_ssh_public_key environment variable 
variable "ssh_public_key" {
  description = "Latitude.sh SSH public key"
}

# Set via a TF_VAR_project environment variable
variable "project" {
  description = "Latitude.sh project id"
}

# Set via a TF_VAR_enable_ssh_update environment variable
variable "enable_ssh_update" {
  description = "Allow creation of the ssh key to cause the servers to update"
}

# Set via a TF_VAR_enable_accountsdb_destroy environment variable
variable "enable_accountsdb_destroy" {
  description = "Allow for destruction of the accountsdb server"
}
# Set via a TF_VAR_enable_metrics_destroy environment variable
variable "enable_metrics_destroy" {
  description = "Allow for destruction of the metrics server"
}
