# Set via a TF_VAR_ssh_public_key environment variable 
variable "ssh_public_key" {
  description = "Latitude.sh SSH public key"
}

# Set via a TF_VAR_project environment variable
variable "project" {
  description = "Latitude.sh project id"
}
