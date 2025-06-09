# Determines the location we are going to deploy to
# https://www.latitude.sh/locations
# Set via a TF_VAR_project environment variable
variable "region" {
  description = "Latitude.sh server region slug"
}

# Set via a TF_VAR_project environment variable
variable "project" {
  description = "Latitude.sh project id"
}
